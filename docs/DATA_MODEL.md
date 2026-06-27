# Data model

SQLite, settings JSON, live snapshot shapes, and on-disk paths.

---

## On-disk paths

| Path | Contents |
|------|----------|
| `%LOCALAPPDATA%\pitwall-desktop\pitwall.db` | Sessions, laps, sectors, traces, standings |
| `%LOCALAPPDATA%\pitwall-desktop\settings.json` | User preferences |
| AppData OpenXR layer staging | See [NATIVE_VR.md](NATIVE_VR.md) — implicit API layer install |

**PRAGMA:** `journal_mode=WAL`, `synchronous=NORMAL`

---

## SQLite tables

### `sessions`

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | |
| `ibt_path` | TEXT UNIQUE | Source file |
| `file_hash` | TEXT | SHA256 dedup |
| `track`, `car` | TEXT | From session YAML |
| `session_date` | TEXT | ISO |
| `lap_count` | INTEGER | |
| `best_lap_ms` | REAL | |
| `imported_at` | TEXT | ISO |
| `sector_boundaries_json` | TEXT | Region start pcts from SplitTimeInfo (JSON array) |

### `laps`

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | |
| `session_id` | FK | CASCADE delete |
| `session_num` | INTEGER | iRacing sub-session |
| `session_type` | TEXT | PRACTICE / QUALIFY / RACE |
| `iracing_lap` | INTEGER | Raw counter |
| `lap_number` | INTEGER | Sequential within sub-session |
| `lap_time_ms` | REAL | |
| `valid` | INTEGER | 0/1 |
| `lap_kind` | TEXT | `flying`, `pitOut`, `pitIn`, `pitLane`, `partial` |
| `fuel_start`, `fuel_used` | REAL | |
| `avg_speed` | REAL | |
| `lf_temp` … `rr_temp` | REAL | Lap averages |

**UNIQUE:** `(session_id, session_num, lap_number)`

### `sectors`

| Column | Type |
|--------|------|
| `lap_id` | FK → laps |
| `sector_num` | INTEGER | 1..N (track-dependent) |
| `time_ms` | REAL |

### `lap_traces`

Downsampled: `dist_pct`, `speed`, `throttle`, `brake`, `gear`, `steering`.

### `session_standings`

Live disconnect snapshot; linked to imported IBT by track + recency.

| Column | Notes |
|--------|-------|
| `session_id` | FK, nullable |
| `competitors_json` | Leaderboard rows |
| `traffic_laps_json` | iRacing lap numbers in traffic |
| `session_fastest_ms`, `player_best_ms` | |
| `player_position`, `player_class_position` | |

---

## LiveSnapshot (groups)

Rust: [`live/snapshot.rs`](../src-tauri/src/live/snapshot.rs). TypeScript: `LiveSnapshot` in [`types.ts`](../src/lib/types.ts).

| Group | Fields |
|-------|--------|
| Session | `track`, `car`, `sessionType` |
| Player lap | `lap`, `lapTimeMs`, `lastLapMs`, `bestLapMs`, deltas (from SDK), `lapDistPct`, `currentSector`, `sectorBoundaries[]`, `sectors[]` |
| Car state | `fuelLevel`, `speed`, tire temps, `onPitRoad`, `onTrack` |
| Field | `competitors[]`, `playerPosition`, `playerClassPosition`, gaps, session deltas |
| Race | `sessionFlags`, `incidentCount`, `sessionLapsRemain`, `sessionTimeRemainS`, `pitsOpen` |
| Pack | `packState` |

VR SHM v2 uses a compact binary layout (up to 8 sector slots) — [`pitwall_vr_shm.h`](../openxr-layer/include/pitwall_vr_shm.h).

---

## AppSettings

Persisted to `settings.json`. UI: Live tab → Settings.

| Field | Default | UI / notes |
|-------|---------|------------|
| `ollamaUrl` | `http://localhost:11434` | Ollama |
| `ollamaModel` | `llama3.2` | Ollama |
| `overlayX/Y/Width/Height` | 100, 100, 720, 520 | Desktop overlay window |
| `vrOverlayEnabled` | `false` | Auto-start VR with live |
| `vrMode` | `"native"` | `"native"` or `"web"` |
| `vrOverlayScale` | `1.0` | Legacy + coach VR scale |
| `vrHudOffset` | `0.0` | VR vertical nudge (m) |
| `vrHudOpacity` | `1.0` | VR opacity |
| `vrRecenterHotkey` | `""` | Optional global hotkey |
| `vrFieldPaceMode` | `"best"` | Coach field pace: best/optimal/both |
| `overlayLayout` | see `OverlayLayout::default()` | Per-widget enable + placement |
| `audioCoachEnabled` | `true` | Auto-start with live |
| `audioCoachRate` | `1.0` | WinRT rate |
| `audioCoachVolume` | `1.0` | Playback volume |
| `audioCoachFuelThreshold` | `5.0` | Liters; `0` disables low-fuel |
| `audioPackAlertsEnabled` | `true` | Pack spotter (traffic + clear) |
| `audioFlagsEnabled` | `true` | Flag callouts |
| `audioIncidentsEnabled` | `true` | Incident count |
| `audioFuelRaceEnabled` | `true` | Race fuel strategy |
| `audioGapAlertsEnabled` | `true` | Gap ahead/behind |
| `audioPaceEnabled` | `true` | Sector/lap pace |
| `audioStrategyEnabled` | `true` | Fuel pit planning |
| `audioRaceClockEnabled` | `true` | Time/lap milestones |
| `audioPitsOpenEnabled` | `true` | Pits open |
| `audioCoachChatterLevel` | `"normal"` | minimal / normal / verbose |

---

## Related docs

- [API.md](API.md) — IPC types
- [ARCHITECTURE.md](ARCHITECTURE.md) — system overview
