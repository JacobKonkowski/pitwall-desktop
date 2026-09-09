use pitwall::PitwallFrame;

/// Player-only live telemetry frame (10 Hz). Extra channels keep the SDK
/// subscription aligned with IBT layout even when unused by the tracker.
#[derive(Debug, Clone, PitwallFrame)]
#[allow(dead_code)]
pub struct AnalysisFrame {
    #[field_name = "SessionNum"]
    pub session_num: i32,
    #[field_name = "Lap"]
    pub lap: i32,
    #[field_name = "LapDistPct"]
    pub lap_dist_pct: f32,
    #[field_name = "Speed"]
    pub speed: f32,
    #[field_name = "Throttle"]
    pub throttle: f32,
    #[field_name = "Brake"]
    pub brake: f32,
    #[field_name = "SteeringWheelAngle"]
    pub steering: f32,
    #[field_name = "Gear"]
    pub gear: i32,
    #[field_name = "FuelLevel"]
    pub fuel_level: f32,
    #[field_name = "OnPitRoad"]
    pub on_pit_road: bool,
    #[field_name = "SessionTime"]
    pub session_time: f64,
    #[field_name = "LFtempM"]
    pub lf_temp: f32,
    #[field_name = "RFtempM"]
    pub rf_temp: f32,
    #[field_name = "LRtempM"]
    pub lr_temp: f32,
    #[field_name = "RRtempM"]
    pub rr_temp: f32,
}
