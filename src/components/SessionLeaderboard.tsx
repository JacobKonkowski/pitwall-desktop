import { useMemo, useState } from "react";
import { formatDelta, formatLapTime } from "../lib/api";
import type { CompetitorEntry } from "../lib/types";

interface Props {
  competitors: CompetitorEntry[];
}

type Mode = "overall" | "class";

export function SessionLeaderboard({ competitors }: Props) {
  const [mode, setMode] = useState<Mode>("overall");

  const player = useMemo(() => competitors.find((c) => c.isPlayer), [competitors]);
  const playerBest = player?.bestLapMs ?? null;

  const rows = useMemo(() => {
    if (mode === "class" && player) {
      return competitors
        .filter((c) => c.classId === player.classId)
        .slice()
        .sort((a, b) => rank(a.classPosition) - rank(b.classPosition));
    }
    return competitors.slice().sort((a, b) => rank(a.position) - rank(b.position));
  }, [competitors, mode, player]);

  if (competitors.length === 0) {
    return (
      <div className="panel">
        <h3>Leaderboard</h3>
        <p className="muted">Waiting for competitor data…</p>
      </div>
    );
  }

  const showClassToggle = player != null;

  return (
    <div className="panel session-leaderboard">
      <div className="leaderboard-header">
        <h3>Leaderboard</h3>
        {showClassToggle && (
          <div className="btn-row">
            <button
              className={mode === "overall" ? "tab active" : "tab"}
              onClick={() => setMode("overall")}
            >
              Overall
            </button>
            <button
              className={mode === "class" ? "tab active" : "tab"}
              onClick={() => setMode("class")}
            >
              Class
            </button>
          </div>
        )}
      </div>
      <table className="leaderboard-table">
        <thead>
          <tr>
            <th>P</th>
            <th>#</th>
            <th>Driver</th>
            <th>Best</th>
            <th>Last</th>
            <th>Δ you</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((c) => {
            const pos = mode === "class" ? c.classPosition : c.position;
            const deltaYou =
              !c.isPlayer && c.bestLapMs != null && playerBest != null
                ? c.bestLapMs - playerBest
                : null;
            return (
              <tr key={c.carIdx} className={c.isPlayer ? "leaderboard-you" : undefined}>
                <td>{pos > 0 ? pos : "—"}</td>
                <td>
                  <span
                    className="class-chip"
                    style={c.classColor ? { background: `#${c.classColor}` } : undefined}
                  >
                    {c.carNumber || c.carIdx}
                  </span>
                </td>
                <td className="leaderboard-driver">
                  {c.driverName}
                  {c.onPitRoad && <span className="pit-tag"> PIT</span>}
                </td>
                <td>{formatLapTime(c.bestLapMs)}</td>
                <td>{formatLapTime(c.lastLapMs)}</td>
                <td className={deltaYou != null ? (deltaYou > 0 ? "slow" : "fast") : undefined}>
                  {deltaYou != null ? formatDelta(deltaYou) : "—"}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

/** Sort helper: cars without a valid position (0) go to the bottom. */
function rank(position: number): number {
  return position > 0 ? position : Number.MAX_SAFE_INTEGER;
}
