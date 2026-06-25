use crate::storage::models::LapKind;

use super::lap_segmenter::{lap_completed, lap_dist_range};
use super::types::RawFrame;

const PIT_SAMPLE_CAP: usize = 30;
const PIT_SAMPLE_FRACTION: f64 = 0.05;
const PIT_LANE_RATIO: f64 = 0.85;
const PIT_MEANINGFUL_RATIO: f64 = 0.15;

/// Classify a lap from per-frame `OnPitRoad` and distance coverage. Independent of
/// validity — a pit-out lap may be invalid while still labeled `PitOut`.
pub fn classify_lap_kind(frames: &[RawFrame]) -> LapKind {
    if frames.is_empty() {
        return LapKind::Partial;
    }

    let pit_frames = frames.iter().filter(|f| f.on_pit_road).count();
    let pit_ratio = pit_frames as f64 / frames.len() as f64;
    let start_pit = sample_pit_majority(frames, true);
    let end_pit = sample_pit_majority(frames, false);
    let (min_pct, max_pct) = lap_dist_range(frames);

    if pit_ratio > PIT_LANE_RATIO || (start_pit && end_pit && pit_ratio > PIT_MEANINGFUL_RATIO) {
        return LapKind::PitLane;
    }
    if start_pit && !end_pit {
        return LapKind::PitOut;
    }
    if !start_pit && end_pit {
        return LapKind::PitIn;
    }
    if !lap_completed(min_pct, max_pct) {
        return LapKind::Partial;
    }
    LapKind::Flying
}

fn sample_pit_majority(frames: &[RawFrame], from_start: bool) -> bool {
    let sample_len = ((frames.len() as f64 * PIT_SAMPLE_FRACTION).ceil() as usize)
        .clamp(1, PIT_SAMPLE_CAP)
        .min(frames.len());
    let slice = if from_start {
        &frames[..sample_len]
    } else {
        &frames[frames.len() - sample_len..]
    };
    let pit_count = slice.iter().filter(|f| f.on_pit_road).count();
    pit_count * 2 >= slice.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lap_segmenter::is_valid_lap;
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

    fn frames_with_pit_regions(
        count: usize,
        start: f32,
        end: f32,
        pit_until_idx: usize,
        pit_from_idx: usize,
    ) -> Vec<RawFrame> {
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
                    on_pit_road: i < pit_until_idx || i >= pit_from_idx,
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
    fn flying_lap() {
        let frames = lap_frames(900, 0.0, 0.98, false);
        assert_eq!(classify_lap_kind(&frames), LapKind::Flying);
        assert!(is_valid_lap(&frames, Some(91_700.0)));
    }

    #[test]
    fn pit_out_lap() {
        let frames = frames_with_pit_regions(900, 0.0, 0.98, 200, 900);
        assert_eq!(classify_lap_kind(&frames), LapKind::PitOut);
    }

    #[test]
    fn pit_in_lap() {
        let frames = frames_with_pit_regions(900, 0.0, 0.98, 0, 700);
        assert_eq!(classify_lap_kind(&frames), LapKind::PitIn);
    }

    #[test]
    fn pit_lane_lap() {
        let frames = lap_frames(300, 0.0, 0.5, true);
        assert_eq!(classify_lap_kind(&frames), LapKind::PitLane);
    }

    #[test]
    fn partial_lap_okayama_style() {
        let frames = lap_frames(443, 0.0, 0.37, false);
        assert_eq!(classify_lap_kind(&frames), LapKind::Partial);
    }

    #[test]
    fn partial_lap_barcelona_style() {
        let frames = lap_frames(855, 0.0, 0.852, false);
        assert_eq!(classify_lap_kind(&frames), LapKind::Partial);
    }

    #[test]
    fn lap_kind_round_trip() {
        for kind in [
            LapKind::Flying,
            LapKind::PitOut,
            LapKind::PitIn,
            LapKind::PitLane,
            LapKind::Partial,
        ] {
            assert_eq!(LapKind::from_str(kind.as_str()), Some(kind));
        }
    }
}
