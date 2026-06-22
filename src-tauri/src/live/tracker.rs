use pitwall::SessionInfo;

use crate::analysis::types::SectorBoundary;
use crate::ingest::frame::AnalysisFrame;

use super::snapshot::{LiveSectorProgress, LiveSnapshot};

pub struct LiveTracker {
    track: String,
    car: String,
    session_type: String,
    current_lap: i32,
    lap_start_time: f64,
    last_lap_ms: Option<f64>,
    best_lap_ms: Option<f64>,
    sector_start_time: f64,
    completed_sectors: Vec<(i32, f64)>,
    last_bounds_len: usize,
}

impl LiveTracker {
    pub fn new() -> Self {
        Self {
            track: String::new(),
            car: String::new(),
            session_type: String::new(),
            current_lap: 0,
            lap_start_time: 0.0,
            last_lap_ms: None,
            best_lap_ms: None,
            sector_start_time: 0.0,
            completed_sectors: Vec::new(),
            last_bounds_len: 0,
        }
    }

    pub fn set_session_meta(&mut self, session: &SessionInfo) {
        self.track = if session.weekend_info.track_display_name.is_empty() {
            session.weekend_info.track_name.clone()
        } else {
            session.weekend_info.track_display_name.clone()
        };
        self.car = extract_car_name(session);
        let idx = session.session_info.current_session_num;
        self.session_type = session
            .session_info
            .sessions
            .iter()
            .find(|s| s.session_num == idx)
            .and_then(|s| {
                s.session_name
                    .as_ref()
                    .filter(|n| !n.is_empty())
                    .cloned()
                    .or_else(|| Some(s.session_type.clone()))
            })
            .unwrap_or_else(|| "Session".into());
    }

    pub fn update(&mut self, frame: &AnalysisFrame, bounds: &[SectorBoundary]) {
        if frame.lap != self.current_lap {
            if self.current_lap > 0 && frame.lap > self.current_lap {
                let lap_ms = (frame.session_time - self.lap_start_time) * 1000.0;
                if lap_ms > 10_000.0 {
                    self.last_lap_ms = Some(lap_ms);
                    self.best_lap_ms = Some(
                        self.best_lap_ms
                            .map(|b| b.min(lap_ms))
                            .unwrap_or(lap_ms),
                    );
                }
            }
            self.current_lap = frame.lap;
            self.lap_start_time = frame.session_time;
            self.sector_start_time = frame.session_time;
            self.completed_sectors.clear();
        }

        // Detect sector boundary crossings within current lap
        if bounds.len() != self.last_bounds_len {
            self.completed_sectors.clear();
            self.sector_start_time = frame.session_time;
            self.last_bounds_len = bounds.len();
        }

        for boundary in bounds {
            if self.completed_sectors.iter().any(|(n, _)| *n == boundary.sector_num) {
                continue;
            }
            if frame.lap_dist_pct as f64 >= boundary.start_pct {
                let sector_ms = (frame.session_time - self.sector_start_time) * 1000.0;
                self.completed_sectors.push((boundary.sector_num, sector_ms));
                self.sector_start_time = frame.session_time;
            }
        }
    }

    pub fn snapshot_from_frame(&mut self, frame: &AnalysisFrame, bounds: &[SectorBoundary]) -> LiveSnapshot {
        self.update(frame, bounds);
        let lap_time_ms = (frame.session_time - self.lap_start_time).max(0.0) * 1000.0;
        let delta_to_best = self
            .best_lap_ms
            .filter(|b| *b > 0.0 && lap_time_ms > 0.0)
            .map(|b| lap_time_ms - b);
        let delta_to_last = self
            .last_lap_ms
            .filter(|l| *l > 0.0 && lap_time_ms > 0.0)
            .map(|l| lap_time_ms - l);

        let current_sector = self.completed_sectors.len() as i32 + 1;

        LiveSnapshot {
            track: self.track.clone(),
            car: self.car.clone(),
            session_type: self.session_type.clone(),
            lap: self.current_lap,
            lap_time_ms,
            last_lap_ms: self.last_lap_ms,
            best_lap_ms: self.best_lap_ms,
            delta_to_best_ms: delta_to_best,
            delta_to_last_ms: delta_to_last,
            fuel_level: frame.fuel_level,
            speed: frame.speed,
            lap_dist_pct: frame.lap_dist_pct,
            current_sector: current_sector.min(3),
            sectors: (1..=3)
                .map(|n| {
                    let done = self.completed_sectors.iter().find(|(s, _)| *s == n);
                    LiveSectorProgress {
                        sector_num: n,
                        time_ms: done.map(|(_, ms)| *ms),
                        completed: done.is_some(),
                    }
                })
                .collect(),
            lf_temp: frame.lf_temp,
            rf_temp: frame.rf_temp,
            lr_temp: frame.lr_temp,
            rr_temp: frame.rr_temp,
        }
    }
}

fn extract_car_name(session: &SessionInfo) -> String {
    if let Some(driver_info) = &session.driver_info {
        let car_idx = driver_info.driver_car_idx.unwrap_or(-1);
        if let Some(drivers) = &driver_info.drivers {
            for driver in drivers {
                if driver.car_idx == car_idx {
                    return driver
                        .car_screen_name
                        .clone()
                        .or_else(|| driver.car_path.clone())
                        .unwrap_or_else(|| "Unknown Car".into());
                }
            }
        }
    }
    "Unknown Car".into()
}
