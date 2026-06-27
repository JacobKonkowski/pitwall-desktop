import type { LiveSnapshot } from "../lib/types";

interface Props {
  snap: LiveSnapshot;
}

/** Top-down proximity dots for cars within +/-3s; ahead toward the top. */
export function RadarWidget({ snap }: Props) {
  const cars = snap.competitors.filter(
    (c) => !c.isPlayer && c.gapToPlayerS != null && Math.abs(c.gapToPlayerS) <= 3,
  );
  return (
    <div className="pw-radar">
      <div className="pw-radar-ring" />
      <div className="pw-radar-me" />
      {cars.map((c) => {
        const gap = c.gapToPlayerS ?? 0;
        const top = 50 - (gap / 3) * 42;
        const lateral =
          Math.abs(gap) <= 1.5 ? (c.lapDistPct - snap.lapDistPct) * 80 : 0;
        const left = 50 + lateral;
        return (
          <div
            key={c.carIdx}
            className={`pw-radar-car${c.onPitRoad ? " pit" : ""}`}
            style={{ top: `${top}%`, left: `${left}%` }}
            title={`#${c.carNumber || c.carIdx} ${c.driverName}`}
          />
        );
      })}
    </div>
  );
}
