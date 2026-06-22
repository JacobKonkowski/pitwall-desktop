import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppSettings,
  AudioCoachStatus,
  CoachReport,
  CoachSummaryResult,
  FuelSummary,
  ImportStatus,
  IracingConfigCheck,
  LapTrace,
  LiveSnapshot,
  LiveStatus,
  SessionDetail,
  SessionSummary,
  TireSummary,
  VrOverlayStatus,
} from "./types";

export async function listSessions(): Promise<SessionSummary[]> {
  return invoke("list_sessions");
}

export async function getSession(sessionId: number): Promise<SessionDetail | null> {
  return invoke("get_session", { sessionId });
}

export async function getLapTraces(lapIds: number[]): Promise<LapTrace[]> {
  return invoke("get_lap_traces", { lapIds });
}

export async function getFuelSummary(sessionId: number): Promise<FuelSummary> {
  return invoke("get_fuel_summary", { sessionId });
}

export async function getTireSummary(sessionId: number): Promise<TireSummary> {
  return invoke("get_tire_summary", { sessionId });
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

export async function startLiveMonitor(): Promise<void> {
  return invoke("start_live_monitor");
}

export async function stopLiveMonitor(): Promise<void> {
  return invoke("stop_live_monitor");
}

export async function getLiveStatus(): Promise<LiveStatus> {
  return invoke("get_live_status");
}

export async function getLiveSnapshot(): Promise<LiveSnapshot> {
  return invoke("get_live_snapshot");
}

export async function getCoachReport(sessionId: number): Promise<CoachReport> {
  return invoke("get_coach_report", { sessionId });
}

export async function generateCoachSummary(sessionId: number): Promise<CoachSummaryResult> {
  return invoke("generate_coach_summary", { sessionId });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings_cmd", { settings });
}

export async function openDesktopOverlay(): Promise<void> {
  return invoke("open_desktop_overlay_cmd");
}

export async function closeDesktopOverlay(): Promise<void> {
  return invoke("close_desktop_overlay_cmd");
}

export async function isDesktopOverlayOpen(): Promise<boolean> {
  return invoke("is_desktop_overlay_open_cmd");
}

export async function startVrOverlay(): Promise<void> {
  return invoke("start_vr_overlay");
}

export async function stopVrOverlay(): Promise<void> {
  return invoke("stop_vr_overlay");
}

export async function getVrOverlayStatus(): Promise<VrOverlayStatus> {
  return invoke("get_vr_overlay_status");
}

export async function checkVrHudHealth(): Promise<boolean> {
  return invoke("check_vr_hud_health");
}

export async function openVrHudPreview(): Promise<void> {
  return invoke("open_vr_hud_preview_cmd");
}

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

export function onImportComplete(callback: (sessionId: number) => void) {
  return listen<number>("import-complete", (event) => callback(event.payload));
}

export function onImportStatus(callback: (status: ImportStatus) => void) {
  return listen<ImportStatus>("import-status", (event) => callback(event.payload));
}

export function onLiveTelemetry(callback: (snap: LiveSnapshot) => void) {
  return listen<LiveSnapshot>("live-telemetry", (event) => callback(event.payload));
}

export function onLiveStatus(callback: (status: LiveStatus) => void) {
  return listen<LiveStatus>("live-status", (event) => callback(event.payload));
}

export function formatLapTime(ms: number | null | undefined): string {
  if (ms == null || ms <= 0) return "—";
  const totalSec = ms / 1000;
  const min = Math.floor(totalSec / 60);
  const sec = totalSec - min * 60;
  return min > 0 ? `${min}:${sec.toFixed(3).padStart(6, "0")}` : sec.toFixed(3);
}

export function formatDelta(ms: number | null | undefined): string {
  if (ms == null) return "—";
  const sign = ms >= 0 ? "+" : "";
  return `${sign}${(ms / 1000).toFixed(3)}`;
}

export function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}
