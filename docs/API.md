# IPC API reference

PitWall Desktop uses **Tauri invoke** (commands) and **events** between the Rust backend and React frontend. TypeScript wrappers live in [`src/lib/api.ts`](../src/lib/api.ts); Rust handlers in [`src-tauri/src/commands/mod.rs`](../src-tauri/src/commands/mod.rs).

## Generate full API reference

```powershell
npm run docs:api
```

Opens locally (gitignored):

| Output | Path |
|--------|------|
| Rust (rustdoc) | `docs/.api-out/rust/pitwall_desktop_lib/index.html` |
| TypeScript (TypeDoc) | `docs/.api-out/ts/index.html` |

On non-Windows: run `cargo doc --no-deps --lib` from `src-tauri/`, then `npx typedoc` from the repo root.

---

## Commands (37)

Arguments use **camelCase** in JSON from the frontend (Tauri serde convention).

### Sessions and analysis

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `list_sessions` | `listSessions` | — | `SessionSummary[]` | All sessions, newest first |
| `get_session` | `getSession` | `sessionId` | `SessionDetail \| null` | Laps, sectors, metadata |
| `get_lap_traces` | `getLapTraces` | `lapIds` | `LapTrace[]` | Downsampled traces for compare chart |
| `get_fuel_summary` | `getFuelSummary` | `sessionId` | `FuelSummary` | Per-lap fuel usage |
| `get_tire_summary` | `getTireSummary` | `sessionId` | `TireSummary` | Per-lap tire temps |
| `get_coach_report` | `getCoachReport` | `sessionId` | `CoachReport` | Rule-based insights (+ trace/standings when available) |
| `get_session_standings` | `getSessionStandings` | `sessionId` | `SessionStandings \| null` | Linked live standings snapshot |
| `generate_coach_summary` | `generateCoachSummary` | `sessionId` | `CoachSummaryResult` | Ollama AI summary |

### Import

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `import_ibt` | `importIbt` | `path` | `string` | Import one IBT file |
| `import_folder_cmd` | `importFolder` | — | `number` | Scan default telemetry folder |
| `check_iracing_config_cmd` | `checkIracingConfig` | — | `IracingConfigCheck` | Validate `app.ini` |
| `get_import_status` | `getImportStatus` | — | `ImportStatus` | Current import progress |
| `pick_ibt_file` | `pickIbtFile` | — | `string \| null` | Native file picker |
| `clear_database_cmd` | `clearDatabase` | — | `number` | **Debug builds only** — wipe DB |

### Live monitor

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `start_live_monitor` | `startLiveMonitor` | — | — | Connect to iRacing; may auto-start VR/audio per settings |
| `stop_live_monitor` | `stopLiveMonitor` | — | — | Stop live, VR, and audio |
| `get_live_status` | `getLiveStatus` | — | `LiveStatus` | Connection state |
| `get_live_snapshot` | `getLiveSnapshot` | — | `LiveSnapshot` | Latest telemetry snapshot |

### Settings

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `get_settings` | `getSettings` | — | `AppSettings` | Load settings |
| `save_settings_cmd` | `saveSettings` | `settings` | — | Persist settings |

### Desktop overlay

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `open_desktop_overlay_cmd` | `openDesktopOverlay` | — | — | Open `live-overlay` window |
| `close_desktop_overlay_cmd` | `closeDesktopOverlay` | — | — | Close overlay |
| `is_desktop_overlay_open_cmd` | `isDesktopOverlayOpen` | — | `boolean` | Overlay window state |

### VR overlay

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `start_vr_overlay` | `startVrOverlay` | — | — | Requires live monitor |
| `stop_vr_overlay` | `stopVrOverlay` | — | — | Stop VR compositor / HUD server |
| `get_vr_overlay_status` | `getVrOverlayStatus` | — | `VrOverlayStatus` | Active mode and URL |
| `get_native_vr_status` | `getNativeVrStatus` | — | `NativeVrStatus` | SHM / layer health |
| `is_vr_layer_installed` | `isVrLayerInstalled` | — | `boolean` | OpenXR registry check |
| `install_vr_layer` | `installVrLayer` | — | — | Register implicit API layer |
| `uninstall_vr_layer` | `uninstallVrLayer` | — | — | Remove layer registration |
| `get_vr_layer_diagnostics` | `getVrLayerDiagnostics` | — | `VrLayerDiagnostics` | Install path, DLL, issues |
| `check_vr_hud_health` | `checkVrHudHealth` | — | `boolean` | Web fallback HTTP probe |
| `open_vr_hud_preview_cmd` | `openVrHudPreview` | — | — | Browser preview of web HUD |

### Audio coach

| Command | TS wrapper | Args | Returns | Purpose |
|---------|------------|------|---------|---------|
| `start_audio_coach` | `startAudioCoach` | — | — | Requires live monitor |
| `stop_audio_coach` | `stopAudioCoach` | — | — | Stop speech queue |
| `get_audio_coach_status` | `getAudioCoachStatus` | — | `AudioCoachStatus` | Active + last message |
| `get_audio_coach_message` | `getAudioCoachMessage` | — | `string` | Last spoken line |

---

## Events (4)

Subscribe via `listen()` in [`api.ts`](../src/lib/api.ts).

| Event | Payload | Emitter | Rate / trigger |
|-------|---------|---------|----------------|
| `import-status` | `ImportStatus` | `import_ibt`, `import_runner` | During import |
| `import-complete` | `sessionId: number` | `import_runner` | Successful import |
| `live-telemetry` | `LiveSnapshot` | `live/mod.rs` | ~10 Hz while connected |
| `live-status` | `LiveStatus` | `live/mod.rs` | ~10 Hz while connected |

---

## Type index

Serde types are defined in Rust and mirrored in [`src/lib/types.ts`](../src/lib/types.ts).

| Type | Rust module | Notes |
|------|-------------|-------|
| `AppSettings` | `settings/mod.rs` | Full field list in [DATA_MODEL.md](DATA_MODEL.md) |
| `LiveSnapshot` | `live/snapshot.rs` | Live telemetry + field data |
| `LiveStatus` | `live/snapshot.rs` | `LiveConnectionState` + message |
| `SessionDetail` | `storage/` | Session + laps |
| `CoachReport` | `analysis/coach.rs` | Post-session insights |
| `SessionStandings` | `storage/` | Post-disconnect field snapshot |
| `ImportStatus` | `storage/` | Import progress |
| `AudioCoachStatus` | `audio/mod.rs` | Runtime audio state |
| `VrOverlayStatus`, `NativeVrStatus`, `VrLayerDiagnostics` | `vr/` | VR modes and diagnostics |

For struct fields, run `npm run docs:api` and open the rustdoc / TypeDoc pages above.

---

## Tauri capabilities

| File | Window | Permissions |
|------|--------|-------------|
| `capabilities/default.json` | `main` | `core:default`, `dialog:default` |
| `capabilities/overlay.json` | `live-overlay` | `core:default` |
