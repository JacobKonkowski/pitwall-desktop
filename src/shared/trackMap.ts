/** Track outline geometry helpers, mirroring `race_refinery_analysis::track_map`. */
import type { OutlinePoint, TrackOutline, TrailSample } from "./types";

export interface MapPoint {
  x: number;
  y: number;
}

/** Which pedal dominates a sample, used to color a lap path. */
export type PedalTone = "brake" | "throttle" | "coast";

/** A run of consecutive samples sharing one tone. */
export interface PedalSegment {
  tone: PedalTone;
  points: MapPoint[];
}

/** Below this, a pedal is treated as released. */
const PEDAL_DEADZONE = 0.05;

/**
 * Position on an outline at a lap fraction, wrapping across the finish line.
 *
 * Points are sorted by `pct`; the gap between the last and first point is the
 * closing segment, so a car at 0.99 interpolates toward the start line.
 */
export function pointAt(points: OutlinePoint[], pct: number): MapPoint | null {
  if (points.length === 0) return null;
  if (points.length === 1) return { x: points[0].x, y: points[0].y };

  const t = ((pct % 1) + 1) % 1;
  const first = points[0];
  const last = points[points.length - 1];

  if (t <= first.pct || t >= last.pct) {
    const span = 1 - last.pct + first.pct;
    if (span <= 0) return { x: first.x, y: first.y };
    const travelled = t >= last.pct ? t - last.pct : 1 - last.pct + t;
    return lerp(last, first, travelled / span);
  }

  let hi = points.findIndex((p) => p.pct > t);
  if (hi < 1) hi = points.length - 1;
  const a = points[hi - 1];
  const b = points[hi];
  const span = b.pct - a.pct;
  return lerp(a, b, span > 0 ? (t - a.pct) / span : 0);
}

function lerp(a: OutlinePoint, b: OutlinePoint, u: number): MapPoint {
  return { x: a.x + (b.x - a.x) * u, y: a.y + (b.y - a.y) * u };
}

/**
 * Project a GPS sample into the outline's unit box.
 *
 * `null` for outlines cached before the projection was stored — those sessions
 * need a re-import before their racing line can be drawn.
 */
export function projectSample(
  outline: TrackOutline,
  lat: number,
  lon: number,
): MapPoint | null {
  const p = outline.projection;
  if (!p) return null;
  const lonScale = 111_320 * Math.cos((p.originLat * Math.PI) / 180);
  const xM = (lon - p.originLon) * lonScale;
  const yM = (p.originLat - lat) * 111_320;
  return {
    x: p.offsetX + (xM - p.minX) * p.scale,
    y: p.offsetY + (yM - p.minY) * p.scale,
  };
}

/** Whether these samples carry GPS the outline can place as a true racing line. */
export function hasRacingLine(
  outline: TrackOutline | null,
  samples: TrailSample[] | null | undefined,
): boolean {
  if (!outline?.projection || !samples) return false;
  return samples.some((s) => s.lat != null && s.lon != null);
}

/**
 * Screen path for a lap's samples: the true racing line when GPS is present,
 * otherwise the samples placed along the shared circuit by lap distance.
 */
export function tracePath(outline: TrackOutline, samples: TrailSample[]): MapPoint[] {
  const gps = hasRacingLine(outline, samples);
  const path: MapPoint[] = [];
  for (const s of samples) {
    const at =
      gps && s.lat != null && s.lon != null
        ? projectSample(outline, s.lat, s.lon)
        : pointAt(outline.points, s.distPct);
    if (at) path.push(at);
  }
  return path;
}

type PedalSample = Pick<TrailSample, "throttle" | "brake" | "throttleRaw" | "brakeRaw">;

/** Throttle the driver asked for: raw when stored, else applied. */
export function driverThrottle(p: PedalSample): number {
  return p.throttleRaw ?? p.throttle;
}

/** Brake the driver asked for: raw when stored, else applied. */
export function driverBrake(p: PedalSample): number {
  return p.brakeRaw ?? p.brake;
}

/** Which pedal dominates: brake wins ties so braking zones stay visible. */
export function pedalTone(throttle: number, brake: number): PedalTone {
  if (brake > PEDAL_DEADZONE && brake >= throttle) return "brake";
  if (throttle > PEDAL_DEADZONE) return "throttle";
  return "coast";
}

/**
 * Split a lap into runs of constant pedal tone, ready to stroke as separate
 * paths. Each run repeats its predecessor's last point so the strokes join.
 */
export function pedalSegments(
  outline: TrackOutline,
  samples: TrailSample[],
): PedalSegment[] {
  const path = tracePath(outline, samples);
  if (path.length < 2) return [];

  const segments: PedalSegment[] = [];
  for (let i = 0; i < path.length; i++) {
    // Driver intent, so downshift blips don't paint braking zones green.
    const tone = pedalTone(driverThrottle(samples[i]), driverBrake(samples[i]));
    const current = segments[segments.length - 1];
    if (current && current.tone === tone) {
      current.points.push(path[i]);
      continue;
    }
    // Start the new run at the previous point so there is no visual gap.
    const bridge = current ? [path[i - 1]] : [];
    segments.push({ tone, points: [...bridge, path[i]] });
  }
  return segments.filter((s) => s.points.length >= 2);
}

/** Open SVG path over a `0 0 1 1` viewBox. */
export function toPathD(points: MapPoint[]): string {
  return points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(4)},${p.y.toFixed(4)}`)
    .join(" ");
}
