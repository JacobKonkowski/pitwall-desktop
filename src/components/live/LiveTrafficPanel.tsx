import { formatLapTime } from "../../lib/api";
import type { LiveSnapshot } from "../../lib/types";
import { fmtGap, packLabel, sectorProgress } from "../../widgets/format";

interface Props {
  snap: LiveSnapshot;
}

function flagLabel(flags: number): string | null {
  if (flags === 0) return null;
  if (flags & 0x00000002) return "Yellow";
  if (flags & 0x00000004) return "Red";
  if (flags & 0x00000008) return "Blue";
  if (flags & 0x00000001) return "Checkered";
  if (flags & 0x00000010) return "Green";
  return "Flag";
}

export function LiveTrafficPanel({ snap }: Props) {
  const pack = packLabel(snap.packState);
  const flag = flagLabel(snap.sessionFlags);

  return (
    <div className="panel live-traffic">
      <h3>Traffic</h3>
      <div className="live-traffic-gaps">
        <div className="live-metric-card">
          <span className="label">Gap ahead</span>
          <span className="value">{fmtGap(snap.gapToCarAheadS)}</span>
        </div>
        <div className="live-metric-card">
          <span className="label">Gap behind</span>
          <span className="value">{fmtGap(snap.gapToCarBehindS)}</span>
        </div>
      </div>
      {pack && (
        <p className={`live-pack-pill ${snap.packState === "clear" ? "fast" : "warn"}`}>{pack}</p>
      )}
      <div className="live-traffic-badges">
        {flag && <span className="live-badge live-badge-warn">{flag}</span>}
        {snap.incidentCount > 0 && (
          <span className="live-badge live-badge-warn">Incidents: {snap.incidentCount}</span>
        )}
      </div>
    </div>
  );
}

export function LiveSectorsPanel({ snap }: Props) {
  return (
    <div className="panel">
      <h3>Sector progress</h3>
      <div className="sector-bars">
        {snap.sectors.map((sector) => {
          const pct = sector.completed ? 100 : sectorProgress(snap, sector.sectorNum);
          return (
            <div key={sector.sectorNum} className="sector-bar-row">
              <span className="sector-label">S{sector.sectorNum}</span>
              <div className="sector-bar-track">
                <div className="sector-bar-fill" style={{ width: `${pct}%` }} />
              </div>
              <span className="sector-time">{formatLapTime(sector.timeMs ?? null)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function LiveTiresPanel({ snap }: Props) {
  return (
    <div className="panel">
      <h3>Tire temps (°C)</h3>
      <div className="tire-temps">
        <span>LF {snap.lfTemp.toFixed(0)}</span>
        <span>RF {snap.rfTemp.toFixed(0)}</span>
        <span>LR {snap.lrTemp.toFixed(0)}</span>
        <span>RR {snap.rrTemp.toFixed(0)}</span>
      </div>
    </div>
  );
}
