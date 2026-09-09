# Architecture

PitWall is a Tauri 2 + React desktop app with a **Cargo workspace** of domain crates
and a thin `src-tauri` composition root. See [FOUNDATION.md](FOUNDATION.md) for the
dependency table and contributor playbook.

```
┌─────────────────────────────────────────────────────────┐
│  React shell + feature registry (Analyze | Live)        │
│  widgets catalog → monitor windows + VR HUD             │
└─────────────┬───────────────────────────┬───────────────┘
              │ invoke / events           │
┌─────────────▼───────────────────────────▼───────────────┐
│  pitwall-desktop (commands / AppState)                  │
├──────────┬──────────┬──────────┬──────────┬─────────────┤
│ ingest   │ live     │ audio    │ monitor  │ vr          │
│ analysis │ storage  │ settings │ telemetry│             │
└──────────┴──────────┴──────────┴──────────┴─────────────┘
```

## Frontend

| Path | Role |
|------|------|
| `src/shell/` | AppShell, feature nav |
| `src/features/registry.ts` | Analyze, Live |
| `src/features/analyze/` | Post-session UI |
| `src/features/live/` | Live / coach / monitor / VR controls |
| `src/shared/` | api, types, format, toast, i18n |
| `src/widgets/` | Shared presentational widgets |
| `src/monitor/` | Monitor window entry (when present) |

## Live data paths

1. **UI** — ~10 Hz snapshot  
2. **Monitor** — always-on-top Tauri windows per enabled widget  
3. **Native VR** — ~30 Hz SHM for OpenXR layer  
4. **Web HUD** — `:17342`  
5. **Audio** — 250 ms rule engine poll  

## Related

- [FOUNDATION.md](FOUNDATION.md) — crates and rules  
- [NATIVE_VR.md](NATIVE_VR.md) — OpenXR details  
- [PLUGINS.md](PLUGINS.md) — extension seams  
