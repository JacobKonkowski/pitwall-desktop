# Troubleshooting

Consolidated fixes for common PitWall issues. Setup basics: [SETUP.md](SETUP.md).

---

## Live won't connect / "Waiting for iRacing"

- Set `irsdkEnableMem=1` in `Documents\iRacing\app.ini` under `[irsdk]`
- Restart iRacing after editing `app.ini`
- iRacing must be in an **active session** (not menu only)
- Click **Start live monitor** before or after joining
- Check Live tab message — `reconnecting` means backoff retry (normal briefly)

---

## Sectors wrong or missing audio at sector end

- Sectors use iRacing YAML boundaries; sector 0 at start is ignored by design
- S3 completes at lap end — see [LIVE_TELEMETRY.md](LIVE_TELEMETRY.md)
- Mid-lap join infers current sector from lap distance %
- If live sectors look wrong, confirm mem telemetry is enabled and you are on track

---

## Audio coach silent or missing phrases

- Windows output device and volume
- **Start live monitor** first, then **Start audio coach** (or enable auto-start)
- Check per-category toggles and chatter level in Settings
- Missing WAV → run clip export — [AUDIO_COACH.md](AUDIO_COACH.md)
- WinRT failures fall back to logs; rebuild clips with `generate-audio-clips.ps1`

---

## VR layer not loading / HUD invisible

- iRacing display: **OpenXR** (not SteamVR-only)
- Native mode: **Install VR layer**, restart iRacing
- Check **VR layer diagnostics** in Live panel — DLL path, registry, `PITWALL_VR_DISABLE` unset
- AppData staging path — [NATIVE_VR.md](NATIVE_VR.md)
- Web fallback: OpenKneeboard Web Dashboard tab, URL `http://127.0.0.1:17342/vr`, live monitor running

---

## Ollama summary fails

- `ollama serve` running; model pulled (`ollama pull llama3.2`)
- Settings URL `http://localhost:11434` and model name match
- Post-session only — not used live

---

## Import stuck, slow, or duplicates

- Large IBT takes time — watch progress bar
- Only one import at a time (`import_gate`)
- Duplicates skipped by hash/path
- **Scan Folder** imports everything in telemetry dir — can be slow

---

## Build / dev errors

| Error | Fix |
|-------|-----|
| `rustc` too old | `rustup update` (need 1.89+) |
| Port 1420 in use | Close other Vite/Tauri instances |
| `npm` not found | Install Node 18+ |
| API docs fail CI | Run `npm run docs:api` locally |

---

## Clear database

**Debug builds only** — sidebar **Clear database**. Release builds reject `clear_database_cmd`.

---

## Still stuck?

Check [ARCHITECTURE.md](ARCHITECTURE.md) data flow, [API.md](API.md) for IPC, or open an issue with Live status message and log output (`RUST_LOG=pitwall_desktop_lib=debug`).
