export interface SessionSummary {
  id: number;
  ibtPath: string;
  track: string;
  car: string;
  sessionDate: string;
  lapCount: number;
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
  valid: boolean;
  fuelStart: number | null;
  fuelUsed: number | null;
  avgSpeed: number | null;
  lfTemp: number | null;
  rfTemp: number | null;
  lrTemp: number | null;
  rrTemp: number | null;
  sectors: SectorTime[];
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

export interface FuelLapSummary {
  lapNumber: number;
  fuelUsed: number;
  fuelRemaining: number;
  lapsRemainingEstimate: number | null;
}

export interface FuelSummary {
  laps: FuelLapSummary[];
  tankCapacity: number | null;
}

export interface TireLapSummary {
  lapNumber: number;
  lfTemp: number;
  rfTemp: number;
  lrTemp: number;
  rrTemp: number;
}

export interface TireSummary {
  laps: TireLapSummary[];
  note: string;
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

export interface CoachInsight {
  kind: string;
  title: string;
  detail: string;
  severity: string;
  lapNumbers: number[];
  sectorNum: number | null;
  deltaMs: number | null;
}

export interface SessionCoachStats {
  validLapCount: number;
  consistencyMs: number | null;
  bestLapMs: number | null;
  avgLapMs: number | null;
  weakestSector: number | null;
  weakestSectorLossMs: number | null;
}

export interface CoachReport {
  sessionId: number;
  insights: CoachInsight[];
  summary: SessionCoachStats;
}

export interface CompetitorStanding {
  position: number;
  classPosition: number;
  carNumber: string;
  driverName: string;
  classId: number;
  classColor: string;
  bestLapMs: number | null;
  isPlayer: boolean;
}

export interface SessionStandings {
  id: number;
  sessionId: number | null;
  track: string;
  sessionType: string;
  sessionDate: string;
  sessionFastestMs: number | null;
  playerBestMs: number | null;
  playerPosition: number | null;
  playerClassPosition: number | null;
  competitors: CompetitorStanding[];
  trafficLaps: number[];
  createdAt: string;
}

export interface CoachSummaryResult {
  markdown: string;
  model: string;
}

export type WidgetKind = "coach" | "standings" | "relative" | "radar";

/** Index of each widget in {@link OverlayLayout.widgets}; matches the Rust order
 *  and the VR overlay slot / kind. */
export const WIDGET_KINDS: WidgetKind[] = ["coach", "standings", "relative", "radar"];

export const WIDGET_LABELS: Record<WidgetKind, string> = {
  coach: "Coach HUD",
  standings: "Standings",
  relative: "Relative",
  radar: "Radar",
};

export interface WidgetPlacement {
  enabled: boolean;
  desktopX: number;
  desktopY: number;
  desktopW: number;
  desktopH: number;
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

export interface AppSettings {
  ollamaUrl: string;
  ollamaModel: string;
  overlayX: number;
  overlayY: number;
  overlayWidth: number;
  overlayHeight: number;
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
}

export interface AudioCoachStatus {
  active: boolean;
  lastMessage: string;
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
  compositorActive: boolean;
  lastFrameAgeMs: number | null;
}
