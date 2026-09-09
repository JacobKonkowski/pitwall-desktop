use crate::live::LiveSnapshot;

use super::super::session_mode::SessionMode;

/// Session facts stable for the current track + session type.
#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub fuel_capacity: Option<f32>,
    pub tyre_compound: Option<String>,
    pub player_car_idx: i32,
    pub player_irating: Option<i32>,
    pub player_licence: Option<String>,
}

/// Read-only view passed to race-engine rules each tick.
pub struct RaceContext<'a> {
    pub snap: &'a LiveSnapshot,
    pub session_mode: SessionMode,
    pub meta: &'a SessionMeta,
    pub fuel_per_lap: &'a [f32],
}

impl<'a> RaceContext<'a> {
    pub fn new(snap: &'a LiveSnapshot, meta: &'a SessionMeta, fuel_per_lap: &'a [f32]) -> Self {
        Self {
            session_mode: SessionMode::from_session_type(&snap.session_type),
            snap,
            meta,
            fuel_per_lap,
        }
    }
}
