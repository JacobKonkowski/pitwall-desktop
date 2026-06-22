use anyhow::{Context, Result};
use pitwall::{VarData, VariableInfo, VariableSchema};

use crate::analysis::RawFrame;

/// Pre-resolved variable offsets for fast per-frame extraction (no FramePacket/allocation).
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
    lf_temp: VariableInfo,
    rf_temp: VariableInfo,
    lr_temp: VariableInfo,
    rr_temp: VariableInfo,
}

impl FastFrameExtractor {
    pub fn from_schema(schema: &VariableSchema) -> Result<Self> {
        fn req(schema: &VariableSchema, name: &str) -> Result<VariableInfo> {
            schema
                .get_variable(name)
                .cloned()
                .with_context(|| format!("telemetry variable '{name}' not found in IBT"))
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
            lf_temp: req(schema, "LFtempCL")?,
            rf_temp: req(schema, "RFtempCL")?,
            lr_temp: req(schema, "LRtempCL")?,
            rr_temp: req(schema, "RRtempCL")?,
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
            lf_temp: read_f32(data, &self.lf_temp),
            rf_temp: read_f32(data, &self.rf_temp),
            lr_temp: read_f32(data, &self.lr_temp),
            rr_temp: read_f32(data, &self.rr_temp),
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
