# Troubleshooting

## Import / Analyze

| Symptom | Check |
|---------|--------|
| No sessions | Disk recording (`irsdkEnableDisk=1`), Alt+L, watcher watching telemetry folder |
| Empty after upgrade | Schema v2 drops old analysis tables — reimport IBTs |
| Empty after the Race Refinery rename | Clean break: data moved from `%LOCALAPPDATA%\pitwall-desktop\` to `%LOCALAPPDATA%\race-refinery\` with no migration. Reimport IBTs and redo settings; delete the old folder when done |
| Duplicate skip | Same file hash already imported |
| Import stuck | `get_import_status`; restart app if a previous import crashed |
| Odd duplicate lap times | Cleanup clears sticky times and drops phantom `Lap == 0` buckets — reopen the session or reimport. See [ANALYSIS.md](ANALYSIS.md) |

## Live telemetry

| Symptom | Check |
|---------|--------|
| Never connects | `irsdkEnableMem=1`, iRacing running, Start live monitor |
| Snapshot stale | Leave/rejoin session; restart monitor |
| Demo only | Demo clock is synthetic — stop it before trusting sim data |

## Audio coach

| Symptom | Check |
|---------|--------|
| Test Coach works, live silent | Missing WAVs under `src-tauri/resources/audio/coach/default/` — regenerate clips. Look for `Coach clips unavailable, continuing TTS-only` in the log |
| Numbers sound robotic | Settings voice should be "Race Refinery voice (neural)". If it says "not installed", run `scripts/fetch-piper-voice.ps1` (dev) or reinstall; the log shows `Piper voice not installed` |
| No Test Coach either | Audio output device; coach volume above 0 in settings; for a Windows voice, WinRT voices installed |
| Pack / clear wrong | Pack uses `CarLeftRight` enum; confirm on-track / not pit-road suppression |
| Clip key missing | Phrase in `scripts/audio-phrases.txt` + regenerate; player skips missing files |

```powershell
.\scripts\generate-audio-clips.ps1 -Only <key>
```

## Native VR

| Symptom | Check |
|---------|--------|
| Layer not ready | Install VR layer; DLL beside staged manifest; unset `RACE_REFINERY_VR_DISABLE` |
| No HUD after the Race Refinery rename | The old PitWall layer cannot read the new shared memory. Click Install VR layer again; it removes the old `pitwall_openxr_layer.json` registration and registers `race_refinery_openxr_layer.json` |
| Blank headset | Other OpenXR API layers off; OpenXR (not OpenVR); restart iRacing after install |
| Compositor false | Diagnostics use **producer write age**, not a layer heartbeat file. Fresh write age + layer installed ⇒ `compositorActive` proxy |
| Test pattern missing | Start HUD with empty live track data; coach slot enabled |
| Web preview | `http://127.0.0.1:17342/vr` after Start HUD in web mode or open preview |

The OpenXR layer performs **no disk I/O in `xrEndFrame`**. Do not expect `layer-heartbeat` files.

## Debug

- `clear_database_cmd` exists for wipe/reimport (no dedicated UI button in the shell).
- Backend logs: `RUST_LOG=race_refinery_desktop_lib=debug`.
