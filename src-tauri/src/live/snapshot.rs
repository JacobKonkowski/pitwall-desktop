use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum LiveConnectionState {
    #[default]
    Disconnected,
    WaitingForSession,
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
}
