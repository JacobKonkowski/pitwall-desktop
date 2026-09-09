# API

Frontend IPC lives in `src/shared/api.ts` and `src/shared/types.ts`. TypeDoc: `npm run docs:api` (entry points under `src/shared`).

Backend commands are registered in `src-tauri/src/lib.rs` from `commands/mod.rs`.

**37 commands** covering Analyze storage, Live, settings, audio coach, monitor overlays, and VR/HUD.

## Analyze / storage

| Command | TS helper | Notes |
|---------|-----------|--------|
| `list_sessions` | `listSessions` | Session summaries (display cleanup applied) |
| `get_session` | `getSession` | Session + laps (display cleanup applied) |
| `get_lap_traces` | `getLapTraces` | Trace points for one lap |
| `compare_laps` | `compareLaps` | Two-lap comparison payload |
| `import_ibt` | `importIbt` | Single file (pipeline cleanup on write) |
| `import_folder_cmd` | `importFolder` | Folder scan |
| `check_iracing_config_cmd` | `checkIracingConfig` | mem/disk flags |
| `get_import_status` | `getImportStatus` | Watcher / import progress |
| `pick_ibt_file` | `pickIbtFile` | Dialog |
| `clear_database_cmd` | `clearDatabase` | Debug wipe |
| `delete_session_cmd` | `deleteSession` | Per-session delete |

## Live

| Command | TS helper | Notes |
|---------|-----------|--------|
| `start_live_monitor` | `startLiveMonitor` | |
| `stop_live_monitor` | `stopLiveMonitor` | |
| `get_live_status` | `getLiveStatus` | |
| `get_live_snapshot` | `getLiveSnapshot` | |
| `start_demo_clock` | `startDemoClock` | Offline exercise |
| `stop_demo_clock` | `stopDemoClock` | |

## Settings

| Command | TS helper | Notes |
|---------|-----------|--------|
| `get_settings` | `getSettings` | Full `AppSettings` |
| `save_settings_cmd` | `saveSettings` | Persists + may emit `settings-changed` |

## Audio

| Command | TS helper | Notes |
|---------|-----------|--------|
| `start_audio_coach` | `startAudioCoach` | |
| `stop_audio_coach` | `stopAudioCoach` | |
| `get_audio_coach_status` | `getAudioCoachStatus` | |
| `get_audio_coach_message` | `getAudioCoachMessage` | |
| `test_audio_coach` | `testAudioCoach` | TTS-only sample |

## Monitor overlays

| Command | TS helper | Notes |
|---------|-----------|--------|
| `start_monitor_overlay` | `startMonitorOverlay` | One always-on-top window per enabled widget |
| `stop_monitor_overlay` | `stopMonitorOverlay` | |
| `get_monitor_overlay_status` | `getMonitorOverlayStatus` | Active flag, message, open labels |

## VR / HUD

| Command | TS helper | Notes |
|---------|-----------|--------|
| `start_vr_overlay` | `startVrOverlay` | Native or web per settings |
| `stop_vr_overlay` | `stopVrOverlay` | |
| `get_vr_overlay_status` | `getVrOverlayStatus` | |
| `get_native_vr_status` | `getNativeVrStatus` | Write age, overlay count, last error |
| `is_vr_layer_installed` | `isVrLayerInstalled` | |
| `install_vr_layer` | `installVrLayer` | |
| `uninstall_vr_layer` | `uninstallVrLayer` | |
| `get_vr_layer_diagnostics` | `getVrLayerDiagnostics` | Ready / DLL / issues |
| `check_vr_hud_health` | `checkVrHudHealth` | Web HUD health |
| `open_vr_hud_preview_cmd` | `openVrHudPreview` | Opens browser preview |

## Events

- Live snapshot / status updates (see Live page listeners)
- `settings-changed` after save

## Notes for contributors

TS may still declare helpers for `patch_settings_cmd` / `list_tts_voices_cmd` — they are **not** in the Rust invoke handler until re-added.

## Capabilities

Main and monitor windows (`main`, `monitor-*`) use `src-tauri/capabilities/default.json`.