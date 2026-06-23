import { Fragment } from "react";
import { formatDelta, formatLapTime } from "../lib/api";
import type { LapSummary } from "../lib/types";

interface Props {
  laps: LapSummary[];
  selectedLaps: number[];
  highlightedLaps?: number[];
  onToggleLap: (lapId: number) => void;
}

export function LapTable({ laps, selectedLaps, highlightedLaps = [], onToggleLap }: Props) {
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
            <th>Delta</th>
            <th>S1</th>
            <th>S2</th>
            <th>S3</th>
            <th>Fuel</th>
            <th>Valid</th>
          </tr>
        </thead>
        <tbody>
          {laps.map((lap, index) => {
            const selected = selectedLaps.includes(lap.id);
            const highlighted = highlightedLaps.includes(lap.lapNumber);
            const showStageHeader =
              index === 0 || lap.sessionNum !== laps[index - 1].sessionNum;
            const s1 = lap.sectors.find((s) => s.sectorNum === 1)?.timeMs;
            const s2 = lap.sectors.find((s) => s.sectorNum === 2)?.timeMs;
            const s3 = lap.sectors.find((s) => s.sectorNum === 3)?.timeMs;
            return (
              <Fragment key={lap.id}>
                {showStageHeader && (
                  <tr className="stage-row">
                    <td colSpan={10}>
                      <strong>{lap.sessionType}</strong>
                    </td>
                  </tr>
                )}
                <tr className={[selected ? "selected" : "", highlighted ? "highlighted" : "", !lap.valid ? "invalid-lap" : ""].filter(Boolean).join(" ")}>
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
                  <td>{formatLapTime(lap.lapTimeMs)}</td>
                  <td className={lap.valid && lap.deltaToBestMs != null && lap.deltaToBestMs > 0 ? "slow" : lap.valid ? "fast" : "muted"}>
                    {lap.valid ? formatDelta(lap.deltaToBestMs) : "—"}
                  </td>
                  <td className={lap.valid ? "" : "muted"}>{lap.valid ? formatLapTime(s1) : "—"}</td>
                  <td className={lap.valid ? "" : "muted"}>{lap.valid ? formatLapTime(s2) : "—"}</td>
                  <td className={lap.valid ? "" : "muted"}>{lap.valid ? formatLapTime(s3) : "—"}</td>
                  <td>{lap.fuelUsed != null ? lap.fuelUsed.toFixed(2) : "—"}</td>
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
