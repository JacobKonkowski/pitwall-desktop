# Design notes

Short rationale for non-obvious decisions (ADR-lite). Each entry: context → decision → consequences.

---

## Sector 0 at 0% is ignored

**Context:** iRacing exposes a sector marker at the start/finish line that does not represent a timed sector.

**Decision:** Both live (`tracker.rs`) and post-session (`sector_splitter.rs`) skip sector 0 crossings at 0% lap distance.

**Consequences:** S1/S2/S3 align with in-sim sector times; no spurious sub-second "sector" at lap start.

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

**Decision:** Seqlock protocol in `shm.rs` / `pitwall_vr_shm.h` — reader retries on torn reads.

**Consequences:** No mutex in the compositor hot path; occasional retry on conflict.

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

## Related docs

- [AUDIO_COACH.md](AUDIO_COACH.md) — priority details
- [LIVE_TELEMETRY.md](LIVE_TELEMETRY.md) — sector and merge logic
- [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md) — original layer research
