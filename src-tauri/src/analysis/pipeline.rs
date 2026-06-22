use std::collections::HashMap;

use rayon::prelude::*;

use super::fuel_tire::{fuel_stats, tire_averages};
use super::lap_segmenter::{
    average_speed, compute_lap_time_ms, downsample_traces, is_valid_lap, segment_laps,
};
use super::sector_splitter::compute_sector_times;
use super::types::{LapFrames, RawFrame, SectorBoundary};
use crate::storage::StoredLap;

pub fn analyze_session(
    frames: Vec<RawFrame>,
    sector_boundaries: Vec<SectorBoundary>,
    session_labels: HashMap<i32, String>,
) -> Vec<StoredLap> {
    let lap_groups = segment_laps(frames, &session_labels);

    lap_groups
        .into_par_iter()
        .map(|group| analyze_lap(group, &sector_boundaries))
        .collect()
}

fn analyze_lap(group: LapFrames, sector_boundaries: &[SectorBoundary]) -> StoredLap {
    let lap_time_ms = compute_lap_time_ms(&group.frames);
    let valid = is_valid_lap(&group.frames, lap_time_ms);
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

    StoredLap {
        session_num: group.session_num,
        session_type: group.session_type,
        iracing_lap: group.iracing_lap,
        lap_number: group.lap_number,
        lap_time_ms,
        valid,
        fuel_start,
        fuel_used,
        avg_speed: average_speed(&group.frames),
        lf_temp,
        rf_temp,
        lr_temp,
        rr_temp,
        sectors,
        traces,
    }
}
