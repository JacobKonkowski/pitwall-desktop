use std::collections::{HashMap, HashSet};

use crate::live::LiveSnapshot;
use crate::settings::AppSettings;

/// Tracks what we've already spoken so alerts stay useful, not repetitive.
pub struct CoachEngine {
    tracked_lap: i32,
    announced_sectors: HashSet<i32>,
    last_lap_time_announced: Option<f64>,
    best_sector_ms: HashMap<i32, f64>,
    best_lap_ms: Option<f64>,
    fuel_at_lap_start: Option<f32>,
    fuel_per_lap: Vec<f32>,
    spoke_session_intro: bool,
    spoke_fuel_low: bool,
}

impl CoachEngine {
    pub fn new() -> Self {
        Self {
            tracked_lap: 0,
            announced_sectors: HashSet::new(),
            last_lap_time_announced: None,
            best_sector_ms: HashMap::new(),
            best_lap_ms: None,
            fuel_at_lap_start: None,
            fuel_per_lap: Vec::new(),
            spoke_session_intro: false,
            spoke_fuel_low: false,
        }
    }

    pub fn poll(&mut self, snap: &LiveSnapshot, settings: &AppSettings) -> Vec<String> {
        let mut out = Vec::new();

        if snap.lap <= 0 && snap.track.is_empty() {
            return out;
        }

        if !self.spoke_session_intro && !snap.track.is_empty() {
            self.spoke_session_intro = true;
            out.push(format!(
                "PitWall coach online. {}. {}",
                snap.track,
                if snap.session_type.is_empty() {
                    "Good luck.".into()
                } else {
                    format!("{} session.", snap.session_type)
                }
            ));
        }

        if snap.lap != self.tracked_lap {
            self.tracked_lap = snap.lap;
            self.announced_sectors.clear();
            if snap.lap > 0 && self.fuel_at_lap_start.is_none() {
                self.fuel_at_lap_start = Some(snap.fuel_level);
            }
        }

        for sector in &snap.sectors {
            if !sector.completed || self.announced_sectors.contains(&sector.sector_num) {
                continue;
            }
            if let Some(msg) = self.announce_sector(snap, sector.sector_num, sector.time_ms) {
                out.push(msg);
            }
            self.announced_sectors.insert(sector.sector_num);
        }

        // Lap completed (last_lap_ms updated)
        if let Some(lap_ms) = snap.last_lap_ms {
            if self.last_lap_time_announced != Some(lap_ms) && lap_ms > 10_000.0 {
                let completed_lap = snap.lap.saturating_sub(1).max(1);
                if let Some(msg) = self.announce_lap_complete(snap, completed_lap, lap_ms) {
                    out.push(msg);
                }
                self.last_lap_time_announced = Some(lap_ms);
                self.announced_sectors.clear();
                self.fuel_at_lap_start = Some(snap.fuel_level);
            }
        }

        // Fuel alerts
        if settings.audio_coach_fuel_threshold > 0.0 && snap.fuel_level > 0.0 {
            if let Some(msg) = self.announce_fuel(snap, settings.audio_coach_fuel_threshold) {
                out.push(msg);
            }
        }

        out
    }

    fn announce_sector(&mut self, snap: &LiveSnapshot, sector_num: i32, time_ms: Option<f64>) -> Option<String> {
        let Some(ms) = time_ms else {
            return None;
        };
        if ms < 1000.0 {
            return None;
        }

        let prev_best = self.best_sector_ms.get(&sector_num).copied();
        let is_pb = prev_best.map(|b| ms < b - 20.0).unwrap_or(true);
        if is_pb {
            self.best_sector_ms.insert(sector_num, ms);
        }

        let time_str = format_duration_short(ms);
        let delta_str = prev_best.and_then(|b| {
            if is_pb && b > ms + 20.0 {
                Some(format!(" {:.0} tenths quicker than before.", (b - ms) / 100.0))
            } else if !is_pb {
                Some(format_delta_phrase(ms - b))
            } else {
                None
            }
        });

        let live_delta = snap.delta_to_best_ms.filter(|d| d.abs() > 80.0 && snap.lap_dist_pct > 0.05);
        let pace_hint = live_delta.map(|d| {
            if d > 0.0 {
                format!(" Currently {:.0} tenths off best lap pace.", d / 100.0)
            } else {
                " On personal best pace.".into()
            }
        });

        let mut msg = if is_pb && prev_best.is_some() {
            format!("Sector {sector_num}. {time_str}. New best sector.")
        } else if is_pb {
            format!("Sector {sector_num}. {time_str}.")
        } else {
            format!(
                "Sector {sector_num}. {time_str}.{}",
                delta_str.unwrap_or_default()
            )
        };

        if let Some(hint) = pace_hint {
            msg.push_str(&hint);
        }

        Some(msg)
    }

    fn announce_lap_complete(&mut self, snap: &LiveSnapshot, lap_num: i32, lap_ms: f64) -> Option<String> {
        let prev_best = self.best_lap_ms;
        let is_pb = prev_best.map(|b| lap_ms < b - 50.0).unwrap_or(true);
        if is_pb {
            self.best_lap_ms = Some(lap_ms);
        }

        // Fuel use this lap
        if let Some(start_fuel) = self.fuel_at_lap_start {
            if start_fuel > snap.fuel_level && start_fuel - snap.fuel_level > 0.05 {
                let used = start_fuel - snap.fuel_level;
                self.fuel_per_lap.push(used);
                if self.fuel_per_lap.len() > 8 {
                    self.fuel_per_lap.remove(0);
                }
            }
        }

        let time_str = format_duration_long(lap_ms);
        let mut parts = vec![format!("Lap {lap_num}. {time_str}.")];

        if is_pb && prev_best.is_some() {
            parts.push("New personal best.".into());
        } else if let Some(best) = prev_best {
            parts.push(format_delta_phrase(lap_ms - best));
        }

        if let Some(last) = snap.delta_to_last_ms {
            if last.abs() > 80.0 && prev_best.is_some() && !is_pb {
                parts.push(format!(
                    "{} versus previous lap.",
                    format_delta_phrase(last).trim()
                ));
            }
        }

        if snap.fuel_level > 0.0 {
            if let Some(laps_left) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) {
                if laps_left <= 3.0 {
                    parts.push(format!(
                        "Fuel {:.0} liters. Pit in {:.0} laps.",
                        snap.fuel_level,
                        laps_left.ceil()
                    ));
                } else {
                    parts.push(format!(
                        "Fuel {:.0} liters. About {:.0} laps remaining.",
                        snap.fuel_level,
                        laps_left.round()
                    ));
                }
            } else {
                parts.push(format!("Fuel {:.0} liters.", snap.fuel_level));
            }
        }

        Some(parts.join(" "))
    }

    fn announce_fuel(&mut self, snap: &LiveSnapshot, threshold: f32) -> Option<String> {
        if self.spoke_fuel_low || snap.fuel_level > threshold {
            return None;
        }
        self.spoke_fuel_low = true;

        let mut msg = format!(
            "Fuel low. {:.0} liters left.",
            snap.fuel_level
        );

        if let Some(laps_left) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) {
            if laps_left <= 1.5 {
                msg.push_str(" Pit this lap or next.");
            } else {
                msg.push_str(&format!(" About {:.0} laps of fuel.", laps_left.round()));
            }
        }

        Some(msg)
    }
}

fn estimate_laps_remaining(fuel_level: f32, fuel_per_lap: &[f32]) -> Option<f32> {
    if fuel_per_lap.is_empty() || fuel_level <= 0.0 {
        return None;
    }
    let avg: f32 = fuel_per_lap.iter().sum::<f32>() / fuel_per_lap.len() as f32;
    if avg < 0.05 {
        return None;
    }
    Some(fuel_level / avg)
}

fn format_duration_short(ms: f64) -> String {
    let total = ms / 1000.0;
    if total >= 60.0 {
        let min = (total / 60.0).floor() as i32;
        let sec = total - (min as f64 * 60.0);
        format!("{min} minute{} {:.1} seconds", if min == 1 { "" } else { "s" }, sec)
    } else {
        format!("{:.1} seconds", total)
    }
}

fn format_duration_long(ms: f64) -> String {
    format_duration_short(ms)
}

fn format_delta_phrase(delta_ms: f64) -> String {
    let abs = delta_ms.abs();
    if abs < 50.0 {
        return " Matching your best.".into();
    }
    let dir = if delta_ms > 0.0 { "slower" } else { "faster" };
    if abs < 1000.0 {
        format!(" {:.0} tenths {dir}.", abs / 100.0)
    } else {
        format!(" {:.1} seconds {dir}.", abs / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_phrase_tenths() {
        assert!(format_delta_phrase(350.0).contains("tenths slower"));
        assert!(format_delta_phrase(-280.0).contains("faster"));
    }

    #[test]
    fn fuel_estimate() {
        let laps = vec![2.0, 2.1, 1.9];
        let left = estimate_laps_remaining(6.0, &laps).unwrap();
        assert!((left - 3.0).abs() < 0.5);
    }
}
