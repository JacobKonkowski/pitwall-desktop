# PitWall Desktop

[![CI](https://github.com/JacobKonkowski/pitwall-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/JacobKonkowski/pitwall-desktop/actions/workflows/ci.yml)

iRacing telemetry analysis for Windows — post-session IBT import, live shared-memory telemetry, rule-based coaching, optional local AI summaries, and in-race HUD overlays (desktop + VR via OpenKneeboard).

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
| [OpenKneeboard](https://openkneeboard.com/) (optional) | In-headset HUD via Web Dashboard — OpenXR, no SteamVR |

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
- **Coach panel** — rule-based insights
- **AI summary** — optional Ollama brief (structured data only, not raw IBT)

### Live (in-session)

- Real-time telemetry at 10 Hz via `pitwall::Pitwall::connect()`
- **Desktop overlay** — always-on-top pop-out for a companion monitor
- **In-headset HUD** — `http://127.0.0.1:17342/vr` for OpenKneeboard (OpenXR, no SteamVR)
- **Audio coach** — spoken lap/sector summaries, fuel and pit alerts (Windows TTS)

## Build & run

| Command | Purpose |
|---------|---------|
| `npm run tauri dev` | Dev mode |
| `npm run build` | Frontend only |
| `npm run tauri build` | Release installer |

## Data storage

| Path | Contents |
|------|----------|
| `%LOCALAPPDATA%\pitwall-desktop\pitwall.db` | SQLite sessions, laps, sectors, traces |
| `%LOCALAPPDATA%\pitwall-desktop\settings.json` | Ollama, overlay, VR/audio preferences |

## Documentation

| Doc | Description |
|-----|-------------|
| [docs/SETUP.md](docs/SETUP.md) | Step-by-step install, iRacing, live, VR, Ollama, troubleshooting |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Technical audit — modules, IPC, schema, data flow |
| [docs/VR_NATIVE_SPIKE.md](docs/VR_NATIVE_SPIKE.md) | Native OpenXR in-headset HUD research & decision (no-go; OpenKneeboard is the path) |

## Stack

Tauri 2 · React 19 · TypeScript · [pitwall](https://crates.io/crates/pitwall) · SQLite · Recharts · Ollama (optional) · OpenKneeboard (optional VR)

## Out of scope (future)

- MoTeC export, multi-car analysis, real-time LLM coaching, community lap percentiles
- Native OpenXR API layer (today: OpenKneeboard web dashboard path)

## License

MIT — see [LICENSE](LICENSE)
