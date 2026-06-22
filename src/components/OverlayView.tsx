import { useEffect, useState } from "react";
import { formatDelta, formatLapTime, getLiveSnapshot, onLiveTelemetry } from "../lib/api";
import type { LiveSnapshot } from "../lib/types";
import "../overlay.css";

export function OverlayView() {
  const [snap, setSnap] = useState<LiveSnapshot | null>(null);

  useEffect(() => {
    getLiveSnapshot().then((s) => {
      if (s.track) setSnap(s);
    });
    let unlisten: (() => void) | undefined;
    onLiveTelemetry((payload) => setSnap(payload)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  if (!snap) {
    return (
      <div className="overlay-root">
        <span className="overlay-wait">Waiting for live telemetry…</span>
      </div>
    );
  }

  return (
    <div className="overlay-root">
      <div className="overlay-title">{snap.track}</div>
      <div className="overlay-lap">
        Lap {snap.lap} · {formatLapTime(snap.lapTimeMs)}
      </div>
      <div className={`overlay-delta ${snap.deltaToBestMs != null && snap.deltaToBestMs > 0 ? "slow" : "fast"}`}>
        Δ {formatDelta(snap.deltaToBestMs)}
      </div>
      <div className="overlay-fuel">Fuel {snap.fuelLevel.toFixed(1)} L</div>
      <div className="overlay-sectors">
        {[1, 2, 3].map((n) => {
          const sector = snap.sectors.find((s) => s.sectorNum === n);
          const done = sector?.completed;
          const active = snap.currentSector === n;
          return (
            <div key={n} className="overlay-sector">
              <span>S{n}</span>
              <div className="overlay-sector-bar">
                <div
                  className="overlay-sector-fill"
                  style={{
                    width: done ? "100%" : active ? `${snap.lapDistPct * 100}%` : "0%",
                  }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
