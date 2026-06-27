import { formatDelta, formatLapTime } from "../../lib/api";
import type { LiveSnapshot } from "../../lib/types";

interface Props {
  snap: LiveSnapshot;
}

function formatPosition(snap: LiveSnapshot): string {
  const cls = snap.playerClassPosition;
  const overall = snap.playerPosition;
  if (cls != null && overall != null && cls !== overall) return `P${cls} (P${overall})`;
  return `P${cls ?? overall ?? "—"}`;
}

export function LiveMetricsGrid({ snap }: Props) {
  return (
    <div className="live-metrics">
      <div className="live-metric-card">
        <span className="label">Lap</span>
        <span className="value">{snap.lap}</span>
      </div>
      <div className="live-metric-card">
        <span className="label">Lap time</span>
        <span className="value">{formatLapTime(snap.lapTimeMs)}</span>
      </div>
      <div className="live-metric-card">
        <span className="label">Δ best</span>
        <span className={`value ${snap.deltaToBestMs != null && snap.deltaToBestMs > 0 ? "slow" : "fast"}`}>
          {formatDelta(snap.deltaToBestMs)}
        </span>
      </div>
      {(snap.playerPosition != null || snap.playerClassPosition != null) && (
        <div className="live-metric-card">
          <span className="label">Position</span>
          <span className="value">{formatPosition(snap)}</span>
        </div>
      )}
      {snap.deltaToSessionBestMs != null && (
        <div className="live-metric-card">
          <span className="label">Δ session best</span>
          <span className={`value ${snap.deltaToSessionBestMs > 0 ? "slow" : "fast"}`}>
            {formatDelta(snap.deltaToSessionBestMs)}
          </span>
        </div>
      )}
      {snap.deltaToSessionOptimalMs != null && (
        <div className="live-metric-card">
          <span className="label">Δ session optimal</span>
          <span className={`value ${snap.deltaToSessionOptimalMs > 0 ? "slow" : "fast"}`}>
            {formatDelta(snap.deltaToSessionOptimalMs)}
          </span>
        </div>
      )}
      <div className="live-metric-card">
        <span className="label">Last lap</span>
        <span className={`value ${snap.lastLapValid ? "" : "muted"}`}>
          {formatLapTime(snap.lastLapMs)}
        </span>
      </div>
      {snap.deltaToLastMs != null && (
        <div className="live-metric-card">
          <span className="label">Δ last</span>
          <span className={`value ${snap.deltaToLastMs > 0 ? "slow" : "fast"}`}>
            {formatDelta(snap.deltaToLastMs)}
          </span>
        </div>
      )}
      <div className="live-metric-card">
        <span className="label">Fuel</span>
        <span className="value">{snap.fuelLevel.toFixed(1)} L</span>
      </div>
      <div className="live-metric-card">
        <span className="label">Speed</span>
        <span className="value">{snap.speed.toFixed(0)}</span>
      </div>
    </div>
  );
}

export function LiveRaceContext({ snap }: Props) {
  const formatRemain = (sec: number | null) => {
    if (sec == null) return "—";
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return m > 0 ? `${m}:${s.toString().padStart(2, "0")}` : `${s}s`;
  };

  return (
    <div className="live-race-context">
      {snap.sessionLapsRemain != null && (
        <span className="live-badge">Laps left: {snap.sessionLapsRemain}</span>
      )}
      {snap.sessionTimeRemainS != null && (
        <span className="live-badge">Time left: {formatRemain(snap.sessionTimeRemainS)}</span>
      )}
      {snap.pitsOpen && <span className="live-badge live-badge-ok">Pits open</span>}
      {snap.onPitRoad && <span className="live-badge live-badge-warn">Pit road</span>}
      {!snap.onTrack && !snap.onPitRoad && <span className="live-badge">Off track</span>}
    </div>
  );
}
