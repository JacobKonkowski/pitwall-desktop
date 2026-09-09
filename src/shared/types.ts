/**
 * Shared serde types for the Tauri IPC boundary (`camelCase` JSON).
 * Rust definitions: `src-tauri/src/storage/models.rs`, `analysis/compare.rs`,
 * and (when registered) live / settings / audio / vr modules.
 *
 * Lap facts mirror iRacing exactly — the sim's `_OK` flags, pit-road samples, and
 * distance coverage. There is no invented lap kind or opaque "valid" flag.
 */

export interface SessionSummary {
  id: number;
  ibtPath: string;
  track: string;
  car: string;
  sessionDate: string;
  lapCount: number;
  /** Fastest pace-eligible lap, if any. */
  bestLapMs: number | null;
  importedAt: string;
}

export interface SectorTime {
  sectorNum: number;
  timeMs: number;
}

export interface LapSummary {
  id: number;
  sessionNum: number;
  sessionType: string;
  iracingLap: number;
  lapNumber: number;
  lapTimeMs: number | null;
  /** `LapDeltaToBestLap_OK`, or `null` when the channel was absent in the IBT. */
  deltaBestOk: boolean | null;
  deltaSessionBestOk: boolean | null;
  onPitRoadStart: boolean;
  onPitRoadEnd: boolean;
  lapDistPctMin: number | null;
  lapDistPctMax: number | null;
  /** Derived: reported time and both `_OK` flags true. */
  paceEligible: boolean;
  fuelStart: number | null;
  fuelUsed: number | null;
  avgSpeed: number | null;
  lfTemp: number | null;
  rfTemp: number | null;
  lrTemp: number | null;
  rrTemp: number | null;
  sectors: SectorTime[];
  /** Delta to the fastest pace-eligible lap in this sub-session. */
  deltaToBestMs: number | null;
}

export interface SessionDetail {
  session: SessionSummary;
  laps: LapSummary[];
}

export interface TracePoint {
  distPct: number;
  speed: number;
  throttle: number;
  brake: number;
  gear: number;
  steering: number;
}

export interface LapTrace {
  lapId: number;
  lapNumber: number;
  points: TracePoint[];
}

export interface ImportStatus {
  active: boolean;
  currentFile: string | null;
  progressPct: number;
  message: string;
}

export interface IracingConfigCheck {
  appIniPath: string;
  telemetryDir: string;
  memEnabled: boolean;
  diskEnabled: boolean;
  warnings: string[];
}

/* --- Comparison (from `compare_laps`) --- */

export interface SectorDelta {
  sectorNum: number;
  candidateMs: number | null;
  referenceMs: number | null;
  deltaMs: number | null;
}

export interface AlignedPoint {
  distPct: number;
  candidateSpeed: number | null;
  referenceSpeed: number | null;
  candidateThrottle: number | null;
  referenceThrottle: number | null;
  candidateBrake: number | null;
  referenceBrake: number | null;
  candidateGear: number | null;
  referenceGear: number | null;
  candidateSteering: number | null;
  referenceSteering: number | null;
  /** Optional backend cumulative time delta (ms). Client may approximate if absent. */
  cumulativeDeltaMs?: number | null;
}

export interface LapComparison {
  candidateLapId: number;
  referenceLapId: number;
  candidateTimeMs: number | null;
  referenceTimeMs: number | null;
  deltaMs: number | null;
  sectorDeltas: SectorDelta[];
  series: AlignedPoint[];
}

/* --- Live telemetry --- */

export type LiveConnectionState =
  | "disconnected"
  | "waitingForSession"
  | "reconnecting"
  | "connected"
  | "error";

export interface LiveStatus {
  state: LiveConnectionState;
  message: string;
}

export interface LiveSectorProgress {
  sectorNum: number;
  timeMs: number | null;
  completed: boolean;
}

export type PackState =
  | "off"
  | "clear"
  | "carLeft"
  | "carRight"
  | "threeWide"
  | "twoCarsLeft"
  | "twoCarsRight";

export interface CompetitorEntry {
  carIdx: number;
  driverName: string;
  carNumber: string;
  classId: number;
  classColor: string;
  position: number;
  classPosition: number;
  bestLapMs: number | null;
  lastLapMs: number | null;
  onPitRoad: boolean;
  isPlayer: boolean;
  lapDistPct: number;
  gapToPlayerS: number | null;
}

/** Live telemetry + field context from `get_live_snapshot` / `live-telemetry` events. */
export interface LiveSnapshot {
  track: string;
  car: string;
  sessionType: string;
  lap: number;
  lapTimeMs: number;
  lastLapMs: number | null;
  lastLapValid: boolean;
  bestLapMs: number | null;
  deltaToBestMs: number | null;
  deltaToLastMs: number | null;
  fuelLevel: number;
  speed: number;
  lapDistPct: number;
  currentSector: number;
  sectorBoundaries: number[];
  sectors: LiveSectorProgress[];
  lfTemp: number;
  rfTemp: number;
  lrTemp: number;
  rrTemp: number;
  onPitRoad: boolean;
  competitors: CompetitorEntry[];
  playerPosition: number | null;
  playerClassPosition: number | null;
  sessionFastestLapMs: number | null;
  deltaToSessionBestMs: number | null;
  deltaToSessionOptimalMs: number | null;
  gapToCarAheadS: number | null;
  gapToCarBehindS: number | null;
  packState: PackState;
  sessionFlags: number;
  incidentCount: number;
  sessionLapsRemain: number | null;
  sessionTimeRemainS: number | null;
  pitsOpen: boolean;
  onTrack: boolean;
}

export type WidgetKind = "coach" | "standings" | "relative" | "radar";

export const WIDGET_KINDS: WidgetKind[] = ["coach", "standings", "relative", "radar"];

export const WIDGET_LABELS: Record<WidgetKind, string> = {
  coach: "Coach HUD",
  standings: "Standings",
  relative: "Relative",
  radar: "Radar",
};

export interface WidgetPlacement {
  /** Shared enable flag for monitor windows and VR slots. */
  enabled: boolean;
  /** Monitor window screen position / size (pixels). */
  desktopX: number;
  desktopY: number;
  desktopW: number;
  desktopH: number;
  /** VR placement (meters / multipliers). */
  vrOffsetY: number;
  vrScale: number;
  vrOpacity: number;
}

export interface OverlayLayout {
  widgets: WidgetPlacement[];
  fieldPaceMode: string;
}

/** Default layout, mirroring `OverlayLayout::default()` in the Rust settings. */
export function defaultOverlayLayout(): OverlayLayout {
  const base = (over: Partial<WidgetPlacement>): WidgetPlacement => ({
    enabled: false,
    desktopX: 24,
    desktopY: 24,
    desktopW: 320,
    desktopH: 180,
    vrOffsetY: 0,
    vrScale: 1,
    vrOpacity: 1,
    ...over,
  });
  return {
    widgets: [
      base({ enabled: true, desktopX: 24, desktopY: 24, desktopW: 360, desktopH: 200 }),
      base({ desktopX: 24, desktopY: 244, desktopW: 320, desktopH: 300 }),
      base({ desktopX: 360, desktopY: 244, desktopW: 300, desktopH: 240 }),
      base({ desktopX: 404, desktopY: 24, desktopW: 200, desktopH: 200 }),
    ],
    fieldPaceMode: "best",
  };
}

/** User preferences persisted to `%LOCALAPPDATA%\\pitwall-desktop\\settings.json`. */
export interface AppSettings {
  ollamaUrl?: string;
  ollamaModel?: string;
  overlayX?: number;
  overlayY?: number;
  overlayWidth?: number;
  overlayHeight?: number;
  vrOverlayEnabled: boolean;
  vrOverlayScale: number;
  vrMode: string;
  vrHudOffset: number;
  vrHudOpacity: number;
  vrRecenterHotkey: string;
  vrFieldPaceMode: string;
  overlayLayout: OverlayLayout;
  audioCoachEnabled: boolean;
  audioCoachRate: number;
  audioCoachVolume: number;
  audioCoachFuelThreshold: number;
  audioPackAlertsEnabled: boolean;
  audioFlagsEnabled: boolean;
  audioIncidentsEnabled: boolean;
  audioFuelRaceEnabled: boolean;
  audioGapAlertsEnabled: boolean;
  audioPaceEnabled: boolean;
  audioStrategyEnabled: boolean;
  audioRaceClockEnabled: boolean;
  audioPitsOpenEnabled: boolean;
  audioCoachChatterLevel: "minimal" | "normal" | "verbose";
  audioCoachVoice: string;
  audioSessionIntroEnabled: boolean;
  audioPositionCalloutsEnabled: boolean;
  audioTyreAlertsEnabled: boolean;
  audioInvalidLapEnabled: boolean;
  audioRadioEffectsEnabled: boolean;
  audioPackPrecursorsEnabled: boolean;
  audioFuelStrategySensitivity: "normal" | "conservative";
  audioInterMessageGapMs: number;
  audioVoiceCommandsEnabled?: boolean;
  audioVoicePushToTalkKey?: string;
}

export interface AudioCoachStatus {
  active: boolean;
  lastMessage: string;
}

export interface MonitorOverlayStatus {
  active: boolean;
  message: string;
  windows: string[];
}

export interface VrOverlayStatus {
  active: boolean;
  runtime: string;
  message: string;
  hudUrl: string;
  mode: string;
  layerInstalled: boolean;
}

export interface NativeVrStatus {
  active: boolean;
  layerInstalled: boolean;
  /** Proxy: fresh SHM publish + layer installed (no layer heartbeat file). */
  compositorActive: boolean;
  telemetryPublishing: boolean;
  lastFrameAgeMs: number | null;
  writeAgeMs: number | null;
  overlayCount: number;
  lastError: string | null;
}

export interface VrLayerDiagnostics {
  registered: boolean;
  manifestPath: string | null;
  dllPresent: boolean;
  dllPath: string | null;
  layerDisabled: boolean;
  ready: boolean;
  iracingOpenXrVrMode: number | null;
  iracingOpenXrEnabled: boolean | null;
  issues: string[];
}

export interface TtsVoiceInfo {
  displayName: string;
  language: string;
  gender: string;
  neural: boolean;
}
