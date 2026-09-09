//! Shared telemetry domain types.
//!
//! These mirror iRacing SDK channels one-to-one. They contain **no** invented
//! taxonomy (no "lap kind", no heuristic "validity"): only raw samples plus the
//! `_OK` flags the sim itself publishes. Both IBT ingest and any future live
//! source produce these same types, so the analysis layer never needs to know
//! where the frames came from.
//!
//! This module performs no I/O and has no Tauri dependency.

/// One telemetry sample. Field names map directly to iRacing SDK channels.
#[derive(Debug, Clone)]
pub struct RawFrame {
    /// `SessionNum` — iRacing sub-session index (practice / qualify / race each
    /// have their own number).
    pub session_num: i32,
    /// `Lap` — lap counter within the sub-session.
    pub lap: i32,
    /// `LapDistPct` — fraction around the lap [0, 1).
    pub lap_dist_pct: f32,
    /// `Speed` — m/s.
    pub speed: f32,
    /// `Throttle` — 0..1.
    pub throttle: f32,
    /// `Brake` — 0..1.
    pub brake: f32,
    /// `SteeringWheelAngle` — radians.
    pub steering: f32,
    /// `Gear`.
    pub gear: i32,
    /// `FuelLevel` — liters.
    pub fuel_level: f32,
    /// `OnPitRoad` — between the pit cones.
    pub on_pit_road: bool,
    /// `SessionTime` — seconds since session start. Used only for sector-crossing
    /// interpolation, never as a lap time.
    pub session_time: f64,
    /// `LapLastLapTime` — official time of the just-completed lap, in seconds
    /// (negative sentinel = unset). Present on the first frame(s) of the next lap.
    pub lap_last_lap_time: Option<f32>,
    /// `LapDeltaToBestLap_OK` — the sim's own "this lap's delta is valid" flag.
    /// `None` when the channel is absent from the source (e.g. older IBT files).
    pub delta_best_ok: Option<bool>,
    /// `LapDeltaToSessionBestLap_OK`.
    pub delta_session_best_ok: Option<bool>,
    /// `LFtempM` — left-front middle surface tire temp (°C).
    pub lf_temp: f32,
    /// `RFtempM`.
    pub rf_temp: f32,
    /// `LRtempM`.
    pub lr_temp: f32,
    /// `RRtempM`.
    pub rr_temp: f32,
}

/// A sector split line from the session YAML `SplitTimeInfo.Sectors[]`.
/// `start_pct` is where the region *begins* (sector 0 sits at 0%).
#[derive(Debug, Clone, PartialEq)]
pub struct SectorBoundary {
    pub sector_num: i32,
    pub start_pct: f64,
}

/// Session-level metadata resolved from the IBT session YAML.
#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub track: String,
    pub car: String,
    pub session_date: String,
    pub sector_boundaries: Vec<SectorBoundary>,
    /// Map of `SessionNum` -> human label (e.g. "Practice", "Race").
    pub session_labels: std::collections::HashMap<i32, String>,
}
