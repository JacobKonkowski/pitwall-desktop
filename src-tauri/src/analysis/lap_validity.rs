use crate::storage::models::LapKind;

use super::lap_segmenter::lap_completed;
use super::types::RawFrame;

/// Per-lap telemetry summary used for IBT heuristic validity checks.
#[derive(Debug, Clone, PartialEq)]
pub struct LapTelemetryMetrics {
    pub frame_count: usize,
    pub pit_frame_count: usize,
    pub lap_time_ms: Option<f64>,
    pub min_dist_pct: f32,
    pub max_dist_pct: f32,
}

pub fn metrics_from_frames(frames: &[RawFrame], lap_time_ms: Option<f64>) -> LapTelemetryMetrics {
    let pit_frame_count = frames.iter().filter(|f| f.on_pit_road).count();
    let (min_dist_pct, max_dist_pct) = super::lap_segmenter::lap_dist_range(frames);
    LapTelemetryMetrics {
        frame_count: frames.len(),
        pit_frame_count,
        lap_time_ms,
        min_dist_pct,
        max_dist_pct,
    }
}

/// IBT import heuristics: completion, pit ratio, time range, frame count.
pub fn telemetry_passes_heuristics(metrics: &LapTelemetryMetrics) -> bool {
    if metrics.frame_count < 30 {
        return false;
    }
    let pit_ratio = metrics.pit_frame_count as f64 / metrics.frame_count as f64;
    if pit_ratio > 0.15 {
        return false;
    }
    match metrics.lap_time_ms {
        Some(ms) if ms >= 10_000.0 && ms <= 600_000.0 => {}
        _ => return false,
    }
    lap_completed(metrics.min_dist_pct, metrics.max_dist_pct)
}

/// Stored DB `valid` column: flying laps that pass telemetry heuristics.
pub fn include_in_stats_ibt(kind: LapKind, telemetry_ok: bool) -> bool {
    kind == LapKind::Flying && telemetry_ok
}

/// Live coach `lastLapValid`: flying, completed, and iRacing OK flags.
pub fn include_in_stats_live(kind: LapKind, completed: bool, iracing_ok: bool) -> bool {
    kind == LapKind::Flying && completed && iracing_ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::types::RawFrame;
    use crate::storage::models::LapKind;

    fn lap_frames(count: usize, start: f32, end: f32, on_pit_road: bool) -> Vec<RawFrame> {
        (0..count)
            .map(|i| {
                let t = i as f64 / (count.max(2) - 1) as f64;
                RawFrame {
                    session_num: 0,
                    lap: 1,
                    lap_dist_pct: start + (end - start) * t as f32,
                    speed: 50.0,
                    throttle: 0.0,
                    brake: 0.0,
                    steering: 0.0,
                    gear: 3,
                    fuel_level: 50.0,
                    on_pit_road,
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
    fn telemetry_heuristics_reject_partial_lap() {
        let frames = lap_frames(443, 0.0, 0.37, false);
        let metrics = metrics_from_frames(&frames, Some(44_200.0));
        assert!(!telemetry_passes_heuristics(&metrics));
    }

    #[test]
    fn telemetry_heuristics_accept_full_lap() {
        let frames = lap_frames(900, 0.0, 0.98, false);
        let metrics = metrics_from_frames(&frames, Some(91_700.0));
        assert!(telemetry_passes_heuristics(&metrics));
    }

    #[test]
    fn include_in_stats_ibt_requires_flying() {
        assert!(!include_in_stats_ibt(LapKind::PitOut, true));
        assert!(include_in_stats_ibt(LapKind::Flying, true));
        assert!(!include_in_stats_ibt(LapKind::Flying, false));
    }

    #[test]
    fn include_in_stats_live_requires_all_three() {
        assert!(include_in_stats_live(LapKind::Flying, true, true));
        assert!(!include_in_stats_live(LapKind::PitOut, true, true));
        assert!(!include_in_stats_live(LapKind::Flying, false, true));
        assert!(!include_in_stats_live(LapKind::Flying, true, false));
    }

    #[test]
    fn classify_from_signals_pit_out() {
        use super::super::lap_kind::classify_from_signals;
        assert_eq!(
            classify_from_signals(0.05, true, true, false),
            LapKind::PitOut
        );
    }

    #[test]
    fn classify_from_signals_flying() {
        use super::super::lap_kind::classify_from_signals;
        assert_eq!(
            classify_from_signals(0.0, true, false, false),
            LapKind::Flying
        );
    }
}
