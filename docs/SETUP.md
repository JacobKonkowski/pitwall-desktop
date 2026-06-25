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
| OpenXR VR runtime | optional | Native in-headset HUD (Meta Quest Link, SteamVR, VDXR) |
| [OpenKneeboard](https://openkneeboard.com/) | optional | Web Dashboard fallback for the in-headset HUD |

**Not required:** SteamVR for the HUD, or a GPU for coaching (rules + hybrid audio run on CPU). Building the native VR layer from source needs CMake + an MSVC C++ toolchain — see [NATIVE_VR.md](NATIVE_VR.md).

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

With live monitor running, click **Pop out overlay (desktop)** for a transparent
always-on-top window on a second monitor. It renders the same widgets as the VR
HUD — coach, standings, relative, and radar.

- Enable or disable widgets under Settings → **Overlay widgets** (this also
  controls which widgets show in VR).
- Drag a widget by its top edge to move it; drag the bottom-right corner to
  resize it. Positions persist per widget.
- Drag an empty part of the window to move the whole overlay.

### Audio coach (hybrid WAV + WinRT)

With live monitor running:

1. Click **Start audio coach**
2. Or enable **Auto-start audio coach with live monitor** in Settings

**Runtime:** Pre-recorded WAV clips for flags, pack calls, and fuel phrases;
Windows WinRT speech for dynamic lap times, gaps, position, and deltas. No ML
models run while iRacing is open.

Spoken alerts include:

- Session intro (track name)
- Flags, spotter pack, optional clear callout
- Sector and lap pace with personal-best and session-best deltas (qual/practice)
- Gap ahead/behind on lap complete and when closing
- Race fuel strategy, race clock milestones, pits open (race/qual)
- Incident count (Nx style)

Tune under Settings → **Radio chatter level** and per-category toggles (pace,
strategy, gaps, race clock).

**Regenerating voice clips (maintainers only — dev machine, not while racing):**

Neural WinRT synthesis runs **only** in the export tool. The shipped app plays the
resulting WAV files; it does not load any neural model.

```powershell
# List installed Windows neural voices
.\scripts\generate-audio-clips.ps1 -ListVoices

# Export all phrases with default en-US neural voice
.\scripts\generate-audio-clips.ps1

# Pick a voice by substring (e.g. Jenny, Guy, Aria)
.\scripts\generate-audio-clips.ps1 -Voice "Jenny"

# Silence placeholders (CI / quick layout tests)
.\scripts\generate-audio-clips.ps1 -Engine Placeholder
```

Equivalent Cargo commands (from repo root):

```powershell
cargo run --manifest-path src-tauri\Cargo.toml --bin gen-audio-clips -- --list-voices
cargo run --manifest-path src-tauri\Cargo.toml --bin gen-audio-clips -- --engine winrt
cargo run --manifest-path src-tauri\Cargo.toml --bin gen-audio-clips -- --engine placeholder
```

Phrases live in `scripts/audio-phrases.txt`. Output goes to
`src-tauri/resources/audio/coach/default/` (`*.wav` + `manifest.json`). Commit
those files so builds bundle your chosen voice.

Adjust **Fuel warning (liters)** in Settings (default `5`; set `0` to disable).

---

## 6. VR in-headset HUD (no SteamVR)

Use iRacing in **OpenXR** mode (not SteamVR-only).

### Native (recommended — no OpenKneeboard)

PitWall composites the HUD in the headset through its own OpenXR layer. Full
build/install details are in [NATIVE_VR.md](NATIVE_VR.md).

1. Settings → **VR mode: Native**
2. PitWall **Live** → **Start live monitor** → **Start in-headset HUD**
3. If prompted, click **Install VR layer**, then restart iRacing
4. Under Settings → **Overlay widgets**, enable the widgets you want (coach,
   standings, relative, radar) and tune each one's **VR height / scale /
   opacity**; set **Field pace** for the coach. The head-locked widgets update
   live. The same enabled set also appears on the desktop pop-out.

### OpenKneeboard fallback

If you prefer the web path or the native layer is unavailable:

1. Settings → **VR mode: Web fallback**
2. PitWall **Live** → **Start live monitor** → **Start in-headset HUD**
3. Click **Preview in browser** to confirm the HUD shows live data
4. Copy the URL shown: `http://127.0.0.1:17342/vr`
5. Install [OpenKneeboard](https://openkneeboard.com/) → **Settings** → **Tabs**
   → **Add tab** → **Web Dashboard** → paste the URL
6. Start iRacing in VR, join a session, use OpenKneeboard recenter binding

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

See **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** for consolidated fixes. Quick checks:

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

- iRacing display mode: **OpenXR** (required for both native and OpenKneeboard)
- **Native mode:** install the VR layer and restart iRacing; confirm the panel
  reads "VR layer installed" and the compositor is active. Ensure
  `PITWALL_VR_DISABLE` is not set. See [NATIVE_VR.md](NATIVE_VR.md)
- **Web fallback:** OpenKneeboard must be running with the Web Dashboard tab; do
  not add `iRacingUI.exe` to OpenKneeboard's games list — use the sim exe or
  leave the games list empty for OpenXR

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
| `npm run docs:api` | Generate local rustdoc + TypeDoc |

---

## 10. Further reading

- [docs/README.md](README.md) — documentation hub
- [README](../README.md) — feature overview
- [ARCHITECTURE.md](ARCHITECTURE.md) — technical audit
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — consolidated troubleshooting
