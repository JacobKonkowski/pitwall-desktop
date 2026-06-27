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



export function LapTable({ laps, selectedLaps, highlightedLaps = [], onToggleLap }: Props) {

  const sectorCols = useMemo(() => {

    const max = maxSectorCount(laps);

    return max > 0 ? Array.from({ length: max }, (_, i) => i + 1) : [];

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

                  <td className={nonFlying ? "muted" : ""}>{kindLabel}</td>

                  <td className={lap.valid && lap.deltaToBestMs != null && lap.deltaToBestMs > 0 ? "slow" : lap.valid ? "fast" : "muted"}>

                    {lap.valid ? formatDelta(lap.deltaToBestMs) : "—"}

                  </td>

                  {sectorCols.map((n) => {

                    const time = lap.sectors.find((s) => s.sectorNum === n)?.timeMs;

                    return (

                      <td key={n} className={showSectors ? "" : "muted"}>

                        {showSectors ? formatLapTime(time ?? null) : "—"}

                      </td>

                    );

                  })}

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


