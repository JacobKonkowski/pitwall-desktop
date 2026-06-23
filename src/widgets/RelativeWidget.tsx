import type { LiveSnapshot } from "../lib/types";
import { positionLabel } from "./format";

interface Props {
  snap: LiveSnapshot;
}

/** Cars within +/-8s of the player, ahead listed above, behind below. */
export function RelativeWidget({ snap }: Props) {
  const near = snap.competitors
    .filter((c) => !c.isPlayer && c.gapToPlayerS != null && Math.abs(c.gapToPlayerS) <= 8)
    .sort((a, b) => (b.gapToPlayerS ?? 0) - (a.gapToPlayerS ?? 0));
  const ahead = near.filter((c) => (c.gapToPlayerS ?? 0) >= 0);
  const behind = near.filter((c) => (c.gapToPlayerS ?? 0) < 0);

  const row = (c: (typeof near)[number]) => {
    const gap = c.gapToPlayerS ?? 0;
    return (
      <div key={c.carIdx} className="pw-board-row relative">
        <span
          className="pw-board-num"
          style={c.classColor ? { background: `#${c.classColor}` } : undefined}
        >
          {c.carNumber || c.carIdx}
        </span>
        <span className="pw-board-name">{c.driverName}</span>
        <span className={gap >= 0 ? "fast" : "slow"}>
          {gap >= 0 ? "+" : ""}
          {gap.toFixed(1)}s
        </span>
      </div>
    );
  };

  return (
    <div className="pw-board">
      <h1>Relative</h1>
      {ahead.map(row)}
      <div className="pw-board-row relative you">
        <span className="pw-board-num">YOU</span>
        <span className="pw-board-name">{positionLabel(snap)}</span>
        <span />
      </div>
      {behind.map(row)}
    </div>
  );
}
