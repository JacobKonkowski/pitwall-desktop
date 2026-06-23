use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::live::{LiveSnapshot, PackState};
use crate::settings::AppSettings;

/// iRacing `SessionFlags` bit masks (irsdk_Flags) used for edge detection.
mod flags {
    pub const CHECKERED: u32 = 0x0000_0001;
    pub const WHITE: u32 = 0x0000_0002;
    pub const GREEN: u32 = 0x0000_0004;
    pub const YELLOW: u32 = 0x0000_0008;
    pub const RED: u32 = 0x0000_0010;
    pub const BLUE: u32 = 0x0000_0020;
    pub const YELLOW_WAVING: u32 = 0x0000_0100;
    pub const GREEN_HELD: u32 = 0x0000_0400;
}

/// How long to wait before repeating the same side-by-side alert.
const PACK_COOLDOWN: Duration = Duration::from_secs(4);

/// Alert priority. When several alerts are eligible in one tick we speak only the
/// highest-priority one; the rest are reconsidered next tick (natural deferral).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Pace = 0,
    Race = 1,
    Pack = 2,
    Safety = 3,
    Critical = 4,
}

/// What state to update once an alert is actually spoken.
enum Mark {
    Intro,
    Sector { num: i32, is_pb: bool, ms: f64 },
    LapComplete,
    Flags,
    Incident,
    Pack,
    FuelLow,
    FuelToEnd,
    PitToFinish,
}

struct Candidate {
    priority: Priority,
    text: String,
    mark: Mark,
}

/// Tracks what we've already spoken so alerts stay useful, not repetitive.
pub struct CoachEngine {
    tracked_lap: i32,
    announced_sectors: HashSet<i32>,
    best_sector_ms: HashMap<i32, f64>,
    best_lap_ms: Option<f64>,
    fuel_at_lap_start: Option<f32>,
    fuel_per_lap: Vec<f32>,
    spoke_session_intro: bool,
    spoke_fuel_low: bool,
    // New-lap bookkeeping is done once per lap regardless of whether the callout
    // gets spoken, so fuel/best tracking never drifts.
    last_lap_seen: Option<f64>,
    pending_lap_msg: Option<String>,
    // Edge-detection baselines for session-wide alerts.
    last_flags: u32,
    flags_baseline_set: bool,
    last_incident_count: i32,
    incident_baseline_set: bool,
    last_pack_state: PackState,
    last_pack_spoken: Option<Instant>,
    spoke_fuel_to_end: bool,
    spoke_pit_to_finish: bool,
}

impl CoachEngine {
    pub fn new() -> Self {
        Self {
            tracked_lap: 0,
            announced_sectors: HashSet::new(),
            best_sector_ms: HashMap::new(),
            best_lap_ms: None,
            fuel_at_lap_start: None,
            fuel_per_lap: Vec::new(),
            spoke_session_intro: false,
            spoke_fuel_low: false,
            last_lap_seen: None,
            pending_lap_msg: None,
            last_flags: 0,
            flags_baseline_set: false,
            last_incident_count: 0,
            incident_baseline_set: false,
            last_pack_state: PackState::Off,
            last_pack_spoken: None,
            spoke_fuel_to_end: false,
            spoke_pit_to_finish: false,
        }
    }

    pub fn poll(&mut self, snap: &LiveSnapshot, settings: &AppSettings) -> Vec<String> {
        if snap.lap <= 0 && snap.track.is_empty() {
            return Vec::new();
        }

        self.maintenance(snap);

        let mut candidates: Vec<Candidate> = Vec::new();
        self.gather_intro(snap, &mut candidates);
        self.gather_flags(snap, settings, &mut candidates);
        self.gather_incident(snap, settings, &mut candidates);
        self.gather_pack(snap, settings, &mut candidates);
        self.gather_race_fuel(snap, settings, &mut candidates);
        self.gather_lap(&mut candidates);
        self.gather_sectors(snap, &mut candidates);
        self.gather_fuel_low(snap, settings, &mut candidates);

        // Suppress low-priority chatter in the pits or when off track. Flags and
        // incidents (Safety/Critical) still come through.
        let suppressed = snap.on_pit_road || !snap.on_track;
        if suppressed {
            candidates.retain(|c| c.priority >= Priority::Safety);
        }

        // Speak only the single highest-priority alert this tick.
        let Some(idx) = pick_highest(&candidates) else {
            return Vec::new();
        };
        let chosen = candidates.swap_remove(idx);
        self.apply_mark(snap, chosen.mark);
        vec![chosen.text]
    }

    /// Per-tick bookkeeping that must happen whether or not we speak.
    fn maintenance(&mut self, snap: &LiveSnapshot) {
        if !self.flags_baseline_set {
            self.last_flags = snap.session_flags;
            self.flags_baseline_set = true;
        }
        if !self.incident_baseline_set {
            self.last_incident_count = snap.incident_count;
            self.incident_baseline_set = true;
        }

        if snap.lap != self.tracked_lap {
            self.tracked_lap = snap.lap;
            self.announced_sectors.clear();
            if snap.lap > 0 && self.fuel_at_lap_start.is_none() {
                self.fuel_at_lap_start = Some(snap.fuel_level);
            }
        }

        // Mark completed sectors with no usable time as announced so they never
        // surface as candidates.
        for sector in &snap.sectors {
            if sector.completed
                && !self.announced_sectors.contains(&sector.sector_num)
                && sector.time_ms.map(|ms| ms < 1000.0).unwrap_or(true)
            {
                self.announced_sectors.insert(sector.sector_num);
            }
        }

        // New lap time: record fuel use, update best lap, and queue the callout.
        if let Some(lap_ms) = snap.last_lap_ms {
            if lap_ms > 10_000.0 && self.last_lap_seen != Some(lap_ms) {
                self.last_lap_seen = Some(lap_ms);
                let completed_lap = snap.lap.saturating_sub(1).max(1);
                if snap.last_lap_valid {
                    self.pending_lap_msg = Some(self.lap_complete_message(snap, completed_lap, lap_ms));
                    self.record_fuel_use(snap);
                    self.best_lap_ms = Some(self.best_lap_ms.map(|b| b.min(lap_ms)).unwrap_or(lap_ms));
                } else {
                    let time_str = format_duration_long(lap_ms);
                    self.pending_lap_msg =
                        Some(format!("Lap {completed_lap}. {time_str}. Out lap."));
                }
                self.fuel_at_lap_start = Some(snap.fuel_level);
            }
        }

        // When the player is clear/off, keep the pack baseline current so the
        // next side-by-side moment re-announces.
        if !snap.pack_state.is_traffic() {
            self.last_pack_state = snap.pack_state;
        }
    }

    fn gather_intro(&self, snap: &LiveSnapshot, out: &mut Vec<Candidate>) {
        if self.spoke_session_intro || snap.track.is_empty() {
            return;
        }
        let text = format!(
            "PitWall coach online. {}. {}",
            snap.track,
            if snap.session_type.is_empty() {
                "Good luck.".into()
            } else {
                format!("{} session.", snap.session_type)
            }
        );
        out.push(Candidate { priority: Priority::Critical, text, mark: Mark::Intro });
    }

    fn gather_flags(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_flags_enabled || !self.flags_baseline_set {
            return;
        }
        let cur = snap.session_flags;
        let newly_set = |mask: u32| (cur & mask) != 0 && (self.last_flags & mask) == 0;

        let alert = if newly_set(flags::RED) {
            Some((Priority::Critical, "Red flag."))
        } else if newly_set(flags::CHECKERED) {
            Some((Priority::Critical, "Checkered flag."))
        } else if newly_set(flags::YELLOW) || newly_set(flags::YELLOW_WAVING) {
            Some((Priority::Safety, "Yellow flag. Caution."))
        } else if newly_set(flags::BLUE) {
            Some((Priority::Safety, "Blue flag. Faster car approaching."))
        } else if newly_set(flags::GREEN) || newly_set(flags::GREEN_HELD) {
            Some((Priority::Safety, "Green flag. Go."))
        } else if newly_set(flags::WHITE) {
            Some((Priority::Safety, "White flag. Last lap."))
        } else {
            None
        };

        if let Some((priority, text)) = alert {
            out.push(Candidate { priority, text: text.into(), mark: Mark::Flags });
        }
    }

    fn gather_incident(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_incidents_enabled || !self.incident_baseline_set {
            return;
        }
        if snap.incident_count > self.last_incident_count {
            let text = format!("Incident count now {}.", snap.incident_count);
            out.push(Candidate { priority: Priority::Safety, text, mark: Mark::Incident });
        }
    }

    fn gather_pack(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_pack_alerts_enabled || !snap.pack_state.is_traffic() {
            return;
        }
        let changed = snap.pack_state != self.last_pack_state;
        let cooled = self
            .last_pack_spoken
            .map(|t| t.elapsed() >= PACK_COOLDOWN)
            .unwrap_or(true);
        if !changed && !cooled {
            return;
        }
        let text = match snap.pack_state {
            PackState::CarLeft => "Car on your left.",
            PackState::CarRight => "Car on your right.",
            PackState::ThreeWide => "Three wide. You're in the middle.",
            PackState::TwoCarsLeft => "Two cars on your left.",
            PackState::TwoCarsRight => "Two cars on your right.",
            PackState::Clear | PackState::Off => return,
        };
        out.push(Candidate { priority: Priority::Pack, text: text.into(), mark: Mark::Pack });
    }

    fn gather_race_fuel(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_fuel_race_enabled {
            return;
        }
        let Some(laps_remain) = snap.session_laps_remain else {
            return;
        };
        if laps_remain <= 0 || laps_remain > 60 {
            return;
        }
        let Some(laps_of_fuel) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) else {
            return;
        };

        if laps_of_fuel + 0.3 < laps_remain as f32 && !self.spoke_pit_to_finish {
            let short_by = (laps_remain as f32 - laps_of_fuel).ceil() as i32;
            let text = format!(
                "Short on fuel. About {} lap{} short of the finish. Plan a stop.",
                short_by,
                if short_by == 1 { "" } else { "s" }
            );
            out.push(Candidate { priority: Priority::Race, text, mark: Mark::PitToFinish });
        } else if laps_of_fuel >= laps_remain as f32 && laps_remain <= 5 && !self.spoke_fuel_to_end {
            out.push(Candidate {
                priority: Priority::Race,
                text: "Fuel is good to the finish.".into(),
                mark: Mark::FuelToEnd,
            });
        }
    }

    fn gather_lap(&self, out: &mut Vec<Candidate>) {
        if let Some(text) = &self.pending_lap_msg {
            out.push(Candidate {
                priority: Priority::Pace,
                text: text.clone(),
                mark: Mark::LapComplete,
            });
        }
    }

    fn gather_sectors(&self, snap: &LiveSnapshot, out: &mut Vec<Candidate>) {
        for sector in &snap.sectors {
            if !sector.completed || self.announced_sectors.contains(&sector.sector_num) {
                continue;
            }
            let Some(ms) = sector.time_ms else { continue };
            if ms < 1000.0 {
                continue;
            }
            let prev_best = self.best_sector_ms.get(&sector.sector_num).copied();
            let is_pb = prev_best.map(|b| ms < b - 20.0).unwrap_or(true);
            let text = self.sector_message(snap, sector.sector_num, ms, prev_best, is_pb);
            out.push(Candidate {
                priority: Priority::Pace,
                text,
                mark: Mark::Sector { num: sector.sector_num, is_pb, ms },
            });
            // Only surface the earliest pending sector each tick.
            break;
        }
    }

    fn gather_fuel_low(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if self.spoke_fuel_low
            || settings.audio_coach_fuel_threshold <= 0.0
            || snap.fuel_level <= 0.0
            || snap.fuel_level > settings.audio_coach_fuel_threshold
        {
            return;
        }
        let mut text = format!("Fuel low. {:.0} liters left.", snap.fuel_level);
        if let Some(laps_left) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) {
            if laps_left <= 1.5 {
                text.push_str(" Pit this lap or next.");
            } else {
                text.push_str(&format!(" About {:.0} laps of fuel.", laps_left.round()));
            }
        }
        out.push(Candidate { priority: Priority::Race, text, mark: Mark::FuelLow });
    }

    fn apply_mark(&mut self, snap: &LiveSnapshot, mark: Mark) {
        match mark {
            Mark::Intro => self.spoke_session_intro = true,
            Mark::Sector { num, is_pb, ms } => {
                if is_pb {
                    self.best_sector_ms.insert(num, ms);
                }
                self.announced_sectors.insert(num);
            }
            Mark::LapComplete => self.pending_lap_msg = None,
            Mark::Flags => self.last_flags = snap.session_flags,
            Mark::Incident => self.last_incident_count = snap.incident_count,
            Mark::Pack => {
                self.last_pack_state = snap.pack_state;
                self.last_pack_spoken = Some(Instant::now());
            }
            Mark::FuelLow => self.spoke_fuel_low = true,
            Mark::FuelToEnd => self.spoke_fuel_to_end = true,
            Mark::PitToFinish => self.spoke_pit_to_finish = true,
        }
    }

    fn record_fuel_use(&mut self, snap: &LiveSnapshot) {
        if let Some(start_fuel) = self.fuel_at_lap_start {
            if start_fuel > snap.fuel_level && start_fuel - snap.fuel_level > 0.05 {
                self.fuel_per_lap.push(start_fuel - snap.fuel_level);
                if self.fuel_per_lap.len() > 8 {
                    self.fuel_per_lap.remove(0);
                }
            }
        }
    }

    fn sector_message(
        &self,
        snap: &LiveSnapshot,
        sector_num: i32,
        ms: f64,
        prev_best: Option<f64>,
        is_pb: bool,
    ) -> String {
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
            format!("Sector {sector_num}. {time_str}.{}", delta_str.unwrap_or_default())
        };
        if let Some(hint) = pace_hint {
            msg.push_str(&hint);
        }
        msg
    }

    fn lap_complete_message(&self, snap: &LiveSnapshot, lap_num: i32, lap_ms: f64) -> String {
        let prev_best = self.best_lap_ms;
        let is_pb = prev_best.map(|b| lap_ms < b - 50.0).unwrap_or(true);

        let time_str = format_duration_long(lap_ms);
        let mut parts = vec![format!("Lap {lap_num}. {time_str}.")];

        if is_pb && prev_best.is_some() {
            parts.push("New personal best.".into());
        } else if let Some(best) = prev_best {
            parts.push(format_delta_phrase(lap_ms - best));
        }

        if let Some(last) = snap.delta_to_last_ms {
            if last.abs() > 80.0 && prev_best.is_some() && !is_pb {
                parts.push(format!("{} versus previous lap.", format_delta_phrase(last).trim()));
            }
        }

        if let Some(pos) = snap.player_class_position.or(snap.player_position) {
            if pos > 0 {
                parts.push(format!("P{pos}."));
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

        parts.join(" ")
    }
}

fn pick_highest(candidates: &[Candidate]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.priority)
        .map(|(i, _)| i)
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

    fn settings() -> AppSettings {
        AppSettings::default()
    }

    fn base_snapshot() -> LiveSnapshot {
        let mut snap = LiveSnapshot::default();
        snap.track = "Test Track".into();
        snap.session_type = "Race".into();
        snap.lap = 1;
        snap.on_track = true;
        snap
    }

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

    #[test]
    fn intro_spoken_first() {
        let mut engine = CoachEngine::new();
        let msgs = engine.poll(&base_snapshot(), &settings());
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("PitWall coach online"));
    }

    #[test]
    fn red_flag_beats_pack_alert() {
        let mut engine = CoachEngine::new();
        // Consume the intro first.
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.session_flags = flags::RED;
        snap.pack_state = PackState::ThreeWide;
        let msgs = engine.poll(&snap, &settings());
        assert_eq!(msgs, vec!["Red flag.".to_string()]);
    }

    #[test]
    fn pack_alert_suppressed_in_pits() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.pack_state = PackState::CarLeft;
        snap.on_pit_road = true;
        let msgs = engine.poll(&snap, &settings());
        assert!(msgs.is_empty());
    }

    #[test]
    fn incident_increase_announced() {
        let mut engine = CoachEngine::new();
        // Baseline at 0 incidents.
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.incident_count = 4;
        let msgs = engine.poll(&snap, &settings());
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("Incident count now 4"));
    }

    #[test]
    fn flags_disabled_no_announce() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut cfg = settings();
        cfg.audio_flags_enabled = false;
        let mut snap = base_snapshot();
        snap.session_flags = flags::YELLOW;
        let msgs = engine.poll(&snap, &cfg);
        assert!(msgs.is_empty());
    }
}
