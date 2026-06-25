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
| `AnalysisFrame` | Max 10 Hz | Player lap, sectors, fuel, temps, lap dist |
| `CarIdxFrame` | Max 4 Hz | All cars — positions, gaps, flags, pack |

`session_updates()` provides track/car name, sector boundaries, and driver roster (`competitors::build_roster`).

---

## Sector tracking

`LiveTracker` ([`tracker.rs`](../src-tauri/src/live/tracker.rs)) detects sector crossings by **lap distance %** edge crossing (aligned with post-session [`sector_splitter.rs`](../src-tauri/src/analysis/sector_splitter.rs)):

- Sector 0 at 0% is ignored
- Sector 3 completes at lap end (not only at a YAML boundary)
- **Mid-lap join** — current sector inferred from `lapDistPct`
- `pending_lap_sectors` — sectors completed on the lap being closed, passed to audio at lap change

Live sector times feed the audio coach and overlay widgets.

---

## Field merge

`merge_car_idx` folds `CarIdxFrame` into the snapshot:

- **Leaderboard** — [`competitors.rs`](../src-tauri/src/live/competitors.rs): positions, class, best/last lap, gap to player
- **Gaps** — `CarIdxF2Time` differences between adjacent cars (ahead/behind player)
- **Session deltas** — `LapDeltaToSessionBestLap`, `LapDeltaToSessionOptimalLap`
- **Pack** — [`pack.rs`](../src-tauri/src/live/pack.rs) from `CarLeftRight` (Int32 enum)
- **Flags, incidents, fuel, session remain, pits open, on-track**

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

Compact `LiveSnapshot` mirror + per-widget placement written to `Local\PitWallVR`. Layout defined in [`openxr-layer/include/pitwall_vr_shm.h`](../openxr-layer/include/pitwall_vr_shm.h). See [NATIVE_VR.md](NATIVE_VR.md).

---

## Related docs

- [COMPARISON.md](COMPARISON.md) — SDK fields used
- [AUDIO_COACH.md](AUDIO_COACH.md) — what the coach reads from the snapshot
- [DATA_MODEL.md](DATA_MODEL.md) — `LiveSnapshot` field groups
