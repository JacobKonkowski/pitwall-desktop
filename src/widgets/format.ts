import type { CompetitorEntry, LiveSnapshot, PackState } from "../lib/types";

/** Signed delta in seconds, e.g. "+0.123" / "-0.080". */
export function fmtDelta(ms: number | null | undefined): string {
  if (ms == null) return "\u2014";
  const sign = ms >= 0 ? "+" : "";
  return `${sign}${(ms / 1000).toFixed(3)}`;
}

/** Gap in seconds, magnitude only, e.g. "1.2s". */
export function fmtGap(sec: number | null | undefined): string {
  if (sec == null) return "\u2014";
  return `${Math.abs(sec).toFixed(1)}s`;
}

/** "fast" (gain), "slow" (loss) or "neutral" (unknown) for delta coloring. */
export function deltaClass(ms: number | null | undefined): "fast" | "slow" | "neutral" {
  if (ms == null) return "neutral";
  return ms > 0 ? "slow" : ms < 0 ? "fast" : "neutral";
}

/** "P3 · P5" (class · overall) or "P3" when they match / only one is known. */
export function positionLabel(snap: LiveSnapshot): string {
  const cls = snap.playerClassPosition;
  const overall = snap.playerPosition;
  if (cls != null && overall != null && cls !== overall) return `P${cls} \u00b7 P${overall}`;
  if (cls != null) return `P${cls}`;
  if (overall != null) return `P${overall}`;
  return "";
}

const PACK_LABELS: Record<PackState, string> = {
  off: "",
  clear: "CLEAR",
  carLeft: "\u25C0 CAR",
  carRight: "CAR \u25B6",
  threeWide: "3-WIDE",
  twoCarsLeft: "2 LEFT",
  twoCarsRight: "2 RIGHT",
};

export function packLabel(state: PackState): string {
  return PACK_LABELS[state] ?? "";
}

/** True once any meaningful telemetry has arrived. */
export function hasLiveData(snap: LiveSnapshot | null): snap is LiveSnapshot {
  return !!snap && (!!snap.track || snap.lap > 0 || snap.fuelLevel > 0);
}

/** Progress 0..100 of the given sector, mirroring the VR HUD logic. */
export function sectorProgress(snap: LiveSnapshot, sectorNum: number): number {
  const sector = snap.sectors.find((s) => s.sectorNum === sectorNum);
  if (sector?.completed) return 100;
  if (snap.currentSector !== sectorNum) return 0;
  const bounds = [0, 0.33, 0.66, 1];
  const start = bounds[sectorNum - 1] ?? 0;
  const end = bounds[sectorNum] ?? 1;
  const span = end - start;
  if (span <= 0) return 0;
  return Math.min(100, Math.max(0, ((snap.lapDistPct - start) / span) * 100));
}

/** Competitors sorted by overall position, cars without a position last. */
export function sortByPosition(list: CompetitorEntry[]): CompetitorEntry[] {
  return list
    .slice()
    .sort((a, b) => rank(a.position) - rank(b.position));
}

function rank(position: number): number {
  return position > 0 ? position : Number.MAX_SAFE_INTEGER;
}
