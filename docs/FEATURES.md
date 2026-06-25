# Features (user guide)

What PitWall does on each tab and overlay, in plain language.

---

## Analyze tab (after the session)

1. **Import telemetry** — Scan Folder, pick a file, or leave PitWall open while driving (auto-import from `Documents\iRacing\telemetry\`).
2. **Pick a session** — Sidebar lists track, car, date, lap count.
3. **Lap table** — Practice / qual / race groups, sector times (S1–S2–S3), delta to your best, lap kind (flying vs pit).
4. **Compare** — Select two laps for speed / throttle / brake traces.
5. **Fuel & tires** — Per-lap charts; tire note: some cars update wear only in pits.
6. **Coach** — Automatic rule-based tips (consistency, weak sectors, fuel, pace vs field, traffic).
7. **Standings** — Who you raced against, if a live snapshot linked to this IBT.
8. **AI summary** — Optional Ollama paragraph (needs Ollama running locally).

---

## Live tab (in the session)

1. **Start live monitor** — Connects to iRacing shared memory (~10 Hz UI).
2. **Metrics** — Lap time, deltas, sectors, fuel, temps, flags.
3. **Leaderboard** — Overall or class view; gaps vs your pace.
4. **Desktop overlay** — Pop-out transparent window with draggable widgets (coach, standings, relative, radar).
5. **In-headset HUD** — Native OpenXR layer (default) or OpenKneeboard web fallback.
6. **Audio coach** — Spoken race engineer (WAV + Windows speech). Toggle categories and chatter level in Settings.

Settings on the Live tab control Ollama, overlay layout, VR mode, and all audio toggles.

---

## Overlay widgets

Four slots shared between desktop pop-out and VR:

| Widget | Shows |
|--------|-------|
| Coach | Lap time, deltas, sector, fuel, field pace, flag badge |
| Standings | Compact leaderboard |
| Relative | Cars ahead/behind |
| Radar | Pack / spotter-style view |

Enable and position under **Settings → Overlay widgets**.

---

## VR modes

| Mode | What you need |
|------|----------------|
| **Native** (default) | OpenXR in iRacing, PitWall VR layer installed — see [NATIVE_VR.md](NATIVE_VR.md) |
| **Web fallback** | OpenKneeboard Web Dashboard tab at `http://127.0.0.1:17342/vr` |

---

## What PitWall does not do

- MoTeC export or external telemetry services (Garage61, VRS)
- Other drivers' sector times or traces live (iRacing SDK limit)
- Real-time LLM coaching while driving
- Multi-car IBT analysis in one view
- 4-wide pack detection

See [README](../README.md) out-of-scope list for future ideas.

---

## Related docs

- [SETUP.md](SETUP.md) — how to enable each feature
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — when something does not work
- [COMPARISON.md](COMPARISON.md) — live vs post-session data
