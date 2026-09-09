# Troubleshooting

## Import / Analyze

| Symptom | Check |
|---------|--------|
| No sessions | Disk recording (`irsdkEnableDisk=1`), Alt+L, watcher watching telemetry folder |
| Empty after upgrade | Schema v2 drops old analysis tables — reimport IBTs |
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
| Test Coach works, live silent | Missing WAVs under `resources/audio/coach/default/` — regenerate clips |
| No Test Coach either | Windows speech / WinRT voices installed; coach not muted in settings |
| Pack / clear wrong | Pack uses `CarLeftRight` enum; confirm on-track / not pit-road suppression |
| Clip key missing | Phrase in `scripts/audio-phrases.txt` + regenerate; player skips missing files |

```powershell
.\scripts\generate-audio-clips.ps1 -Engine WinRT
```

## Native VR

| Symptom | Check |
|---------|--------|
| Layer not ready | Install VR layer; DLL beside staged manifest; unset `PITWALL_VR_DISABLE` |
| Blank headset | Other OpenXR API layers off; OpenXR (not OpenVR); restart iRacing after install |
| Compositor false | Diagnostics use **producer write age**, not a layer heartbeat file. Fresh write age + layer installed ⇒ `compositorActive` proxy |
| Test pattern missing | Start HUD with empty live track data; coach slot enabled |
| Web preview | `http://127.0.0.1:17342/vr` after Start HUD in web mode or open preview |

The OpenXR layer performs **no disk I/O in `xrEndFrame`**. Do not expect `layer-heartbeat` files.

## Debug

- `clear_database_cmd` exists for wipe/reimport (no dedicated UI button in the shell).
- Backend logs: `RUST_LOG=pitwall_desktop_lib=debug`.
