//! Per-lap aggregates computed directly from frames: fuel usage, tire-temp
//! averages, average speed, and downsampled traces for charts/comparison.

use super::types::{RawFrame, TracePoint};

/// Keep every Nth frame for chart traces (~10 Hz -> ~1.6 Hz at 6).
const DOWNSAMPLE_EVERY: usize = 6;

/// `(fuel_start, fuel_used)` in liters from the first/last frame fuel level.
pub fn fuel_stats(frames: &[RawFrame]) -> (Option<f64>, Option<f64>) {
    let first = frames.first().map(|f| f.fuel_level as f64);
    let last = frames.last().map(|f| f.fuel_level as f64);
    match (first, last) {
        (Some(start), Some(end)) => (Some(start), Some((start - end).max(0.0))),
        _ => (None, None),
    }
}

/// `(lf, rf, lr, rr)` mean tire temps, `None` when there are no frames.
pub fn tire_averages(
    frames: &[RawFrame],
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if frames.is_empty() {
        return (None, None, None, None);
    }
    let n = frames.len() as f64;
    let sum = frames.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, f| {
        (
            acc.0 + f.lf_temp as f64,
            acc.1 + f.rf_temp as f64,
            acc.2 + f.lr_temp as f64,
            acc.3 + f.rr_temp as f64,
        )
    });
    (
        Some(sum.0 / n),
        Some(sum.1 / n),
        Some(sum.2 / n),
        Some(sum.3 / n),
    )
}

/// Mean speed (m/s), `None` when there are no frames.
pub fn average_speed(frames: &[RawFrame]) -> Option<f64> {
    if frames.is_empty() {
        return None;
    }
    let sum: f64 = frames.iter().map(|f| f.speed as f64).sum();
    Some(sum / frames.len() as f64)
}

/// Downsample frames into chart trace points.
pub fn downsample_traces(frames: &[RawFrame]) -> Vec<TracePoint> {
    frames
        .iter()
        .enumerate()
        .filter(|(i, _)| i % DOWNSAMPLE_EVERY == 0)
        .map(|(_, f)| TracePoint {
            dist_pct: f.lap_dist_pct as f64,
            speed: f.speed as f64,
            throttle: f.throttle as f64,
            brake: f.brake as f64,
            gear: f.gear,
            steering: f.steering as f64,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(fuel: f32, speed: f32, temp: f32) -> RawFrame {
        RawFrame {
            session_num: 0,
            lap: 1,
            lap_dist_pct: 0.0,
            speed,
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            gear: 3,
            fuel_level: fuel,
            on_pit_road: false,
            session_time: 0.0,
            lap_last_lap_time: None,
            delta_best_ok: None,
            delta_session_best_ok: None,
            lf_temp: temp,
            rf_temp: temp,
            lr_temp: temp,
            rr_temp: temp,
        }
    }

    #[test]
    fn fuel_used_is_start_minus_end() {
        let frames = vec![frame(50.0, 0.0, 0.0), frame(48.5, 0.0, 0.0)];
        let (start, used) = fuel_stats(&frames);
        assert_eq!(start, Some(50.0));
        assert!((used.unwrap() - 1.5).abs() < 1e-6);
    }

    #[test]
    fn average_speed_and_temps() {
        let frames = vec![frame(50.0, 40.0, 80.0), frame(50.0, 60.0, 90.0)];
        assert_eq!(average_speed(&frames), Some(50.0));
        let (lf, _, _, _) = tire_averages(&frames);
        assert_eq!(lf, Some(85.0));
    }
}
