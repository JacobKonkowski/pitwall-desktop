use std::collections::HashMap;

use rayon::prelude::*;

use super::fuel_tire::{fuel_stats, tire_averages};
use super::lap_kind::classify_lap_kind;
use super::lap_segmenter::{
    average_speed, compute_lap_time_ms, downsample_traces, max_lap_dist_pct, segment_laps,
    MIN_LAP_MAX_PCT,
};
use super::lap_validity::{include_in_stats_ibt, metrics_from_frames, telemetry_passes_heuristics};
use super::sector_splitter::compute_sector_times;
use super::types::{LapFrames, RawFrame, SectorBoundary};
use crate::storage::models::LapKind;
use crate::storage::StoredLap;

/// Laps faster than this fraction of the sub-session median are re-checked.
const OUTLIER_TIME_RATIO: f64 = 0.93;

struct AnalyzedLap {
    lap: StoredLap,
    max_dist_pct: f32,
}

pub fn analyze_session(
    frames: Vec<RawFrame>,
    sector_boundaries: Vec<SectorBoundary>,
    session_labels: HashMap<i32, String>,
) -> Vec<StoredLap> {
    let lap_groups = segment_laps(frames, &session_labels);

    let mut analyzed: Vec<AnalyzedLap> = lap_groups
        .into_par_iter()
        .map(|group| analyze_lap(group, &sector_boundaries))
        .collect();

    apply_session_outlier_filter(&mut analyzed);

    analyzed.into_iter().map(|entry| entry.lap).collect()
}

fn analyze_lap(group: LapFrames, sector_boundaries: &[SectorBoundary]) -> AnalyzedLap {
    let lap_time_ms = compute_lap_time_ms(&group.frames);
    let lap_kind = classify_lap_kind(&group.frames);
    let metrics = metrics_from_frames(&group.frames, lap_time_ms);
    let valid = include_in_stats_ibt(lap_kind, telemetry_passes_heuristics(&metrics));
    let max_dist_pct = max_lap_dist_pct(&group.frames);
    let (fuel_start, fuel_used) = fuel_stats(&group.frames);
    let (lf_temp, rf_temp, lr_temp, rr_temp) = tire_averages(&group.frames);
    let sectors = if valid {
        compute_sector_times(&group.frames, sector_boundaries)
    } else {
        Vec::new()
    };
    let traces = if valid {
        downsample_traces(&group.frames)
    } else {
        Vec::new()
    };

    AnalyzedLap {
        lap: StoredLap {
            session_num: group.session_num,
            session_type: group.session_type,
            iracing_lap: group.iracing_lap,
            lap_number: group.lap_number,
            lap_time_ms,
            valid,
            lap_kind,
            fuel_start,
            fuel_used,
            avg_speed: average_speed(&group.frames),
            lf_temp,
            rf_temp,
            lr_temp,
            rr_temp,
            sectors,
            traces,
        },
        max_dist_pct,
    }
}

/// Second pass: invalidate suspiciously fast laps that did not reach the finish line.
fn apply_session_outlier_filter(analyzed: &mut [AnalyzedLap]) {
    let session_nums: Vec<i32> = analyzed
        .iter()
        .map(|entry| entry.lap.session_num)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for session_num in session_nums {
        let times: Vec<f64> = analyzed
            .iter()
            .filter(|entry| entry.lap.session_num == session_num && entry.lap.valid)
            .filter_map(|entry| entry.lap.lap_time_ms)
            .filter(|t| *t > 0.0)
            .collect();

        if times.len() < 3 {
            continue;
        }

        let mut sorted = times.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];
        if median <= 0.0 {
            continue;
        }

        let threshold = median * OUTLIER_TIME_RATIO;

        for entry in analyzed.iter_mut().filter(|e| e.lap.session_num == session_num) {
            if !entry.lap.valid || entry.lap.lap_kind != LapKind::Flying {
                continue;
            }
            let Some(lap_time) = entry.lap.lap_time_ms else {
                continue;
            };
            if lap_time < threshold && entry.max_dist_pct < MIN_LAP_MAX_PCT {
                entry.lap.valid = false;
                entry.lap.sectors.clear();
                entry.lap.traces.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::types::RawFrame;
    use crate::storage::models::LapKind;
    use crate::storage::StoredLap;

    fn entry(lap_number: i32, time_ms: f64, valid: bool, max_pct: f32) -> AnalyzedLap {
        AnalyzedLap {
            lap: StoredLap {
                session_num: 0,
                session_type: "Practice".into(),
                iracing_lap: lap_number,
                lap_number,
                lap_time_ms: Some(time_ms),
                valid,
                lap_kind: LapKind::Flying,
                fuel_start: None,
                fuel_used: None,
                avg_speed: None,
                lf_temp: None,
                rf_temp: None,
                lr_temp: None,
                rr_temp: None,
                sectors: Vec::new(),
                traces: Vec::new(),
            },
            max_dist_pct: max_pct,
        }
    }

    fn pit_out_frames(count: usize) -> Vec<RawFrame> {
        (0..count)
            .map(|i| {
                let t = i as f64 / (count.max(2) - 1) as f64;
                RawFrame {
                    session_num: 0,
                    lap: 1,
                    lap_dist_pct: t as f32 * 0.98,
                    speed: 50.0,
                    throttle: 0.0,
                    brake: 0.0,
                    steering: 0.0,
                    gear: 3,
                    fuel_level: 50.0,
                    on_pit_road: i < count / 10,
                    session_time: t * 90.0,
                    lf_temp: 0.0,
                    rf_temp: 0.0,
                    lr_temp: 0.0,
                    rr_temp: 0.0,
                }
            })
            .collect()
    }

    #[test]
    fn outlier_pass_invalidates_fast_incomplete_lap() {
        let mut analyzed = vec![
            entry(1, 100_000.0, true, 0.99),
            entry(2, 100_000.0, true, 0.99),
            entry(3, 85_000.0, true, 0.852),
            entry(4, 100_000.0, true, 0.99),
        ];
        apply_session_outlier_filter(&mut analyzed);
        assert!(!analyzed[2].lap.valid);
    }

    #[test]
    fn pit_out_lap_has_no_sectors() {
        let mut labels = std::collections::HashMap::new();
        labels.insert(0, "Practice".into());
        let laps = analyze_session(pit_out_frames(900), default_split_boundaries(), labels);
        let lap = laps.first().expect("one lap");
        assert_eq!(lap.lap_kind, LapKind::PitOut);
        assert!(!lap.valid);
        assert!(lap.sectors.is_empty());
    }

    fn default_split_boundaries() -> Vec<SectorBoundary> {
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
}
