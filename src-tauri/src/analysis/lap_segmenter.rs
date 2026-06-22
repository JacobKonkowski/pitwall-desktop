use std::collections::HashMap;

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

pub fn is_valid_lap(frames: &[RawFrame], lap_time_ms: Option<f64>) -> bool {
    if frames.len() < 30 {
        return false;
    }
    let pit_ratio = frames.iter().filter(|f| f.on_pit_road).count() as f64 / frames.len() as f64;
    if pit_ratio > 0.15 {
        return false;
    }
    if let Some(ms) = lap_time_ms {
        if ms < 10_000.0 || ms > 600_000.0 {
            return false;
        }
    } else {
        return false;
    }
    true
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
