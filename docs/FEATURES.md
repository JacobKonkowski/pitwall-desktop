# Features

PitWall exposes two features via `src/features/registry.ts`: **Analyze** and **Live**.

## Analyze

| Capability | Notes |
|------------|--------|
| Session browser | Lists imported IBTs; delete per session |
| Import | File / folder pickers; folder watcher auto-import |
| Config tip | Reminds when disk recording looks disabled |
| Lap table | Session type grouping; sectors; `paceEligible` (official time + both `_OK` flags + near-full coverage) |
| Compare | Two-lap traces via `compare_laps` |
| Fuel / tire panels | From stored lap aggregates |
| Insights strip | Deterministic client-side bullets from your laps |

Phantom reset buckets and sticky duplicate lap times are cleaned in the analysis pipeline (and when loading older sessions). See [ANALYSIS.md](ANALYSIS.md).

## Live

| Capability | Notes |
|------------|--------|
| Live monitor | Shared-memory telemetry snapshot + status |
| Leaderboard | Positions, best/last, gaps |
| Coach preview | Last coach message + widget preview |
| Audio coach | Rule engine priorities; WAV clips + WinRT TTS |
| Test Coach | One-shot TTS path (works without WAV assets) |
| Demo clock | Synthetic session clock / offline exercise |
| Native VR HUD | OpenXR API layer + shared memory (default `vrMode: native`) |
| Web HUD | HTTP server `:17342` for browser preview |
| Monitor overlays | Always-on-top transparent windows per enabled widget |
| Layer install / diagnostics | Registry stage, DLL presence, producer write age |

Overlay layout settings configure a **shared widget catalog** (coach / standings / relative / radar). Enable once; place twice (`desktop*` for monitor windows, `vr*` for the headset). The Live page shows an in-app coach preview; the same slot config drives monitor, native VR, and the web HUD.

## Settings

Persisted via `get_settings` / `save_settings_cmd`. Live page exposes common audio toggles, monitor overlay start/stop, and VR actions. Full `AppSettings` includes VR mode/opacity, overlay layout, and coach chatter / category flags.
