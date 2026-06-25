#[derive(Debug, Clone)]
pub struct RawFrame {
    /// iRacing sub-session index (practice / qual / race each have their own number).
    pub session_num: i32,
    pub lap: i32,
    pub lap_dist_pct: f32,
    pub speed: f32,
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub gear: i32,
    pub fuel_level: f32,
    pub on_pit_road: bool,
    pub session_time: f64,
    pub lf_temp: f32,
    pub rf_temp: f32,
    pub lr_temp: f32,
    pub rr_temp: f32,
}

#[derive(Debug, Clone)]
pub struct LapFrames {
    pub session_num: i32,
    pub session_type: String,
    pub iracing_lap: i32,
    /// Lap index within this sub-session (1-based, assigned after segmentation).
    pub lap_number: i32,
    pub frames: Vec<RawFrame>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectorBoundary {
    pub sector_num: i32,
    pub start_pct: f64,
}
