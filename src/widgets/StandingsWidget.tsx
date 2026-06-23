import { formatLapTime } from "../lib/api";
import type { LiveSnapshot } from "../lib/types";
import { sortByPosition } from "./format";

interface Props {
  snap: LiveSnapshot;
}

export function StandingsWidget({ snap }: Props) {
  const rows = sortByPosition(snap.competitors).slice(0, 12);
  return (
    <div className="pw-board">
      <h1>Standings</h1>
      {rows.length === 0 ? (
        <p className="pw-board-empty">No competitor data</p>
      ) : (
        rows.map((c) => (
          <div key={c.carIdx} className={`pw-board-row standings${c.isPlayer ? " you" : ""}`}>
            <span className="pw-board-pos">{c.position > 0 ? c.position : "\u2014"}</span>
            <span
              className="pw-board-num"
              style={c.classColor ? { background: `#${c.classColor}` } : undefined}
            >
              {c.carNumber || c.carIdx}
            </span>
            <span className="pw-board-name">
              {c.driverName}
              {c.onPitRoad && <span className="warn"> PIT</span>}
            </span>
            <span className="pw-board-lap">{formatLapTime(c.bestLapMs)}</span>
          </div>
        ))
      )}
    </div>
  );
}
