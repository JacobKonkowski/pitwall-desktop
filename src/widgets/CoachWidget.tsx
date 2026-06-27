import { formatLapTime } from "../lib/api";
import type { LiveSnapshot } from "../lib/types";
import { deltaClass, fmtDelta, fmtGap, packLabel, positionLabel, sectorProgress } from "./format";

interface Props {
  snap: LiveSnapshot;
  fieldPaceMode: string;
}

function FieldPace({ snap, mode }: { snap: LiveSnapshot; mode: string }) {
  const best = snap.deltaToSessionBestMs;
  const optimal = snap.deltaToSessionOptimalMs;
  if (mode === "optimal") {
    return optimal != null ? (
      <span className={deltaClass(optimal)}>OPT {fmtDelta(optimal)}</span>
    ) : null;
  }
  if (mode === "both") {
    return (
      <>
        {best != null && <span className={deltaClass(best)}>FLD {fmtDelta(best)}</span>}
        {optimal != null && <span className={deltaClass(optimal)}>OPT {fmtDelta(optimal)}</span>}
      </>
    );
  }
  return best != null ? <span className={deltaClass(best)}>FLD {fmtDelta(best)}</span> : null;
}

export function CoachWidget({ snap, fieldPaceMode }: Props) {
  const pack = packLabel(snap.packState);
  const pos = positionLabel(snap);
  return (
    <div className="pw-coach">
      <div className="pw-coach-top">
        {pos && <span className="pw-coach-pos">{pos}</span>}
        {snap.sessionFlags !== 0 && <span className="pw-badge warn">FLAG</span>}
      </div>
      <div className="pw-coach-hero">
        <div className="pw-coach-lapnum">LAP {snap.lap || 0}</div>
        <div className="pw-coach-laptime">{formatLapTime(snap.lapTimeMs)}</div>
      </div>
      <div className="pw-coach-gaps">
        <div className="pw-gap">
          <span className="pw-gap-lbl">AHEAD</span>
          <span className="pw-gap-val">{fmtGap(snap.gapToCarAheadS)}</span>
        </div>
        <div className="pw-gap">
          <span className="pw-gap-lbl">BEHIND</span>
          <span className="pw-gap-val">{fmtGap(snap.gapToCarBehindS)}</span>
        </div>
      </div>
      <div className="pw-coach-deltas">
        <span className={deltaClass(snap.deltaToBestMs)}>{"\u0394"}B {fmtDelta(snap.deltaToBestMs)}</span>
        <span className={deltaClass(snap.deltaToLastMs)}>{"\u0394"}L {fmtDelta(snap.deltaToLastMs)}</span>
        <FieldPace snap={snap} mode={fieldPaceMode} />
      </div>
      {pack && <div className={`pw-coach-pack ${snap.packState === "clear" ? "fast" : "warn"}`}>{pack}</div>}
      <div className="pw-coach-sectors">
        {snap.sectors.map((sector) => (
          <div key={sector.sectorNum} className="pw-sector">
            <div
              className="pw-sector-fill"
              style={{ width: `${sectorProgress(snap, sector.sectorNum)}%` }}
            />
          </div>
        ))}
      </div>
      <div className="pw-coach-footer">
        {snap.fuelLevel.toFixed(1)} L {"\u00b7"} {Math.round(snap.speed)}
      </div>
    </div>
  );
}
