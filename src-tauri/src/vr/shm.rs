//! Producer half of the PitWall VR shared-memory contract.
//!
//! Writes a compact mirror of [`LiveSnapshot`](crate::live::LiveSnapshot) plus
//! per-overlay placement into a named Windows file mapping that the
//! `pitwall-openxr-layer` DLL reads each frame. The binary layout MUST match
//! `openxr-layer/include/pitwall_vr_shm.h` exactly — every field is 4 bytes (or
//! a char array of length divisible by 4) so neither side needs explicit
//! packing, and 64-bit values are split into lo/hi `u32` pairs.

use crate::live::{LiveSnapshot, PackState};

pub const MAGIC: u32 = 0x5256_5750; // "PWVR"
pub const VERSION: u32 = 1;
pub const SHM_NAME: &str = r"Local\PitWallVR";

pub const MAX_OVERLAYS: usize = 4;
pub const MAX_COMPETITORS: usize = 64;
pub const NUM_LEN: usize = 8;
pub const NAME_LEN: usize = 40;
pub const TRACK_LEN: usize = 64;
pub const SESSION_LEN: usize = 32;

// Overlay slot kinds. The array index in `overlay_layout.widgets` equals the
// kind, which equals the slot the C++ layer composites.
#[allow(dead_code)]
pub const KIND_COACH: u32 = 0;
pub const KIND_STANDINGS: u32 = 1;
pub const KIND_RELATIVE: u32 = 2;
pub const KIND_RADAR: u32 = 3;

pub const LOCK_VIEW: u32 = 0;
#[allow(dead_code)]
pub const LOCK_LOCAL: u32 = 1;

pub const FLAG_IS_PLAYER: u32 = 0x1;
pub const FLAG_ON_PIT_ROAD: u32 = 0x2;

pub const FIELD_PACE_BEST: u32 = 0;
pub const FIELD_PACE_OPTIMAL: u32 = 1;
pub const FIELD_PACE_BOTH: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PwOverlay {
    pub enabled: u32,
    pub kind: u32,
    pub lock_space: u32,
    pub opacity: f32,
    pub pos: [f32; 3],
    pub rot: [f32; 4],
    pub size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PwCompetitor {
    pub position: i32,
    pub class_position: i32,
    pub class_id: i32,
    pub best_lap_ms: f32,
    pub last_lap_ms: f32,
    pub lap_dist_pct: f32,
    pub gap_to_player_s: f32,
    pub flags: u32,
    pub number: [u8; NUM_LEN],
    pub name: [u8; NAME_LEN],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PwSnapshot {
    pub lap: i32,
    pub lap_time_ms: f32,
    pub last_lap_ms: f32,
    pub best_lap_ms: f32,
    pub delta_best_ms: f32,
    pub delta_last_ms: f32,
    pub delta_field_best_ms: f32,
    pub delta_field_optimal_ms: f32,
    pub player_position: i32,
    pub player_class_position: i32,
    pub gap_ahead_s: f32,
    pub gap_behind_s: f32,
    pub fuel_level: f32,
    pub speed: f32,
    pub lap_dist_pct: f32,
    pub current_sector: i32,
    pub pack_state: u32,
    pub session_flags: u32,
    pub incident_count: i32,
    pub session_laps_remain: i32,
    pub session_time_remain_s: f32,
    pub on_track: u32,
    pub field_pace_mode: u32,
    pub sector_pct: [f32; 3],
    pub sector_done: [u32; 3],
    pub competitor_count: u32,
    pub track: [u8; TRACK_LEN],
    pub session_type: [u8; SESSION_LEN],
    pub competitors: [PwCompetitor; MAX_COMPETITORS],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PwSharedBlock {
    pub magic: u32,
    pub version: u32,
    pub seq: u32,
    pub overlay_count: u32,
    pub write_ms_lo: u32,
    pub write_ms_hi: u32,
    pub overlays: [PwOverlay; MAX_OVERLAYS],
    pub snapshot: PwSnapshot,
}

impl PwSharedBlock {
    /// A zeroed block with the header identity filled in.
    fn empty() -> Self {
        // All fields are plain old data (POD); zeroing is a valid initial state.
        let mut block: PwSharedBlock = unsafe { std::mem::zeroed() };
        block.magic = MAGIC;
        block.version = VERSION;
        block
    }
}

/// Placement for one overlay slot, sourced from `settings.overlay_layout`.
#[derive(Clone, Copy)]
pub struct SlotPlacement {
    pub enabled: bool,
    pub vertical_offset: f32,
    pub scale: f32,
    pub opacity: f32,
}

/// Base VR pose (position + quad size in meters) for a widget kind, before the
/// user's per-widget vertical offset and scale are applied. Slots are spread
/// around the windshield so enabling several at once does not overlap, and the
/// quad aspect ratios match the per-kind swapchain dimensions in the layer so
/// the rendered HUD is not stretched.
fn base_pose(kind: u32) -> ([f32; 3], [f32; 2]) {
    match kind {
        // Tall list, lower-left (512 x 640 texture).
        KIND_STANDINGS => ([-0.85, -0.30, -1.4], [0.46, 0.575]),
        // Square board, lower-right (512 x 512 texture).
        KIND_RELATIVE => ([0.85, -0.30, -1.4], [0.46, 0.46]),
        // Square dish, low-center (512 x 512 texture).
        KIND_RADAR => ([0.0, -0.55, -1.25], [0.40, 0.40]),
        // Coach (and any unknown kind): wide-short, centered upper windshield
        // (1024 x 288 texture).
        _ => ([0.0, 0.0, -1.2], [0.80, 0.225]),
    }
}

/// `None` floats are encoded as NaN; the C++ reader checks `isnan`.
fn opt_f32(v: Option<f64>) -> f32 {
    v.map(|x| x as f32).unwrap_or(f32::NAN)
}

fn opt_f32_s(v: Option<f32>) -> f32 {
    v.unwrap_or(f32::NAN)
}

fn copy_str<const N: usize>(s: &str, dst: &mut [u8; N]) {
    let bytes = s.as_bytes();
    let n = bytes.len().min(N - 1);
    dst[..n].copy_from_slice(&bytes[..n]);
    dst[n] = 0;
}

/// Build the shared block from the current snapshot and per-slot placement.
///
/// `slots` is indexed by widget kind (0 = coach, 1 = standings, 2 = relative,
/// 3 = radar); each enabled slot becomes a composition-layer quad. Disabled
/// slots are packed out so the layer only iterates the active ones.
pub fn build_block(
    snap: &LiveSnapshot,
    slots: &[SlotPlacement; MAX_OVERLAYS],
    field_pace_mode: u32,
) -> PwSharedBlock {
    let mut block = PwSharedBlock::empty();

    let s = &mut block.snapshot;
    s.lap = snap.lap;
    s.lap_time_ms = snap.lap_time_ms as f32;
    s.last_lap_ms = opt_f32(snap.last_lap_ms);
    s.best_lap_ms = opt_f32(snap.best_lap_ms);
    s.delta_best_ms = opt_f32(snap.delta_to_best_ms);
    s.delta_last_ms = opt_f32(snap.delta_to_last_ms);
    s.delta_field_best_ms = opt_f32(snap.delta_to_session_best_ms);
    s.delta_field_optimal_ms = opt_f32(snap.delta_to_session_optimal_ms);
    s.player_position = snap.player_position.unwrap_or(0);
    s.player_class_position = snap.player_class_position.unwrap_or(0);
    s.gap_ahead_s = opt_f32_s(snap.gap_to_car_ahead_s);
    s.gap_behind_s = opt_f32_s(snap.gap_to_car_behind_s);
    s.fuel_level = snap.fuel_level;
    s.speed = snap.speed;
    s.lap_dist_pct = snap.lap_dist_pct;
    s.current_sector = snap.current_sector;
    s.pack_state = pack_ordinal(snap.pack_state);
    s.session_flags = snap.session_flags;
    s.incident_count = snap.incident_count;
    s.session_laps_remain = snap.session_laps_remain.unwrap_or(-1);
    s.session_time_remain_s = opt_f32(snap.session_time_remain_s);
    s.on_track = snap.on_track as u32;
    s.field_pace_mode = field_pace_mode;
    copy_str(&snap.track, &mut s.track);
    copy_str(&snap.session_type, &mut s.session_type);

    for n in 0..3 {
        let sector = snap.sectors.iter().find(|x| x.sector_num == (n as i32 + 1));
        s.sector_done[n] = sector.map(|x| x.completed as u32).unwrap_or(0);
        s.sector_pct[n] = sector_progress(snap, n as i32 + 1);
    }

    let count = snap.competitors.len().min(MAX_COMPETITORS);
    s.competitor_count = count as u32;
    for (i, c) in snap.competitors.iter().take(count).enumerate() {
        let dst = &mut s.competitors[i];
        dst.position = c.position;
        dst.class_position = c.class_position;
        dst.class_id = c.class_id;
        dst.best_lap_ms = opt_f32(c.best_lap_ms);
        dst.last_lap_ms = opt_f32(c.last_lap_ms);
        dst.lap_dist_pct = c.lap_dist_pct;
        dst.gap_to_player_s = opt_f32_s(c.gap_to_player_s);
        dst.flags = (c.is_player as u32 * FLAG_IS_PLAYER)
            | (c.on_pit_road as u32 * FLAG_ON_PIT_ROAD);
        copy_str(&c.car_number, &mut dst.number);
        copy_str(&c.driver_name, &mut dst.name);
    }

    // Fixed slots: overlay[kind] always represents that kind so the layer can
    // keep a stable, correctly-sized swapchain per slot. Disabled widgets carry
    // `enabled = 0` and are skipped by the compositor.
    for kind in 0..MAX_OVERLAYS as u32 {
        let slot = slots[kind as usize];
        let (base_pos, base_size) = base_pose(kind);
        let scale = slot.scale.max(0.1);
        block.overlays[kind as usize] = PwOverlay {
            enabled: slot.enabled as u32,
            kind,
            lock_space: LOCK_VIEW,
            opacity: slot.opacity.clamp(0.0, 1.0),
            pos: [base_pos[0], base_pos[1] + slot.vertical_offset, base_pos[2]],
            rot: [0.0, 0.0, 0.0, 1.0],
            size: [base_size[0] * scale, base_size[1] * scale],
        };
    }
    block.overlay_count = MAX_OVERLAYS as u32;

    block
}

fn pack_ordinal(p: PackState) -> u32 {
    match p {
        PackState::Off => 0,
        PackState::Clear => 1,
        PackState::CarLeft => 2,
        PackState::CarRight => 3,
        PackState::ThreeWide => 4,
        PackState::TwoCarsLeft => 5,
        PackState::TwoCarsRight => 6,
    }
}

/// Progress (0..1) of the given sector, mirroring the web HUD logic.
fn sector_progress(snap: &LiveSnapshot, sector_num: i32) -> f32 {
    if let Some(sec) = snap.sectors.iter().find(|x| x.sector_num == sector_num) {
        if sec.completed {
            return 1.0;
        }
    }
    if snap.current_sector != sector_num {
        return 0.0;
    }
    let bounds = [0.0_f32, 0.33, 0.66, 1.0];
    let start = bounds[(sector_num - 1).clamp(0, 3) as usize];
    let end = bounds[sector_num.clamp(0, 3) as usize];
    let span = end - start;
    if span <= 0.0 {
        return 0.0;
    }
    ((snap.lap_dist_pct - start) / span).clamp(0.0, 1.0)
}

#[cfg(windows)]
pub use windows_impl::ShmWriter;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::sync::atomic::{fence, Ordering};

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows::Win32::System::Memory::{
        CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_WRITE,
        MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
    };

    /// Owns the named file mapping and publishes blocks with a seqlock so the
    /// reader never composites a half-written frame.
    pub struct ShmWriter {
        mapping: HANDLE,
        view: *mut PwSharedBlock,
        seq: u32,
    }

    // The raw view pointer is only ever touched from the single producer thread.
    unsafe impl Send for ShmWriter {}

    impl ShmWriter {
        pub fn open() -> anyhow::Result<Self> {
            let name: Vec<u16> = SHM_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let size = std::mem::size_of::<PwSharedBlock>() as u32;
            unsafe {
                let mapping = CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    None,
                    PAGE_READWRITE,
                    0,
                    size,
                    PCWSTR(name.as_ptr()),
                )?;
                let addr: MEMORY_MAPPED_VIEW_ADDRESS =
                    MapViewOfFile(mapping, FILE_MAP_WRITE, 0, 0, size as usize);
                if addr.Value.is_null() {
                    let _ = CloseHandle(mapping);
                    anyhow::bail!("MapViewOfFile failed for {SHM_NAME}");
                }
                let view = addr.Value as *mut PwSharedBlock;
                // Initialize the header so an early reader sees a valid, even seq.
                view.write(PwSharedBlock::empty());
                Ok(Self { mapping, view, seq: 0 })
            }
        }

        /// Publish a fully built block under the seqlock protocol.
        pub fn publish(&mut self, mut block: PwSharedBlock) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            block.write_ms_lo = now as u32;
            block.write_ms_hi = (now >> 32) as u32;

            self.seq = self.seq.wrapping_add(2);
            let odd = self.seq | 1;
            let even = self.seq & !1;
            block.seq = odd;
            unsafe {
                // Mark "writing" (odd), fence, copy body, fence, mark "stable" (even).
                self.view.write(block);
                fence(Ordering::Release);
                std::ptr::addr_of_mut!((*self.view).seq).write_volatile(even);
            }
        }
    }

    impl Drop for ShmWriter {
        fn drop(&mut self) {
            unsafe {
                if !self.view.is_null() {
                    let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                        Value: self.view as *mut _,
                    });
                }
                if !self.mapping.is_invalid() {
                    let _ = CloseHandle(self.mapping);
                }
            }
        }
    }
}

#[cfg(not(windows))]
pub use stub_impl::ShmWriter;

#[cfg(not(windows))]
mod stub_impl {
    use super::*;

    /// Non-Windows builds have no OpenXR layer; the writer is a no-op so the
    /// rest of the app still compiles for development on other platforms.
    pub struct ShmWriter;

    impl ShmWriter {
        pub fn open() -> anyhow::Result<Self> {
            anyhow::bail!("native VR shared memory is only available on Windows")
        }
        pub fn publish(&mut self, _block: PwSharedBlock) {}
    }
}
