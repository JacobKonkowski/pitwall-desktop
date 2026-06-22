use super::types::{RawFrame, SectorBoundary};

const MIN_SECTOR_MS: f64 = 0.5;

/// Normalize iRacing sector boundaries. Session YAML includes sector 0 at 0% (start/finish);
/// only sector_num > 0 are split lines where sector N marks the end of sector N timing.
pub fn normalize_sector_boundaries(boundaries: &[SectorBoundary]) -> Vec<SectorBoundary> {
    let mut sorted: Vec<SectorBoundary> = boundaries
        .iter()
        .filter(|b| b.sector_num > 0)
        .cloned()
        .collect();
    if sorted.is_empty() {
        sorted = default_split_boundaries();
    }
    sorted.sort_by(|a, b| {
        a.start_pct
            .partial_cmp(&b.start_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted
}

pub fn default_split_boundaries() -> Vec<SectorBoundary> {
    vec![
        SectorBoundary {
            sector_num: 1,
            start_pct: 0.33,
        },
        SectorBoundary {
            sector_num: 2,
            start_pct: 0.66,
        },
    ]
}

pub fn compute_sector_times(frames: &[RawFrame], boundaries: &[SectorBoundary]) -> Vec<(i32, f64)> {
    if frames.len() < 2 {
        return Vec::new();
    }

    let bounds = normalize_sector_boundaries(boundaries);
    let start_time = frames.first().map(|f| f.session_time).unwrap_or(0.0);
    let first_pct = frames.first().map(|f| f.lap_dist_pct as f64).unwrap_or(0.0);

    let mut crossings: Vec<(i32, f64)> = Vec::new();
    let mut next_idx = 0usize;

    // Lap buckets often start slightly after 0%; skip split lines already passed.
    while next_idx < bounds.len() && first_pct >= bounds[next_idx].start_pct {
        next_idx += 1;
    }

    for window in frames.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        while next_idx < bounds.len() {
            let boundary = &bounds[next_idx];
            let pct = boundary.start_pct;
            let crossed =
                prev.lap_dist_pct as f64 <= pct && curr.lap_dist_pct as f64 > pct;
            if crossed {
                let ratio = if (curr.lap_dist_pct - prev.lap_dist_pct).abs() > f32::EPSILON {
                    ((pct - prev.lap_dist_pct as f64)
                        / (curr.lap_dist_pct - prev.lap_dist_pct) as f64)
                        .clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let time = prev.session_time + (curr.session_time - prev.session_time) * ratio;
                crossings.push((boundary.sector_num, (time - start_time) * 1000.0));
                next_idx += 1;
            } else {
                break;
            }
        }
    }

    let mut sector_times = Vec::new();
    let mut prev_ms = 0.0;
    for (sector_num, cumulative_ms) in crossings {
        let split = (cumulative_ms - prev_ms).max(0.0);
        if split >= MIN_SECTOR_MS {
            sector_times.push((sector_num, split));
        }
        prev_ms = cumulative_ms;
    }

    if let Some(last_frame) = frames.last() {
        let total_ms = (last_frame.session_time - start_time) * 1000.0;
        let final_sector = bounds.last().map(|b| b.sector_num + 1).unwrap_or(1);
        let final_split = (total_ms - prev_ms).max(0.0);
        if final_split >= MIN_SECTOR_MS {
            sector_times.push((final_sector, final_split));
        }
    }

    sector_times
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(pct: f32, session_time: f64) -> RawFrame {
        RawFrame {
            session_num: 0,
            lap: 1,
            lap_dist_pct: pct,
            speed: 0.0,
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            gear: 0,
            fuel_level: 0.0,
            on_pit_road: false,
            session_time,
            lf_temp: 0.0,
            rf_temp: 0.0,
            lr_temp: 0.0,
            rr_temp: 0.0,
        }
    }

    #[test]
    fn ignores_sector_zero_start_line() {
        let boundaries = vec![
            SectorBoundary {
                sector_num: 0,
                start_pct: 0.0,
            },
            SectorBoundary {
                sector_num: 1,
                start_pct: 0.34,
            },
            SectorBoundary {
                sector_num: 2,
                start_pct: 0.72,
            },
        ];
        let frames: Vec<_> = (0..100)
            .map(|i| {
                let pct = i as f32 / 99.0;
                frame(pct, pct as f64 * 100.0)
            })
            .collect();

        let sectors = compute_sector_times(&frames, &boundaries);
        assert_eq!(sectors.len(), 3);
        assert_eq!(sectors[0].0, 1);
        assert_eq!(sectors[1].0, 2);
        assert_eq!(sectors[2].0, 3);
        let total: f64 = sectors.iter().map(|(_, ms)| ms).sum();
        assert!((total - 100_000.0).abs() < 500.0);
    }
}
