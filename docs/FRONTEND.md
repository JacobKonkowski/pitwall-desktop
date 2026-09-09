# Frontend

## Entry

`index.html` → `src/main.tsx` → `shell/AppShell.tsx`.

## Layout

```
src/
  shell/           AppShell, FeatureNav
  features/
    registry.ts    Feature[] — Analyze + Live
    analyze/       AnalyzePage, browser, laps, compare, insights, fuel…
    live/          LivePage, SessionLeaderboard
  shared/          api.ts, types.ts, format, toast
  widgets/         Coach, Standings, Relative, Radar + widgets.css
  styles/          tokens.css, app.css
```

Adding a surface: create `features/<id>/`, export a `Feature`, append to `registry.ts`. Nav appears automatically when more than one feature is registered.

## IPC

All `invoke` wrappers and shared DTOs live in **`src/shared/`** (not `src/lib/`). Typedoc entry points match that layout.

## Widgets

Widgets render coach / standings / relative / radar for the Live in-app preview and share shapes with VR SHM slots.

## Styling

Global look: `src/styles/`. Widget-specific: `src/widgets/widgets.css`.
