//! In-headset HUD for iRacing VR.
//!
//! Two modes share one service:
//! * **Native** (default) — writes the live snapshot into shared memory for the
//!   `pitwall-openxr-layer` DLL, which composites quads directly in the headset.
//!   No OpenKneeboard or RaceLab required.
//! * **Web** (fallback) — serves the HUD over HTTP for an OpenKneeboard Web
//!   Dashboard tab.

mod hud_server;
mod layer_install;
mod shm;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;

use crate::live::LiveService;
use crate::settings::AppSettings;

pub use hud_server::{check_hud_health, hud_url, open_hud_preview, HUD_PORT};
pub use layer_install::{install_layer, is_layer_installed, uninstall_layer, MANIFEST_FILE};

pub struct VrOverlayService {
    cancel: Mutex<Option<CancellationToken>>,
    status: Mutex<VrOverlayStatus>,
    /// Wall-clock ms of the most recent native frame published to shared memory.
    last_frame_ms: Mutex<Option<u64>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VrOverlayStatus {
    pub active: bool,
    pub runtime: String,
    pub message: String,
    pub hud_url: String,
    /// "native" or "web".
    pub mode: String,
    /// Whether the OpenXR API layer is registered with the loader.
    pub layer_installed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NativeVrStatus {
    pub active: bool,
    pub layer_installed: bool,
    pub compositor_active: bool,
    /// Age of the last published frame in ms (None if nothing published yet).
    pub last_frame_age_ms: Option<u64>,
}

impl VrOverlayService {
    pub fn new() -> Self {
        Self {
            cancel: Mutex::new(None),
            status: Mutex::new(VrOverlayStatus {
                hud_url: hud_url(),
                mode: "native".into(),
                ..Default::default()
            }),
            last_frame_ms: Mutex::new(None),
        }
    }

    pub fn is_active(&self) -> bool {
        self.cancel.lock().is_some()
    }

    pub fn status(&self) -> VrOverlayStatus {
        let mut s = self.status.lock().clone();
        s.hud_url = hud_url();
        s.layer_installed = layer_install::is_layer_installed();
        s
    }

    pub fn native_status(&self) -> NativeVrStatus {
        let now = now_ms();
        let last = *self.last_frame_ms.lock();
        NativeVrStatus {
            active: self.is_active(),
            layer_installed: layer_install::is_layer_installed(),
            compositor_active: last.map(|t| now.saturating_sub(t) < 2000).unwrap_or(false),
            last_frame_age_ms: last.map(|t| now.saturating_sub(t)),
        }
    }

    pub fn stop(&self) {
        if let Some(token) = self.cancel.lock().take() {
            token.cancel();
        }
        *self.last_frame_ms.lock() = None;
        let mode = self.status.lock().mode.clone();
        *self.status.lock() = VrOverlayStatus {
            active: false,
            runtime: String::new(),
            message: "In-headset HUD stopped".into(),
            hud_url: hud_url(),
            mode,
            layer_installed: layer_install::is_layer_installed(),
        };
    }

    /// Start the in-headset HUD in the mode selected by `settings.vr_mode`.
    pub fn start(self: &Arc<Self>, live: Arc<LiveService>, settings: AppSettings) {
        if self.is_active() {
            return;
        }

        let native = settings.vr_mode != "web";
        let token = CancellationToken::new();
        *self.cancel.lock() = Some(token.clone());

        if native {
            self.start_native(live, settings, token);
        } else {
            self.start_web(live, token);
        }
    }

    fn start_web(self: &Arc<Self>, live: Arc<LiveService>, token: CancellationToken) {
        *self.status.lock() = VrOverlayStatus {
            active: true,
            runtime: "OpenXR (Web HUD)".into(),
            message: "Starting in-headset HUD server…".into(),
            hud_url: hud_url(),
            mode: "web".into(),
            layer_installed: layer_install::is_layer_installed(),
        };
        let service = Arc::clone(self);
        thread::spawn(move || {
            if let Err(e) = hud_server::run_hud_server(service.clone(), live, token) {
                service.status.lock().message = format!("HUD server error: {e:#}");
                service.status.lock().active = false;
            }
            *service.cancel.lock() = None;
        });
    }

    fn start_native(
        self: &Arc<Self>,
        live: Arc<LiveService>,
        settings: AppSettings,
        token: CancellationToken,
    ) {
        let installed = layer_install::is_layer_installed();
        *self.status.lock() = VrOverlayStatus {
            active: true,
            runtime: "OpenXR (native layer)".into(),
            message: if installed {
                "Publishing HUD to in-headset compositor".into()
            } else {
                "VR layer not installed — install it, then restart iRacing".into()
            },
            hud_url: hud_url(),
            mode: "native".into(),
            layer_installed: installed,
        };

        let service = Arc::clone(self);
        thread::spawn(move || {
            if let Err(e) = run_native_loop(service.clone(), live, settings, token) {
                service.status.lock().message = format!("Native VR error: {e:#}");
                service.status.lock().active = false;
            }
            *service.cancel.lock() = None;
        });
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn field_pace_ordinal(mode: &str) -> u32 {
    match mode {
        "optimal" => shm::FIELD_PACE_OPTIMAL,
        "both" => shm::FIELD_PACE_BOTH,
        _ => shm::FIELD_PACE_BEST,
    }
}

/// Publish the live snapshot to shared memory at ~30 Hz until cancelled. The
/// placement is re-read from settings each tick so the in-app sliders move the
/// HUD live.
fn run_native_loop(
    service: Arc<VrOverlayService>,
    live: Arc<LiveService>,
    initial: AppSettings,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let mut writer = shm::ShmWriter::open()?;
    let _ = initial; // settings are re-read each tick below

    while !cancel.is_cancelled() {
        let settings = crate::settings::load_settings();
        let snap = live.snapshot.lock().clone();
        let layout = &settings.overlay_layout;
        let mut slots = [shm::SlotPlacement {
            enabled: false,
            vertical_offset: 0.0,
            scale: 1.0,
            opacity: 1.0,
        }; shm::MAX_OVERLAYS];
        for (i, w) in layout.widgets.iter().enumerate() {
            slots[i] = shm::SlotPlacement {
                enabled: w.enabled,
                vertical_offset: w.vr_offset_y,
                scale: w.vr_scale.max(0.1),
                opacity: w.vr_opacity.clamp(0.0, 1.0),
            };
        }
        let field_pace = field_pace_ordinal(&layout.field_pace_mode);
        writer.publish(shm::build_block(&snap, &slots, field_pace));
        *service.last_frame_ms.lock() = Some(now_ms());

        thread::sleep(Duration::from_millis(33));
    }

    *service.last_frame_ms.lock() = None;
    Ok(())
}
