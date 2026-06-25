use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: i64,
    pub ibt_path: String,
    pub track: String,
    pub car: String,
    pub session_date: String,
    pub lap_count: i32,
    pub best_lap_ms: Option<f64>,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorTime {
    pub sector_num: i32,
    pub time_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LapKind {
    #[default]
    Flying,
    PitOut,
    PitIn,
    PitLane,
    Partial,
}

impl LapKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LapKind::Flying => "flying",
            LapKind::PitOut => "pit_out",
            LapKind::PitIn => "pit_in",
            LapKind::PitLane => "pit_lane",
            LapKind::Partial => "partial",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "flying" => Some(LapKind::Flying),
            "pit_out" => Some(LapKind::PitOut),
            "pit_in" => Some(LapKind::PitIn),
            "pit_lane" => Some(LapKind::PitLane),
            "partial" => Some(LapKind::Partial),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapSummary {
    pub id: i64,
    pub session_num: i32,
    pub session_type: String,
    pub iracing_lap: i32,
    pub lap_number: i32,
    pub lap_time_ms: Option<f64>,
    pub valid: bool,
    pub lap_kind: LapKind,
    pub fuel_start: Option<f64>,
    pub fuel_used: Option<f64>,
    pub avg_speed: Option<f64>,
    pub lf_temp: Option<f64>,
    pub rf_temp: Option<f64>,
    pub lr_temp: Option<f64>,
    pub rr_temp: Option<f64>,
    pub sectors: Vec<SectorTime>,
    pub delta_to_best_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session: SessionSummary,
    pub laps: Vec<LapSummary>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapTrace {
    pub lap_id: i64,
    pub lap_number: i32,
    pub points: Vec<TracePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuelLapSummary {
    pub lap_number: i32,
    pub fuel_used: f64,
    pub fuel_remaining: f64,
    pub laps_remaining_estimate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuelSummary {
    pub laps: Vec<FuelLapSummary>,
    pub tank_capacity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TireLapSummary {
    pub lap_number: i32,
    pub lf_temp: f64,
    pub rf_temp: f64,
    pub lr_temp: f64,
    pub rr_temp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TireSummary {
    pub laps: Vec<TireLapSummary>,
    pub note: String,
}

/// One competitor row captured in a post-session standings snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitorStanding {
    pub position: i32,
    pub class_position: i32,
    pub car_number: String,
    pub driver_name: String,
    pub class_id: i32,
    pub class_color: String,
    pub best_lap_ms: Option<f64>,
    pub is_player: bool,
}

/// Snapshot of the live leaderboard at the moment a live session ended, linked
/// to an imported IBT session when one can be matched.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStandings {
    pub id: i64,
    pub session_id: Option<i64>,
    pub track: String,
    pub session_type: String,
    pub session_date: String,
    pub session_fastest_ms: Option<f64>,
    pub player_best_ms: Option<f64>,
    pub player_position: Option<i32>,
    pub player_class_position: Option<i32>,
    pub competitors: Vec<CompetitorStanding>,
    pub traffic_laps: Vec<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStatus {
    pub active: bool,
    pub current_file: Option<String>,
    pub progress_pct: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IracingConfigCheck {
    pub app_ini_path: String,
    pub telemetry_dir: String,
    pub mem_enabled: bool,
    pub disk_enabled: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoredLap {
    /// iRacing sub-session index (practice / qual / race).
    pub session_num: i32,
    /// Human label, e.g. "Practice", "Open Qualifying", "Race".
    pub session_type: String,
    /// Raw lap counter from telemetry (resets each sub-session; lap 0 = out lap).
    pub iracing_lap: i32,
    /// Lap number within this sub-session (1-based).
    pub lap_number: i32,
    pub lap_time_ms: Option<f64>,
    pub valid: bool,
    pub lap_kind: LapKind,
    pub fuel_start: Option<f64>,
    pub fuel_used: Option<f64>,
    pub avg_speed: Option<f64>,
    pub lf_temp: Option<f64>,
    pub rf_temp: Option<f64>,
    pub lr_temp: Option<f64>,
    pub rr_temp: Option<f64>,
    pub sectors: Vec<(i32, f64)>,
    pub traces: Vec<TracePoint>,
}
