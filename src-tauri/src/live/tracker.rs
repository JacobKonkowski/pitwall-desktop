use pitwall::SessionInfo;

use crate::analysis::lap_segmenter::is_valid_lap_metrics;
use crate::analysis::sector_splitter::normalize_sector_boundaries;
use crate::analysis::types::SectorBoundary;
use crate::ingest::frame::AnalysisFrame;

use super::competitors::{build_roster, RosterEntry};
use super::snapshot::{LiveSectorProgress, LiveSnapshot};

const MIN_SECTOR_MS: f64 = 1000.0;

#[derive(Default)]
struct LapAccum {
    frames: u32,
    pit_frames: u32,
    min_dist_pct: f32,
    max_dist_pct: f32,
}

pub struct LiveTracker {
    track: String,
    car: String,
    session_type: String,
    current_lap: i32,
    lap_start_time: f64,
    last_lap_ms: Option<f64>,
    last_lap_valid: bool,
    best_lap_ms: Option<f64>,
    sector_start_time: f64,
    completed_sectors: Vec<(i32, f64)>,
    normalized_bounds: Vec<SectorBoundary>,
    next_boundary_idx: usize,
    prev_lap_dist_pct: Option<f32>,
    prev_session_time: Option<f64>,
    /// Sectors from the lap that just ended; exposed once on the first snapshot after a lap change.
    pending_lap_sectors: Option<Vec<(i32, f64)>>,
    roster: Vec<RosterEntry>,
    player_car_idx: i32,
    lap_accum: LapAccum,
    /// Latest iRacing `LapDeltaTo*_OK` flags; sampled each frame and read at lap finish.
    latest_iracing_lap_ok: bool,
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
            last_lap_valid: false,
            best_lap_ms: None,
            sector_start_time: 0.0,
            completed_sectors: Vec::new(),
            normalized_bounds: Vec::new(),
            next_boundary_idx: 0,
            prev_lap_dist_pct: None,
            prev_session_time: None,
            pending_lap_sectors: None,
            roster: Vec::new(),
            player_car_idx: -1,
            lap_accum: LapAccum::default(),
            latest_iracing_lap_ok: true,
        }
    }

    pub fn track(&self) -> &str {
        &self.track
    }

    pub fn roster(&self) -> &[RosterEntry] {
        &self.roster
    }

    pub fn player_car_idx(&self) -> i32 {
        self.player_car_idx
    }

    /// Update the latest iRacing lap-validity signal from the CarIdx stream.
    pub fn note_iracing_lap_ok(&mut self, ok: bool) {
        self.latest_iracing_lap_ok = ok;
    }

    /// Clear per-session lap and sector state. Used when the track changes
    /// (driver moved to a new session) so deltas and bests don't carry over.
    pub fn reset_session(&mut self) {
        self.current_lap = 0;
        self.lap_start_time = 0.0;
        self.last_lap_ms = None;
        self.last_lap_valid = false;
        self.best_lap_ms = None;
        self.sector_start_time = 0.0;
        self.completed_sectors.clear();
        self.normalized_bounds.clear();
        self.next_boundary_idx = 0;
        self.prev_lap_dist_pct = None;
        self.prev_session_time = None;
        self.pending_lap_sectors = None;
        self.lap_accum = LapAccum::default();
        self.latest_iracing_lap_ok = true;
    }

    pub fn set_session_meta(&mut self, session: &SessionInfo) {
        self.roster = build_roster(session);
        if let Some(driver_info) = &session.driver_info {
            if let Some(idx) = driver_info.driver_car_idx {
                self.player_car_idx = idx;
            }
        }
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
        self.sync_bounds(bounds, frame.lap_dist_pct);

        if frame.lap != self.current_lap {
            if self.current_lap > 0 && frame.lap > self.current_lap {
                self.record_final_sector(frame.session_time);
                self.pending_lap_sectors = Some(self.completed_sectors.clone());
                let lap_ms = (frame.session_time - self.lap_start_time) * 1000.0;
                let valid = is_valid_lap_metrics(
                    self.lap_accum.frames as usize,
                    self.lap_accum.pit_frames as usize,
                    Some(lap_ms),
                    self.lap_accum.min_dist_pct,
                    self.lap_accum.max_dist_pct,
                ) && self.latest_iracing_lap_ok;

                self.last_lap_valid = valid;
                if valid && lap_ms > 10_000.0 {
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
            self.lap_accum = LapAccum::default();
            self.skip_passed_boundaries(frame.lap_dist_pct);
            self.prev_lap_dist_pct = Some(frame.lap_dist_pct);
            self.prev_session_time = Some(frame.session_time);
            self.record_lap_frame(frame);
            return;
        }

        self.record_lap_frame(frame);

        if let Some(prev_pct) = self.prev_lap_dist_pct {
            let prev_time = self.prev_session_time.unwrap_or(frame.session_time);
            self.detect_sector_crossings(prev_pct, prev_time, frame.lap_dist_pct, frame.session_time);
        }
        self.prev_lap_dist_pct = Some(frame.lap_dist_pct);
        self.prev_session_time = Some(frame.session_time);
    }

    fn sync_bounds(&mut self, bounds: &[SectorBoundary], lap_dist_pct: f32) {
        let normalized = normalize_sector_boundaries(bounds);
        if normalized != self.normalized_bounds {
            self.normalized_bounds = normalized;
            self.next_boundary_idx = 0;
            self.skip_passed_boundaries(lap_dist_pct);
        }
    }

    fn skip_passed_boundaries(&mut self, lap_dist_pct: f32) {
        while self.next_boundary_idx < self.normalized_bounds.len() {
            let pct = self.normalized_bounds[self.next_boundary_idx].start_pct;
            if lap_dist_pct as f64 >= pct {
                self.next_boundary_idx += 1;
            } else {
                break;
            }
        }
    }

    fn detect_sector_crossings(
        &mut self,
        prev_pct: f32,
        prev_time: f64,
        curr_pct: f32,
        session_time: f64,
    ) {
        while self.next_boundary_idx < self.normalized_bounds.len() {
            let boundary = &self.normalized_bounds[self.next_boundary_idx];
            let pct = boundary.start_pct;
            let crossed = prev_pct as f64 <= pct && curr_pct as f64 > pct;
            if !crossed {
                break;
            }
            let ratio = if (curr_pct - prev_pct).abs() > f32::EPSILON {
                ((pct - prev_pct as f64) / (curr_pct - prev_pct) as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let cross_time = prev_time + (session_time - prev_time) * ratio;
            let sector_ms = (cross_time - self.sector_start_time) * 1000.0;
            if sector_ms >= MIN_SECTOR_MS {
                self.completed_sectors.push((boundary.sector_num, sector_ms));
            }
            self.sector_start_time = cross_time;
            self.next_boundary_idx += 1;
        }
    }

    fn record_final_sector(&mut self, lap_end_time: f64) {
        if self.normalized_bounds.is_empty() {
            return;
        }
        let final_sector = self
            .normalized_bounds
            .last()
            .map(|b| b.sector_num + 1)
            .unwrap_or(1);
        if self.completed_sectors.iter().any(|(n, _)| *n == final_sector) {
            return;
        }
        let sector_ms = (lap_end_time - self.sector_start_time) * 1000.0;
        if sector_ms >= MIN_SECTOR_MS {
            self.completed_sectors.push((final_sector, sector_ms));
        }
    }

    fn record_lap_frame(&mut self, frame: &AnalysisFrame) {
        self.lap_accum.frames += 1;
        if frame.on_pit_road {
            self.lap_accum.pit_frames += 1;
        }
        if self.lap_accum.frames == 1 {
            self.lap_accum.min_dist_pct = frame.lap_dist_pct;
            self.lap_accum.max_dist_pct = frame.lap_dist_pct;
        } else {
            self.lap_accum.min_dist_pct = self.lap_accum.min_dist_pct.min(frame.lap_dist_pct);
            self.lap_accum.max_dist_pct = self.lap_accum.max_dist_pct.max(frame.lap_dist_pct);
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

        let sector_source = self
            .pending_lap_sectors
            .take()
            .unwrap_or_else(|| self.completed_sectors.clone());

        LiveSnapshot {
            track: self.track.clone(),
            car: self.car.clone(),
            session_type: self.session_type.clone(),
            lap: self.current_lap,
            lap_time_ms,
            last_lap_ms: self.last_lap_ms,
            last_lap_valid: self.last_lap_valid,
            best_lap_ms: self.best_lap_ms,
            delta_to_best_ms: delta_to_best,
            delta_to_last_ms: delta_to_last,
            fuel_level: frame.fuel_level,
            speed: frame.speed,
            lap_dist_pct: frame.lap_dist_pct,
            current_sector: current_sector.min(3),
            sectors: (1..=3)
                .map(|n| {
                    let done = sector_source.iter().find(|(s, _)| *s == n);
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
            on_pit_road: frame.on_pit_road,
            // Competitor and session-wide fields are merged in by the live loop
            // from the separate CarIdx telemetry stream.
            ..Default::default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::types::SectorBoundary;

    fn boundary(sector_num: i32, start_pct: f64) -> SectorBoundary {
        SectorBoundary {
            sector_num,
            start_pct,
        }
    }

    fn frame(lap: i32, pct: f32, session_time: f64) -> AnalysisFrame {
        AnalysisFrame {
            session_num: 0,
            lap,
            lap_dist_pct: pct,
            speed: 0.0,
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            gear: 0,
            fuel_level: 0.0,
            on_pit_road: false,
            session_time,
            lf_temp: 0.0,
            rf_temp: 0.0,
            lr_temp: 0.0,
            rr_temp: 0.0,
        }
    }

    fn default_bounds() -> Vec<SectorBoundary> {
        vec![boundary(1, 0.34), boundary(2, 0.72)]
    }

    #[test]
    fn sector_crossings_at_splits() {
        let mut tracker = LiveTracker::new();
        let bounds = default_bounds();
        tracker.update(&frame(1, 0.0, 0.0), &bounds);
        tracker.update(&frame(1, 0.35, 34.0), &bounds);
        tracker.update(&frame(1, 0.73, 72.0), &bounds);
        assert_eq!(tracker.completed_sectors.len(), 2);

        let snap = tracker.snapshot_from_frame(&frame(2, 0.01, 100.0), &bounds);
        let completed: Vec<_> = snap
            .sectors
            .iter()
            .filter(|s| s.completed)
            .map(|s| s.sector_num)
            .collect();
        assert_eq!(completed, vec![1, 2, 3]);
    }

    #[test]
    fn mid_lap_join_skips_passed_splits() {
        let mut tracker = LiveTracker::new();
        let bounds = default_bounds();
        tracker.update(&frame(1, 0.50, 50.0), &bounds);
        assert_eq!(tracker.completed_sectors.len(), 0);
        tracker.update(&frame(1, 0.73, 72.0), &bounds);
        assert_eq!(tracker.completed_sectors.len(), 1);
        assert_eq!(tracker.completed_sectors[0].0, 2);
    }

    #[test]
    fn no_spurious_sectors_on_single_tick() {
        let mut tracker = LiveTracker::new();
        let bounds = default_bounds();
        tracker.update(&frame(1, 0.0, 0.0), &bounds);
        tracker.update(&frame(1, 0.80, 80.0), &bounds);
        assert!(tracker.completed_sectors.len() <= 2);
        for (_, ms) in &tracker.completed_sectors {
            assert!(*ms >= MIN_SECTOR_MS);
        }
    }
}
