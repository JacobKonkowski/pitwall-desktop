import { useMemo, useState } from "react";
import { formatDelta, formatLapTime } from "../../shared/format";
import type { CompetitorEntry } from "../../shared/types";

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
        <div className="panel-header">
          <h2>Leaderboard</h2>
        </div>
        <div className="panel-body">
          <p className="muted">Waiting for competitor data…</p>
        </div>
      </div>
    );
  }

  const showClassToggle = player != null;

  return (
    <div className="panel session-leaderboard">
      <div className="panel-header leaderboard-header">
        <h2>Leaderboard</h2>
        {showClassToggle && (
          <div className="btn-row" style={{ marginLeft: "auto" }}>
            <button
              type="button"
              className={mode === "overall" ? "tab active" : "tab"}
              onClick={() => setMode("overall")}
            >
              Overall
            </button>
            <button
              type="button"
              className={mode === "class" ? "tab active" : "tab"}
              onClick={() => setMode("class")}
            >
              Class
            </button>
          </div>
        )}
      </div>
      <div className="panel-body" style={{ padding: 0 }}>
        <table className="data-table leaderboard-table">
          <thead>
            <tr>
              <th>P</th>
              <th>#</th>
              <th>Driver</th>
              <th className="num">Best</th>
              <th className="num">Last</th>
              <th className="num">Δ you</th>
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
                  <td>
                    {c.driverName}
                    {c.onPitRoad && <span className="pill pit"> PIT</span>}
                  </td>
                  <td className="num">{formatLapTime(c.bestLapMs)}</td>
                  <td className="num">{formatLapTime(c.lastLapMs)}</td>
                  <td className={`num ${deltaYou != null ? (deltaYou > 0 ? "slow" : "fast") : ""}`}>
                    {deltaYou != null ? formatDelta(deltaYou) : "—"}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function rank(position: number): number {
  return position > 0 ? position : Number.MAX_SAFE_INTEGER;
}
