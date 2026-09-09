//! Lap comparison: time delta, per-sector deltas, and distance-aligned traces.
//!
//! Pure math over two laps' stored products. The caller (commands layer) loads
//! the traces/sectors/times from storage and hands them in; this module performs
//! no I/O. "Candidate" is the lap being examined, "reference" is what it is
//! measured against (session best or a user pick).

use serde::{Deserialize, Serialize};

use super::types::TracePoint;

/// Number of points on the shared distance grid used to align two laps.
const GRID_POINTS: usize = 200;

/// One lap's stored products, as needed for a comparison.
pub struct CompareInput<'a> {
    pub lap_id: i64,
    pub lap_time_ms: Option<f64>,
    /// `(sector_num, time_ms)` pairs.
    pub sectors: &'a [(i32, f64)],
    /// Trace points sorted ascending by `dist_pct`.
    pub traces: &'a [TracePoint],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorDelta {
    pub sector_num: i32,
    pub candidate_ms: Option<f64>,
    pub reference_ms: Option<f64>,
    pub delta_ms: Option<f64>,
}

/// One point of the aligned overlay. Channels are `None` where a lap has no data
/// covering that part of the track.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignedPoint {
    pub dist_pct: f64,
    pub candidate_speed: Option<f64>,
    pub reference_speed: Option<f64>,
    pub candidate_throttle: Option<f64>,
    pub reference_throttle: Option<f64>,
    pub candidate_brake: Option<f64>,
    pub reference_brake: Option<f64>,
    pub candidate_gear: Option<f64>,
    pub reference_gear: Option<f64>,
    pub candidate_steering: Option<f64>,
    pub reference_steering: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapComparison {
    pub candidate_lap_id: i64,
    pub reference_lap_id: i64,
    pub candidate_time_ms: Option<f64>,
    pub reference_time_ms: Option<f64>,
    pub delta_ms: Option<f64>,
    pub sector_deltas: Vec<SectorDelta>,
    pub series: Vec<AlignedPoint>,
}

pub fn compare_laps(candidate: &CompareInput, reference: &CompareInput) -> LapComparison {
    let delta_ms = match (candidate.lap_time_ms, reference.lap_time_ms) {
        (Some(c), Some(r)) => Some(c - r),
        _ => None,
    };

    LapComparison {
        candidate_lap_id: candidate.lap_id,
        reference_lap_id: reference.lap_id,
        candidate_time_ms: candidate.lap_time_ms,
        reference_time_ms: reference.lap_time_ms,
        delta_ms,
        sector_deltas: sector_deltas(candidate.sectors, reference.sectors),
        series: aligned_series(candidate.traces, reference.traces),
    }
}

fn sector_deltas(candidate: &[(i32, f64)], reference: &[(i32, f64)]) -> Vec<SectorDelta> {
    let mut nums: Vec<i32> = candidate
        .iter()
        .chain(reference.iter())
        .map(|(n, _)| *n)
        .collect();
    nums.sort_unstable();
    nums.dedup();

    nums.into_iter()
        .map(|sector_num| {
            let cand = candidate.iter().find(|(n, _)| *n == sector_num).map(|(_, t)| *t);
            let refr = reference.iter().find(|(n, _)| *n == sector_num).map(|(_, t)| *t);
            let delta_ms = match (cand, refr) {
                (Some(c), Some(r)) => Some(c - r),
                _ => None,
            };
            SectorDelta {
                sector_num,
                candidate_ms: cand,
                reference_ms: refr,
                delta_ms,
            }
        })
        .collect()
}

fn aligned_series(candidate: &[TracePoint], reference: &[TracePoint]) -> Vec<AlignedPoint> {
    (0..GRID_POINTS)
        .map(|i| {
            let dist_pct = i as f64 / (GRID_POINTS - 1) as f64;
            AlignedPoint {
                dist_pct,
                candidate_speed: interp(candidate, dist_pct, |p| p.speed),
                reference_speed: interp(reference, dist_pct, |p| p.speed),
                candidate_throttle: interp(candidate, dist_pct, |p| p.throttle),
                reference_throttle: interp(reference, dist_pct, |p| p.throttle),
                candidate_brake: interp(candidate, dist_pct, |p| p.brake),
                reference_brake: interp(reference, dist_pct, |p| p.brake),
                candidate_gear: interp(candidate, dist_pct, |p| p.gear as f64),
                reference_gear: interp(reference, dist_pct, |p| p.gear as f64),
                candidate_steering: interp(candidate, dist_pct, |p| p.steering),
                reference_steering: interp(reference, dist_pct, |p| p.steering),
            }
        })
        .collect()
}

/// Linear interpolation of a channel at `x` over points sorted by `dist_pct`.
/// Returns `None` when the trace is empty or `x` falls outside its coverage.
fn interp(points: &[TracePoint], x: f64, accessor: impl Fn(&TracePoint) -> f64) -> Option<f64> {
    if points.len() < 2 {
        return points.first().map(&accessor);
    }
    let first = points.first().unwrap();
    let last = points.last().unwrap();
    if x < first.dist_pct || x > last.dist_pct {
        return None;
    }
    // Binary search for the bracketing pair.
    let idx = points.partition_point(|p| p.dist_pct <= x);
    if idx == 0 {
        return Some(accessor(first));
    }
    if idx >= points.len() {
        return Some(accessor(last));
    }
    let lo = &points[idx - 1];
    let hi = &points[idx];
    let span = hi.dist_pct - lo.dist_pct;
    if span.abs() < f64::EPSILON {
        return Some(accessor(lo));
    }
    let t = (x - lo.dist_pct) / span;
    Some(accessor(lo) + (accessor(hi) - accessor(lo)) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tp(dist: f64, speed: f64, gear: i32, steering: f64) -> TracePoint {
        TracePoint {
            dist_pct: dist,
            speed,
            throttle: 0.0,
            brake: 0.0,
            gear,
            steering,
        }
    }

    #[test]
    fn time_and_sector_deltas() {
        let cand = CompareInput {
            lap_id: 1,
            lap_time_ms: Some(91_000.0),
            sectors: &[(1, 30_000.0), (2, 31_000.0), (3, 30_000.0)],
            traces: &[],
        };
        let refr = CompareInput {
            lap_id: 2,
            lap_time_ms: Some(90_000.0),
            sectors: &[(1, 29_500.0), (2, 31_000.0), (3, 29_500.0)],
            traces: &[],
        };
        let cmp = compare_laps(&cand, &refr);
        assert_eq!(cmp.delta_ms, Some(1_000.0));
        assert_eq!(cmp.sector_deltas.len(), 3);
        assert_eq!(cmp.sector_deltas[0].delta_ms, Some(500.0));
        assert_eq!(cmp.sector_deltas[1].delta_ms, Some(0.0));
    }

    #[test]
    fn aligned_series_interpolates_on_grid() {
        let cand = CompareInput {
            lap_id: 1,
            lap_time_ms: None,
            sectors: &[],
            traces: &[tp(0.0, 100.0, 3, 0.0), tp(1.0, 200.0, 5, 1.0)],
        };
        let refr = CompareInput {
            lap_id: 2,
            lap_time_ms: None,
            sectors: &[],
            traces: &[tp(0.0, 50.0, 2, -0.5), tp(1.0, 150.0, 4, 0.5)],
        };
        let cmp = compare_laps(&cand, &refr);
        assert_eq!(cmp.series.len(), GRID_POINTS);
        // Midpoint should be ~150 for candidate, ~100 for reference.
        let mid = &cmp.series[GRID_POINTS / 2];
        assert!((mid.candidate_speed.unwrap() - 150.0).abs() < 2.0);
        assert!((mid.reference_speed.unwrap() - 100.0).abs() < 2.0);
        // Gear 3→5 and 2→4 → midpoints ~4 and ~3.
        assert!((mid.candidate_gear.unwrap() - 4.0).abs() < 0.1);
        assert!((mid.reference_gear.unwrap() - 3.0).abs() < 0.1);
        // Steering 0→1 and -0.5→0.5 → midpoints ~0.5 and ~0.0.
        assert!((mid.candidate_steering.unwrap() - 0.5).abs() < 0.05);
        assert!(mid.reference_steering.unwrap().abs() < 0.05);
    }
}
