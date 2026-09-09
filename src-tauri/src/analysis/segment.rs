//! Frame -> lap segmentation.
//!
//! Laps are split on any change of `(SessionNum, Lap)`. The official lap time and
//! the `_OK` flags for a completed lap are published by iRacing on the *next*
//! lap's first frame, so we sample them there when closing each bucket.

use std::collections::HashMap;

use super::types::{LapFrames, RawFrame};

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
            // The transition frame carries the completed lap's official time and
            // the sim's validity flags for it.
            let sdk_ms = sdk_lap_time_ms(frame.lap_last_lap_time);
            laps.push(finish_bucket(
                current_session,
                session_labels,
                current_lap,
                std::mem::take(&mut bucket),
                sdk_ms,
                frame.delta_best_ok,
                frame.delta_session_best_ok,
            ));
            current_session = frame.session_num;
            current_lap = frame.lap;
        }
        bucket.push(frame);
    }

    if !bucket.is_empty() {
        // Final lap: no following frame, so no official time or flags.
        laps.push(finish_bucket(
            current_session,
            session_labels,
            current_lap,
            bucket,
            None,
            None,
            None,
        ));
    }

    assign_lap_numbers(&mut laps);
    laps
}

#[allow(clippy::too_many_arguments)]
fn finish_bucket(
    session_num: i32,
    session_labels: &HashMap<i32, String>,
    iracing_lap: i32,
    frames: Vec<RawFrame>,
    sdk_lap_time_ms: Option<f64>,
    delta_best_ok: Option<bool>,
    delta_session_best_ok: Option<bool>,
) -> LapFrames {
    LapFrames {
        session_num,
        session_type: session_labels
            .get(&session_num)
            .cloned()
            .unwrap_or_else(|| format!("Session {session_num}")),
        iracing_lap,
        lap_number: 0,
        sdk_lap_time_ms,
        delta_best_ok,
        delta_session_best_ok,
        frames,
    }
}

/// Number laps 1..N separately within each sub-session.
fn assign_lap_numbers(laps: &mut [LapFrames]) {
    let mut counters: HashMap<i32, i32> = HashMap::new();
    for lap in laps {
        let counter = counters.entry(lap.session_num).or_insert(0);
        *counter += 1;
        lap.lap_number = *counter;
    }
}

/// iRacing reports lap times in seconds (`f32`); a negative value means unset.
/// Convert to integer milliseconds, rounding to the nearest ms to absorb `f32`
/// representation error rather than biasing every time downward.
fn sdk_lap_time_ms(secs: Option<f32>) -> Option<f64> {
    match secs {
        Some(s) if s > 0.0 => Some((s as f64 * 1000.0).round()),
        _ => None,
    }
}

/// Minimum and maximum `LapDistPct` observed across the frames.
pub fn lap_dist_range(frames: &[RawFrame]) -> (f32, f32) {
    frames
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), f| {
            (lo.min(f.lap_dist_pct), hi.max(f.lap_dist_pct))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(session_num: i32, lap: i32, pct: f32, last: Option<f32>, ok: Option<bool>) -> RawFrame {
        RawFrame {
            session_num,
            lap,
            lap_dist_pct: pct,
            speed: 50.0,
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            gear: 3,
            fuel_level: 50.0,
            on_pit_road: false,
            session_time: 0.0,
            lap_last_lap_time: last,
            delta_best_ok: ok,
            delta_session_best_ok: ok,
            lf_temp: 0.0,
            rf_temp: 0.0,
            lr_temp: 0.0,
            rr_temp: 0.0,
        }
    }

    #[test]
    fn splits_on_lap_change_and_samples_transition() {
        let labels = HashMap::from([(0, "Practice".to_string())]);
        let frames = vec![
            frame(0, 1, 0.1, None, None),
            frame(0, 1, 0.9, None, None),
            // transition into lap 2 carries lap 1's official time + OK flags
            frame(0, 2, 0.05, Some(82.653), Some(true)),
            frame(0, 2, 0.9, None, None),
        ];
        let laps = segment_laps(frames, &labels);
        assert_eq!(laps.len(), 2);
        assert_eq!(laps[0].lap_number, 1);
        assert_eq!(laps[0].sdk_lap_time_ms, Some(82_653.0));
        assert_eq!(laps[0].delta_best_ok, Some(true));
        // Final lap has no following frame => no time.
        assert_eq!(laps[1].sdk_lap_time_ms, None);
        assert_eq!(laps[1].delta_best_ok, None);
    }

    #[test]
    fn negative_last_lap_time_is_unset() {
        let labels = HashMap::new();
        let frames = vec![
            frame(0, 1, 0.1, None, None),
            frame(0, 2, 0.05, Some(-1.0), Some(false)),
        ];
        let laps = segment_laps(frames, &labels);
        assert_eq!(laps[0].sdk_lap_time_ms, None);
        assert_eq!(laps[0].delta_best_ok, Some(false));
    }

    #[test]
    fn per_subsession_lap_numbering() {
        let labels = HashMap::new();
        let frames = vec![
            frame(0, 1, 0.5, None, None),
            frame(0, 2, 0.5, Some(60.0), Some(true)),
            frame(1, 1, 0.5, None, None),
            frame(1, 2, 0.5, Some(61.0), Some(true)),
        ];
        let laps = segment_laps(frames, &labels);
        assert_eq!(laps.len(), 4);
        assert_eq!(laps[0].lap_number, 1);
        assert_eq!(laps[1].lap_number, 2);
        assert_eq!(laps[2].session_num, 1);
        assert_eq!(laps[2].lap_number, 1);
    }
}
