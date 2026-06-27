# Frontend

React 19 + TypeScript + Vite. Two HTML entry points share widgets and types with the Rust backend via Tauri IPC.

---

## Entry points

| HTML | TS entry | Window |
|------|----------|--------|
| `index.html` | `main.tsx` | Main — Analyze \| Live \| Settings tabs |
| `overlay.html` | `overlay.tsx` | `live-overlay` — desktop pop-out |

Built as a Vite multi-page app (`vite.config.ts`, dev port **1420**).

---

## Main app structure

[`App.tsx`](../src/App.tsx) — tab shell:

- **Analyze** — `SessionBrowser`, `LapTable`, compare chart, fuel/tire, coach, standings
- **Live** — unified dashboard (`LivePanel` with metrics, traffic, radar, relative, leaderboard)
- **Settings** — [`SettingsPage`](../src/components/SettingsPage.tsx) with section nav (AI, Overlay & VR, Audio, Advanced)

[`LivePanel.tsx`](../src/components/LivePanel.tsx) — live controls and dashboard; configuration lives on the Settings tab.

[`useSettings`](../src/lib/useSettings.ts) — shared settings load/patch with debounced save. Overlay listens for `settings-changed` events.

---

## Event-driven live UI

Prefer events over polling:

```typescript
onLiveTelemetry((snap) => { /* update UI */ });
onLiveStatus((status) => { /* connection state */ });
```

Same pattern in [`OverlayView.tsx`](../src/components/OverlayView.tsx) for the pop-out window.

Import progress: `onImportStatus`, `onImportComplete`.

---

## Widget system

[`src/widgets/`](../src/widgets/) — shared by desktop overlay and VR reference layout:

| Index | Kind | Component |
|-------|------|-----------|
| 0 | coach | `CoachWidget` |
| 1 | standings | `StandingsWidget` |
| 2 | relative | `RelativeWidget` |
| 3 | radar | `RadarWidget` |

`settings.overlayLayout` drives enabled state, desktop pixel placement, and VR offset/scale/opacity. Index matches Rust `WIDGET_*` constants and OpenXR layer slots.

---

## API layer

[`src/lib/api.ts`](../src/lib/api.ts) — `invoke()` per Tauri command + `listen()` helpers.

[`src/lib/types.ts`](../src/lib/types.ts) — mirrors Rust `serde` structs (`camelCase`).

Run `npm run docs:api` for TypeDoc output. Contract table: [API.md](API.md).

---

## Component map

| File | When to read |
|------|--------------|
| `SessionBrowser.tsx` | Import, session list, clear DB |
| `LapTable.tsx` | Lap selection, sectors, coach highlights |
| `LapCompareChart.tsx` | Two-lap Recharts traces |
| `CoachPanel.tsx` | Rule insights + Ollama button |
| `SessionStandingsPanel.tsx` | Linked live standings |
| `SessionLeaderboard.tsx` | Live field table |
| `OverlayView.tsx` | Draggable widget shell |
| `widgets/*.tsx` | Individual HUD panels |

---

## Styling

- `App.css` — main window
- `overlay.css` — transparent overlay window
- `widgets/widgets.css` — shared widget chrome

---

## Related docs

- [API.md](API.md) — commands and events
- [FEATURES.md](FEATURES.md) — user-facing UI tour
- [NATIVE_VR.md](NATIVE_VR.md) — VR compositor (C++ layer reads same widget config)
