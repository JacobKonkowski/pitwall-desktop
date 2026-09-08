import { Fragment, useMemo } from "react";
import { formatDelta, formatLapTime } from "../lib/api";
import type { LapKind, LapSummary } from "../lib/types";

interface Props {
  laps: LapSummary[];
  selectedLaps: number[];
  highlightedLaps?: number[];
  onToggleLap: (lapId: number) => void;
}

function formatLapKind(kind: LapKind): string {
  switch (kind) {
    case "flying":
      return "Fly";
    case "pitOut":
      return "Out";
    case "pitIn":
      return "In";
    case "pitLane":
      return "Pit";
    case "partial":
      return "Part";
  }
}

function maxSectorCount(laps: LapSummary[]): number {
  return laps.reduce(
    (max, lap) => Math.max(max, ...lap.sectors.map((s) => s.sectorNum), 0),
    0,
  );
}

function buildColorScale(
  laps: LapSummary[],
  getValue: (l: LapSummary) => number | null,
): Map<number, string> {
  const entries = laps
    .filter((l) => l.valid && l.lapKind === "flying")
    .flatMap((l) => { const v = getValue(l); return v != null ? [{ id: l.id, v }] : []; });
  if (entries.length === 0) return new Map();

  const sorted = entries.map((e) => e.v).sort((a, b) => a - b);
  const lo = sorted[Math.floor((sorted.length - 1) * 0.1)];
  const hi = sorted[Math.floor((sorted.length - 1) * 0.9)];
  const range = hi - lo;

  return new Map(
    entries.map((e) => {
      const t = range === 0 ? 0 : Math.max(0, Math.min(1, (e.v - lo) / range));
      return [e.id, `hsl(${Math.round(120 - t * 120)}, 65%, 60%)`];
    }),
  );
}

export function LapTable({ laps, selectedLaps, highlightedLaps = [], onToggleLap }: Props) {
  const sectorCols = useMemo(() => {
    const max = maxSectorCount(laps);
    return max > 0 ? Array.from({ length: max }, (_, i) => i + 1) : [];
  }, [laps]);

  const fastestLapId = useMemo(() => {
    const flying = laps.filter((l) => l.valid && l.lapKind === "flying" && l.lapTimeMs != null);
    if (flying.length === 0) return null;
    return flying.reduce((best, l) => (l.lapTimeMs! < best.lapTimeMs! ? l : best)).id;
  }, [laps]);

  const lapTimeColors = useMemo(() => buildColorScale(laps, (l) => l.lapTimeMs), [laps]);
  const fuelColors = useMemo(() => buildColorScale(laps, (l) => l.fuelUsed), [laps]);

  const fastestSectorTimes = useMemo(() => {
    const best = new Map<number, number>();
    for (const lap of laps) {
      if (!lap.valid || lap.lapKind !== "flying") continue;
      for (const s of lap.sectors) {
        if (s.timeMs == null) continue;
        const current = best.get(s.sectorNum);
        if (current == null || s.timeMs < current) best.set(s.sectorNum, s.timeMs);
      }
    }
    return best;
  }, [laps]);

  const colSpan = 8 + sectorCols.length;

  return (
    <div className="panel lap-table">
      <div className="panel-header">
        <h2>Laps</h2>
        <span className="muted">Select up to 2 laps to compare traces</span>
      </div>
      <table className="data-table compact">
        <thead>
          <tr>
            <th></th>
            <th>Stage</th>
            <th>#</th>
            <th>Time</th>
            <th>Type</th>
            <th>Delta</th>
            {sectorCols.map((n) => (
              <th key={n}>S{n}</th>
            ))}
            <th>Fuel</th>
            <th>Valid</th>
          </tr>
        </thead>
        <tbody>
          {laps.map((lap, index) => {
            const selected = selectedLaps.includes(lap.id);
            const highlighted = highlightedLaps.includes(lap.lapNumber);
            const isFastest = lap.id === fastestLapId;
            const showStageHeader =
              index === 0 || lap.sessionNum !== laps[index - 1].sessionNum;
            const kindLabel = formatLapKind(lap.lapKind);
            const nonFlying = lap.lapKind !== "flying";
            const showSectors = lap.valid && lap.lapKind === "flying";
            return (
              <Fragment key={lap.id}>
                {showStageHeader && (
                  <tr className="stage-row">
                    <td colSpan={colSpan}>
                      <strong>{lap.sessionType}</strong>
                    </td>
                  </tr>
                )}
                <tr className={[selected ? "selected" : "", highlighted ? "highlighted" : "", isFastest ? "fastest-lap" : "", !lap.valid ? "invalid-lap" : ""].filter(Boolean).join(" ")}>
                  <td>
                    <input
                      type="checkbox"
                      checked={selected}
                      onChange={() => onToggleLap(lap.id)}
                      disabled={!lap.valid}
                    />
                  </td>
                  <td className="muted stage-label">{lap.sessionType}</td>
                  <td>{lap.lapNumber}</td>
                  <td style={lapTimeColors.has(lap.id) ? { color: lapTimeColors.get(lap.id) } : undefined}>{formatLapTime(lap.lapTimeMs)}</td>
                  <td className={nonFlying ? "muted" : ""}>{kindLabel}</td>
                  <td className={lap.valid && lap.deltaToBestMs != null && lap.deltaToBestMs > 0 ? "slow" : lap.valid ? "fast" : "muted"}>
                    {lap.valid ? formatDelta(lap.deltaToBestMs) : "—"}
                  </td>
                  {sectorCols.map((n) => {
                    const time = lap.sectors.find((s) => s.sectorNum === n)?.timeMs;
                    const isFastestSector =
                      showSectors && time != null && time === fastestSectorTimes.get(n);
                    return (
                      <td
                        key={n}
                        className={showSectors ? "" : "muted"}
                        style={isFastestSector ? { color: "#b388ff" } : undefined}
                      >
                        {showSectors ? formatLapTime(time ?? null) : "—"}
                      </td>
                    );
                  })}
                  <td style={fuelColors.has(lap.id) ? { color: fuelColors.get(lap.id) } : undefined}>{lap.fuelUsed != null ? lap.fuelUsed.toFixed(2) : "—"}</td>
                  <td>{lap.valid ? "✓" : "—"}</td>
                </tr>
              </Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}