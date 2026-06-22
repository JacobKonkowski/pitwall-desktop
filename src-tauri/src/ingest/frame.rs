use pitwall::PitwallFrame;

#[derive(Debug, Clone, PitwallFrame)]
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
    #[field_name = "LFtempCL"]
    pub lf_temp: f32,
    #[field_name = "RFtempCL"]
    pub rf_temp: f32,
    #[field_name = "LRtempCL"]
    pub lr_temp: f32,
    #[field_name = "RRtempCL"]
    pub rr_temp: f32,
}

impl From<AnalysisFrame> for crate::analysis::RawFrame {
    fn from(f: AnalysisFrame) -> Self {
        Self {
            session_num: f.session_num,
            lap: f.lap,
            lap_dist_pct: f.lap_dist_pct,
            speed: f.speed,
            throttle: f.throttle,
            brake: f.brake,
            steering: f.steering,
            gear: f.gear,
            fuel_level: f.fuel_level,
            on_pit_road: f.on_pit_road,
            session_time: f.session_time,
            lf_temp: f.lf_temp,
            rf_temp: f.rf_temp,
            lr_temp: f.lr_temp,
            rr_temp: f.rr_temp,
        }
    }
}
