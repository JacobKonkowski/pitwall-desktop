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
        // +/-3s maps across the dish; ahead (positive) sits toward the top.
        const top = 50 - (gap / 3) * 42;
        return (
          <div
            key={c.carIdx}
            className="pw-radar-car"
            style={{ top: `${top}%` }}
            title={`#${c.carNumber || c.carIdx} ${c.driverName}`}
          />
        );
      })}
    </div>
  );
}
