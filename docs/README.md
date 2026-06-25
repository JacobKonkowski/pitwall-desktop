# PitWall Desktop — Documentation

PitWall is a Windows Tauri app for iRacing: post-session IBT analysis, live telemetry, rule-based coaching, optional Ollama summaries, desktop overlays, native VR HUD, and a hybrid audio race engineer.

**Last updated:** June 2026 (v0.1.0, Path B audio).

---

## For drivers and users

| Doc | What you'll learn |
|-----|-------------------|
| [SETUP.md](SETUP.md) | Install, iRacing `app.ini`, first import, live monitor, VR, Ollama |
| [FEATURES.md](FEATURES.md) | What each tab and overlay does, in plain language |
| [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Live won't connect, silent audio, VR layer, sectors, import |
| [COMPARISON.md](COMPARISON.md) | Live field data vs your IBT — what's possible |
| [NATIVE_VR.md](NATIVE_VR.md) | Native in-headset HUD — build, install, Quest Link |

**Quick path:** SETUP → FEATURES → drive → TROUBLESHOOTING if something breaks.

---

## For contributors

| Doc | What you'll learn |
|-----|-------------------|
| [CONTRIBUTING.md](CONTRIBUTING.md) | Dev commands, tests, CI, doc maintenance |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System map — modules, data flow, audit status |
| [API.md](API.md) | Tauri commands, events, types (`npm run docs:api` for rustdoc + TypeDoc) |
| [AUDIO_COACH.md](AUDIO_COACH.md) | Path B speech pipeline, messages, clip export |
| [LIVE_TELEMETRY.md](LIVE_TELEMETRY.md) | Live loop, sectors, competitors, VR SHM |
| [ANALYSIS.md](ANALYSIS.md) | IBT import, lap/sector pipeline, post-session coach |
| [FRONTEND.md](FRONTEND.md) | React entry points, widgets, events |
| [DATA_MODEL.md](DATA_MODEL.md) | SQLite schema, settings fields, on-disk paths |
| [DESIGN_NOTES.md](DESIGN_NOTES.md) | Why we made specific technical choices |

**Quick path:** CONTRIBUTING → ARCHITECTURE → area deep-dive for your change.

---

## Other references

| Location | Topic |
|----------|-------|
| [openxr-layer/README.md](../openxr-layer/README.md) | C++ OpenXR layer build |
| [scripts/](../scripts/) | Audio clip generation, helpers |
| [VR_NATIVE_SPIKE.md](VR_NATIVE_SPIKE.md) | Historical OpenXR research (see NATIVE_VR for current path) |

---

## Keeping docs in sync

When you change code, update the matching doc:

| Code change | Update |
|-------------|--------|
| New Tauri command / event | `commands/mod.rs` doc comment, `api.ts`, **API.md** |
| New serde / IPC type | `types.ts`, rustdoc, **DATA_MODEL.md** if persisted |
| New `AppSettings` field | **DATA_MODEL.md**, SETUP or LivePanel if user-facing |
| New audio message / clip | **AUDIO_COACH.md**, `scripts/audio-phrases.txt` |
| Live telemetry field | **LIVE_TELEMETRY.md**, COMPARISON if SDK-related |
| VR SHM layout | **NATIVE_VR.md**, `pitwall_vr_shm.h` |

Run `npm run docs:api` locally after IPC changes to verify rustdoc and TypeDoc still build.
