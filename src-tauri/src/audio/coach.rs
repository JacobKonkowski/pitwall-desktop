use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::live::{LiveSnapshot, PackState};
use crate::settings::{AppSettings, ChatterLevel};

use super::phrasing::{
    format_delta_tts, format_duration_long, format_gap_seconds, lap_time_tts, sector_time_tts,
};
use super::queue::SpeechPriority;
use super::session_mode::SessionMode;
use super::speech::{SpeechPlan, SpeechUnit};

/// iRacing `SessionFlags` bit masks (irsdk_Flags) used for edge detection.
mod flags {
    pub const CHECKERED: u32 = 0x0000_0001;
    pub const WHITE: u32 = 0x0000_0002;
    pub const GREEN: u32 = 0x0000_0004;
    pub const YELLOW: u32 = 0x0000_0008;
    pub const RED: u32 = 0x0000_0010;
    pub const BLUE: u32 = 0x0000_0020;
    pub const BLACK: u32 = 0x0000_0040;
    pub const YELLOW_WAVING: u32 = 0x0000_0100;
    pub const GREEN_HELD: u32 = 0x0000_0400;
}

/// Minimum interval before repeating the same traffic-side callout while cars stay alongside.
const PACK_TRAFFIC_REMINDER: Duration = Duration::from_secs(3);
const GAP_CHANGE_THRESHOLD_S: f32 = 0.3;
const GAP_COOLDOWN: Duration = Duration::from_secs(9);

enum Mark {
    Intro,
    Sector { num: i32, is_pb: bool, ms: f64 },
    LapComplete,
    Flags,
    Incident,
    Pack,
    PackClear,
    FuelLow,
    FuelToEnd,
    PitToFinish,
    GapChange,
    RaceClock,
    PitsOpen,
}

struct Candidate {
    priority: SpeechPriority,
    plan: SpeechPlan,
    mark: Mark,
}

pub struct CoachEngine {
    session_key: String,
    tracked_lap: i32,
    announced_sectors: HashSet<i32>,
    best_sector_ms: HashMap<i32, f64>,
    best_lap_ms: Option<f64>,
    fuel_at_lap_start: Option<f32>,
    fuel_per_lap: Vec<f32>,
    spoke_session_intro: bool,
    spoke_fuel_low: bool,
    last_lap_seen: Option<f64>,
    pending_lap_plan: Option<SpeechPlan>,
    last_flags: u32,
    flags_baseline_set: bool,
    last_incident_count: i32,
    incident_baseline_set: bool,
    last_pack_state: PackState,
    /// Last traffic alert (not updated by clear callouts).
    last_traffic_spoken: Option<Instant>,
    spoke_fuel_to_end: bool,
    spoke_pit_to_finish: bool,
    last_announced_gap_ahead: Option<f32>,
    last_announced_gap_behind: Option<f32>,
    last_gap_spoken: Option<Instant>,
    last_announced_position: Option<i32>,
    spoke_five_laps: bool,
    spoke_final_lap: bool,
    spoke_five_minutes: bool,
    spoke_one_minute: bool,
    last_time_remain_s: Option<f64>,
    prev_pits_open: bool,
    pits_open_announced: bool,
}

impl CoachEngine {
    pub fn new() -> Self {
        Self {
            session_key: String::new(),
            tracked_lap: 0,
            announced_sectors: HashSet::new(),
            best_sector_ms: HashMap::new(),
            best_lap_ms: None,
            fuel_at_lap_start: None,
            fuel_per_lap: Vec::new(),
            spoke_session_intro: false,
            spoke_fuel_low: false,
            last_lap_seen: None,
            pending_lap_plan: None,
            last_flags: 0,
            flags_baseline_set: false,
            last_incident_count: 0,
            incident_baseline_set: false,
            last_pack_state: PackState::Off,
            last_traffic_spoken: None,
            spoke_fuel_to_end: false,
            spoke_pit_to_finish: false,
            last_announced_gap_ahead: None,
            last_announced_gap_behind: None,
            last_gap_spoken: None,
            last_announced_position: None,
            spoke_five_laps: false,
            spoke_final_lap: false,
            spoke_five_minutes: false,
            spoke_one_minute: false,
            last_time_remain_s: None,
            prev_pits_open: false,
            pits_open_announced: false,
        }
    }

    pub fn reset_session(&mut self) {
        *self = Self::new();
    }

    /// Poll for at most one new speech plan; caller enqueues into `SpeechQueue`.
    pub fn poll(&mut self, snap: &LiveSnapshot, settings: &AppSettings) -> Option<(SpeechPriority, SpeechPlan)> {
        if snap.lap <= 0 && snap.track.is_empty() {
            return None;
        }

        self.maybe_reset_session(snap);
        let pack_before_maintenance = self.last_pack_state;
        self.maintenance(snap, settings);

        let mut candidates: Vec<Candidate> = Vec::new();
        self.gather_intro(snap, settings, &mut candidates);
        self.gather_flags(snap, settings, &mut candidates);
        self.gather_incident(snap, settings, &mut candidates);
        self.gather_pack(snap, settings, &mut candidates);
        self.gather_pack_clear(snap, settings, pack_before_maintenance, &mut candidates);
        self.gather_race_fuel(snap, settings, &mut candidates);
        self.gather_race_clock(snap, settings, &mut candidates);
        self.gather_pits_open(snap, settings, &mut candidates);
        self.gather_gap_change(snap, settings, &mut candidates);
        self.gather_lap(&mut candidates);
        self.gather_sectors(snap, settings, &mut candidates);
        self.gather_fuel_low(snap, settings, &mut candidates);

        let suppressed = snap.on_pit_road || !snap.on_track;
        if suppressed {
            candidates.retain(|c| c.priority >= SpeechPriority::SAFETY);
        }

        let idx = pick_highest(&candidates)?;
        let chosen = candidates.swap_remove(idx);
        self.apply_mark(snap, chosen.mark);
        self.prev_pits_open = snap.pits_open;
        Some((chosen.priority, chosen.plan))
    }

    fn maybe_reset_session(&mut self, snap: &LiveSnapshot) {
        let key = format!("{}|{}", snap.track, snap.session_type);
        if self.session_key.is_empty() {
            self.session_key = key;
            return;
        }
        if key != self.session_key {
            self.reset_session();
            self.session_key = key;
        }
    }

    fn maintenance(&mut self, snap: &LiveSnapshot, settings: &AppSettings) {
        if !self.flags_baseline_set {
            self.last_flags = snap.session_flags;
            self.flags_baseline_set = true;
        }
        if !self.incident_baseline_set {
            self.last_incident_count = snap.incident_count;
            self.incident_baseline_set = true;
        }

        if snap.lap != self.tracked_lap {
            let previously_announced = self.announced_sectors.clone();
            self.tracked_lap = snap.lap;
            self.announced_sectors.clear();
            for sector in &snap.sectors {
                if sector.completed && previously_announced.contains(&sector.sector_num) {
                    self.announced_sectors.insert(sector.sector_num);
                }
            }
            if snap.lap > 0 && self.fuel_at_lap_start.is_none() {
                self.fuel_at_lap_start = Some(snap.fuel_level);
            }
        }

        for sector in &snap.sectors {
            if sector.completed
                && !self.announced_sectors.contains(&sector.sector_num)
                && sector.time_ms.map(|ms| ms < 1000.0).unwrap_or(true)
            {
                self.announced_sectors.insert(sector.sector_num);
            }
        }

        if let Some(lap_ms) = snap.last_lap_ms {
            if lap_ms > 10_000.0 && self.last_lap_seen != Some(lap_ms) {
                self.last_lap_seen = Some(lap_ms);
                let completed_lap = snap.lap.saturating_sub(1).max(1);
                if snap.last_lap_valid {
                    self.pending_lap_plan =
                        Some(self.lap_complete_plan(snap, settings, completed_lap, lap_ms));
                    self.record_fuel_use(snap);
                    self.best_lap_ms = Some(self.best_lap_ms.map(|b| b.min(lap_ms)).unwrap_or(lap_ms));
                } else {
                    let time_str = format_duration_long(lap_ms);
                    self.pending_lap_plan = Some(SpeechPlan::sequence(vec![
                        SpeechUnit::Clip("lap".into()),
                        SpeechUnit::Tts(format!("{completed_lap}, {time_str}. Out lap.")),
                    ]));
                }
                self.fuel_at_lap_start = Some(snap.fuel_level);
                if settings.audio_fuel_race_enabled {
                    self.spoke_fuel_low = false;
                }
            }
        }

        if !snap.pack_state.is_traffic() && snap.pack_state != PackState::Clear {
            self.last_pack_state = snap.pack_state;
        }

        self.last_time_remain_s = snap.session_time_remain_s;
    }

    fn gather_intro(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_session_intro_enabled || self.spoke_session_intro || snap.track.is_empty() {
            return;
        }
        let session = if snap.session_type.is_empty() {
            SpeechUnit::Clip("intro_good_luck".into())
        } else {
            SpeechUnit::Tts(format!("{}. Good luck.", snap.session_type))
        };
        out.push(Candidate {
            priority: SpeechPriority::CRITICAL,
            plan: SpeechPlan::sequence(vec![
                SpeechUnit::Clip("intro_online".into()),
                SpeechUnit::Tts(snap.track.clone()),
                session,
            ]),
            mark: Mark::Intro,
        });
    }

    fn gather_flags(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_flags_enabled || !self.flags_baseline_set {
            return;
        }
        let cur = snap.session_flags;
        let newly_set = |mask: u32| (cur & mask) != 0 && (self.last_flags & mask) == 0;

        let alert = if newly_set(flags::RED) {
            Some((SpeechPriority::CRITICAL, "flag_red"))
        } else if newly_set(flags::CHECKERED) {
            Some((SpeechPriority::CRITICAL, "flag_checkered"))
        } else if newly_set(flags::BLACK) {
            Some((SpeechPriority::CRITICAL, "flag_black"))
        } else if newly_set(flags::YELLOW_WAVING) {
            Some((SpeechPriority::SAFETY, "flag_yellow_waving"))
        } else if newly_set(flags::YELLOW) {
            Some((SpeechPriority::SAFETY, "flag_yellow"))
        } else if newly_set(flags::BLUE) {
            Some((SpeechPriority::SAFETY, "flag_blue"))
        } else if newly_set(flags::GREEN) || newly_set(flags::GREEN_HELD) {
            Some((SpeechPriority::SAFETY, "flag_green"))
        } else if newly_set(flags::WHITE) {
            Some((SpeechPriority::SAFETY, "flag_white"))
        } else {
            None
        };

        if let Some((priority, clip)) = alert {
            out.push(Candidate {
                priority,
                plan: SpeechPlan::clip(clip),
                mark: Mark::Flags,
            });
        }
    }

    fn gather_incident(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_incidents_enabled || !self.incident_baseline_set {
            return;
        }
        if snap.incident_count > self.last_incident_count {
            let count = snap.incident_count;
            let mut units = vec![
                SpeechUnit::Clip("incident_intro".into()),
                SpeechUnit::Tts(format!("{count}x")),
            ];
            if let Some(limit) = snap.incident_limit {
                if limit > 0 && count >= limit.saturating_sub(2) {
                    units.push(SpeechUnit::Tts(format!("Limit is {limit}.")));
                }
            }
            out.push(Candidate {
                priority: SpeechPriority::SAFETY,
                plan: SpeechPlan::sequence(units),
                mark: Mark::Incident,
            });
        }
    }

    fn gather_pack(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_pack_alerts_enabled || !snap.pack_state.is_traffic() {
            return;
        }
        if snap.on_pit_road || !snap.on_track {
            return;
        }
        let changed = snap.pack_state != self.last_pack_state;
        if !changed {
            let remind = self
                .last_traffic_spoken
                .map(|t| t.elapsed() >= PACK_TRAFFIC_REMINDER)
                .unwrap_or(true);
            if !remind {
                return;
            }
        }
        let clip = match snap.pack_state {
            PackState::CarLeft => "pack_car_left",
            PackState::CarRight => "pack_car_right",
            PackState::ThreeWide => "pack_three_wide_middle",
            PackState::TwoCarsLeft => "pack_two_left",
            PackState::TwoCarsRight => "pack_two_right",
            PackState::Clear | PackState::Off => return,
        };
        out.push(Candidate {
            priority: SpeechPriority::SAFETY,
            plan: SpeechPlan::clip(clip),
            mark: Mark::Pack,
        });
    }

    fn gather_pack_clear(
        &self,
        snap: &LiveSnapshot,
        settings: &AppSettings,
        prev_pack: PackState,
        out: &mut Vec<Candidate>,
    ) {
        if !settings.audio_pack_alerts_enabled {
            return;
        }
        if snap.on_pit_road || !snap.on_track {
            return;
        }
        if snap.pack_state != PackState::Clear {
            return;
        }
        if !prev_pack.is_traffic() {
            return;
        }
        let clip = pack_clear_clip(prev_pack);
        out.push(Candidate {
            priority: SpeechPriority::SAFETY,
            plan: SpeechPlan::clip(clip),
            mark: Mark::PackClear,
        });
    }

    fn gather_gap_change(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_gap_alerts_enabled || !chatter_is_verbose(settings) {
            return;
        }
        if snap.on_pit_road || !snap.on_track {
            return;
        }
        let cooled = self
            .last_gap_spoken
            .map(|t| t.elapsed() >= GAP_COOLDOWN)
            .unwrap_or(true);
        if !cooled {
            return;
        }

        if let (Some(cur), Some(prev)) = (snap.gap_to_car_ahead_s, self.last_announced_gap_ahead) {
            let delta = cur - prev;
            if delta.abs() >= GAP_CHANGE_THRESHOLD_S {
                let clip = if delta < 0.0 {
                    "gaining_ahead"
                } else {
                    "losing_ahead"
                };
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip(clip),
                    mark: Mark::GapChange,
                });
                return;
            }
        }
        if let (Some(cur), Some(prev)) = (snap.gap_to_car_behind_s, self.last_announced_gap_behind) {
            let delta = cur - prev;
            if delta.abs() >= GAP_CHANGE_THRESHOLD_S {
                let clip = if delta > 0.0 {
                    "gaining_behind"
                } else {
                    "losing_behind"
                };
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip(clip),
                    mark: Mark::GapChange,
                });
            }
        }
    }

    fn gather_race_fuel(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_fuel_race_enabled || !settings.audio_strategy_enabled {
            return;
        }
        if !SessionMode::from_session_type(&snap.session_type).is_race() {
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
            out.push(Candidate {
                priority: SpeechPriority::RACE,
                plan: SpeechPlan::sequence(vec![
                    SpeechUnit::Clip("fuel_short_on_fuel".into()),
                    SpeechUnit::Tts(format!(
                        "About {short_by} lap{} short.",
                        if short_by == 1 { "" } else { "s" }
                    )),
                    SpeechUnit::Clip("fuel_plan_stop".into()),
                ]),
                mark: Mark::PitToFinish,
            });
        } else if laps_of_fuel >= laps_remain as f32 && laps_remain <= 5 && !self.spoke_fuel_to_end {
            out.push(Candidate {
                priority: SpeechPriority::RACE,
                plan: SpeechPlan::clip("fuel_good_to_finish"),
                mark: Mark::FuelToEnd,
            });
        }
    }

    fn gather_race_clock(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_race_clock_enabled || !settings.audio_strategy_enabled {
            return;
        }
        if !chatter_allows_verbose(settings) {
            return;
        }
        if !SessionMode::from_session_type(&snap.session_type).is_race() {
            return;
        }
        if let Some(laps) = snap.session_laps_remain {
            if laps == 5 && !self.spoke_five_laps {
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip("race_five_laps"),
                    mark: Mark::RaceClock,
                });
                return;
            }
            if laps == 1 && !self.spoke_final_lap {
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip("race_final_lap"),
                    mark: Mark::RaceClock,
                });
                return;
            }
        }
        if let (Some(cur), Some(prev)) = (snap.session_time_remain_s, self.last_time_remain_s) {
            if prev > 300.0 && cur <= 300.0 && !self.spoke_five_minutes {
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip("race_five_minutes"),
                    mark: Mark::RaceClock,
                });
                return;
            }
            if prev > 60.0 && cur <= 60.0 && !self.spoke_one_minute {
                out.push(Candidate {
                    priority: SpeechPriority::RACE,
                    plan: SpeechPlan::clip("race_one_minute"),
                    mark: Mark::RaceClock,
                });
            }
        }
    }

    fn gather_pits_open(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_pits_open_enabled || !settings.audio_strategy_enabled {
            return;
        }
        let mode = SessionMode::from_session_type(&snap.session_type);
        if mode.is_practice() {
            return;
        }
        if snap.pits_open && !self.prev_pits_open && !self.pits_open_announced {
            out.push(Candidate {
                priority: SpeechPriority::RACE,
                plan: SpeechPlan::clip("pits_open"),
                mark: Mark::PitsOpen,
            });
        }
    }

    fn gather_lap(&self, out: &mut Vec<Candidate>) {
        if let Some(plan) = &self.pending_lap_plan {
            out.push(Candidate {
                priority: SpeechPriority::PACE,
                plan: plan.clone(),
                mark: Mark::LapComplete,
            });
        }
    }

    fn gather_sectors(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_pace_enabled {
            return;
        }
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
            let plan = self.sector_plan(snap, settings, sector.sector_num, ms, prev_best, is_pb);
            out.push(Candidate {
                priority: SpeechPriority::PACE,
                plan,
                mark: Mark::Sector {
                    num: sector.sector_num,
                    is_pb,
                    ms,
                },
            });
            break;
        }
    }

    fn gather_fuel_low(&self, snap: &LiveSnapshot, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if self.spoke_fuel_low
            || !settings.audio_strategy_enabled
            || settings.audio_coach_fuel_threshold <= 0.0
            || snap.fuel_level <= 0.0
            || snap.fuel_level > settings.audio_coach_fuel_threshold
        {
            return;
        }
        let mut units = vec![
            SpeechUnit::Clip("fuel_low".into()),
            SpeechUnit::Tts(format!("{:.0} liters", snap.fuel_level)),
        ];
        if let Some(laps_left) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) {
            if laps_left <= 1.5 {
                units.push(SpeechUnit::Clip("fuel_pit_this_lap".into()));
            } else {
                units.push(SpeechUnit::Tts(format!(
                    "About {:.0} laps of fuel.",
                    laps_left.round()
                )));
            }
        }
        out.push(Candidate {
            priority: SpeechPriority::RACE,
            plan: SpeechPlan::sequence(units),
            mark: Mark::FuelLow,
        });
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
            Mark::LapComplete => {
                self.pending_lap_plan = None;
                if let Some(pos) = snap.player_class_position.or(snap.player_position) {
                    if pos > 0 {
                        self.last_announced_position = Some(pos);
                    }
                }
                self.last_announced_gap_ahead = snap.gap_to_car_ahead_s;
                self.last_announced_gap_behind = snap.gap_to_car_behind_s;
            }
            Mark::Flags => self.last_flags = snap.session_flags,
            Mark::Incident => self.last_incident_count = snap.incident_count,
            Mark::Pack => {
                self.last_pack_state = snap.pack_state;
                self.last_traffic_spoken = Some(Instant::now());
            }
            Mark::PackClear => {
                self.last_pack_state = snap.pack_state;
            }
            Mark::FuelLow => self.spoke_fuel_low = true,
            Mark::FuelToEnd => self.spoke_fuel_to_end = true,
            Mark::PitToFinish => self.spoke_pit_to_finish = true,
            Mark::GapChange => {
                self.last_announced_gap_ahead = snap.gap_to_car_ahead_s;
                self.last_announced_gap_behind = snap.gap_to_car_behind_s;
                self.last_gap_spoken = Some(Instant::now());
            }
            Mark::RaceClock => {
                if snap.session_laps_remain == Some(5) {
                    self.spoke_five_laps = true;
                }
                if snap.session_laps_remain == Some(1) {
                    self.spoke_final_lap = true;
                }
                if snap.session_time_remain_s.map(|t| t <= 300.0).unwrap_or(false) {
                    self.spoke_five_minutes = true;
                }
                if snap.session_time_remain_s.map(|t| t <= 60.0).unwrap_or(false) {
                    self.spoke_one_minute = true;
                }
            }
            Mark::PitsOpen => self.pits_open_announced = true,
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

    fn sector_plan(
        &self,
        snap: &LiveSnapshot,
        settings: &AppSettings,
        sector_num: i32,
        ms: f64,
        prev_best: Option<f64>,
        is_pb: bool,
    ) -> SpeechPlan {
        let mode = SessionMode::from_session_type(&snap.session_type);
        let mut units = vec![
            SpeechUnit::Clip("sector".into()),
            SpeechUnit::Tts(sector_time_tts(sector_num, ms)),
        ];

        if is_pb && prev_best.is_some() {
            units.push(SpeechUnit::Clip("pb_sector".into()));
        } else if let Some(b) = prev_best {
            if !is_pb {
                push_pace_delta_units(&mut units, ms - b);
            }
        }

        let show_live_pace = !( !is_pb && prev_best.is_some() && ms > prev_best.unwrap() + 20.0);
        if show_live_pace {
            if let Some(d) = snap.delta_to_best_ms.filter(|d| d.abs() > 80.0 && snap.lap_dist_pct > 0.05)
            {
                if d > 0.0 {
                    units.push(SpeechUnit::Clip("pace_off_pb_intro".into()));
                    units.push(SpeechUnit::Tts(format_delta_tts(d)));
                } else {
                    units.push(SpeechUnit::Clip("pace_on_pb".into()));
                }
            }
        }

        if chatter_allows_verbose(settings) && (mode.is_qual() || mode.is_practice()) {
            if let Some(d) = snap
                .delta_to_session_best_ms
                .filter(|d| d.abs() > 80.0)
            {
                push_pace_delta_units(&mut units, d);
            }
        }

        SpeechPlan::sequence(units)
    }

    fn lap_complete_plan(
        &self,
        snap: &LiveSnapshot,
        settings: &AppSettings,
        lap_num: i32,
        lap_ms: f64,
    ) -> SpeechPlan {
        let mode = SessionMode::from_session_type(&snap.session_type);
        let prev_best = self.best_lap_ms;
        let is_pb = prev_best.map(|b| lap_ms < b - 50.0).unwrap_or(true);

        let mut units = vec![
            SpeechUnit::Clip("lap".into()),
            SpeechUnit::Tts(lap_time_tts(lap_num, lap_ms)),
        ];

        if is_pb && prev_best.is_some() {
            units.push(SpeechUnit::Clip("pb_new".into()));
        } else if let Some(best) = prev_best {
            push_pace_delta_units(&mut units, lap_ms - best);
        }

        if let Some(last) = snap.delta_to_last_ms {
            if last.abs() > 80.0 && prev_best.is_some() && !is_pb {
                push_pace_delta_with_suffix(&mut units, last, "versus previous lap.");
            }
        }

        if mode.is_qual() || (mode.is_practice() && chatter_is_verbose(settings)) {
            if let Some(d) = snap
                .delta_to_session_best_ms
                .filter(|d| d.abs() > 80.0)
            {
                push_pace_delta_with_suffix(&mut units, d, "off session best.");
            }
        }

        if mode.is_race() || mode.is_qual() {
            if settings.audio_position_callouts_enabled && chatter_allows_normal(settings) {
                if let Some(pos) = snap.player_class_position.or(snap.player_position) {
                    if let Some(prev) = self.last_announced_position {
                        if pos > 0 && prev > 0 && pos != prev {
                            let clip = if pos < prev {
                                "position_up"
                            } else {
                                "position_down"
                            };
                            units.push(SpeechUnit::Clip(clip.into()));
                            units.push(SpeechUnit::Tts(format!("P{pos}")));
                        } else if pos > 0 {
                            units.push(SpeechUnit::Tts(format!("P{pos}")));
                        }
                    } else if pos > 0 {
                        units.push(SpeechUnit::Tts(format!("P{pos}")));
                    }
                }
            }
        }

        if settings.audio_gap_alerts_enabled
            && (mode.is_race() || mode.is_qual() || chatter_is_verbose(settings))
        {
            if let Some(g) = snap.gap_to_car_ahead_s.filter(|g| *g >= 0.0) {
                units.push(SpeechUnit::Clip("gap_ahead".into()));
                units.push(SpeechUnit::Tts(format_gap_seconds(g)));
            }
            if let Some(g) = snap.gap_to_car_behind_s.filter(|g| *g >= 0.0) {
                units.push(SpeechUnit::Clip("gap_behind".into()));
                units.push(SpeechUnit::Tts(format_gap_seconds(g)));
            }
        }

        if mode.is_race() && settings.audio_strategy_enabled {
            if snap.fuel_level > 0.0 {
                if let Some(laps_left) = estimate_laps_remaining(snap.fuel_level, &self.fuel_per_lap) {
                    if laps_left <= 3.0 {
                        units.push(SpeechUnit::Tts(format!(
                            "Fuel {:.0} liters. Pit in {:.0} laps.",
                            snap.fuel_level,
                            laps_left.ceil()
                        )));
                    } else {
                        units.push(SpeechUnit::Tts(format!(
                            "Fuel {:.0} liters. About {:.0} laps remaining.",
                            snap.fuel_level,
                            laps_left.round()
                        )));
                    }
                } else {
                    units.push(SpeechUnit::Tts(format!("Fuel {:.0} liters.", snap.fuel_level)));
                }
            }
        }

        SpeechPlan::sequence(units)
    }
}

fn pack_clear_clip(prev_pack: PackState) -> &'static str {
    match prev_pack {
        PackState::CarLeft | PackState::TwoCarsLeft => "pack_clear_left",
        PackState::CarRight | PackState::TwoCarsRight => "pack_clear_right",
        _ => "pack_clear",
    }
}

fn push_pace_delta_units(units: &mut Vec<SpeechUnit>, delta_ms: f64) {
    let abs = delta_ms.abs();
    if abs < 50.0 {
        units.push(SpeechUnit::Clip("pace_matching_best".into()));
    } else if delta_ms > 0.0 {
        units.push(SpeechUnit::Clip("pace_off_pb_intro".into()));
        units.push(SpeechUnit::Tts(format_delta_tts(delta_ms)));
    } else {
        units.push(SpeechUnit::Tts(format_delta_tts(delta_ms)));
    }
}

fn push_pace_delta_with_suffix(units: &mut Vec<SpeechUnit>, delta_ms: f64, suffix: &str) {
    let abs = delta_ms.abs();
    if abs < 50.0 {
        units.push(SpeechUnit::Clip("pace_matching_best".into()));
        units.push(SpeechUnit::Tts(suffix.into()));
    } else if delta_ms > 0.0 {
        units.push(SpeechUnit::Clip("pace_off_pb_intro".into()));
        units.push(SpeechUnit::Tts(format!(
            "{} {}",
            format_delta_tts(delta_ms).trim_end_matches('.'),
            suffix
        )));
    } else {
        units.push(SpeechUnit::Tts(format!(
            "{} {}",
            format_delta_tts(delta_ms).trim_end_matches('.'),
            suffix
        )));
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

fn chatter_allows_normal(settings: &AppSettings) -> bool {
    settings.audio_coach_chatter_level != ChatterLevel::Minimal
}

fn chatter_is_verbose(settings: &AppSettings) -> bool {
    settings.audio_coach_chatter_level == ChatterLevel::Verbose
}

#[allow(dead_code)]
fn chatter_allows_verbose(settings: &AppSettings) -> bool {
    chatter_allows_normal(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppSettings;

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
    fn intro_spoken_first() {
        let mut engine = CoachEngine::new();
        let plan = engine.poll(&base_snapshot(), &settings());
        assert!(plan.is_some());
        assert!(plan.unwrap().1.display_text().contains("Test Track"));
    }

    #[test]
    fn red_flag_beats_pack_alert() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.session_flags = flags::RED;
        snap.pack_state = PackState::ThreeWide;
        let plan = engine.poll(&snap, &settings()).unwrap();
        assert!(matches!(plan.1, SpeechPlan::Clip(k) if k == "flag_red"));
    }

    #[test]
    fn pack_alert_suppressed_in_pits() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.pack_state = PackState::CarLeft;
        snap.on_pit_road = true;
        assert!(engine.poll(&snap, &settings()).is_none());
    }

    #[test]
    fn incident_increase_announced() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.incident_count = 4;
        let plan = engine.poll(&snap, &settings()).unwrap();
        assert!(plan.1.display_text().contains("4"));
    }

    #[test]
    fn session_reset_on_track_change() {
        let mut engine = CoachEngine::new();
        let intro = engine.poll(&base_snapshot(), &settings());
        assert!(intro.is_some());

        let mut snap = base_snapshot();
        snap.track = "Other Track".into();
        let intro2 = engine.poll(&snap, &settings());
        assert!(intro2.is_some());
        assert!(intro2.unwrap().1.display_text().contains("Other Track"));
    }

    #[test]
    fn race_fuel_muted_in_practice() {
        let mut engine = CoachEngine::new();
        engine.fuel_per_lap = vec![2.0];
        let mut snap = base_snapshot();
        snap.session_type = "Practice".into();
        snap.session_laps_remain = Some(10);
        snap.fuel_level = 5.0;
        engine.poll(&snap, &settings());
        let plan = engine.poll(&snap, &settings());
        assert!(plan.is_none() || !plan.unwrap().1.display_text().contains("fuel_short"));
    }

    #[test]
    fn qual_lap_plan_has_session_delta() {
        let mut engine = CoachEngine::new();
        engine.poll(&base_snapshot(), &settings());

        let mut snap = base_snapshot();
        snap.session_type = "Qualifying".into();
        snap.lap = 2;
        snap.last_lap_valid = true;
        snap.last_lap_ms = Some(92_000.0);
        snap.delta_to_session_best_ms = Some(400.0);
        engine.poll(&snap, &settings());
        let plan = engine.poll(&snap, &settings()).unwrap();
        assert!(plan.1.display_text().contains("session best"));
    }

    fn clip_key(plan: &SpeechPlan) -> Option<&str> {
        match plan {
            SpeechPlan::Clip(k) => Some(k.as_str()),
            _ => None,
        }
    }

    #[test]
    fn pack_clear_after_car_left() {
        let mut engine = CoachEngine::new();
        let s = settings();
        let mut snap = base_snapshot();
        engine.poll(&snap, &s);
        snap.pack_state = PackState::CarLeft;
        assert_eq!(
            clip_key(&engine.poll(&snap, &s).unwrap().1),
            Some("pack_car_left")
        );
        snap.pack_state = PackState::Clear;
        assert_eq!(
            clip_key(&engine.poll(&snap, &s).unwrap().1),
            Some("pack_clear_left")
        );
    }

    #[test]
    fn pack_clear_after_three_wide() {
        let mut engine = CoachEngine::new();
        let s = settings();
        let mut snap = base_snapshot();
        engine.poll(&snap, &s);
        snap.pack_state = PackState::ThreeWide;
        engine.poll(&snap, &s);
        snap.pack_state = PackState::Clear;
        assert_eq!(
            clip_key(&engine.poll(&snap, &s).unwrap().1),
            Some("pack_clear")
        );
    }

    #[test]
    fn pack_traffic_immediate_after_clear() {
        let mut engine = CoachEngine::new();
        let s = settings();
        let mut snap = base_snapshot();
        engine.poll(&snap, &s);
        snap.pack_state = PackState::CarLeft;
        engine.poll(&snap, &s);
        snap.pack_state = PackState::Clear;
        engine.poll(&snap, &s);
        snap.pack_state = PackState::CarLeft;
        assert_eq!(
            clip_key(&engine.poll(&snap, &s).unwrap().1),
            Some("pack_car_left")
        );
    }

    #[test]
    fn pack_clear_requires_pack_alerts() {
        let mut engine = CoachEngine::new();
        let mut s = settings();
        let mut snap = base_snapshot();
        engine.poll(&snap, &s);
        snap.pack_state = PackState::CarLeft;
        engine.poll(&snap, &s);
        snap.pack_state = PackState::Clear;
        s.audio_pack_alerts_enabled = false;
        assert!(engine.poll(&snap, &s).is_none());
    }

    #[test]
    fn pack_clear_suppressed_in_pits() {
        let mut engine = CoachEngine::new();
        let s = settings();
        let mut snap = base_snapshot();
        engine.poll(&snap, &s);
        snap.pack_state = PackState::CarLeft;
        engine.poll(&snap, &s);
        snap.pack_state = PackState::Clear;
        snap.on_pit_road = true;
        assert!(engine.poll(&snap, &s).is_none());
    }

    #[test]
    fn pace_matching_best_on_small_delta() {
        let mut units = Vec::new();
        push_pace_delta_units(&mut units, 30.0);
        assert!(matches!(&units[0], SpeechUnit::Clip(k) if k == "pace_matching_best"));
    }
}
