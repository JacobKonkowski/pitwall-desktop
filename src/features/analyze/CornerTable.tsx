import { Fragment, useEffect, useMemo, useState } from "react";
import type { CornerConsistency, CornerDelta, LapSummary } from "../../shared/types";
import { cornerConsistency } from "../../shared/api";
import { deltaClass, formatDelta, formatSpeedKph } from "../../shared/format";
import { CornerDetail, type ConsistencyState } from "./CornerDetail";
import { isCleanLap, matchConsistency } from "./cornerConsistency";

/** Corners whose loss is below this are not called out as "worst". */
const NOTABLE_LOSS_MS = 50;
const WORST_COUNT = 3;
/** Extra ABS / TC time on the candidate worth calling out in the insight. */
const NOTABLE_ASSIST_MS = 150;
const COLUMNS = 8;

interface Props {
  corners: CornerDelta[];
  estimated: boolean;
  /** Session laps; the clean ones from the reference's sub-session feed consistency. */
  laps: LapSummary[];
  candidate: LapSummary;
  reference: LapSummary;
  onHoverDistPct?: (pct: number | null) => void;
  /** Row click: zoom the track map to this corner's apex. */
  onFocusDistPct?: (pct: number) => void;
}

type Loaded =
  | { referenceLapId: number; data: CornerConsistency[] }
  | { referenceLapId: number; error: string };

/** "12 m later" / "8 m earlier" / "same"; `null` when unknown. */
function formatMetres(m: number | null, later: string, earlier: string): string {
  if (m == null) return "—";
  const r = Math.round(m);
  if (Math.abs(r) < 2) return "same";
  return `${Math.abs(r)} m ${r > 0 ? later : earlier}`;
}

/** One-line coaching read of where a corner's time went. */
export function cornerInsight(c: CornerDelta): string {
  const lost = c.timeDeltaMs > 0;
  const phase =
    Math.abs(c.entryDeltaMs) >= Math.abs(c.exitDeltaMs) ? "entry" : "exit";
  const seconds = (Math.abs(c.timeDeltaMs) / 1000).toFixed(3);
  const parts = [`${lost ? "lost" : "gained"} ${seconds}s, mostly on ${phase}`];
  if (c.brakePointDeltaM != null && Math.abs(c.brakePointDeltaM) >= 5) {
    parts.push(`braked ${formatMetres(c.brakePointDeltaM, "later", "earlier")}`);
  }
  if (c.candidateMinSpeed != null && c.referenceMinSpeed != null) {
    const kph = Math.round((c.candidateMinSpeed - c.referenceMinSpeed) * 3.6);
    if (Math.abs(kph) >= 2) {
      parts.push(`${Math.abs(kph)} km/h ${kph < 0 ? "slower" : "faster"} at the slowest point`);
    }
  }
  if (c.throttlePointDeltaM != null && Math.abs(c.throttlePointDeltaM) >= 5) {
    parts.push(`full throttle ${formatMetres(c.throttlePointDeltaM, "later", "earlier")}`);
  }
  const extra = (cand: number | null, ref: number | null) =>
    cand != null && ref != null ? cand - ref : 0;
  const absExtra = extra(c.candidate.absMs, c.reference.absMs);
  if (absExtra >= NOTABLE_ASSIST_MS) {
    parts.push(`${(absExtra / 1000).toFixed(2)}s more ABS`);
  }
  const tcExtra = extra(c.candidate.tcMs, c.reference.tcMs);
  if (tcExtra >= NOTABLE_ASSIST_MS) {
    parts.push(`${(tcExtra / 1000).toFixed(2)}s more TC`);
  }
  return `Corner ${c.number}: ${parts.join("; ")}.`;
}

export function CornerTable({
  corners,
  estimated,
  laps,
  candidate,
  reference,
  onHoverDistPct,
  onFocusDistPct,
}: Props) {
  const [sortByLoss, setSortByLoss] = useState(false);
  const [expanded, setExpanded] = useState<number | null>(null);
  const [loaded, setLoaded] = useState<Loaded | null>(null);

  const cleanLapIds = useMemo(
    () => laps.filter((l) => isCleanLap(l, reference.sessionNum)).map((l) => l.id),
    [laps, reference.sessionNum],
  );
  const needsFetch = expanded != null && loaded?.referenceLapId !== reference.id;

  useEffect(() => {
    if (!needsFetch) return;
    let cancelled = false;
    const referenceLapId = reference.id;
    cornerConsistency(referenceLapId, cleanLapIds)
      .then((data) => !cancelled && setLoaded({ referenceLapId, data }))
      .catch((e) => !cancelled && setLoaded({ referenceLapId, error: String(e) }));
    return () => {
      cancelled = true;
    };
  }, [needsFetch, reference.id, cleanLapIds]);

  const consistencyFor = (c: CornerDelta): ConsistencyState => {
    if (!loaded || loaded.referenceLapId !== reference.id) return { status: "loading" };
    if ("error" in loaded) return { status: "error", message: loaded.error };
    return { status: "ready", corner: matchConsistency(c, loaded.data) };
  };

  const worst = useMemo(
    () =>
      new Set(
        [...corners]
          .filter((c) => c.timeDeltaMs >= NOTABLE_LOSS_MS)
          .sort((a, b) => b.timeDeltaMs - a.timeDeltaMs)
          .slice(0, WORST_COUNT)
          .map((c) => c.number),
      ),
    [corners],
  );
  const rows = useMemo(
    () =>
      sortByLoss ? [...corners].sort((a, b) => b.timeDeltaMs - a.timeDeltaMs) : corners,
    [corners, sortByLoss],
  );
  const maxAbs = useMemo(
    () => Math.max(1, ...corners.map((c) => Math.abs(c.timeDeltaMs))),
    [corners],
  );
  const biggest = corners.reduce<CornerDelta | null>(
    (best, c) => (c.timeDeltaMs > (best?.timeDeltaMs ?? NOTABLE_LOSS_MS) ? c : best),
    null,
  );

  if (corners.length === 0) {
    return <p className="muted">No corners detected on the reference lap.</p>;
  }

  return (
    <div className="corner-analysis">
      <div className="corner-header">
        <div className="chart-title">
          Corners{estimated ? " (estimated timing)" : ""}
        </div>
        <label className="corner-sort muted">
          <input
            type="checkbox"
            checked={sortByLoss}
            onChange={(e) => setSortByLoss(e.target.checked)}
          />
          Biggest loss first
        </label>
      </div>
      {biggest ? <p className="corner-insight">{cornerInsight(biggest)}</p> : null}
      {estimated ? (
        <p className="muted corner-note">
          One of these laps was imported before PitWall stored lap timing, so corner
          times are estimated from speed. Re-import the session for exact numbers.
        </p>
      ) : null}
      <table className="data-table corner-table">
        <thead>
          <tr>
            <th title="Detected from the reference lap's speed; may not match official turn numbers">
              Corner
            </th>
            <th className="num">Δ time</th>
            <th className="corner-bar-col" />
            <th className="num" title="Reference lap's slowest point → segment start">
              Entry
            </th>
            <th className="num" title="Reference lap's slowest point → segment end">
              Exit
            </th>
            <th className="num">Brake point</th>
            <th className="num" title="Slowest speed through the corner, candidate / reference">
              Min km/h
            </th>
            <th className="num">Full throttle</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((c) => (
            <Fragment key={c.number}>
              <tr
                className={[
                  "corner-focusable",
                  worst.has(c.number) ? "corner-worst" : "",
                  expanded === c.number ? "corner-expanded" : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                title="Click for corner detail and lap consistency"
                aria-expanded={expanded === c.number}
                onMouseEnter={() => onHoverDistPct?.(c.apexPct)}
                onMouseLeave={() => onHoverDistPct?.(null)}
                onClick={() => {
                  setExpanded((n) => (n === c.number ? null : c.number));
                  onFocusDistPct?.(c.apexPct);
                }}
              >
                <td>
                  C{c.number}
                  <span className="muted corner-pos"> {(c.apexPct * 100).toFixed(0)}%</span>
                </td>
                <td className={`num ${deltaClass(c.timeDeltaMs)}`}>
                  {formatDelta(c.timeDeltaMs)}
                </td>
                <td className="corner-bar-col">
                  <DeltaBar ms={c.timeDeltaMs} maxAbs={maxAbs} />
                </td>
                <td className={`num ${deltaClass(c.entryDeltaMs)}`}>
                  {formatDelta(c.entryDeltaMs)}
                </td>
                <td className={`num ${deltaClass(c.exitDeltaMs)}`}>
                  {formatDelta(c.exitDeltaMs)}
                </td>
                <td className="num">{formatMetres(c.brakePointDeltaM, "later", "earlier")}</td>
                <td className="num">
                  {formatSpeedKph(c.candidateMinSpeed)} / {formatSpeedKph(c.referenceMinSpeed)}
                </td>
                <td className="num">
                  {formatMetres(c.throttlePointDeltaM, "later", "earlier")}
                </td>
              </tr>
              {expanded === c.number ? (
                <tr className="corner-detail-row">
                  <td colSpan={COLUMNS}>
                    <CornerDetail
                      corner={c}
                      consistency={consistencyFor(c)}
                      laps={laps}
                      candidateLapId={candidate.id}
                      referenceLapId={reference.id}
                    />
                  </td>
                </tr>
              ) : null}
            </Fragment>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Centred bar: losses grow right (red), gains grow left (green). */
function DeltaBar({ ms, maxAbs }: { ms: number; maxAbs: number }) {
  const width = `${(Math.abs(ms) / maxAbs) * 50}%`;
  return (
    <div className="corner-bar">
      <div
        className={`corner-bar-fill ${ms >= 0 ? "slow" : "fast"}`}
        style={ms >= 0 ? { left: "50%", width } : { right: "50%", width }}
      />
    </div>
  );
}
