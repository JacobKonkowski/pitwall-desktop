# Design notes

Short rationale for non-obvious decisions (ADR-lite). Each entry: context → decision → consequences.

---

## Sector splits: iRacing SplitTimeInfo model

**Context:** iRacing session YAML lists `SplitTimeInfo.Sectors[]` with `SectorStartPct` marking where each timed region **begins** (sector 0 at 0%). Track layouts vary (3, 4, or more sectors). The previous engine capped interior splits at two, hardcoded S1–S3 in the UI, and computed lap times from `SessionTime` deltas.

**Decision:** [`sector_splitter.rs`](../src-tauri/src/analysis/sector_splitter.rs) builds region starts from YAML (includes 0%, drops ~100% finish marker), derives N sectors dynamically, and times crossings from `LapDistPct` + `SessionTime`. Display uses 1-indexed labels S1..SN. `current_sector_from_pct` follows the SDK pattern (max region where `pct > start`). When YAML has no sector data, suppress sector display (show "—") rather than guessing equal thirds. Live lap start clamps `LapDistPct > 0.9` to 0.0 when `Lap` increments before the distance wrap.

**Consequences:** Live and IBT import share one canonical engine. Four-sector tracks (e.g. 0/26/51/69%) show S1–S4 matching in-sim timing. UI, audio coach, VR SHM, and lap table columns scale with sector count.

---

## Live lap clock: SDK fields

**Context:** Player lap time, last/best, and deltas were computed from `SessionTime` deltas in the tracker, which could drift from the in-sim timing box.

**Decision:** [`CarIdxFrame`](../src-tauri/src/live/car_idx_frame.rs) subscribes to `LapCurrentLapTime`, `LapLastLapTime`, `LapBestLapTime`, `LapDeltaToBestLap`, `LapDeltaToLastLap`, and `SessionBestLapTime`. [`merge_car_idx`](../src-tauri/src/live/mod.rs) writes these into `LiveSnapshot` each tick. Sector crossing still uses `SessionTime` (SDK provides no live sector-time telemetry).

**Consequences:** Live HUD matches iRacing lap clock and deltas. IBT import still uses SessionTime frame deltas (documented deviation).

---

## Gaps use `CarIdxF2Time`, not physical distance

**Context:** SDK exposes time-behind-leader per car, not reliable on-track distance to neighbors.

**Decision:** Gap ahead/behind = difference in `CarIdxF2Time` between adjacent positions.

**Consequences:** Gaps match iRacing timing screens; not useful for spatial "meters to car ahead" in traffic.

---

## `CarLeftRight` is Int32 enum, not a bitfield

**Context:** Early code treated pack state as flags; iRacing sends discrete enum values.

**Decision:** `car_idx_frame.rs` reads Int32; `pack.rs` maps explicit variants (clear, left, right, three-wide, two-wide).

**Consequences:** Correct three-wide and two-wide callouts; no false combos from bitwise OR.

---

## One alert per poll + speech queue

**Context:** Unbounded TTS would stack unintelligibly under yellow + pack + lap complete.

**Decision:** Coach picks highest priority per 250 ms tick; `SpeechQueue` plays one plan at a time.

**Consequences:** Lower priority waits; nothing is silently dropped except by chatter-level filtering.

---

## Path B: clips for fixed phrases, WinRT for numbers

**Context:** Neural TTS in-process adds latency, GPU/CPU load, and packaging complexity during a race.

**Decision:** Ship WAV clips for flags/pack/fuel phrases; WinRT synthesizes only dynamic numbers/strings at runtime. Neural voices used only in `gen-audio-clips` at dev time.

**Consequences:** Predictable latency; voice quality depends on committed WAVs; no ONNX in the hot path.

---

## Native OpenXR layer vs pre-rendered SHM pixels

**Context:** Compositing pre-rendered browser bitmaps in SHM was explored; text clarity and widget layout suffered.

**Decision:** C++ implicit API layer hooks `xrEndFrame`, draws with Direct2D per widget slot, reads structured SHM.

**Consequences:** Crisp text in VR; requires layer install and OpenXR mode; see [NATIVE_VR.md](NATIVE_VR.md).

---

## Seqlock on `Local\PitWallVR`

**Context:** Rust writer (~30 Hz) and C++ reader (per frame) share one memory block.

**Decision:** Seqlock protocol in `shm.rs` / `pitwall_vr_shm.h` — reader retries on torn reads. SHM layout version 2 expands sector slots to 8.

**Consequences:** No mutex in the compositor hot path; occasional retry on conflict. Layer and app must agree on version.

---

## Standings link by track + recency

**Context:** Live disconnect and IBT import are separate events with no shared session ID from iRacing.

**Decision:** Match `session_standings` to imported IBT by track name and import time window.

**Consequences:** Occasional mismatch if multiple sessions same track same day; good enough for amateur coaching.

---

## Pit / off-track suppression

**Context:** Pace and pack callouts in the pits are noise.

**Decision:** Mute pack, race, pace, gap, and strategy on pit road or off track; keep flags and incidents.

**Consequences:** Cleaner radio; player still hears safety-critical calls in the paddock.

---

## Lap validity model

Stored DB field `valid` means **`include_in_stats`** — whether a lap counts toward coaching, session best, and sector analysis. No schema rename.

| Context | Policy |
|---------|--------|
| **IBT import** | `lap_kind == Flying` and telemetry heuristics (frame count, pit ratio ≤15%, lap time 10s–600s, distance completion) |
| **IBT outlier pass** | May clear `valid` on suspiciously fast incomplete **Flying** laps only |
| **Live coach (`lastLapValid`)** | `Flying && lap_completed && iracing_ok` where `iracing_ok` = `LapDeltaToBestLap_OK && LapDeltaToSessionBestLap_OK` |

**What `valid` gates:** coach insights, sector times on import, trace storage, lap table stats filters, session best lap selection.

**Intentional exclusions:** pit-out, pit-in, pit-lane, and partial laps are never stats-eligible even if telemetry looks clean. Live path does not apply IBT time/pit heuristics.

**Remaining deviation:** IBT import has no persisted `iracing_ok` field; validity is heuristic-only offline.

---

## Remaining SDK deviations (documented)

| Area | Behavior |
|------|----------|
| IBT lap times | SessionTime delta per lap bucket |
| Lap validity (live) | `Flying + completed + LapDeltaToBest/SessionBest OK` via `include_in_stats_live` |
| Lap validity (IBT) | `Flying + telemetry heuristics + outlier pass`; see Lap validity model |
| `LapKind` | PitWall pit/out/partial taxonomy |
| Flying-only sectors on import | Coaching policy |
| Sampling | Player 10 Hz, CarIdx 4 Hz vs SDK 60 Hz |
| Other drivers' sectors live | SDK limitation |

---

## Related docs

- [AUDIO_COACH.md](AUDIO_COACH.md) — priority details
- [LIVE_TELEMETRY.md](LIVE_TELEMETRY.md) — sector and merge logic
- [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md) — original layer research
