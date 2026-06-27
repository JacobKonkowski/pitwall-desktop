use pitwall::SessionInfo;
use tracing::debug;

use super::types::{RawFrame, SectorBoundary};

/// Minimum sector duration; matches live audio coach filter.
pub const MIN_SECTOR_MS: f64 = 1000.0;

const SF_WRAP_PREV_MIN: f32 = 0.9;
const SF_WRAP_CURR_MAX: f32 = 0.1;
const PCT_REGRESSION_EPS: f32 = 0.05;
/// Ignore finish-line markers at or near 100%.
const FINISH_MAX_PCT: f64 = 0.999;
const PCT_DEDUPE_EPS: f64 = 0.001;

/// Sorted region start positions (includes 0% when sector data exists).
///
/// iRacing `SplitTimeInfo.Sectors[].SectorStartPct` marks where each sector
/// **begins**. Sector 0 at 0% is the start of the first timed region.
pub fn region_starts(boundaries: &[SectorBoundary]) -> Vec<f64> {
    let mut starts: Vec<f64> = boundaries
        .iter()
        .map(|b| b.start_pct)
        .filter(|&pct| pct < FINISH_MAX_PCT)
        .collect();

    if starts.is_empty() {
        return Vec::new();
    }

    starts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    starts.dedup_by(|a, b| (*a - *b).abs() < PCT_DEDUPE_EPS);

    if starts[0] > PCT_DEDUPE_EPS {
        starts.insert(0, 0.0);
    }

    starts
}

/// Split lines used for crossing detection: region starts after 0% (interior boundaries).
pub fn normalize_sector_boundaries(boundaries: &[SectorBoundary]) -> Vec<SectorBoundary> {
    let regions = region_starts(boundaries);
    if regions.len() < 2 {
        return Vec::new();
    }

    regions
        .iter()
        .skip(1)
        .enumerate()
        .map(|(idx, &start_pct)| SectorBoundary {
            sector_num: (idx + 1) as i32,
            start_pct,
        })
        .collect()
}

/// Number of timed sectors (S1..SN) for the track layout.
pub fn sector_count(boundaries: &[SectorBoundary]) -> usize {
    region_starts(boundaries).len()
}

/// Region start pcts plus implicit finish at 1.0 — for UI progress bars.
pub fn display_sector_boundaries(boundaries: &[SectorBoundary]) -> Vec<f64> {
    let mut display = region_starts(boundaries);
    if display.is_empty() {
        return display;
    }
    if *display.last().unwrap_or(&0.0) < 1.0 - PCT_DEDUPE_EPS {
        display.push(1.0);
    }
    display
}

/// SDK-style current sector: max region index where `pct > start`, 1-indexed for display.
pub fn current_sector_from_pct(pct: f32, boundaries: &[SectorBoundary]) -> i32 {
    let regions = region_starts(boundaries);
    if regions.is_empty() {
        return 1;
    }
    let pct = pct as f64;
    let idx = regions
        .iter()
        .rposition(|&start| pct > start)
        .unwrap_or(0);
    (idx + 1) as i32
}

/// Read sector split lines from session YAML (shared by live telemetry and IBT import).
///
/// Returns empty when split data is missing so callers can suppress sector display
/// rather than fabricating 33%/66% defaults.
pub fn extract_sector_boundaries(session: &SessionInfo) -> Vec<SectorBoundary> {
    let Some(split) = &session.split_time_info else {
        return Vec::new();
    };
    let Some(sectors) = &split.sectors else {
        return Vec::new();
    };
    sectors
        .iter()
        .filter_map(|s| {
            Some(SectorBoundary {
                sector_num: s.sector_num.unwrap_or(0),
                start_pct: s.sector_start_pct?,
            })
        })
        .collect()
}

/// Raw region starts from session YAML (for persistence / trace coach).
pub fn extract_region_starts(session: &SessionInfo) -> Vec<f64> {
    let Some(split) = &session.split_time_info else {
        return Vec::new();
    };
    let Some(sectors) = &split.sectors else {
        return Vec::new();
    };
    let raw: Vec<SectorBoundary> = sectors
        .iter()
        .filter_map(|s| {
            Some(SectorBoundary {
                sector_num: s.sector_num.unwrap_or(0),
                start_pct: s.sector_start_pct?,
            })
        })
        .collect();
    region_starts(&raw)
}

/// Per-lap sector timing state (live telemetry and batch import share this logic).
#[derive(Debug, Clone)]
pub struct SectorSplitState {
    normalized_bounds: Vec<SectorBoundary>,
    next_boundary_idx: usize,
    sector_start_time: f64,
    completed: Vec<(i32, f64)>,
}

impl SectorSplitState {
    pub fn new(boundaries: &[SectorBoundary], lap_start_time: f64, start_pct: f32) -> Self {
        let normalized_bounds = normalize_sector_boundaries(boundaries);
        let mut state = Self {
            normalized_bounds,
            next_boundary_idx: 0,
            sector_start_time: lap_start_time,
            completed: Vec::new(),
        };
        state.skip_passed_boundaries(start_pct);
        state
    }

    pub fn sync_bounds(&mut self, boundaries: &[SectorBoundary], current_pct: f32) {
        let normalized = normalize_sector_boundaries(boundaries);
        if normalized != self.normalized_bounds {
            self.normalized_bounds = normalized;
            self.next_boundary_idx = 0;
            self.skip_passed_boundaries(current_pct);
        }
    }

    pub fn reset_lap(&mut self, boundaries: &[SectorBoundary], lap_start_time: f64, start_pct: f32) {
        self.normalized_bounds = normalize_sector_boundaries(boundaries);
        self.next_boundary_idx = 0;
        self.sector_start_time = lap_start_time;
        self.completed.clear();
        self.skip_passed_boundaries(start_pct);
    }

    pub fn advance(&mut self, prev_pct: f32, prev_time: f64, curr_pct: f32, curr_time: f64) {
        if Self::is_pct_regression(prev_pct, curr_pct) {
            return;
        }
        if Self::is_sf_wrap(prev_pct, curr_pct) {
            self.next_boundary_idx = 0;
            self.skip_passed_boundaries(curr_pct);
        }
        self.detect_forward_crossings(prev_pct, prev_time, curr_pct, curr_time);
    }

    pub fn finish_lap(&mut self, lap_end_time: f64) -> Vec<(i32, f64)> {
        self.record_final_sector(lap_end_time);
        self.completed.clone()
    }

    pub fn completed_sectors(&self) -> &[(i32, f64)] {
        &self.completed
    }

    fn is_sf_wrap(prev_pct: f32, curr_pct: f32) -> bool {
        prev_pct > SF_WRAP_PREV_MIN && curr_pct < SF_WRAP_CURR_MAX
    }

    fn is_pct_regression(prev_pct: f32, curr_pct: f32) -> bool {
        !Self::is_sf_wrap(prev_pct, curr_pct) && curr_pct + PCT_REGRESSION_EPS < prev_pct
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

    fn detect_forward_crossings(
        &mut self,
        prev_pct: f32,
        prev_time: f64,
        curr_pct: f32,
        curr_time: f64,
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
            let cross_time = prev_time + (curr_time - prev_time) * ratio;
            let sector_num = (self.next_boundary_idx + 1) as i32;
            let sector_ms = (cross_time - self.sector_start_time) * 1000.0;
            if sector_ms >= MIN_SECTOR_MS {
                debug!(
                    sector = sector_num,
                    pct,
                    ms = sector_ms,
                    "sector crossing recorded"
                );
                self.completed.push((sector_num, sector_ms));
            }
            self.sector_start_time = cross_time;
            self.next_boundary_idx += 1;
        }
    }

    fn record_final_sector(&mut self, lap_end_time: f64) {
        if self.normalized_bounds.is_empty() {
            return;
        }
        let final_sector = (self.normalized_bounds.len() + 1) as i32;
        if self.completed.iter().any(|(n, _)| *n == final_sector) {
            return;
        }
        let sector_ms = (lap_end_time - self.sector_start_time) * 1000.0;
        if sector_ms >= MIN_SECTOR_MS {
            debug!(
                sector = final_sector,
                ms = sector_ms,
                "final sector recorded at lap finish"
            );
            self.completed.push((final_sector, sector_ms));
        }
    }
}

pub fn compute_sector_times(frames: &[RawFrame], boundaries: &[SectorBoundary]) -> Vec<(i32, f64)> {
    if frames.len() < 2 {
        return Vec::new();
    }

    let normalized = normalize_sector_boundaries(boundaries);
    if normalized.is_empty() {
        return Vec::new();
    }

    let start_time = frames.first().map(|f| f.session_time).unwrap_or(0.0);
    let start_pct = frames.first().map(|f| f.lap_dist_pct).unwrap_or(0.0);
    let mut state = SectorSplitState::new(boundaries, start_time, start_pct);

    for window in frames.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        state.advance(
            prev.lap_dist_pct,
            prev.session_time,
            curr.lap_dist_pct,
            curr.session_time,
        );
    }

    let lap_end = frames.last().map(|f| f.session_time).unwrap_or(start_time);
    state.finish_lap(lap_end)
}

#[cfg(test)]
pub fn assert_sectors_sum_to_lap(sectors: &[(i32, f64)], lap_ms: f64, tolerance_ms: f64) {
    let sum: f64 = sectors.iter().map(|(_, ms)| ms).sum();
    assert!(
        (sum - lap_ms).abs() <= tolerance_ms,
        "sector sum {sum} != lap time {lap_ms} (delta {} ms)",
        (sum - lap_ms).abs()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(pct: f32, session_time: f64) -> RawFrame {
        RawFrame {
            session_num: 0,
            lap: 1,
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

    fn linear_frames(count: usize, start_pct: f32, end_pct: f32, duration_s: f64) -> Vec<RawFrame> {
        (0..count)
            .map(|i| {
                let t = i as f64 / (count.max(2) - 1) as f64;
                let pct = start_pct + (end_pct - start_pct) * t as f32;
                frame(pct, t * duration_s)
            })
            .collect()
    }

    fn default_bounds() -> Vec<SectorBoundary> {
        vec![
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.34,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.72,
            },
        ]
    }

    fn four_sector_yaml() -> Vec<SectorBoundary> {
        vec![
            SectorBoundary {
                sector_num: 0,
                start_pct: 0.0,
            },
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.259885,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.509689,
            },
            SectorBoundary {
                sector_num: 3,
                start_pct: 0.694809,
            },
        ]
    }

    #[test]
    fn region_starts_includes_zero() {
        let boundaries = vec![
            SectorBoundary {
                sector_num: 0,
                start_pct: 0.0,
            },
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.34,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.72,
            },
        ];
        let regions = region_starts(&boundaries);
        assert_eq!(regions.len(), 3);
        assert!((regions[0] - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn four_sector_track_produces_four_sectors() {
        let boundaries = four_sector_yaml();
        assert_eq!(sector_count(&boundaries), 4);
        let normalized = normalize_sector_boundaries(&boundaries);
        assert_eq!(normalized.len(), 3);

        let frames = linear_frames(100, 0.0, 0.98, 82.8);
        let sectors = compute_sector_times(&frames, &boundaries);
        assert_eq!(sectors.len(), 4);
        assert_eq!(sectors[3].0, 4, "final sector must be S4");
        assert_sectors_sum_to_lap(&sectors, 82_800.0, 500.0);
    }

    #[test]
    fn current_sector_from_pct_four_sectors() {
        let boundaries = four_sector_yaml();
        assert_eq!(current_sector_from_pct(0.0, &boundaries), 1);
        assert_eq!(current_sector_from_pct(0.27, &boundaries), 2);
        assert_eq!(current_sector_from_pct(0.55, &boundaries), 3);
        assert_eq!(current_sector_from_pct(0.80, &boundaries), 4);
    }

    #[test]
    fn one_indexed_yaml_with_zero_start() {
        let boundaries = vec![
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.0,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.34,
            },
            SectorBoundary {
                sector_num: 3,
                start_pct: 0.72,
            },
        ];
        assert_eq!(sector_count(&boundaries), 3);
        let normalized = normalize_sector_boundaries(&boundaries);
        assert_eq!(normalized.len(), 2);

        let frames = linear_frames(100, 0.0, 0.98, 100.0);
        let sectors = compute_sector_times(&frames, &boundaries);
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 100_000.0, 500.0);
    }

    #[test]
    fn drops_trailing_finish_marker() {
        let boundaries = vec![
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.34,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.72,
            },
            SectorBoundary {
                sector_num: 3,
                start_pct: 0.999,
            },
        ];
        assert_eq!(sector_count(&boundaries), 3);

        let frames = linear_frames(100, 0.0, 0.98, 100.0);
        let sectors = compute_sector_times(&frames, &boundaries);
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 100_000.0, 500.0);
    }

    #[test]
    fn flying_sectors_sum_to_lap_time() {
        let frames = linear_frames(900, 0.0, 0.98, 91.7);
        let sectors = compute_sector_times(&frames, &default_bounds());
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 91_700.0, 500.0);
    }

    #[test]
    fn flying_sectors_sf_wrap() {
        let mut frames = linear_frames(450, 0.97, 1.0, 45.0);
        frames.extend(linear_frames(450, 0.0, 0.95, 45.0).into_iter().map(|mut f| {
            f.session_time += 45.0;
            f
        }));
        let sectors = compute_sector_times(&frames, &default_bounds());
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 90_000.0, 500.0);
    }

    #[test]
    fn flying_sectors_okayama_style() {
        let frames = linear_frames(873, 0.0, 0.873, 87.3);
        let sectors = compute_sector_times(&frames, &default_bounds());
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 87_300.0, 500.0);
    }

    #[test]
    fn flying_sectors_pct_regression() {
        let mut frames = linear_frames(300, 0.0, 0.38, 30.0);
        let jump = frame(0.08, 30.0);
        frames.push(jump);
        frames.extend(
            linear_frames(600, 0.08, 0.98, 60.0)
                .into_iter()
                .skip(1)
                .map(|mut f| {
                    f.session_time += 30.0;
                    f
                }),
        );
        let sectors = compute_sector_times(&frames, &default_bounds());
        assert_eq!(sectors.len(), 3);
        assert_sectors_sum_to_lap(&sectors, 90_000.0, 500.0);
    }

    #[test]
    fn streaming_matches_batch() {
        let frames = linear_frames(900, 0.0, 0.98, 91.7);
        let bounds = default_bounds();
        let batch = compute_sector_times(&frames, &bounds);

        let start_time = frames[0].session_time;
        let mut state = SectorSplitState::new(&bounds, start_time, frames[0].lap_dist_pct);
        for window in frames.windows(2) {
            let prev = &window[0];
            let curr = &window[1];
            state.advance(
                prev.lap_dist_pct,
                prev.session_time,
                curr.lap_dist_pct,
                curr.session_time,
            );
        }
        let stream = state.finish_lap(frames.last().unwrap().session_time);

        assert_eq!(batch, stream);
    }

    #[test]
    fn display_boundaries_include_finish() {
        let display = display_sector_boundaries(&default_bounds());
        assert_eq!(display.len(), 4);
        assert!((display[0] - 0.0).abs() < f64::EPSILON);
        assert!((display.last().copied().unwrap_or(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_yaml_returns_empty() {
        assert!(region_starts(&[]).is_empty());
        assert!(normalize_sector_boundaries(&[]).is_empty());
        let sectors = compute_sector_times(&linear_frames(100, 0.0, 0.98, 100.0), &[]);
        assert!(sectors.is_empty());
    }
}
