/**
 * Tauri IPC wrappers for the PitWall backend.
 *
 * Commands use `invoke()`; live/import updates use `listen()` helpers below.
 * Analyze, live, audio, monitor, and VR handlers are registered in
 * `src-tauri/src/commands/mod.rs`.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask } from "@tauri-apps/plugin-dialog";
import type {
  AppSettings,
  AudioCoachStatus,
  ImportStatus,
  IracingConfigCheck,
  LapComparison,
  LapTrace,
  LiveSnapshot,
  LiveStatus,
  MonitorOverlayStatus,
  NativeVrStatus,
  SessionDetail,
  SessionSummary,
  TtsVoiceInfo,
  VrLayerDiagnostics,
  VrOverlayStatus,
} from "./types";

/* --- Analyze / sessions --- */

export async function listSessions(): Promise<SessionSummary[]> {
  return invoke("list_sessions");
}

export async function getSession(sessionId: number): Promise<SessionDetail | null> {
  return invoke("get_session", { sessionId });
}

export async function getLapTraces(lapIds: number[]): Promise<LapTrace[]> {
  return invoke("get_lap_traces", { lapIds });
}

export async function compareLaps(
  candidateLapId: number,
  referenceLapId: number,
): Promise<LapComparison> {
  return invoke("compare_laps", { candidateLapId, referenceLapId });
}

export async function importIbt(path: string): Promise<string> {
  return invoke("import_ibt", { path });
}

export async function importFolder(): Promise<number> {
  return invoke("import_folder_cmd");
}

export async function checkIracingConfig(): Promise<IracingConfigCheck> {
  return invoke("check_iracing_config_cmd");
}

export async function getImportStatus(): Promise<ImportStatus> {
  return invoke("get_import_status");
}

export async function pickIbtFile(): Promise<string | null> {
  return invoke("pick_ibt_file");
}

export async function clearDatabase(): Promise<number> {
  return invoke("clear_database_cmd");
}

export async function deleteSession(sessionId: number): Promise<boolean> {
  return invoke("delete_session_cmd", { sessionId });
}

/** Native yes/no dialog (Tauri webview blocks `window.confirm`). */
export async function confirmDialog(
  message: string,
  title = "Confirm",
): Promise<boolean> {
  return ask(message, { title, kind: "warning" });
}

export function onImportComplete(callback: (sessionId: number) => void) {
  return listen<number>("import-complete", (event) => callback(event.payload));
}

export function onImportStatus(callback: (status: ImportStatus) => void) {
  return listen<ImportStatus>("import-status", (event) => callback(event.payload));
}

/* --- Live --- */

export async function startLiveMonitor(): Promise<void> {
  return invoke("start_live_monitor");
}

export async function stopLiveMonitor(): Promise<void> {
  return invoke("stop_live_monitor");
}

export async function startDemoClock(): Promise<void> {
  return invoke("start_demo_clock");
}

export async function stopDemoClock(): Promise<void> {
  return invoke("stop_demo_clock");
}

export async function getLiveStatus(): Promise<LiveStatus> {
  return invoke("get_live_status");
}

export async function getLiveSnapshot(): Promise<LiveSnapshot> {
  return invoke("get_live_snapshot");
}

export function onLiveTelemetry(callback: (snap: LiveSnapshot) => void) {
  return listen<LiveSnapshot>("live-telemetry", (event) => callback(event.payload));
}

export function onLiveStatus(callback: (status: LiveStatus) => void) {
  return listen<LiveStatus>("live-status", (event) => callback(event.payload));
}

/* --- Settings --- */

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings_cmd", { settings });
}

export async function patchSettings(patch: Partial<AppSettings>): Promise<AppSettings> {
  return invoke("patch_settings_cmd", { patch });
}

export function onSettingsChanged(callback: (settings: AppSettings) => void) {
  return listen<AppSettings>("settings-changed", (event) => callback(event.payload));
}

/* --- Audio coach --- */

export async function startAudioCoach(): Promise<void> {
  return invoke("start_audio_coach");
}

export async function stopAudioCoach(): Promise<void> {
  return invoke("stop_audio_coach");
}

export async function getAudioCoachStatus(): Promise<AudioCoachStatus> {
  return invoke("get_audio_coach_status");
}

export async function getAudioCoachMessage(): Promise<string> {
  return invoke("get_audio_coach_message");
}

export async function testAudioCoach(): Promise<void> {
  return invoke("test_audio_coach");
}

export async function listTtsVoices(): Promise<TtsVoiceInfo[]> {
  return invoke("list_tts_voices_cmd");
}

/* --- VR / HUD --- */

export async function startVrOverlay(): Promise<void> {
  return invoke("start_vr_overlay");
}

export async function stopVrOverlay(): Promise<void> {
  return invoke("stop_vr_overlay");
}

export async function getVrOverlayStatus(): Promise<VrOverlayStatus> {
  return invoke("get_vr_overlay_status");
}

export async function getNativeVrStatus(): Promise<NativeVrStatus> {
  return invoke("get_native_vr_status");
}

export async function isVrLayerInstalled(): Promise<boolean> {
  return invoke("is_vr_layer_installed");
}

export async function installVrLayer(): Promise<void> {
  return invoke("install_vr_layer");
}

export async function uninstallVrLayer(): Promise<void> {
  return invoke("uninstall_vr_layer");
}

export async function getVrLayerDiagnostics(): Promise<VrLayerDiagnostics> {
  return invoke("get_vr_layer_diagnostics");
}

export async function checkVrHudHealth(): Promise<boolean> {
  return invoke("check_vr_hud_health");
}

export async function openVrHudPreview(): Promise<void> {
  return invoke("open_vr_hud_preview_cmd");
}

/* --- Monitor overlays --- */

export async function startMonitorOverlay(): Promise<void> {
  return invoke("start_monitor_overlay");
}

export async function stopMonitorOverlay(): Promise<void> {
  return invoke("stop_monitor_overlay");
}

export async function getMonitorOverlayStatus(): Promise<MonitorOverlayStatus> {
  return invoke("get_monitor_overlay_status");
}

export function buildOpenKneeboardUrl(settings: AppSettings, baseUrl: string): string {
  const layoutMap: Record<string, string> = {
    coach: "ironman",
    standings: "standings",
    relative: "relative",
    radar: "radar",
  };
  const kinds = ["coach", "standings", "relative", "radar"];
  const enabled = settings.overlayLayout.widgets
    .map((w, i) => ({ w, kind: kinds[i] }))
    .filter(({ w }) => w.enabled);
  const layout = enabled.length > 0 ? layoutMap[enabled[0].kind] ?? "ironman" : "ironman";
  const pace = settings.overlayLayout.fieldPaceMode || "best";
  const url = new URL(baseUrl.endsWith("/vr") ? baseUrl : `${baseUrl.replace(/\/$/, "")}/vr`);
  url.searchParams.set("layout", layout);
  url.searchParams.set("pace", pace);
  return url.toString();
}
