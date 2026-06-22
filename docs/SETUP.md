# PitWall Desktop — Setup Guide

Complete setup for development and daily use on Windows with iRacing.

**Repository:** [github.com/JacobKonkowski/pitwall-desktop](https://github.com/JacobKonkowski/pitwall-desktop)

---

## 1. What you need

| Software | Version | Purpose |
|----------|---------|---------|
| [Windows](https://www.microsoft.com/windows) | 10/11 | Tauri desktop target |
| [Rust](https://rustup.rs/) | 1.89+ | Backend (`pitwall` crate MSRV) |
| [Node.js](https://nodejs.org/) | 18+ | Frontend + Tauri CLI |
| [iRacing](https://www.iracing.com/) | — | Telemetry source |
| [Ollama](https://ollama.com/) | optional | Local AI coach summaries |
| [OpenKneeboard](https://openkneeboard.com/) | optional | In-headset HUD (OpenXR, no SteamVR) |

**Not required:** SteamVR, CMake, or a GPU for coaching (rules + TTS run on CPU).

---

## 2. Clone and install

```powershell
git clone https://github.com/JacobKonkowski/pitwall-desktop.git
cd pitwall-desktop
```

### Automated setup

```powershell
.\setup.ps1 -SkipBuild   # npm install only (recommended first time)
npm run tauri dev        # run in dev mode
```

Full build (frontend + debug installer):

```powershell
.\setup.ps1
```

### Manual setup

```powershell
npm install
npm run tauri dev
```

Release installer:

```powershell
npm run build
npm run tauri build
```

Output: `src-tauri\target\release\bundle\`

---

## 3. iRacing configuration

Edit `Documents\iRacing\app.ini`:

```ini
[irsdk]
irsdkEnableMem=1
irsdkEnableDisk=1
```

| Flag | Required for |
|------|----------------|
| `irsdkEnableDisk=1` | IBT files in `Documents\iRacing\telemetry\` |
| `irsdkEnableMem=1` | Live telemetry panel, overlays, audio coach |

Restart iRacing after changing `app.ini`.

### Record telemetry

1. Enter a session (test drive, practice, etc.)
2. Press **Alt+L** in-car to start/stop disk recording
3. `.ibt` files appear in `Documents\iRacing\telemetry\`

PitWall auto-imports new files from that folder while the app is running.

---

## 4. First run — post-session analysis

1. Launch PitWall: `npm run tauri dev` (or the installed `.exe`)
2. If the yellow banner appears, fix `app.ini` as above
3. **Import** telemetry:
   - **Scan Folder** — imports all IBT files in the telemetry directory
   - **Import IBT** — pick a single file
   - Or drive a session with PitWall open; new files import automatically
4. Select a session in the sidebar
5. Review laps, sectors, fuel/tire charts
6. Scroll to **Coach** for rule-based insights; click **Generate AI summary** if Ollama is running

### Database location

```
%LOCALAPPDATA%\pitwall-desktop\pitwall.db
```

Settings:

```
%LOCALAPPDATA%\pitwall-desktop\settings.json
```

---

## 5. Live telemetry (in-session)

1. Ensure `irsdkEnableMem=1` and iRacing is running
2. Open PitWall → **Live** tab → **Start live monitor**
3. Join or enter an iRacing session — data should appear within ~2 seconds

The Live panel shows lap time, deltas, sector bars, fuel, and tire temps at 10 Hz.

### Desktop overlay (companion monitor)

With live monitor running:

**Pop out overlay (desktop)** — small always-on-top HUD for a second monitor.

### Audio coach (TTS)

With live monitor running:

1. Click **Start audio coach**
2. Or enable **Auto-start audio coach with live monitor** in Settings

Spoken alerts include:

- Session intro (track name)
- Sector times and deltas vs your best sectors
- Lap summaries with personal-best callouts
- Fuel level and estimated laps remaining
- Low-fuel pit warnings

Adjust **Fuel warning (liters)** in Settings (default `5`; set `0` to disable).

Uses Windows built-in TTS — no extra voice software required.

---

## 6. VR in-headset HUD (no SteamVR)

Use iRacing in **OpenXR** mode (not SteamVR-only).

1. PitWall **Live** → **Start live monitor** → **Start in-headset HUD**
2. Click **Preview in browser** to confirm the HUD shows live data
3. Copy the URL shown: `http://127.0.0.1:17342/vr`
4. Install [OpenKneeboard](https://openkneeboard.com/)
5. OpenKneeboard → **Settings** → **Tabs** → **Add tab** → **Web Dashboard**
6. Paste the PitWall URL
7. Start iRacing in VR, join a session, use OpenKneeboard recenter binding

Same approach used by many iRacing VR overlays (iOverlay, RaceLab, etc.) — OpenXR via OpenKneeboard, not SteamVR.

---

## 7. Optional — Ollama AI summaries

Post-session only (not live — avoids distraction and latency).

```powershell
# Install Ollama from https://ollama.com/, then:
ollama pull llama3.2
ollama serve
```

In PitWall **Live** → **Settings**, confirm:

- Ollama URL: `http://localhost:11434`
- Model: `llama3.2` (or your installed model)

On the **Analyze** tab, open a session → **Coach** → **Generate AI summary**.

Only structured lap/insight JSON is sent to Ollama — not raw IBT files.

---

## 8. Troubleshooting

### Live panel stays on "Waiting for iRacing"

- Confirm `irsdkEnableMem=1` in `app.ini`
- iRacing must be running with an active session (not just the UI)
- Restart iRacing after editing `app.ini`
- Click **Start live monitor** before or after joining the session

### Import stuck or slow

- Large IBT files take time; watch the progress bar
- Only one import runs at a time
- Duplicate files (same hash/path) are skipped

### Sectors look wrong

- PitWall uses iRacing sector boundaries from session YAML
- S3 is always computed (S1/S2 from boundary crossings)

### Audio coach silent

- Check Windows sound output and volume
- Ensure **Start audio coach** is active (green state)
- Live monitor must be running first

### VR HUD not visible

- OpenKneeboard must be running with the Web Dashboard tab
- iRacing display mode: **OpenXR** (recommended)
- Do not add `iRacingUI.exe` to OpenKneeboard's games list — use the sim exe or leave games list empty for OpenXR

### Build errors

| Error | Fix |
|-------|-----|
| `rustc` version too old | `rustup update` (need 1.89+) |
| Port 1420 in use | Close other Vite/Tauri dev instances |
| `npm` not found | Install Node.js 18+ |

### Clear all imported data (dev builds only)

Sidebar → **Clear database** (debug builds only).

---

## 9. Project commands reference

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Development app with hot reload |
| `npm run build` | TypeScript + Vite production build |
| `npm run tauri build` | Release `.msi` / installer |
| `cargo test` (in `src-tauri/`) | Run Rust unit tests |

---

## 10. Further reading

- [README](../README.md) — feature overview
- [ARCHITECTURE.md](ARCHITECTURE.md) — technical audit, IPC, schema
