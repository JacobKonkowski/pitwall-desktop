use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::storage::{LapSummary, SessionDetail, SessionStandings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachReport {
    pub session_id: i64,
    pub insights: Vec<CoachInsight>,
    pub summary: SessionCoachStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachInsight {
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub severity: String,
    pub lap_numbers: Vec<i32>,
    pub sector_num: Option<i32>,
    pub delta_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCoachStats {
    pub valid_lap_count: i32,
    pub consistency_ms: Option<f64>,
    pub best_lap_ms: Option<f64>,
    pub avg_lap_ms: Option<f64>,
    pub weakest_sector: Option<i32>,
    pub weakest_sector_loss_ms: Option<f64>,
}

pub fn build_coach_report(session_id: i64, detail: &SessionDetail) -> CoachReport {
    let valid_laps: Vec<&LapSummary> = detail
        .laps
        .iter()
        .filter(|l| l.valid && l.lap_time_ms.unwrap_or(0.0) > 0.0)
        .collect();

    let lap_times: Vec<f64> = valid_laps
        .iter()
        .filter_map(|l| l.lap_time_ms)
        .collect();

    let best_lap_ms = lap_times
        .iter()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg_lap_ms = if lap_times.is_empty() {
        None
    } else {
        Some(lap_times.iter().sum::<f64>() / lap_times.len() as f64)
    };

    let consistency_ms = if lap_times.len() >= 2 {
        let mean = lap_times.iter().sum::<f64>() / lap_times.len() as f64;
        let variance = lap_times.iter().map(|t| (t - mean).powi(2)).sum::<f64>()
            / lap_times.len() as f64;
        Some(variance.sqrt())
    } else {
        None
    };

    let mut insights = Vec::new();

    if let Some(std) = consistency_ms {
        let label = if std < 500.0 {
            "high"
        } else if std < 1500.0 {
            "medium"
        } else {
            "low"
        };
        insights.push(CoachInsight {
            kind: "consistency".into(),
            title: "Lap consistency".into(),
            detail: format!(
                "Lap time standard deviation is {:.3}s ({label} consistency).",
                std / 1000.0
            ),
            severity: if std < 500.0 {
                "good".into()
            } else if std < 1500.0 {
                "info".into()
            } else {
                "warn".into()
            },
            lap_numbers: Vec::new(),
            sector_num: None,
            delta_ms: Some(std),
        });
    }

    // Weakest sector vs best lap (per sub-session)
    let mut weakest_sector: Option<i32> = None;
    let mut weakest_loss: Option<f64> = None;

    for session_num in detail
        .laps
        .iter()
        .map(|l| l.session_num)
        .collect::<std::collections::HashSet<_>>()
    {
        let stage_laps: Vec<&LapSummary> = valid_laps
            .iter()
            .copied()
            .filter(|l| l.session_num == session_num)
            .collect();
        if stage_laps.is_empty() {
            continue;
        }

        let best = stage_laps
            .iter()
            .filter_map(|l| l.lap_time_ms)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let Some(best_lap) = stage_laps.iter().find(|l| l.lap_time_ms == best) else {
            continue;
        };

        for sector_num in 1..=3 {
            let best_sector = best_lap
                .sectors
                .iter()
                .find(|s| s.sector_num == sector_num)
                .map(|s| s.time_ms);
            let Some(best_ms) = best_sector else { continue };

            let mut total_loss = 0.0;
            let mut count = 0;
            let mut slow_laps = Vec::new();

            for lap in &stage_laps {
                if lap.id == best_lap.id {
                    continue;
                }
                if let Some(s) = lap.sectors.iter().find(|s| s.sector_num == sector_num) {
                    let loss = s.time_ms - best_ms;
                    if loss > 50.0 {
                        total_loss += loss;
                        count += 1;
                        slow_laps.push(lap.lap_number);
                    }
                }
            }

            if count > 0 {
                let avg_loss = total_loss / count as f64;
                if weakest_loss.map(|w| avg_loss > w).unwrap_or(true) {
                    weakest_loss = Some(avg_loss);
                    weakest_sector = Some(sector_num);
                    let stage_name = stage_laps
                        .first()
                        .map(|l| l.session_type.as_str())
                        .unwrap_or("Session");
                    insights.push(CoachInsight {
                        kind: "sector_weakness".into(),
                        title: format!("{stage_name} — Sector {sector_num}"),
                        detail: format!(
                            "Average loss of {:.3}s in S{sector_num} vs your best lap ({} laps affected).",
                            avg_loss / 1000.0,
                            count
                        ),
                        severity: if avg_loss > 500.0 {
                            "warn".into()
                        } else {
                            "info".into()
                        },
                        lap_numbers: slow_laps,
                        sector_num: Some(sector_num),
                        delta_ms: Some(avg_loss),
                    });
                }
            }
        }
    }

    // Fuel trend
    let fuel_laps: Vec<_> = detail
        .laps
        .iter()
        .filter(|l| l.valid && l.fuel_used.unwrap_or(0.0) > 0.01)
        .collect();
    if fuel_laps.len() >= 3 {
        let avg_fuel: f64 = fuel_laps.iter().filter_map(|l| l.fuel_used).sum::<f64>()
            / fuel_laps.len() as f64;
        let last_fuel = fuel_laps.last().and_then(|l| l.fuel_used);
        if let Some(last) = last_fuel {
            if last > avg_fuel * 1.15 {
                insights.push(CoachInsight {
                    kind: "fuel".into(),
                    title: "Fuel usage spike".into(),
                    detail: format!(
                        "Last lap used {:.2}L vs {:.2}L average — check traffic or lift-and-coast.",
                        last, avg_fuel
                    ),
                    severity: "info".into(),
                    lap_numbers: vec![fuel_laps.last().map(|l| l.lap_number).unwrap_or(0)],
                    sector_num: None,
                    delta_ms: None,
                });
            }
        }
    }

    CoachReport {
        session_id,
        insights,
        summary: SessionCoachStats {
            valid_lap_count: valid_laps.len() as i32,
            consistency_ms,
            best_lap_ms,
            avg_lap_ms,
            weakest_sector,
            weakest_sector_loss_ms: weakest_loss,
        },
    }
}

/// Add insights derived from a linked live standings snapshot: how the player's
/// pace compared to the field, and which slow laps were likely traffic.
pub fn append_standings_insights(
    insights: &mut Vec<CoachInsight>,
    detail: &SessionDetail,
    standings: &SessionStandings,
) {
    let player_best = detail
        .laps
        .iter()
        .filter(|l| l.valid)
        .filter_map(|l| l.lap_time_ms)
        .filter(|t| *t > 0.0)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if let (Some(best), Some(fastest)) = (player_best, standings.session_fastest_ms) {
        if fastest > 0.0 {
            let gap = best - fastest;
            if gap > 50.0 {
                insights.push(CoachInsight {
                    kind: "session_pace".into(),
                    title: "Pace vs the field".into(),
                    detail: format!(
                        "Your best lap was {:.3}s off the session's fastest lap.",
                        gap / 1000.0
                    ),
                    severity: if gap > 1000.0 { "warn".into() } else { "info".into() },
                    lap_numbers: Vec::new(),
                    sector_num: None,
                    delta_ms: Some(gap),
                });
            } else {
                insights.push(CoachInsight {
                    kind: "session_pace".into(),
                    title: "Pace vs the field".into(),
                    detail: "You set the fastest lap of the session.".into(),
                    severity: "good".into(),
                    lap_numbers: Vec::new(),
                    sector_num: None,
                    delta_ms: Some(gap),
                });
            }
        }
    }

    if !standings.traffic_laps.is_empty() {
        let traffic: HashSet<i32> = standings.traffic_laps.iter().copied().collect();
        let mut lost: Vec<i32> = detail
            .laps
            .iter()
            .filter(|l| l.valid && traffic.contains(&l.iracing_lap))
            .filter(|l| l.delta_to_best_ms.map(|d| d > 500.0).unwrap_or(false))
            .map(|l| l.lap_number)
            .collect();
        lost.sort_unstable();
        lost.dedup();

        if !lost.is_empty() {
            let lap_list = lost
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            insights.push(CoachInsight {
                kind: "traffic_pace".into(),
                title: "Time lost in traffic".into(),
                detail: format!(
                    "Lap(s) {lap_list} were slow and run side-by-side with other cars — likely traffic, not a pace problem."
                ),
                severity: "info".into(),
                lap_numbers: lost,
                sector_num: None,
                delta_ms: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SessionSummary;

    fn lap(lap_number: i32, iracing_lap: i32, lap_time_ms: f64, delta_to_best_ms: f64) -> LapSummary {
        LapSummary {
            id: lap_number as i64,
            session_num: 0,
            session_type: "Race".into(),
            iracing_lap,
            lap_number,
            lap_time_ms: Some(lap_time_ms),
            valid: true,
            fuel_start: None,
            fuel_used: None,
            avg_speed: None,
            lf_temp: None,
            rf_temp: None,
            lr_temp: None,
            rr_temp: None,
            sectors: Vec::new(),
            delta_to_best_ms: Some(delta_to_best_ms),
        }
    }

    fn detail(laps: Vec<LapSummary>) -> SessionDetail {
        SessionDetail {
            session: SessionSummary {
                id: 1,
                ibt_path: String::new(),
                track: "Test".into(),
                car: "Car".into(),
                session_date: String::new(),
                lap_count: laps.len() as i32,
                best_lap_ms: None,
                imported_at: String::new(),
            },
            laps,
        }
    }

    fn standings(fastest: Option<f64>, traffic: Vec<i32>) -> SessionStandings {
        SessionStandings {
            id: 1,
            session_id: Some(1),
            track: "Test".into(),
            session_type: "Race".into(),
            session_date: String::new(),
            session_fastest_ms: fastest,
            player_best_ms: None,
            player_position: Some(3),
            player_class_position: Some(2),
            competitors: Vec::new(),
            traffic_laps: traffic,
            created_at: String::new(),
        }
    }

    #[test]
    fn session_pace_reports_gap_to_field() {
        let d = detail(vec![lap(1, 1, 90_500.0, 0.0)]);
        let s = standings(Some(90_000.0), Vec::new());
        let mut insights = Vec::new();
        append_standings_insights(&mut insights, &d, &s);
        let pace = insights.iter().find(|i| i.kind == "session_pace").unwrap();
        assert!(pace.detail.contains("0.500s off"));
    }

    #[test]
    fn session_pace_credits_fastest_lap() {
        let d = detail(vec![lap(1, 1, 90_000.0, 0.0)]);
        let s = standings(Some(90_000.0), Vec::new());
        let mut insights = Vec::new();
        append_standings_insights(&mut insights, &d, &s);
        let pace = insights.iter().find(|i| i.kind == "session_pace").unwrap();
        assert_eq!(pace.severity, "good");
    }

    #[test]
    fn traffic_pace_flags_slow_traffic_laps() {
        let d = detail(vec![
            lap(1, 5, 91_000.0, 1_000.0), // slow and in traffic -> flagged
            lap(2, 6, 90_100.0, 100.0),   // in traffic but fast -> ignored
            lap(3, 7, 92_000.0, 2_000.0), // slow but not in traffic -> ignored
        ]);
        let s = standings(Some(90_000.0), vec![5, 6]);
        let mut insights = Vec::new();
        append_standings_insights(&mut insights, &d, &s);
        let traffic = insights.iter().find(|i| i.kind == "traffic_pace").unwrap();
        assert_eq!(traffic.lap_numbers, vec![1]);
    }
}
