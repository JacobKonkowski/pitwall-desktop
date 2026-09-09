# Setup

## Prerequisites

1. **Rust** 1.89+ (`rustup`)
2. **Node.js** 18+
3. **iRacing** with disk + memory telemetry enabled
4. Optional: an OpenXR VR runtime for the in-headset HUD

Analyze Insights are computed in the app from imported laps (no external AI service).

## First install

```powershell
git clone https://github.com/JacobKonkowski/pitwall-desktop.git
cd pitwall-desktop
.\setup.ps1          # or .\setup.ps1 -SkipBuild
npm run tauri dev
```

Build the OpenXR layer when you want the native in-headset HUD (see [NATIVE_VR.md](NATIVE_VR.md)).

## iRacing `app.ini`

Edit `Documents\iRacing\app.ini` (or OneDrive Documents equivalent):

```ini
irsdkEnableMem=1
irsdkEnableDisk=1
```

Restart iRacing after changing. Record with **Alt+L** → `Documents\iRacing\telemetry\*.ibt`.

## Generate coach WAV clips (dev)

Live coach prefers baked WAVs under `src-tauri/resources/audio/coach/default/`.

```powershell
.\scripts\generate-audio-clips.ps1 -Engine WinRT
# Fallback (silent placeholders for CI / layout):
.\scripts\generate-audio-clips.ps1 -Engine Placeholder
```

Test Coach uses TTS-only and works without WAVs; live coach needs the clip set matching `manifest.json`.

---

## Race tonight checklist

1. **app.ini** — `irsdkEnableMem=1` and `irsdkEnableDisk=1`, then restart iRacing.
2. **Import** — Confirm Analyze can see sessions (auto-watcher or Import). Reimport after schema upgrades.
3. **Test Coach** — Live tab → Test Coach. You should hear speech (TTS path).
4. **Demo clock** — Start Demo Clock on Live to exercise UI / SHM test pattern without a session.
5. **HUD preview** — Open HUD preview → browser at `http://127.0.0.1:17342/vr`.
6. **Other OpenXR layers** — Disable any other OpenXR API layers that composite overlays. Only one layer stack should own compositing while you test PitWall.
7. **Install PitWall layer** — Live → Install VR layer (stages DLL under AppData + registry). Restart iRacing in **OpenXR** VR.
8. **Test pattern** — Start HUD (native). With no live track data, PitWall publishes a VR test pattern so the coach quad should appear.
9. **Drive** — Start live monitor + audio coach; confirm write age stays low in Diagnostics and pack/flag calls work with clips.

If the headset stays blank: layer ready + DLL present, OpenXR (not OpenVR), no conflicting API layers, restart sim after install. See [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

---

## Daily workflow

| Goal | Where |
|------|--------|
| Review last race | Analyze → pick session → Insights / Compare / Fuel |
| Practice with coach | Live → Start live + Start audio |
| Headset HUD | Live → Install layer once → Start HUD |
| Offline UI check | Live → Demo clock / Test Coach / HUD preview |

## Audio clip regeneration

After editing `scripts/audio-phrases.txt`, re-run the PowerShell script and commit WAVs + `manifest.json`.
