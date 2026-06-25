use serde::{Deserialize, Serialize};

use super::competitors::CompetitorEntry;
use super::pack::PackState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum LiveConnectionState {
    #[default]
    Disconnected,
    WaitingForSession,
    Reconnecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveStatus {
    pub state: LiveConnectionState,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveSectorProgress {
    pub sector_num: i32,
    pub time_ms: Option<f64>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub track: String,
    pub car: String,
    pub session_type: String,
    pub lap: i32,
    pub lap_time_ms: f64,
    pub last_lap_ms: Option<f64>,
    /// Whether the most recently completed lap passed validity checks.
    pub last_lap_valid: bool,
    pub best_lap_ms: Option<f64>,
    pub delta_to_best_ms: Option<f64>,
    pub delta_to_last_ms: Option<f64>,
    pub fuel_level: f32,
    pub speed: f32,
    pub lap_dist_pct: f32,
    pub current_sector: i32,
    pub sectors: Vec<LiveSectorProgress>,
    pub lf_temp: f32,
    pub rf_temp: f32,
    pub lr_temp: f32,
    pub rr_temp: f32,
    pub on_pit_road: bool,

    // Multi-driver field awareness (populated from the CarIdx telemetry stream).
    pub competitors: Vec<CompetitorEntry>,
    pub player_position: Option<i32>,
    pub player_class_position: Option<i32>,
    pub session_fastest_lap_ms: Option<f64>,
    pub delta_to_session_best_ms: Option<f64>,
    pub delta_to_session_optimal_ms: Option<f64>,
    pub gap_to_car_ahead_s: Option<f32>,
    pub gap_to_car_behind_s: Option<f32>,
    pub pack_state: PackState,

    // Session-wide state consumed by the audio coach and HUD.
    pub session_flags: u32,
    pub incident_count: i32,
    /// Session incident limit when available from session info.
    pub incident_limit: Option<i32>,
    pub session_laps_remain: Option<i32>,
    pub session_time_remain_s: Option<f64>,
    pub pits_open: bool,
    pub on_track: bool,
}
