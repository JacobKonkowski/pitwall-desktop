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
2. **Unified dashboard** — Lap time, deltas, race clock, gaps, pack state, flags, sectors, tires, radar, relative board, and full leaderboard on one screen.
3. **Desktop overlay** — Pop-out transparent window with draggable widgets (coach, standings, relative, radar).
4. **In-headset HUD** — Native OpenXR layer (default) or OpenKneeboard web fallback (URL includes layout/pace from settings).
5. **Audio coach** — Spoken race engineer (WAV + Windows speech).

## Settings tab

Global configuration with sections:

- **AI** — Ollama URL/model for post-session summaries
- **Overlay & VR** — Widget layout, VR mode, recenter hotkey, field pace
- **Audio coach** — Voice, category toggles, chatter level, rate/volume
- **Advanced** — Export/import settings JSON

Session browser on Analyze also supports search, sort, per-session delete, and folder scan timing summary.

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
