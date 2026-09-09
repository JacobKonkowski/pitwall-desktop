/**
 * Client-side session stats derived from lap summaries (no backend coach).
 */
import type { LapSummary } from "../../shared/types";

export interface SessionStats {
  theoreticalBestMs: number | null;
  consistencyMs: number | null;
  weakSector: number | null;
  weakSectorLossMs: number | null;
  fuelOutlierLap: number | null;
  fuelOutlierUsed: number | null;
  paceLapCount: number;
}

function groupBySubsession(laps: LapSummary[]): Map<number, LapSummary[]> {
  const map = new Map<number, LapSummary[]>();
  for (const lap of laps) {
    const list = map.get(lap.sessionNum) ?? [];
    list.push(lap);
    map.set(lap.sessionNum, list);
  }
  return map;
}

function theoreticalForGroup(laps: LapSummary[]): number | null {
  const pace = laps.filter((l) => l.paceEligible);
  if (pace.length === 0) return null;
  const bestBySector = new Map<number, number>();
  for (const lap of pace) {
    for (const s of lap.sectors) {
      const prev = bestBySector.get(s.sectorNum);
      if (prev == null || s.timeMs < prev) bestBySector.set(s.sectorNum, s.timeMs);
    }
  }
  if (bestBySector.size === 0) return null;
  let sum = 0;
  for (const t of bestBySector.values()) sum += t;
  return sum;
}

function stdev(values: number[]): number | null {
  if (values.length < 2) return null;
  const mean = values.reduce((a, b) => a + b, 0) / values.length;
  const variance = values.reduce((a, v) => a + (v - mean) ** 2, 0) / values.length;
  return Math.sqrt(variance);
}

function weakSectorForGroup(laps: LapSummary[]): { sector: number; lossMs: number } | null {
  const pace = laps.filter((l) => l.paceEligible);
  if (pace.length < 2) return null;
  const bestBySector = new Map<number, number>();
  const sums = new Map<number, { total: number; n: number }>();
  for (const lap of pace) {
    for (const s of lap.sectors) {
      const prev = bestBySector.get(s.sectorNum);
      if (prev == null || s.timeMs < prev) bestBySector.set(s.sectorNum, s.timeMs);
      const agg = sums.get(s.sectorNum) ?? { total: 0, n: 0 };
      agg.total += s.timeMs;
      agg.n += 1;
      sums.set(s.sectorNum, agg);
    }
  }
  let worst: { sector: number; lossMs: number } | null = null;
  for (const [sector, agg] of sums) {
    const best = bestBySector.get(sector);
    if (best == null || agg.n === 0) continue;
    const lossMs = agg.total / agg.n - best;
    if (!worst || lossMs > worst.lossMs) worst = { sector, lossMs };
  }
  return worst && worst.lossMs > 50 ? worst : null;
}

function fuelOutlier(laps: LapSummary[]): { lapNumber: number; fuelUsed: number } | null {
  const samples = laps
    .filter((l) => l.paceEligible && l.fuelUsed != null && l.fuelUsed > 0)
    .map((l) => ({ lapNumber: l.lapNumber, fuelUsed: l.fuelUsed! }));
  if (samples.length < 3) return null;
  const sorted = samples.map((s) => s.fuelUsed).sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  const median =
    sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
  if (median <= 0) return null;
  let worst: { lapNumber: number; fuelUsed: number; delta: number } | null = null;
  for (const s of samples) {
    const delta = Math.abs(s.fuelUsed - median);
    if (delta / median < 0.12) continue;
    if (!worst || delta > worst.delta) worst = { ...s, delta };
  }
  return worst ? { lapNumber: worst.lapNumber, fuelUsed: worst.fuelUsed } : null;
}

/** Derive theoretical best, consistency, weak sector, and fuel outlier from laps. */
export function computeSessionStats(laps: LapSummary[]): SessionStats {
  const groups = groupBySubsession(laps);
  let theoreticalBestMs: number | null = null;
  let weak: { sector: number; lossMs: number } | null = null;

  for (const group of groups.values()) {
    const theo = theoreticalForGroup(group);
    if (theo != null && (theoreticalBestMs == null || theo < theoreticalBestMs)) {
      theoreticalBestMs = theo;
    }
    const w = weakSectorForGroup(group);
    if (w && (!weak || w.lossMs > weak.lossMs)) weak = w;
  }

  const paceTimes = laps
    .filter((l) => l.paceEligible && l.lapTimeMs != null)
    .map((l) => l.lapTimeMs!);
  const consistencyMs = stdev(paceTimes);
  const fuel = fuelOutlier(laps);

  return {
    theoreticalBestMs,
    consistencyMs,
    weakSector: weak?.sector ?? null,
    weakSectorLossMs: weak?.lossMs ?? null,
    fuelOutlierLap: fuel?.lapNumber ?? null,
    fuelOutlierUsed: fuel?.fuelUsed ?? null,
    paceLapCount: paceTimes.length,
  };
}

/** Basename of an IBT path, truncated for the header. */
export function truncateIbtName(path: string, max = 36): string {
  const base = path.replace(/^.*[/\\]/, "") || path;
  if (base.length <= max) return base;
  const keep = Math.max(8, max - 1);
  return `${base.slice(0, keep)}…`;
}
