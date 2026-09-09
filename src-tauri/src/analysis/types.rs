//! Analysis-owned domain types.
//!
//! An [`AnalyzedLap`] carries only iRacing-native facts plus values computed by
//! math on those facts (sectors, trace samples, aggregates). There is no
//! invented lap taxonomy and no heuristic validity flag.

use serde::{Deserialize, Serialize};

pub use crate::telemetry::{RawFrame, SectorBoundary, SessionMeta};

/// A downsampled telemetry point kept for charts and comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracePoint {
    pub dist_pct: f64,
    pub speed: f64,
    pub throttle: f64,
    pub brake: f64,
    pub gear: i32,
    pub steering: f64,
}

/// Frames grouped into a single lap, with the SDK values sampled at the
/// transition into the following lap.
#[derive(Debug, Clone)]
pub struct LapFrames {
    pub session_num: i32,
    pub session_type: String,
    pub iracing_lap: i32,
    /// 1-based index within this sub-session, assigned after segmentation.
    pub lap_number: i32,
    /// `LapLastLapTime` (ms) sampled on the first frame of the next lap.
    pub sdk_lap_time_ms: Option<f64>,
    /// `LapDeltaToBestLap_OK` sampled at that same transition.
    pub delta_best_ok: Option<bool>,
    /// `LapDeltaToSessionBestLap_OK` at the transition.
    pub delta_session_best_ok: Option<bool>,
    pub frames: Vec<RawFrame>,
}

/// A fully analyzed lap: SDK facts + computed products. No `valid`/`lap_kind`.
#[derive(Debug, Clone)]
pub struct AnalyzedLap {
    pub session_num: i32,
    pub session_type: String,
    pub iracing_lap: i32,
    pub lap_number: i32,
    /// Official lap time (`LapLastLapTime`); `None` when the sim did not report one
    /// (e.g. the final, unfinished lap).
    pub lap_time_ms: Option<f64>,
    pub delta_best_ok: Option<bool>,
    pub delta_session_best_ok: Option<bool>,
    /// `OnPitRoad` on the first frame of the lap.
    pub on_pit_road_start: bool,
    /// `OnPitRoad` on the last frame of the lap.
    pub on_pit_road_end: bool,
    pub lap_dist_pct_min: f32,
    pub lap_dist_pct_max: f32,
    pub fuel_start: Option<f64>,
    pub fuel_used: Option<f64>,
    pub avg_speed: Option<f64>,
    pub lf_temp: Option<f64>,
    pub rf_temp: Option<f64>,
    pub lr_temp: Option<f64>,
    pub rr_temp: Option<f64>,
    /// `(sector_num, time_ms)` pairs.
    pub sectors: Vec<(i32, f64)>,
    pub traces: Vec<TracePoint>,
}

impl AnalyzedLap {
    /// Whether this lap counts for the session best / default compare reference.
    ///
    /// Requires an official time, both sim `_OK` flags, and near-full distance
    /// coverage so incomplete/reset fragments never become the "best" lap.
    pub fn pace_eligible(&self) -> bool {
        super::cleanup::pace_eligible_from(
            self.lap_time_ms,
            self.delta_best_ok,
            self.delta_session_best_ok,
            self.lap_dist_pct_max,
        )
    }

    pub fn is_phantom(&self) -> bool {
        super::cleanup::is_phantom_lap(self.iracing_lap, self.lap_dist_pct_max)
    }
}

/// Result of analyzing a whole IBT file.
#[derive(Debug, Clone)]
pub struct AnalyzedSession {
    pub track: String,
    pub car: String,
    pub session_date: String,
    pub laps: Vec<AnalyzedLap>,
}

impl AnalyzedSession {
    /// Fastest pace-eligible lap time across all sub-sessions, if any.
    pub fn best_lap_ms(&self) -> Option<f64> {
        self.laps
            .iter()
            .filter(|l| l.pace_eligible())
            .filter_map(|l| l.lap_time_ms)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }
}
