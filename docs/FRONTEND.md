# Frontend

## Entry

`index.html` → `src/main.tsx` → `shell/AppShell.tsx` (main app).

`monitor.html` → `src/monitor/main.tsx` → per-widget overlay host (window label `monitor-<kind>`).

## Layout

```
src/
  shell/           AppShell, FeatureNav
  features/
    registry.ts    Feature[] — Analyze + Live
    analyze/       AnalyzePage, browser, laps, compare, insights, fuel…
    live/          LivePage, SessionLeaderboard
  monitor/         Transparent overlay window entry
  shared/          api.ts, types.ts, format, toast
  widgets/         Coach, Standings, Relative, Radar + widgets.css
  styles/          tokens.css, app.css
```

Adding a surface: create `features/<id>/`, export a `Feature`, append to `registry.ts`. Nav appears automatically when more than one feature is registered.

## IPC

All `invoke` wrappers and shared DTOs live in **`src/shared/`** (not `src/lib/`). Typedoc entry points match that layout.

## Widgets

Widgets render coach / standings / relative / radar for the Live in-app preview, monitor overlay windows, and share shapes with VR SHM slots. Enable once in `overlayLayout`; place with `desktop*` (monitor) and `vr*` (headset).

## Styling

Global look: `src/styles/`. Widget-specific: `src/widgets/widgets.css`.
