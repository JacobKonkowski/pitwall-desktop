# Data model

SQLite at `%LOCALAPPDATA%\pitwall-desktop\` (see `storage/db.rs`). **`PRAGMA user_version = 2`**.

Opening an older DB drops analysis tables (`sessions`, `laps`, `sectors`, `lap_traces`) and requires reimport.

## Tables

### `sessions`

| Column | Notes |
|--------|--------|
| `ibt_path`, `file_hash` | Dedup + source |
| `track`, `car`, `session_date` | Metadata |
| `lap_count`, `best_lap_ms` | Summary (best is pace-eligible; list/detail may refresh from cleaned laps) |
| `imported_at` | ISO timestamp |

### `laps` (schema v2)

| Column | Notes |
|--------|--------|
| `session_num`, `session_type`, `iracing_lap`, `lap_number` | Identity |
| `lap_time_ms` | Official time when present (sticky copies cleared at import / on read) |
| `delta_best_ok`, `delta_session_best_ok` | iRacing `_OK` integers |
| `on_pit_road_start`, `on_pit_road_end` | Pit-road samples |
| `lap_dist_pct_min`, `lap_dist_pct_max` | Coverage (pace eligibility needs max ≥ 0.95) |
| `pace_eligible` | Derived: time + both `_OK` + full coverage (see [ANALYSIS.md](ANALYSIS.md)) |
| `fuel_*`, `avg_speed`, tire temps | Optional aggregates |

### `sectors` / `lap_traces`

Per-lap sector times and distance-sampled traces (speed, throttle, brake, gear, steering).

## Settings

JSON beside the DB (`settings/mod.rs` → `AppSettings`). Includes:

- `vrMode` (`native` \| `web`), HUD offset/opacity, recenter hotkey
- `overlayLayout` — four VR widget slots + field pace mode
- Audio coach rate/volume/voice, chatter level, category toggles, gaps

Some older geometry fields may still exist in the JSON for migration; the UI drives VR slots via `overlayLayout`.

Frontend types: `src/shared/types.ts`.
