use std::collections::HashMap;

use super::lap_kind::classify_lap_kind;
use super::lap_validity::{
    include_in_stats_ibt, metrics_from_frames, telemetry_passes_heuristics,
};
use super::types::{LapFrames, RawFrame};

const DOWNSAMPLE_EVERY: usize = 6;

pub fn segment_laps(frames: Vec<RawFrame>, session_labels: &HashMap<i32, String>) -> Vec<LapFrames> {
    if frames.is_empty() {
        return Vec::new();
    }

    let mut laps: Vec<LapFrames> = Vec::new();
    let mut current_session = frames[0].session_num;
    let mut current_lap = frames[0].lap;
    let mut bucket: Vec<RawFrame> = Vec::new();

    for frame in frames {
        let session_changed = frame.session_num != current_session;
        let lap_changed = frame.lap != current_lap;
        if (session_changed || lap_changed) && !bucket.is_empty() {
            laps.push(finish_lap_bucket(
                current_session,
                session_labels,
                current_lap,
                bucket,
            ));
            bucket = Vec::new();
            current_session = frame.session_num;
            current_lap = frame.lap;
        }
        bucket.push(frame);
    }

    if !bucket.is_empty() {
        laps.push(finish_lap_bucket(
            current_session,
            session_labels,
            current_lap,
            bucket,
        ));
    }

    assign_lap_numbers(&mut laps);
    laps
}

fn finish_lap_bucket(
    session_num: i32,
    session_labels: &HashMap<i32, String>,
    iracing_lap: i32,
    frames: Vec<RawFrame>,
) -> LapFrames {
    LapFrames {
        session_num,
        session_type: session_labels
            .get(&session_num)
            .cloned()
            .unwrap_or_else(|| format!("Session {session_num}")),
        iracing_lap,
        lap_number: 0,
        frames,
    }
}

/// Number laps 1..N separately for each iRacing sub-session (practice / qual / race).
fn assign_lap_numbers(laps: &mut [LapFrames]) {
    let mut counters: HashMap<i32, i32> = HashMap::new();
    for lap in laps {
        let counter = counters.entry(lap.session_num).or_insert(0);
        *counter += 1;
        lap.lap_number = *counter;
    }
}

/// Minimum `lap_dist_pct` a completed lap must reach (start/finish line). Telemetry
/// is sampled at finite Hz so legitimate laps peak around 0.998 rather than 1.0.
pub const MIN_LAP_MAX_PCT: f32 = 0.95;

/// Some IBT sessions never sample past ~0.87 even on complete laps (e.g. Okayama).
/// Fallback requires both reaching this distance and spanning most of the lap.
const MIN_LAP_MAX_PCT_FALLBACK: f32 = 0.87;
const MIN_LAP_SPAN_FALLBACK: f32 = 0.85;

/// IBT `valid` = flying lap with telemetry heuristics (`include_in_stats_ibt`).
pub fn is_valid_lap(frames: &[RawFrame], lap_time_ms: Option<f64>) -> bool {
    let kind = classify_lap_kind(frames);
    let metrics = metrics_from_frames(frames, lap_time_ms);
    include_in_stats_ibt(kind, telemetry_passes_heuristics(&metrics))
}

/// Whether the car reached the start/finish line on this lap.
pub fn lap_completed(min_pct: f32, max_pct: f32) -> bool {
    if !min_pct.is_finite() || !max_pct.is_finite() {
        return false;
    }
    // Race lap straddling the start/finish line: distance ran on both ends of the loop.
    if min_pct < 0.1 && max_pct > 0.9 {
        return true;
    }
    if max_pct >= MIN_LAP_MAX_PCT {
        return true;
    }
    // Fallback for sessions where telemetry peaks below 0.95 on legitimate laps.
    let span = max_pct - min_pct;
    max_pct >= MIN_LAP_MAX_PCT_FALLBACK && span >= MIN_LAP_SPAN_FALLBACK
}

pub fn lap_dist_range(frames: &[RawFrame]) -> (f32, f32) {
    frames.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), f| {
        (lo.min(f.lap_dist_pct), hi.max(f.lap_dist_pct))
    })
}

pub fn max_lap_dist_pct(frames: &[RawFrame]) -> f32 {
    lap_dist_range(frames).1
}

pub fn downsample_traces(frames: &[RawFrame]) -> Vec<crate::storage::TracePoint> {
    frames
        .iter()
        .enumerate()
        .filter(|(i, _)| i % DOWNSAMPLE_EVERY == 0)
        .map(|(_, f)| crate::storage::TracePoint {
            dist_pct: f.lap_dist_pct as f64,
            speed: f.speed as f64,
            throttle: f.throttle as f64,
            brake: f.brake as f64,
            gear: f.gear,
            steering: f.steering as f64,
        })
        .collect()
}

pub fn compute_lap_time_ms(frames: &[RawFrame]) -> Option<f64> {
    if frames.len() < 2 {
        return None;
    }
    let start = frames.first()?.session_time;
    let end = frames.last()?.session_time;
    let delta = end - start;
    if delta > 0.0 {
        Some(delta * 1000.0)
    } else {
        None
    }
}

pub fn average_speed(frames: &[RawFrame]) -> Option<f64> {
    if frames.is_empty() {
        return None;
    }
    let sum: f64 = frames.iter().map(|f| f.speed as f64).sum();
    Some(sum / frames.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build `count` frames sweeping `lap_dist_pct` linearly from `start` to `end`.
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
    fn partial_lap_is_invalid() {
        // Okayama lap 2: only reaches ~37% of the track before the lap counter ticks.
        let frames = lap_frames(443, 0.0, 0.37, false);
        assert!(!is_valid_lap(&frames, Some(44_200.0)));
    }

    #[test]
    fn near_complete_lap_without_finish_is_invalid() {
        // Barcelona lap 12: reaches ~85% but never crosses the finish line.
        let frames = lap_frames(855, 0.0, 0.852, false);
        assert!(!is_valid_lap(&frames, Some(85_433.0)));
    }

    #[test]
    fn full_lap_is_valid() {
        let frames = lap_frames(900, 0.0, 0.98, false);
        assert!(is_valid_lap(&frames, Some(91_700.0)));
    }

    #[test]
    fn lap_wrapping_start_finish_is_valid() {
        // Race lap that crosses the start/finish line: starts near 0.97, wraps to 0.95.
        let mut frames = lap_frames(450, 0.97, 1.0, false);
        frames.extend(lap_frames(450, 0.0, 0.95, false));
        assert!(is_valid_lap(&frames, Some(90_000.0)));
    }

    #[test]
    fn lap_completed_requires_max_pct() {
        assert!(!lap_completed(0.0, 0.852));
        assert!(lap_completed(0.0, 0.98));
        assert!(lap_completed(0.05, 0.97));
        // Okayama-style: telemetry peaks below 0.95 but spans most of the lap.
        assert!(lap_completed(0.0, 0.873));
    }
}
