//! Trace-based coaching insights.
//!
//! Compares throttle/brake/steering traces of slow laps against the best lap in
//! the same sub-session to explain *why* time was lost in a sector. Each insight
//! is anchored to a (lap, sector) pair where measurable time was lost, and the
//! trace comparison identifies the most likely cause (early lift, late braking,
//! or excess steering).

use std::collections::HashMap;

use crate::storage::{LapSummary, SessionDetail, TracePoint};

use super::coach::CoachInsight;

/// Minimum sector time loss (ms) before we bother explaining the cause.
const MIN_SECTOR_LOSS_MS: f64 = 100.0;
/// Max number of laps (best + slow) per sub-session whose traces we load.
const MAX_LAPS_PER_STAGE: usize = 6;
/// Max trace insights returned for the whole session (avoid noise).
const MAX_TRACE_INSIGHTS: usize = 5;
/// Samples per sector when aligning two laps on a dist_pct grid.
const SECTOR_SAMPLES: usize = 40;

/// Lap ids whose traces are worth loading for trace analysis: the best lap plus
/// the slowest valid laps in each sub-session.
pub fn select_trace_lap_ids(detail: &SessionDetail) -> Vec<i64> {
    let mut ids: Vec<i64> = Vec::new();

    for session_num in sub_session_nums(detail) {
        let mut stage: Vec<&LapSummary> = detail
            .laps
            .iter()
            .filter(|l| l.session_num == session_num && l.valid && l.lap_time_ms.unwrap_or(0.0) > 0.0)
            .collect();
        if stage.len() < 2 {
            continue;
        }
        stage.sort_by(|a, b| {
            a.lap_time_ms
                .unwrap_or(f64::MAX)
                .partial_cmp(&b.lap_time_ms.unwrap_or(f64::MAX))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for lap in stage.iter().take(MAX_LAPS_PER_STAGE) {
            if !ids.contains(&lap.id) {
                ids.push(lap.id);
            }
        }
    }

    ids
}

/// Append trace-based insights to an existing insight list.
pub fn append_trace_insights(
    insights: &mut Vec<CoachInsight>,
    detail: &SessionDetail,
    traces: &HashMap<i64, Vec<TracePoint>>,
) {
    let mut trace_insights: Vec<CoachInsight> = Vec::new();
    let boundaries = &detail.session.sector_boundaries;

    for session_num in sub_session_nums(detail) {
        let stage: Vec<&LapSummary> = detail
            .laps
            .iter()
            .filter(|l| l.session_num == session_num && l.valid && l.lap_time_ms.unwrap_or(0.0) > 0.0)
            .collect();
        if stage.len() < 2 {
            continue;
        }

        let Some(best_lap) = stage
            .iter()
            .copied()
            .min_by(|a, b| {
                a.lap_time_ms
                    .unwrap_or(f64::MAX)
                    .partial_cmp(&b.lap_time_ms.unwrap_or(f64::MAX))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        else {
            continue;
        };

        let Some(best_trace) = traces.get(&best_lap.id).filter(|t| t.len() >= 4) else {
            continue;
        };

        for lap in &stage {
            if lap.id == best_lap.id {
                continue;
            }
            let Some(slow_trace) = traces.get(&lap.id).filter(|t| t.len() >= 4) else {
                continue;
            };

            let max_sector = best_lap
                .sectors
                .iter()
                .map(|s| s.sector_num)
                .max()
                .unwrap_or(0);

            for sector_num in 1..=max_sector {
                let loss = sector_loss_ms(lap, best_lap, sector_num);
                let Some(loss_ms) = loss.filter(|l| *l > MIN_SECTOR_LOSS_MS) else {
                    continue;
                };

                let (lo, hi) = sector_pct_range(sector_num, boundaries);
                if let Some(cause) = detect_cause(best_trace, slow_trace, lo, hi) {
                    trace_insights.push(build_insight(lap, sector_num, loss_ms, cause));
                }
            }
        }
    }

    trace_insights.sort_by(|a, b| {
        b.delta_ms
            .unwrap_or(0.0)
            .partial_cmp(&a.delta_ms.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    trace_insights.truncate(MAX_TRACE_INSIGHTS);
    insights.extend(trace_insights);
}

fn sub_session_nums(detail: &SessionDetail) -> Vec<i32> {
    let mut nums: Vec<i32> = detail
        .laps
        .iter()
        .map(|l| l.session_num)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    nums.sort_unstable();
    nums
}

fn sector_loss_ms(lap: &LapSummary, best_lap: &LapSummary, sector_num: i32) -> Option<f64> {
    let slow = lap.sectors.iter().find(|s| s.sector_num == sector_num)?.time_ms;
    let best = best_lap
        .sectors
        .iter()
        .find(|s| s.sector_num == sector_num)?
        .time_ms;
    Some(slow - best)
}

/// dist_pct range for a sector using persisted session boundaries when available.
fn sector_pct_range(sector_num: i32, boundaries: &[f64]) -> (f64, f64) {
    if boundaries.len() >= 2 {
        let idx = (sector_num - 1).max(0) as usize;
        let lo = boundaries.get(idx).copied().unwrap_or(0.0);
        let hi = boundaries.get(idx + 1).copied().unwrap_or(1.0);
        return (lo, hi);
    }
    match sector_num {
        1 => (0.0, 1.0 / 3.0),
        2 => (1.0 / 3.0, 2.0 / 3.0),
        _ => (2.0 / 3.0, 1.0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cause {
    EarlyLift,
    LateBrake,
    HighSteering,
}

/// Linear-interpolate a trace at the given dist_pct. Returns None if out of range.
fn sample_at(points: &[TracePoint], pct: f64) -> Option<(f64, f64, f64)> {
    if points.len() < 2 {
        return None;
    }
    if pct <= points[0].dist_pct {
        let p = &points[0];
        return Some((p.throttle, p.brake, p.steering.abs()));
    }
    if pct >= points[points.len() - 1].dist_pct {
        let p = &points[points.len() - 1];
        return Some((p.throttle, p.brake, p.steering.abs()));
    }
    for w in points.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if pct >= a.dist_pct && pct <= b.dist_pct {
            let span = b.dist_pct - a.dist_pct;
            let t = if span.abs() > f64::EPSILON {
                (pct - a.dist_pct) / span
            } else {
                0.0
            };
            let throttle = a.throttle + (b.throttle - a.throttle) * t;
            let brake = a.brake + (b.brake - a.brake) * t;
            let steering = a.steering.abs() + (b.steering.abs() - a.steering.abs()) * t;
            return Some((throttle, brake, steering));
        }
    }
    None
}

/// Determine the dominant cause of time loss within a sector dist_pct range.
fn detect_cause(best: &[TracePoint], slow: &[TracePoint], lo: f64, hi: f64) -> Option<Cause> {
    let span = hi - lo;
    if span <= 0.0 {
        return None;
    }

    let mut lift_samples = 0usize;
    let mut steering_diff_sum = 0.0;
    let mut samples = 0usize;

    let mut best_brake_onset: Option<f64> = None;
    let mut slow_brake_onset: Option<f64> = None;

    for i in 0..SECTOR_SAMPLES {
        let pct = lo + span * (i as f64 / (SECTOR_SAMPLES - 1) as f64);
        let (Some((bt, bb, bs)), Some((st, sb, ss))) = (sample_at(best, pct), sample_at(slow, pct))
        else {
            continue;
        };
        samples += 1;

        // Early lift: slow off-throttle while best still committed.
        if bt > 0.5 && st < bt - 0.15 {
            lift_samples += 1;
        }

        // Brake onset positions (first time brake exceeds threshold in sector).
        if best_brake_onset.is_none() && bb > 0.08 {
            best_brake_onset = Some(pct);
        }
        if slow_brake_onset.is_none() && sb > 0.08 {
            slow_brake_onset = Some(pct);
        }

        steering_diff_sum += ss - bs;
    }

    if samples == 0 {
        return None;
    }

    let lift_fraction = lift_samples as f64 / samples as f64;
    let steering_diff_avg = steering_diff_sum / samples as f64;
    let brake_late = match (best_brake_onset, slow_brake_onset) {
        (Some(b), Some(s)) => s - b,
        _ => 0.0,
    };

    // Score each cause; choose the strongest signal that clears its threshold.
    let mut best_cause: Option<(Cause, f64)> = None;
    let mut consider = |cause: Cause, score: f64, threshold: f64| {
        if score >= threshold {
            let normalized = score / threshold;
            if best_cause.map(|(_, s)| normalized > s).unwrap_or(true) {
                best_cause = Some((cause, normalized));
            }
        }
    };

    consider(Cause::EarlyLift, lift_fraction, 0.08);
    consider(Cause::LateBrake, brake_late, 0.02);
    consider(Cause::HighSteering, steering_diff_avg, 0.10);

    best_cause.map(|(c, _)| c)
}

fn build_insight(lap: &LapSummary, sector_num: i32, loss_ms: f64, cause: Cause) -> CoachInsight {
    let loss_s = loss_ms / 1000.0;
    let (kind, title, detail) = match cause {
        Cause::EarlyLift => (
            "early_lift",
            format!("Lap {} — S{sector_num}: early throttle lift", lap.lap_number),
            format!(
                "You came off the throttle earlier than your best lap through S{sector_num} (lost {loss_s:.3}s). Try carrying throttle a touch longer."
            ),
        ),
        Cause::LateBrake => (
            "late_brake",
            format!("Lap {} — S{sector_num}: late braking", lap.lap_number),
            format!(
                "You braked later than your best lap in S{sector_num} (lost {loss_s:.3}s). May be overshooting the corner and compromising exit."
            ),
        ),
        Cause::HighSteering => (
            "high_steering",
            format!("Lap {} — S{sector_num}: extra steering input", lap.lap_number),
            format!(
                "More steering input than your best lap through S{sector_num} (lost {loss_s:.3}s). Likely a missed apex or scrubbing speed."
            ),
        ),
    };

    CoachInsight {
        kind: kind.into(),
        title,
        detail,
        severity: if loss_ms > 500.0 { "warn".into() } else { "info".into() },
        lap_numbers: vec![lap.lap_number],
        sector_num: Some(sector_num),
        delta_ms: Some(loss_ms),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{LapKind, SectorTime, SessionSummary};

    fn tp(dist_pct: f64, throttle: f64, brake: f64, steering: f64) -> TracePoint {
        TracePoint {
            dist_pct,
            speed: 0.0,
            throttle,
            brake,
            gear: 4,
            steering,
        }
    }

    fn flat_trace(throttle: f64, brake: f64, steering: f64) -> Vec<TracePoint> {
        (0..=20)
            .map(|i| tp(i as f64 / 20.0, throttle, brake, steering))
            .collect()
    }

    fn lap(id: i64, lap_number: i32, time_ms: f64, sectors: Vec<(i32, f64)>) -> LapSummary {
        LapSummary {
            id,
            session_num: 0,
            session_type: "Practice".into(),
            iracing_lap: lap_number,
            lap_number,
            lap_time_ms: Some(time_ms),
            valid: true,
            lap_kind: LapKind::Flying,
            fuel_start: None,
            fuel_used: None,
            avg_speed: None,
            lf_temp: None,
            rf_temp: None,
            lr_temp: None,
            rr_temp: None,
            sectors: sectors
                .into_iter()
                .map(|(sector_num, time_ms)| SectorTime { sector_num, time_ms })
                .collect(),
            delta_to_best_ms: None,
        }
    }

    fn detail(laps: Vec<LapSummary>) -> SessionDetail {
        SessionDetail {
            session: SessionSummary {
                id: 1,
                ibt_path: String::new(),
                track: "Test".into(),
                car: "Test".into(),
                session_date: String::new(),
                lap_count: laps.len() as i32,
                best_lap_ms: None,
                imported_at: String::new(),
                sector_boundaries: vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0],
            },
            laps,
        }
    }

    #[test]
    fn sample_interpolates_between_points() {
        let points = vec![tp(0.0, 0.0, 0.0, 0.0), tp(1.0, 1.0, 0.0, 0.0)];
        let (throttle, _, _) = sample_at(&points, 0.5).unwrap();
        assert!((throttle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn detects_early_lift() {
        // Best stays full throttle; slow lifts to 0.2 in sector 1.
        let best = flat_trace(1.0, 0.0, 0.0);
        let slow = flat_trace(0.2, 0.0, 0.0);
        let cause = detect_cause(&best, &slow, 0.0, 1.0 / 3.0);
        assert_eq!(cause, Some(Cause::EarlyLift));
    }

    #[test]
    fn detects_high_steering() {
        let best = flat_trace(1.0, 0.0, 0.05);
        let slow = flat_trace(1.0, 0.0, 0.4);
        let cause = detect_cause(&best, &slow, 0.0, 1.0 / 3.0);
        assert_eq!(cause, Some(Cause::HighSteering));
    }

    #[test]
    fn no_cause_when_traces_match() {
        let best = flat_trace(1.0, 0.0, 0.05);
        let slow = flat_trace(1.0, 0.0, 0.05);
        assert_eq!(detect_cause(&best, &slow, 0.0, 1.0 / 3.0), None);
    }

    #[test]
    fn appends_insight_for_lost_sector() {
        let best = lap(1, 1, 90_000.0, vec![(1, 30_000.0), (2, 30_000.0), (3, 30_000.0)]);
        let slow = lap(2, 2, 91_000.0, vec![(1, 31_000.0), (2, 30_000.0), (3, 30_000.0)]);
        let detail = detail(vec![best, slow]);

        let mut traces = HashMap::new();
        traces.insert(1, flat_trace(1.0, 0.0, 0.0));
        traces.insert(2, {
            // Slow lap lifts throttle in sector 1 (dist 0..0.33).
            let mut t = flat_trace(1.0, 0.0, 0.0);
            for p in t.iter_mut() {
                if p.dist_pct < 1.0 / 3.0 {
                    p.throttle = 0.2;
                }
            }
            t
        });

        let mut insights = Vec::new();
        append_trace_insights(&mut insights, &detail, &traces);

        assert!(insights.iter().any(|i| i.kind == "early_lift" && i.sector_num == Some(1)));
    }

    #[test]
    fn select_lap_ids_includes_best_and_slow() {
        let best = lap(1, 1, 90_000.0, vec![(1, 30_000.0)]);
        let slow = lap(2, 2, 91_000.0, vec![(1, 31_000.0)]);
        let detail = detail(vec![best, slow]);
        let ids = select_trace_lap_ids(&detail);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }
}
