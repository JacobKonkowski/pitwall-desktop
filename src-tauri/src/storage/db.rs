use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::models::*;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY,
    ibt_path TEXT UNIQUE NOT NULL,
    file_hash TEXT NOT NULL,
    track TEXT NOT NULL DEFAULT '',
    car TEXT NOT NULL DEFAULT '',
    session_date TEXT NOT NULL DEFAULT '',
    lap_count INTEGER NOT NULL DEFAULT 0,
    best_lap_ms REAL,
    imported_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS laps (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    session_num INTEGER NOT NULL DEFAULT 0,
    session_type TEXT NOT NULL DEFAULT '',
    iracing_lap INTEGER NOT NULL DEFAULT 0,
    lap_number INTEGER NOT NULL,
    lap_time_ms REAL,
    valid INTEGER NOT NULL DEFAULT 1,
    lap_kind TEXT NOT NULL DEFAULT 'flying',
    fuel_start REAL,
    fuel_used REAL,
    avg_speed REAL,
    lf_temp REAL,
    rf_temp REAL,
    lr_temp REAL,
    rr_temp REAL,
    UNIQUE(session_id, session_num, lap_number)
);

CREATE TABLE IF NOT EXISTS sectors (
    id INTEGER PRIMARY KEY,
    lap_id INTEGER NOT NULL REFERENCES laps(id) ON DELETE CASCADE,
    sector_num INTEGER NOT NULL,
    time_ms REAL NOT NULL,
    UNIQUE(lap_id, sector_num)
);

CREATE TABLE IF NOT EXISTS lap_traces (
    id INTEGER PRIMARY KEY,
    lap_id INTEGER NOT NULL REFERENCES laps(id) ON DELETE CASCADE,
    dist_pct REAL NOT NULL,
    speed REAL NOT NULL,
    throttle REAL NOT NULL,
    brake REAL NOT NULL,
    gear INTEGER NOT NULL,
    steering REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS session_standings (
    id INTEGER PRIMARY KEY,
    session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    track TEXT NOT NULL DEFAULT '',
    session_type TEXT NOT NULL DEFAULT '',
    session_date TEXT NOT NULL DEFAULT '',
    session_fastest_ms REAL,
    player_best_ms REAL,
    player_position INTEGER,
    player_class_position INTEGER,
    competitors_json TEXT NOT NULL DEFAULT '[]',
    traffic_laps_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_laps_session ON laps(session_id);
CREATE INDEX IF NOT EXISTS idx_sectors_lap ON sectors(lap_id);
CREATE INDEX IF NOT EXISTS idx_traces_lap ON lap_traces(lap_id);
CREATE INDEX IF NOT EXISTS idx_standings_session ON session_standings(session_id);
";

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path).context("open sqlite database")?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch(SCHEMA)?;
        migrate_schema(&conn)?;
        Ok(Self { conn })
    }

    fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            if row.get::<_, String>(1)? == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn hash_exists(&self, hash: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE file_hash = ?1",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn path_exists(&self, path: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE ibt_path = ?1",
            params![path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn insert_session(
        &self,
        ibt_path: &str,
        file_hash: &str,
        track: &str,
        car: &str,
        session_date: &str,
        laps: &[StoredLap],
    ) -> Result<i64> {
        let imported_at = chrono::Utc::now().to_rfc3339();
        let valid_laps: Vec<_> = laps.iter().filter(|l| l.valid && l.lap_time_ms.is_some()).collect();
        let best_lap_ms = valid_laps
            .iter()
            .filter_map(|l| l.lap_time_ms)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let lap_count = laps.len() as i32;

        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO sessions (ibt_path, file_hash, track, car, session_date, lap_count, best_lap_ms, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![ibt_path, file_hash, track, car, session_date, lap_count, best_lap_ms, imported_at],
        )?;
        let session_id = tx.last_insert_rowid();

        let mut lap_stmt = tx.prepare(
            "INSERT INTO laps (session_id, session_num, session_type, iracing_lap, lap_number, lap_time_ms, valid, lap_kind, fuel_start, fuel_used, avg_speed, lf_temp, rf_temp, lr_temp, rr_temp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        )?;
        let mut sector_stmt = tx.prepare(
            "INSERT INTO sectors (lap_id, sector_num, time_ms) VALUES (?1, ?2, ?3)",
        )?;
        let mut trace_stmt = tx.prepare(
            "INSERT INTO lap_traces (lap_id, dist_pct, speed, throttle, brake, gear, steering)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;

        for lap in laps {
            lap_stmt.execute(params![
                session_id,
                lap.session_num,
                lap.session_type,
                lap.iracing_lap,
                lap.lap_number,
                lap.lap_time_ms,
                lap.valid as i32,
                lap.lap_kind.as_str(),
                lap.fuel_start,
                lap.fuel_used,
                lap.avg_speed,
                lap.lf_temp,
                lap.rf_temp,
                lap.lr_temp,
                lap.rr_temp,
            ])?;
            let lap_id = tx.last_insert_rowid();

            for (sector_num, time_ms) in &lap.sectors {
                sector_stmt.execute(params![lap_id, sector_num, time_ms])?;
            }

            for point in &lap.traces {
                trace_stmt.execute(params![
                    lap_id,
                    point.dist_pct,
                    point.speed,
                    point.throttle,
                    point.brake,
                    point.gear,
                    point.steering,
                ])?;
            }
        }

        drop(lap_stmt);
        drop(sector_stmt);
        drop(trace_stmt);
        tx.commit()?;
        Ok(session_id)
    }

    /// Remove all imported sessions and related lap data.
    pub fn clear_all(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
        self.conn.execute_batch(
            "DELETE FROM lap_traces;
             DELETE FROM sectors;
             DELETE FROM laps;
             DELETE FROM sessions;
             DELETE FROM session_standings;",
        )?;
        Ok(count as usize)
    }

    /// Persist a post-session standings snapshot. Returns the new row id.
    pub fn insert_standings(&self, standings: &SessionStandings) -> Result<i64> {
        let competitors_json = serde_json::to_string(&standings.competitors)?;
        let traffic_laps_json = serde_json::to_string(&standings.traffic_laps)?;
        self.conn.execute(
            "INSERT INTO session_standings (
                session_id, track, session_type, session_date, session_fastest_ms,
                player_best_ms, player_position, player_class_position,
                competitors_json, traffic_laps_json, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                standings.session_id,
                standings.track,
                standings.session_type,
                standings.session_date,
                standings.session_fastest_ms,
                standings.player_best_ms,
                standings.player_position,
                standings.player_class_position,
                competitors_json,
                traffic_laps_json,
                standings.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Link the most recent unlinked standings snapshot for the same track to a
    /// freshly imported IBT session, so the post-session view can show the field.
    pub fn link_standings_to_session(&self, session_id: i64) -> Result<bool> {
        let track: String = match self.conn.query_row(
            "SELECT track FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        ) {
            Ok(t) => t,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
            Err(e) => return Err(e.into()),
        };

        let standings_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM session_standings
                 WHERE session_id IS NULL AND track = ?1
                 ORDER BY created_at DESC LIMIT 1",
                params![track],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = standings_id {
            self.conn.execute(
                "UPDATE session_standings SET session_id = ?1 WHERE id = ?2",
                params![session_id, id],
            )?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_standings_for_session(&self, session_id: i64) -> Result<Option<SessionStandings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, track, session_type, session_date, session_fastest_ms,
                    player_best_ms, player_position, player_class_position,
                    competitors_json, traffic_laps_json, created_at
             FROM session_standings WHERE session_id = ?1
             ORDER BY created_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![session_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let competitors_json: String = row.get(9)?;
        let traffic_laps_json: String = row.get(10)?;
        Ok(Some(SessionStandings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            track: row.get(2)?,
            session_type: row.get(3)?,
            session_date: row.get(4)?,
            session_fastest_ms: row.get(5)?,
            player_best_ms: row.get(6)?,
            player_position: row.get(7)?,
            player_class_position: row.get(8)?,
            competitors: serde_json::from_str(&competitors_json).unwrap_or_default(),
            traffic_laps: serde_json::from_str(&traffic_laps_json).unwrap_or_default(),
            created_at: row.get(11)?,
        }))
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ibt_path, track, car, session_date, lap_count, best_lap_ms, imported_at
             FROM sessions ORDER BY imported_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SessionSummary {
                id: row.get(0)?,
                ibt_path: row.get(1)?,
                track: row.get(2)?,
                car: row.get(3)?,
                session_date: row.get(4)?,
                lap_count: row.get(5)?,
                best_lap_ms: row.get(6)?,
                imported_at: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_session(&self, session_id: i64) -> Result<Option<SessionDetail>> {
        let session = match self.get_session_summary(session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };
        let laps = self.get_laps_for_session(session_id, session.best_lap_ms)?;
        Ok(Some(SessionDetail { session, laps }))
    }

    fn get_session_summary(&self, session_id: i64) -> Result<Option<SessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ibt_path, track, car, session_date, lap_count, best_lap_ms, imported_at
             FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(SessionSummary {
                id: row.get(0)?,
                ibt_path: row.get(1)?,
                track: row.get(2)?,
                car: row.get(3)?,
                session_date: row.get(4)?,
                lap_count: row.get(5)?,
                best_lap_ms: row.get(6)?,
                imported_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_laps_for_session(&self, session_id: i64, _best_lap_ms: Option<f64>) -> Result<Vec<LapSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_num, session_type, iracing_lap, lap_number, lap_time_ms, valid, lap_kind, fuel_start, fuel_used, avg_speed, lf_temp, rf_temp, lr_temp, rr_temp
             FROM laps WHERE session_id = ?1 ORDER BY session_num, lap_number",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let lap_kind_str: String = row.get(7)?;
            Ok(LapSummary {
                id: row.get(0)?,
                session_num: row.get(1)?,
                session_type: row.get(2)?,
                iracing_lap: row.get(3)?,
                lap_number: row.get(4)?,
                lap_time_ms: row.get(5)?,
                valid: row.get::<_, i32>(6)? != 0,
                lap_kind: LapKind::from_str(&lap_kind_str).unwrap_or(LapKind::Flying),
                fuel_start: row.get(8)?,
                fuel_used: row.get(9)?,
                avg_speed: row.get(10)?,
                lf_temp: row.get(11)?,
                rf_temp: row.get(12)?,
                lr_temp: row.get(13)?,
                rr_temp: row.get(14)?,
                sectors: Vec::new(),
                delta_to_best_ms: None,
            })
        })?;
        let mut laps: Vec<LapSummary> = rows.collect::<Result<Vec<_>, _>>()?;

        let mut best_by_subsession: std::collections::HashMap<i32, f64> = std::collections::HashMap::new();
        for lap in &laps {
            if lap.valid {
                if let Some(lt) = lap.lap_time_ms {
                    best_by_subsession
                        .entry(lap.session_num)
                        .and_modify(|best| {
                            if lt < *best {
                                *best = lt;
                            }
                        })
                        .or_insert(lt);
                }
            }
        }

        for lap in &mut laps {
            // Only valid laps get a delta to best; invalid out/pit/partial laps
            // would otherwise show a misleading gap against the valid-only baseline.
            lap.delta_to_best_ms = match (lap.valid, lap.lap_time_ms, best_by_subsession.get(&lap.session_num)) {
                (true, Some(lt), Some(best)) if lt > 0.0 && *best > 0.0 => Some(lt - best),
                _ => None,
            };
            lap.sectors = self.get_sectors(lap.id)?;
        }
        Ok(laps)
    }

    fn get_sectors(&self, lap_id: i64) -> Result<Vec<SectorTime>> {
        let mut stmt = self.conn.prepare(
            "SELECT sector_num, time_ms FROM sectors WHERE lap_id = ?1 ORDER BY sector_num",
        )?;
        let rows = stmt.query_map(params![lap_id], |row| {
            Ok(SectorTime {
                sector_num: row.get(0)?,
                time_ms: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_lap_traces(&self, lap_ids: &[i64]) -> Result<Vec<LapTrace>> {
        let mut traces = Vec::new();
        for lap_id in lap_ids {
            let lap_number: i32 = self.conn.query_row(
                "SELECT lap_number FROM laps WHERE id = ?1",
                params![lap_id],
                |row| row.get(0),
            )?;
            let mut stmt = self.conn.prepare(
                "SELECT dist_pct, speed, throttle, brake, gear, steering
                 FROM lap_traces WHERE lap_id = ?1 ORDER BY dist_pct",
            )?;
            let points = stmt
                .query_map(params![lap_id], |row| {
                    Ok(TracePoint {
                        dist_pct: row.get(0)?,
                        speed: row.get(1)?,
                        throttle: row.get(2)?,
                        brake: row.get(3)?,
                        gear: row.get(4)?,
                        steering: row.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            traces.push(LapTrace {
                lap_id: *lap_id,
                lap_number,
                points,
            });
        }
        Ok(traces)
    }

    pub fn get_fuel_summary(&self, session_id: i64) -> Result<FuelSummary> {
        let detail = self
            .get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("session not found"))?;
        let mut laps = Vec::new();
        let mut prev_remaining: Option<f64> = None;

        for lap in &detail.laps {
            if !lap.valid {
                continue;
            }
            if let (Some(fuel_start), Some(fuel_used)) = (lap.fuel_start, lap.fuel_used) {
                let fuel_remaining = fuel_start - fuel_used;
                let laps_remaining_estimate = if fuel_used > 0.01 {
                    Some(fuel_remaining / fuel_used)
                } else {
                    prev_remaining
                };
                prev_remaining = laps_remaining_estimate;
                laps.push(FuelLapSummary {
                    lap_number: lap.lap_number,
                    fuel_used,
                    fuel_remaining,
                    laps_remaining_estimate,
                });
            }
        }

        Ok(FuelSummary {
            laps,
            tank_capacity: detail.laps.first().and_then(|l| l.fuel_start),
        })
    }

    pub fn get_tire_summary(&self, session_id: i64) -> Result<TireSummary> {
        let detail = self
            .get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("session not found"))?;
        let laps = detail
            .laps
            .iter()
            .filter(|lap| lap.valid)
            .filter_map(|lap| {
                Some(TireLapSummary {
                    lap_number: lap.lap_number,
                    lf_temp: lap.lf_temp?,
                    rf_temp: lap.rf_temp?,
                    lr_temp: lap.lr_temp?,
                    rr_temp: lap.rr_temp?,
                })
            })
            .collect();

        Ok(TireSummary {
            laps,
            note: "Tire wear updates on some cars only after pit stops. Temps are lap averages.".into(),
        })
    }
}

pub fn db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pitwall-desktop")
        .join("pitwall.db")
}

fn migrate_schema(conn: &Connection) -> Result<()> {
    if !Database::table_has_column(conn, "laps", "session_num")? {
        conn.execute_batch(
            "
        CREATE TABLE laps_migrated (
            id INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            session_num INTEGER NOT NULL DEFAULT 0,
            session_type TEXT NOT NULL DEFAULT '',
            iracing_lap INTEGER NOT NULL DEFAULT 0,
            lap_number INTEGER NOT NULL,
            lap_time_ms REAL,
            valid INTEGER NOT NULL DEFAULT 1,
            lap_kind TEXT NOT NULL DEFAULT 'flying',
            fuel_start REAL,
            fuel_used REAL,
            avg_speed REAL,
            lf_temp REAL,
            rf_temp REAL,
            lr_temp REAL,
            rr_temp REAL,
            UNIQUE(session_id, session_num, lap_number)
        );
        INSERT INTO laps_migrated (
            id, session_id, session_num, session_type, iracing_lap, lap_number,
            lap_time_ms, valid, fuel_start, fuel_used, avg_speed,
            lf_temp, rf_temp, lr_temp, rr_temp
        )
        SELECT
            id, session_id, 0, '', lap_number, lap_number,
            lap_time_ms, valid, fuel_start, fuel_used, avg_speed,
            lf_temp, rf_temp, lr_temp, rr_temp
        FROM laps;
        DROP TABLE laps;
        ALTER TABLE laps_migrated RENAME TO laps;
        CREATE INDEX IF NOT EXISTS idx_laps_session ON laps(session_id);
        ",
        )?;
    }

    if !Database::table_has_column(conn, "laps", "lap_kind")? {
        conn.execute_batch(
            "ALTER TABLE laps ADD COLUMN lap_kind TEXT NOT NULL DEFAULT 'flying';",
        )?;
    }

    Ok(())
}
