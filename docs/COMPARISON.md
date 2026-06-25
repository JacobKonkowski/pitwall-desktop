# Multi-driver comparison

PitWall compares you against the rest of the field in two places: **live** (real-time
field awareness while you drive) and **post-session** (a standings snapshot and coach
insights after the session ends). This document describes what data is available, where
it comes from, and the audio coach alert behavior.

## Capabilities matrix

| Capability | Live | Your IBT | Other driver's IBT |
|------------|------|----------|--------------------|
| Others' best / last lap | Yes (`CarIdxBestLapTime`, `CarIdxLastLapTime`) | No | Yes |
| Others' sectors / traces | No | No | Yes |
| Overall + class position | Yes (`CarIdxPosition`, `CarIdxClassPosition`) | No | N/A |
| Gap to leader / ahead / behind | Yes (`CarIdxF2Time`) | No | N/A |
| Delta to session best / optimal (you) | Yes (`LapDeltaToSessionBestLap`, `LapDeltaToSessionOptimalLap`) | Yes | N/A |
| Pack / 3-wide spotter | Yes (`CarLeftRight`) | No | N/A |
| Flags (yellow / green / blue / checkered / red) | Yes (`SessionFlags`) | No | N/A |
| Your incident count | Yes (`PlayerCarMyIncidentCount`) | Partial | N/A |

**Out of scope:** other drivers' sector times and traces (not exposed by the iRacing
SDK for cars other than yours), shared IBT import, 4-wide detection, and external
data services (Garage61 / VRS).

## Live field awareness

The live monitor opens a second telemetry subscription
([`CarIdxFrame`](../src-tauri/src/live/car_idx_frame.rs)) alongside the player frame.
The per-car arrays are merged with the session driver roster in
[`competitors.rs`](../src-tauri/src/live/competitors.rs) and surfaced on the
[`LiveSnapshot`](../src-tauri/src/live/snapshot.rs):

- **Leaderboard** — overall/class toggle, best/last lap, and delta to your best lap,
  shown on the Live tab ([`SessionLeaderboard.tsx`](../src/components/SessionLeaderboard.tsx)).
- **Session deltas** — delta to the session's best and optimal laps next to your own
  delta-to-best.
- **Gaps** — time to the car ahead and behind, derived from the difference in
  `CarIdxF2Time` (time behind the leader) between adjacent cars.
- **Pack state** — `CarLeftRight` mapped to a [`PackState`](../src-tauri/src/live/pack.rs)
  (clear, car left/right, three-wide, two cars left/right).

The same data drives the in-headset HUD ([`hud_server.rs`](../src-tauri/src/vr/hud_server.rs)),
which adds a position line, gap ahead/behind, the field delta, and a compact pack
indicator.

## Audio coach

Path B hybrid delivery: **WAV clips** for fixed phrases, **WinRT TTS** for dynamic lap times, gaps, and positions. Full pipeline, priority order, session modes, and clip export: **[AUDIO_COACH.md](AUDIO_COACH.md)**.

The coach ([`audio/coach.rs`](../src-tauri/src/audio/coach.rs)) speaks at most one alert per 250 ms tick. Lower-priority alerts wait for the next tick rather than being dropped.

| Priority | Category | Examples |
|----------|----------|----------|
| 1 (highest) | Critical | Red, checkered, black |
| 2 | Safety | Yellow (incl. waving), green, blue, incidents |
| 3 | Pack | Car left/right, three-wide (4 s cooldown) |
| 4 | Race | Fuel-to-finish, low fuel, pit-this-lap |
| 5 | Pace | Sector and lap callouts |
| 6 | Strategy | Race clock, pits open, position |

Pack, race, pace, gap, and strategy alerts are suppressed on pit road or off track; flags and incidents still announce. Toggle categories and **chatter level** in Live settings (`audioGapAlertsEnabled`, `audioPaceEnabled`, `audioRaceClockEnabled`, etc. — see [DATA_MODEL.md](DATA_MODEL.md)).

### Notes on `CarIdxF2Time` and the blue flag

`CarIdxF2Time` and the per-car flag semantics are interpreted conservatively. Gaps are
computed as the absolute difference in F2 time between adjacent cars in the overall
order, and the blue flag is read from the player's `SessionFlags` bitfield. Both should
be sanity-checked against a live or replay session; if iRacing reports them differently
than assumed, adjust the resolver in `competitors.rs` / `audio/coach.rs` without changing
the surrounding feature.

## Post-session standings

When a live session ends, PitWall saves a standings snapshot
([`session_standings`](../src-tauri/src/storage/db.rs)) containing the final positions,
each competitor's best lap, the session's fastest lap, your best, and the lap numbers you
ran side-by-side with traffic. When the matching IBT file is imported, the snapshot is
linked to that session by track and recency.

On the Analyze tab, a linked snapshot powers:

- **Session standings panel** ([`SessionStandingsPanel.tsx`](../src/components/SessionStandingsPanel.tsx))
  — a read-only copy of the final leaderboard with an overall/class toggle.
- **Coach insights** ([`analysis/coach.rs`](../src-tauri/src/analysis/coach.rs)):
  - `session_pace` — your best lap versus the session's fastest lap.
  - `traffic_pace` — slow laps that were also run in traffic, so you can tell lost time
    from a pace problem.

If no live snapshot is linked (for example, an IBT imported without a live session), the
standings panel is hidden and the coach still works from the IBT data alone.

## Multi-class notes

Positions are tracked both overall and within class. The leaderboard and standings panel
offer an overall/class toggle; in a multi-class field the class view filters to your
class and sorts by class position. Class color (from the session roster) is shown as the
chip background next to each car number.
