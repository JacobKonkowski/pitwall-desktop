# Architecture

PitWall is a Tauri 2 + React desktop app with two user goals: **Analyze** imported
IBTs, and **Live** coach + HUD from shared-memory telemetry.

```
┌─────────────────────────────────────────────────────────┐
│  React shell (src/shell) + feature registry             │
│  Analyze │ Live                                         │
└─────────────┬───────────────────────────┬───────────────┘
              │ invoke / events           │
┌─────────────▼───────────────────────────▼───────────────┐
│  Tauri commands (src-tauri/src/commands)                │
├─────────────┬───────────────┬─────────────┬─────────────┤
│ ingest/IBT  │ live/         │ audio/      │ vr/         │
│ analysis/   │ snapshot      │ engine+WAV  │ SHM + HUD   │
│ storage/    │               │ WinRT TTS   │ OpenXR DLL  │
└─────────────┴───────────────┴─────────────┴─────────────┘
```

## Frontend

| Path | Role |
|------|------|
| `src/shell/` | `AppShell`, feature nav |
| `src/features/registry.ts` | Registered features (Analyze, Live) |
| `src/features/analyze/` | Post-session UI |
| `src/features/live/` | Live telemetry / coach / VR controls |
| `src/shared/` | `api.ts`, `types.ts`, format, toast |
| `src/widgets/` | Coach / standings / relative / radar (Live preview + VR data shapes) |
| `src/styles/` | Tokens + app CSS |

Single Vite entry: `index.html` → `main.tsx`.

## Backend modules (`src-tauri/src`)

| Module | Role |
|--------|------|
| `telemetry/` | Shared telemetry helpers for analysis |
| `analysis/` | IBT pipeline, sectors, segment, compare, aggregates, cleanup |
| `ingest/` | Frame extract, IBT import, folder watcher |
| `storage/` | SQLite schema v2 |
| `live/` | Tracker, snapshot, pack, sectors, competitors |
| `audio/` | Coach: `engine/rules/*`, clips, queue, WinRT TTS |
| `vr/` | SHM writer, HUD HTTP server, layer install |
| `settings/` | `AppSettings` JSON (audio + VR + overlay layout) |
| `commands/` | Tauri IPC surface |

## Live data paths

1. **UI** — ~10 Hz snapshot for Live page / widgets
2. **Native VR** — ~30 Hz `PwSharedBlock` into `Local\PitWallVR` for `pitwall-openxr-layer`
3. **Web HUD** — HTTP JSON/HTML on port **17342**
4. **Audio** — rule engine polls snapshot; plays clip sequences + TTS units

## Diagnostics (VR)

Producer-side only: write age, enabled overlay count, last error, layer registry/DLL readiness. The layer does not write heartbeat files (no disk I/O in `xrEndFrame`). `compositorActive` is a proxy (fresh publish ∧ layer installed).

## Binary tools

- `gen-audio-clips` (`src-tauri/src/bin/gen_audio_clips.rs`) — bake WAVs; not used at app runtime.
