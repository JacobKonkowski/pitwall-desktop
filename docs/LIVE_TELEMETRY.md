# Live telemetry

The live monitor connects to iRacing shared memory, builds a rich `LiveSnapshot`, emits UI events at 10 Hz, publishes VR shared memory at ~30 Hz, and persists standings on disconnect.

---

## Connection loop

`LiveService` in [`live/mod.rs`](../src-tauri/src/live/mod.rs):

1. `Pitwall::connect().await` — shared-memory connection
2. On failure: `Reconnecting` state with exponential backoff
3. On success: `Connected`; session intro for audio; sector boundaries from YAML
4. On disconnect: persist standings, scan recent IBT for auto-import, reset snapshot

States (`LiveConnectionState`): `disconnected`, `waitingForSession`, `reconnecting`, `connected`, `error`.

---

## Dual subscriptions

| Stream | Rate | Purpose |
|--------|------|---------|
| `AnalysisFrame` | Max 10 Hz | Lap counter, `LapDistPct`, sector crossings (`SessionTime`), fuel, temps |
| `CarIdxFrame` | Max 4 Hz | Player lap clock, field data, flags, pack, session deltas |

`session_updates()` provides track/car name, sector boundaries, and driver roster (`competitors::build_roster`).

### AnalysisFrame (geometry + sector timing)

- `Lap`, `LapDistPct`, `SessionTime` — sector crossing detection only
- No lap-clock fields (those come from CarIdx)

### CarIdxFrame (lap clock + field)

| SDK field | Snapshot |
|-----------|----------|
| `LapCurrentLapTime` | `lapTimeMs` |
| `LapLastLapTime` | `lastLapMs` |
| `LapBestLapTime` | `bestLapMs` |
| `LapDeltaToBestLap` | `deltaToBestMs` |
| `LapDeltaToLastLap` | `deltaToLastMs` |
| `SessionBestLapTime` | `sessionFastestLapMs` (fallback: min `CarIdxBestLapTime`) |
| `LapDeltaToSessionBestLap` / `…OptimalLap` | session deltas |
| `CarIdx*` arrays | leaderboard, gaps, pack |

---

## Sector tracking

`LiveTracker` ([`tracker.rs`](../src-tauri/src/live/tracker.rs)) detects sector crossings by **lap distance %** edge crossing (aligned with post-session [`sector_splitter.rs`](../src-tauri/src/analysis/sector_splitter.rs)):

- Regions from `SplitTimeInfo.Sectors[].SectorStartPct` (includes 0% start; dynamic S1..SN)
- Final sector completes at lap finish
- **Current sector** — SDK pattern via `current_sector_from_pct`
- **Mid-lap join** — skip passed split lines based on `lapDistPct`
- `pending_lap_sectors` — sectors completed on the lap being closed, passed to audio at lap change
- `sectorBoundaries` on snapshot for UI/VR progress bars

Live sector times feed the audio coach and overlay widgets.

---

## Field merge

`merge_car_idx` folds `CarIdxFrame` into the snapshot:

- **Player lap clock** — SDK fields listed above (overwrites tracker defaults)
- **Leaderboard** — [`competitors.rs`](../src-tauri/src/live/competitors.rs): positions, class, best/last lap, gap to player
- **Gaps** — `CarIdxF2Time` differences between adjacent cars (ahead/behind player)
- **Session deltas** — `LapDeltaToSessionBestLap`, `LapDeltaToSessionOptimalLap`
- **Pack** — [`pack.rs`](../src-tauri/src/live/pack.rs) from `CarLeftRight` (Int32 enum)
- **Flags, incidents, session remain, pits open, on-track**

Traffic laps (side-by-side per `pack_state.is_traffic()`) accumulate for the standings snapshot.

---

## Event throttle

| Consumer | Rate | Mechanism |
|----------|------|-----------|
| React UI (`live-telemetry`, `live-status`) | ~10 Hz | 100 ms emit throttle in live loop |
| VR SHM (`Local\PitWallVR`) | ~30 Hz | `vr/shm.rs` seqlock writer |
| Audio coach | 4 Hz effective | 250 ms poll in `AudioCoachService` |

UI can also poll `get_live_snapshot` on demand.

---

## Standings persistence

On disconnect, `persist_standings` writes `session_standings` (field JSON + traffic lap list). When an IBT from the same track imports within a recency window, the row links to `sessions.id` for coach `session_pace` / `traffic_pace` insights.

---

## Post-session IBT hook

After disconnect, scans `Documents/iRacing/telemetry/` for IBT files modified in the last **10 minutes** and imports them (same pipeline as manual import).

---

## VR shared memory

Compact `LiveSnapshot` mirror + per-widget placement written to `Local\PitWallVR`. Layout v2 supports up to 8 sector slots. Defined in [`openxr-layer/include/pitwall_vr_shm.h`](../openxr-layer/include/pitwall_vr_shm.h). See [NATIVE_VR.md](NATIVE_VR.md).

---

## Related docs

- [COMPARISON.md](COMPARISON.md) — SDK fields used
- [AUDIO_COACH.md](AUDIO_COACH.md) — what the coach reads from the snapshot
- [DATA_MODEL.md](DATA_MODEL.md) — `LiveSnapshot` field groups
- [DESIGN_NOTES.md](DESIGN_NOTES.md) — remaining SDK deviations
