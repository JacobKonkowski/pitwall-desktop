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
| Layer install / diagnostics | Registry stage, DLL presence, producer write age |

Overlay layout settings configure **VR widget slots** (coach / standings / relative / radar). The Live page shows an in-app coach preview; the same slot config drives the native layer and the web HUD.

## Settings

Persisted via `get_settings` / `save_settings_cmd`. Live page exposes common audio toggles and VR actions. Full `AppSettings` includes VR mode/opacity, overlay layout, and coach chatter / category flags.
