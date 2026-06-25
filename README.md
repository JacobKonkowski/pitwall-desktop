# PitWall Desktop

[![CI](https://github.com/JacobKonkowski/pitwall-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/JacobKonkowski/pitwall-desktop/actions/workflows/ci.yml)

iRacing telemetry analysis for Windows — post-session IBT import, live shared-memory telemetry, rule-based coaching, optional local AI summaries, and in-race HUD overlays (desktop + native in-headset VR, with an OpenKneeboard fallback).

**Repository:** [github.com/JacobKonkowski/pitwall-desktop](https://github.com/JacobKonkowski/pitwall-desktop)

## Quick start

```powershell
git clone https://github.com/JacobKonkowski/pitwall-desktop.git
cd pitwall-desktop
.\setup.ps1 -SkipBuild
npm run tauri dev
```

**Full setup guide:** [docs/SETUP.md](docs/SETUP.md)

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| [Rust](https://rustup.rs/) 1.89+ | Required by the `pitwall` crate |
| [Node.js](https://nodejs.org/) 18+ | Frontend build and Tauri CLI |
| iRacing | Disk telemetry for IBT import; shared memory for live monitor |
| [Ollama](https://ollama.com/) (optional) | Local LLM summaries — default `http://localhost:11434` |
| OpenXR VR runtime (optional) | Native in-headset HUD via PitWall's own OpenXR layer (Meta Quest Link, SteamVR, VDXR) |
| [OpenKneeboard](https://openkneeboard.com/) (optional) | Web Dashboard fallback for the in-headset HUD |

### iRacing configuration

Add to `Documents\iRacing\app.ini`:

```ini
irsdkEnableMem=1    ; live telemetry (shared memory)
irsdkEnableDisk=1   ; IBT file recording
```

Record telemetry in-car with **Alt+L**. Files are saved to `Documents\iRacing\telemetry\*.ibt`.

## Features

### Analyze (post-session)

- Auto-import from telemetry folder (file watcher)
- Session browser, lap table with P/Q/R grouping, S1/S2/S3, delta to best
- Two-lap trace compare (speed, throttle, brake)
- Fuel and tire charts
- **Coach panel** — rule-based insights, including pace vs the field and time lost in traffic
- **Session standings** — read-only snapshot of who you raced, linked from the live session
- **AI summary** — optional Ollama brief (structured data only, not raw IBT)

### Live (in-session)

- Real-time telemetry at 10 Hz via `pitwall::Pitwall::connect()`
- **Live leaderboard** — overall/class positions, best/last laps, gap to your pace
- **Session deltas** — delta to the session's best and optimal laps
- **Overlay widgets** — coach, standings, relative, and radar share one config for the desktop pop-out and native VR; enable/disable and field pace in Settings, drag/resize on the monitor, per-widget VR height/scale/opacity in-headset
- **Desktop overlay** — transparent always-on-top window with draggable widget panels for a companion monitor
- **Native in-headset HUD** — PitWall's own OpenXR layer composites the same widgets directly in VR (no OpenKneeboard or RaceLab). OpenKneeboard web fallback at `http://127.0.0.1:17342/vr` remains available. See [docs/NATIVE_VR.md](docs/NATIVE_VR.md)
- **Audio coach** — priority-ranked spoken alerts: hybrid WAV clips + Windows speech for lap times, gaps, flags, pack, race fuel, and sector summaries. See [docs/AUDIO_COACH.md](docs/AUDIO_COACH.md)

### Multi-driver comparison

Live field awareness and post-session standings/coaching that compare you against the
rest of the grid. See [docs/COMPARISON.md](docs/COMPARISON.md) for the full capabilities
matrix, audio priority order, and the iRacing SDK fields used.

## Build & run

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Dev mode |
| `npm run build` | Frontend only |
| `npm run tauri build` | Release installer |
| `npm run docs:api` | Generate local rustdoc + TypeDoc (see [docs/API.md](docs/API.md)) |

## Data storage

| Path | Contents |
|------|----------|
| `%LOCALAPPDATA%\pitwall-desktop\pitwall.db` | SQLite sessions, laps, sectors, traces, standings snapshots |
| `%LOCALAPPDATA%\pitwall-desktop\settings.json` | Ollama, overlay, VR/audio preferences |

## Documentation

**Start here:** [docs/README.md](docs/README.md) — documentation hub for users and contributors.

| Doc | Description |
|-----|-------------|
| [docs/SETUP.md](docs/SETUP.md) | Install, iRacing, live, VR, Ollama |
| [docs/FEATURES.md](docs/FEATURES.md) | User-facing feature guide |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues |
| [docs/API.md](docs/API.md) | Tauri IPC reference (`npm run docs:api` for generated docs) |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System map and audit |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | Contributor guide |
| [docs/AUDIO_COACH.md](docs/AUDIO_COACH.md) | Path B audio pipeline |
| [docs/LIVE_TELEMETRY.md](docs/LIVE_TELEMETRY.md) | Live loop and sectors |
| [docs/COMPARISON.md](docs/COMPARISON.md) | Multi-driver comparison matrix |
| [docs/NATIVE_VR.md](docs/NATIVE_VR.md) | Native in-headset VR |
| [docs/VR_NATIVE_SPIKE.md](docs/VR_NATIVE_SPIKE.md) | Historical OpenXR spike |

## Stack

Tauri 2 · React 19 · TypeScript · [pitwall](https://crates.io/crates/pitwall) · SQLite · Recharts · OpenXR API layer (C++/D3D11/Direct2D) · Ollama (optional) · OpenKneeboard (optional VR fallback)

## Out of scope (future)

- MoTeC export, other drivers' sector/trace analysis, real-time LLM coaching, community lap percentiles
- In-VR drag-to-position, named layout presets, track map overlay, gaze fade — see [docs/NATIVE_VR.md](docs/NATIVE_VR.md)

## Native VR layer (release builds)

The OpenXR DLL is built separately and staged before dev or packaging:

```powershell
cmake -S openxr-layer -B openxr-layer/build -A x64
cmake --build openxr-layer/build --config Release
copy openxr-layer\build\Release\pitwall-openxr-layer.dll  src-tauri\resources\openxr-layer\
copy openxr-layer\manifest\pitwall_openxr_layer.json      src-tauri\resources\openxr-layer\
npm run tauri dev          # or: npm run tauri build
```

See [docs/NATIVE_VR.md](docs/NATIVE_VR.md) for install, Quest Link setup, overlay widgets, and RaceLab migration.

## License

MIT — see [LICENSE](LICENSE)
