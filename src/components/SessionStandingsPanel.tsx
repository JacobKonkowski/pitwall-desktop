import { useEffect, useMemo, useState } from "react";
import { formatDelta, formatLapTime, getSessionStandings } from "../lib/api";
import type { CompetitorStanding, SessionStandings } from "../lib/types";

interface Props {
  sessionId: number;
}

type Mode = "overall" | "class";

export function SessionStandingsPanel({ sessionId }: Props) {
  const [standings, setStandings] = useState<SessionStandings | null>(null);
  const [mode, setMode] = useState<Mode>("overall");

  useEffect(() => {
    let cancelled = false;
    getSessionStandings(sessionId)
      .then((s) => {
        if (!cancelled) setStandings(s);
      })
      .catch(() => {
        if (!cancelled) setStandings(null);
      });
    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  const player = useMemo(
    () => standings?.competitors.find((c) => c.isPlayer) ?? null,
    [standings],
  );
  const playerBest = player?.bestLapMs ?? null;

  const rows = useMemo(() => {
    if (!standings) return [];
    if (mode === "class" && player) {
      return standings.competitors
        .filter((c) => c.classId === player.classId)
        .slice()
        .sort((a, b) => rank(a.classPosition) - rank(b.classPosition));
    }
    return standings.competitors.slice().sort((a, b) => rank(a.position) - rank(b.position));
  }, [standings, mode, player]);

  if (!standings || standings.competitors.length === 0) {
    return (
      <div className="panel session-standings empty-state-inline">
        <h2>Session standings</h2>
        <p className="muted">
          No field data was captured for this session. Standings appear when live telemetry was
          running during the session and linked on import.
        </p>
      </div>
    );
  }

  return (
    <div className="panel session-standings">
      <div className="leaderboard-header">
        <h2>Session standings</h2>
        {player && (
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
      <p className="muted small">
        Captured from the live session
        {standings.sessionFastestMs != null &&
          ` · fastest lap ${formatLapTime(standings.sessionFastestMs)}`}
      </p>
      <table className="leaderboard-table">
        <thead>
          <tr>
            <th>P</th>
            <th>#</th>
            <th>Driver</th>
            <th>Best</th>
            <th>Δ you</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((c: CompetitorStanding) => {
            const pos = mode === "class" ? c.classPosition : c.position;
            const deltaYou =
              !c.isPlayer && c.bestLapMs != null && playerBest != null
                ? c.bestLapMs - playerBest
                : null;
            return (
              <tr key={`${c.position}-${c.carNumber}-${c.driverName}`} className={c.isPlayer ? "leaderboard-you" : undefined}>
                <td>{pos > 0 ? pos : "—"}</td>
                <td>
                  <span
                    className="class-chip"
                    style={c.classColor ? { background: `#${c.classColor}` } : undefined}
                  >
                    {c.carNumber || "—"}
                  </span>
                </td>
                <td className="leaderboard-driver">{c.driverName}</td>
                <td>{formatLapTime(c.bestLapMs)}</td>
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

function rank(position: number): number {
  return position > 0 ? position : Number.MAX_SAFE_INTEGER;
}
