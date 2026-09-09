/** Formatting helpers shared across the Analyze feature. */

/** `1:23.456` (or `23.456` under a minute). Non-positive / null -> em dash. */
export function formatLapTime(ms: number | null | undefined): string {
  if (ms == null || ms <= 0) return "—";
  const totalSec = ms / 1000;
  const min = Math.floor(totalSec / 60);
  const sec = totalSec - min * 60;
  return min > 0 ? `${min}:${sec.toFixed(3).padStart(6, "0")}` : sec.toFixed(3);
}

/** Signed seconds delta, e.g. `+0.184` / `-0.052`. */
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

/** m/s -> km/h, rounded. */
export function formatSpeedKph(ms: number | null | undefined): string {
  if (ms == null) return "—";
  return `${Math.round(ms * 3.6)}`;
}

export function formatPct(fraction: number | null | undefined): string {
  if (fraction == null) return "—";
  return `${(fraction * 100).toFixed(1)}%`;
}

export function formatLiters(l: number | null | undefined): string {
  if (l == null) return "—";
  return `${l.toFixed(2)} L`;
}

export function formatTemp(c: number | null | undefined): string {
  if (c == null) return "—";
  return `${Math.round(c)}°`;
}

/** Sign of a delta for coloring: "fast" (negative), "slow" (positive), or "". */
export function deltaClass(ms: number | null | undefined): "fast" | "slow" | "" {
  if (ms == null || Math.abs(ms) < 1) return "";
  return ms < 0 ? "fast" : "slow";
}
