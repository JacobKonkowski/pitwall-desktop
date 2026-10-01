# Data model

SQLite at `%LOCALAPPDATA%\pitwall-desktop\` (see `storage/db.rs`). **`PRAGMA user_version = 5`**.

Opening a pre-v2 DB drops analysis tables (`sessions`, `laps`, `sectors`, `lap_traces`) and requires reimport. v2 → v5 are additive migrations on `lap_traces` (GPS, elapsed time, raw pedals), so existing sessions survive — they just have no GPS / elapsed time / driver-pedal channels until re-imported.

## Tables

### `sessions`

| Column | Notes |
|--------|--------|
| `ibt_path`, `file_hash` | Dedup + source |
| `track`, `car`, `session_date` | Metadata |
| `lap_count`, `best_lap_ms` | Summary (best is pace-eligible; list/detail may refresh from cleaned laps) |
| `imported_at` | ISO timestamp |

`SessionSummary` also exposes a derived `session_type` (not a stored column): the `laps.session_type` of the highest `session_num` (latest stint) for that session, recomputed alongside `lap_count`/`best_lap_ms` whenever laps are loaded.

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

`lap_traces.lat` / `.lon` (schema v3, nullable) keep the GPS for each sample. They are `NULL` for sessions imported before v3 and for IBTs without GPS channels.

`lap_traces.elapsed_ms` (schema v4, nullable) is the time since the lap's first frame. Older rows leave it `NULL`.

`throttle` / `brake` are the **applied** values (`Throttle` / `Brake`), after auto-blip, traction control and ABS. Schema v5 adds the driver's pedals alongside them, all nullable:

| Column | SDK channel | Notes |
|--------|-------------|-------|
| `throttle_raw` | `ThrottleRaw` | Driver throttle; no downshift blips |
| `brake_raw` | `BrakeRaw` | Driver brake, before ABS |
| `clutch` | `Clutch` | Applied clutch (0 = disengaged, 1 = engaged) |
| `clutch_raw` | `ClutchRaw` | Driver clutch pedal |
| `handbrake_raw` | `HandbrakeRaw` | Driver handbrake |

They are `NULL` for sessions imported before v5 and for IBTs without the channel. Consumers that need driver intent fall back to applied `throttle` / `brake`. Clutch and handbrake are stored only — nothing displays them yet.

## Settings

JSON beside the DB (`settings/mod.rs` → `AppSettings`). Includes:

- `vrMode` (`native` \| `web`), HUD offset/opacity, recenter hotkey
- `overlayLayout` — four widget slots (monitor + VR) + field pace mode
- Audio coach rate/volume/voice, chatter level, category toggles, gaps

Shared `enabled` flags; `desktop*` places monitor windows, `vr*` places the in-headset HUD (enable once, place twice).

Some older geometry fields may still exist in the JSON for migration; the UI drives slots via `overlayLayout`.

Frontend types: `src/shared/types.ts`.
