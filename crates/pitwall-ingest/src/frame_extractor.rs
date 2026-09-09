//! Pre-resolved variable offsets for fast per-frame extraction (no allocation).

use anyhow::{Context, Result};
use pitwall::{VarData, VariableInfo, VariableSchema};

use pitwall_analysis::RawFrame;

pub struct FastFrameExtractor {
    session_num: Option<VariableInfo>,
    lap: VariableInfo,
    lap_dist_pct: VariableInfo,
    speed: VariableInfo,
    throttle: VariableInfo,
    brake: VariableInfo,
    steering: VariableInfo,
    gear: VariableInfo,
    fuel_level: VariableInfo,
    on_pit_road: VariableInfo,
    session_time: VariableInfo,
    // Optional: present in modern IBT files, absent in older ones.
    lap_last_lap_time: Option<VariableInfo>,
    delta_best_ok: Option<VariableInfo>,
    delta_session_best_ok: Option<VariableInfo>,
    lf_temp: Option<VariableInfo>,
    rf_temp: Option<VariableInfo>,
    lr_temp: Option<VariableInfo>,
    rr_temp: Option<VariableInfo>,
}

impl FastFrameExtractor {
    pub fn from_schema(schema: &VariableSchema) -> Result<Self> {
        fn req(schema: &VariableSchema, name: &str) -> Result<VariableInfo> {
            schema
                .get_variable(name)
                .cloned()
                .with_context(|| format!("telemetry variable '{name}' not found in IBT"))
        }

        fn tire_temp(schema: &VariableSchema, mid: &str, carcass: &str) -> Option<VariableInfo> {
            schema
                .get_variable(mid)
                .or_else(|| schema.get_variable(carcass))
                .cloned()
        }

        Ok(Self {
            session_num: schema.get_variable("SessionNum").cloned(),
            lap: req(schema, "Lap")?,
            lap_dist_pct: req(schema, "LapDistPct")?,
            speed: req(schema, "Speed")?,
            throttle: req(schema, "Throttle")?,
            brake: req(schema, "Brake")?,
            steering: req(schema, "SteeringWheelAngle")?,
            gear: req(schema, "Gear")?,
            fuel_level: req(schema, "FuelLevel")?,
            on_pit_road: req(schema, "OnPitRoad")?,
            session_time: req(schema, "SessionTime")?,
            lap_last_lap_time: schema.get_variable("LapLastLapTime").cloned(),
            delta_best_ok: schema.get_variable("LapDeltaToBestLap_OK").cloned(),
            delta_session_best_ok: schema.get_variable("LapDeltaToSessionBestLap_OK").cloned(),
            lf_temp: tire_temp(schema, "LFtempM", "LFtempCM"),
            rf_temp: tire_temp(schema, "RFtempM", "RFtempCM"),
            lr_temp: tire_temp(schema, "LRtempM", "LRtempCM"),
            rr_temp: tire_temp(schema, "RRtempM", "RRtempCM"),
        })
    }

    #[inline]
    pub fn extract(&self, data: &[u8]) -> RawFrame {
        RawFrame {
            session_num: self
                .session_num
                .as_ref()
                .map(|v| read_i32(data, v))
                .unwrap_or(0),
            lap: read_i32(data, &self.lap),
            lap_dist_pct: read_f32(data, &self.lap_dist_pct),
            speed: read_f32(data, &self.speed),
            throttle: read_f32(data, &self.throttle),
            brake: read_f32(data, &self.brake),
            steering: read_f32(data, &self.steering),
            gear: read_i32(data, &self.gear),
            fuel_level: read_f32(data, &self.fuel_level),
            on_pit_road: read_bool(data, &self.on_pit_road),
            session_time: read_f64(data, &self.session_time),
            lap_last_lap_time: self.lap_last_lap_time.as_ref().map(|v| read_f32(data, v)),
            delta_best_ok: self.delta_best_ok.as_ref().map(|v| read_bool(data, v)),
            delta_session_best_ok: self
                .delta_session_best_ok
                .as_ref()
                .map(|v| read_bool(data, v)),
            lf_temp: self.lf_temp.as_ref().map(|v| read_f32(data, v)).unwrap_or(0.0),
            rf_temp: self.rf_temp.as_ref().map(|v| read_f32(data, v)).unwrap_or(0.0),
            lr_temp: self.lr_temp.as_ref().map(|v| read_f32(data, v)).unwrap_or(0.0),
            rr_temp: self.rr_temp.as_ref().map(|v| read_f32(data, v)).unwrap_or(0.0),
        }
    }
}

#[inline]
fn read_f32(data: &[u8], info: &VariableInfo) -> f32 {
    f32::from_bytes(data, info).unwrap_or(0.0)
}

#[inline]
fn read_f64(data: &[u8], info: &VariableInfo) -> f64 {
    f64::from_bytes(data, info).unwrap_or(0.0)
}

#[inline]
fn read_i32(data: &[u8], info: &VariableInfo) -> i32 {
    i32::from_bytes(data, info).unwrap_or(0)
}

#[inline]
fn read_bool(data: &[u8], info: &VariableInfo) -> bool {
    bool::from_bytes(data, info).unwrap_or(false)
}
