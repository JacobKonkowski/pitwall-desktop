import { useEffect, useMemo, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { compareLaps } from "../../shared/api";
import type { AlignedPoint, LapComparison, LapSummary } from "../../shared/types";
import { deltaClass, formatDelta, formatLapTime } from "../../shared/format";

const CAND_COLOR = "#4aa3ff";
const REF_COLOR = "#d29922";
const DELTA_COLOR = "#a371f7";
const SYNC_ID = "compare-dist";

interface Props {
  laps: LapSummary[];
  candidate: LapSummary | null;
  reference: LapSummary | null;
  onChangeReference: (id: number) => void;
}

function lapLabel(lap: LapSummary): string {
  const time = lap.lapTimeMs != null ? ` · ${formatLapTime(lap.lapTimeMs)}` : "";
  const pace = lap.paceEligible ? " ✓" : "";
  return `Lap ${lap.lapNumber}${time}${pace}`;
}

/**
 * Approximate cumulative time gain/loss (ms) from aligned speeds.
 * Uses estimated track length from candidate avg speed × lap time when available.
 */
function approxCumulativeDelta(
  series: AlignedPoint[],
  candidateTimeMs: number | null,
): number[] {
  const speeds = series
    .map((p) => p.candidateSpeed)
    .filter((v): v is number => v != null && v > 1);
  const avgSpeed =
    speeds.length > 0 ? speeds.reduce((a, b) => a + b, 0) / speeds.length : null;
  const trackLenM =
    avgSpeed != null && candidateTimeMs != null && candidateTimeMs > 0
      ? avgSpeed * (candidateTimeMs / 1000)
      : 4000; // fallback nominal length (m)

  const out: number[] = [];
  let cum = 0;
  for (let i = 0; i < series.length; i++) {
    if (i === 0) {
      out.push(series[i].cumulativeDeltaMs ?? 0);
      continue;
    }
    const prev = series[i - 1];
    const cur = series[i];
    if (cur.cumulativeDeltaMs != null) {
      cum = cur.cumulativeDeltaMs;
      out.push(cum);
      continue;
    }
    const dd = cur.distPct - prev.distPct;
    if (dd <= 0) {
      out.push(cum);
      continue;
    }
    const vc = cur.candidateSpeed;
    const vr = cur.referenceSpeed;
    if (vc == null || vr == null || vc < 0.5 || vr < 0.5) {
      out.push(cum);
      continue;
    }
    const ds = dd * trackLenM;
    const dtCand = (ds / vc) * 1000;
    const dtRef = (ds / vr) * 1000;
    cum += dtCand - dtRef;
    out.push(cum);
  }
  return out;
}

export function ComparePanel({ laps, candidate, reference, onChangeReference }: Props) {
  const [comparison, setComparison] = useState<LapComparison | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canCompare = candidate && reference && candidate.id !== reference.id;

  useEffect(() => {
    if (!canCompare) {
      setComparison(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    compareLaps(candidate!.id, reference!.id)
      .then((c) => !cancelled && setComparison(c))
      .catch((e) => !cancelled && setError(String(e)))
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [canCompare, candidate, reference]);

  const chartData = useMemo(() => {
    if (!comparison) return [];
    const cum = approxCumulativeDelta(comparison.series, comparison.candidateTimeMs);
    const toDeg = (rad: number) => +(rad * (180 / Math.PI)).toFixed(1);
    return comparison.series.map((p, i) => ({
      x: +(p.distPct * 100).toFixed(2),
      deltaMs: +cum[i].toFixed(1),
      candSpeed: p.candidateSpeed == null ? null : +(p.candidateSpeed * 3.6).toFixed(1),
      refSpeed: p.referenceSpeed == null ? null : +(p.referenceSpeed * 3.6).toFixed(1),
      candThrottle: p.candidateThrottle == null ? null : +(p.candidateThrottle * 100).toFixed(0),
      refThrottle: p.referenceThrottle == null ? null : +(p.referenceThrottle * 100).toFixed(0),
      candBrake: p.candidateBrake == null ? null : +(p.candidateBrake * 100).toFixed(0),
      refBrake: p.referenceBrake == null ? null : +(p.referenceBrake * 100).toFixed(0),
      candGear: p.candidateGear == null ? null : Math.round(p.candidateGear),
      refGear: p.referenceGear == null ? null : Math.round(p.referenceGear),
      candSteering: p.candidateSteering == null ? null : toDeg(p.candidateSteering),
      refSteering: p.referenceSteering == null ? null : toDeg(p.referenceSteering),
    }));
  }, [comparison]);

  return (
    <div className="panel">
      <div className="panel-header">
        <h2>Compare</h2>
        <div className="compare-controls" style={{ marginLeft: "auto" }}>
          <label htmlFor="ref-select">Reference</label>
          <select
            id="ref-select"
            value={reference?.id ?? ""}
            onChange={(e) => onChangeReference(Number(e.target.value))}
          >
            <option value="" disabled>
              Pick a lap…
            </option>
            {laps.map((lap) => (
              <option key={lap.id} value={lap.id}>
                {lapLabel(lap)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="panel-body">
        {!candidate ? (
          <p className="muted">Select a lap above to compare it against the reference.</p>
        ) : !reference ? (
          <p className="muted">Pick a reference lap to compare against.</p>
        ) : candidate.id === reference.id ? (
          <p className="muted">Candidate and reference are the same lap. Pick another lap.</p>
        ) : loading ? (
          <p className="muted">Comparing…</p>
        ) : error ? (
          <p className="slow">Compare failed: {error}</p>
        ) : comparison ? (
          <>
            <div className="legend">
              <span>
                <span className="swatch" style={{ background: CAND_COLOR }} />
                Candidate — Lap {candidate.lapNumber}
              </span>
              <span>
                <span className="swatch" style={{ background: REF_COLOR }} />
                Reference — Lap {reference.lapNumber}
              </span>
              <span>
                <span className="swatch" style={{ background: DELTA_COLOR }} />
                Time gain/loss
              </span>
            </div>

            <div className="compare-summary">
              <Fact label="Candidate" value={formatLapTime(comparison.candidateTimeMs)} />
              <Fact label="Reference" value={formatLapTime(comparison.referenceTimeMs)} />
              <Fact
                label="Delta"
                value={formatDelta(comparison.deltaMs)}
                className={deltaClass(comparison.deltaMs)}
              />
            </div>

            <SectorDeltaTable comparison={comparison} />

            <div className="chart-title">Time gain / loss vs distance (approx.)</div>
            <Chart
              data={chartData}
              lines={[{ key: "deltaMs", color: DELTA_COLOR }]}
              yFormatter={(v) => `${v >= 0 ? "+" : ""}${(v / 1000).toFixed(3)}s`}
              zeroLine
            />

            <div className="chart-title">Speed (km/h)</div>
            <Chart
              data={chartData}
              lines={[
                { key: "candSpeed", color: CAND_COLOR },
                { key: "refSpeed", color: REF_COLOR },
              ]}
            />

            <div className="chart-title">Throttle (%)</div>
            <Chart
              data={chartData}
              domain={[0, 100]}
              lines={[
                { key: "candThrottle", color: CAND_COLOR },
                { key: "refThrottle", color: REF_COLOR },
              ]}
            />

            <div className="chart-title">Brake (%)</div>
            <Chart
              data={chartData}
              domain={[0, 100]}
              lines={[
                { key: "candBrake", color: CAND_COLOR },
                { key: "refBrake", color: REF_COLOR },
              ]}
            />

            <details className="compare-secondary">
              <summary>Gear &amp; steering</summary>
              <div className="chart-title">Gear</div>
              <Chart
                data={chartData}
                height={140}
                lines={[
                  { key: "candGear", color: CAND_COLOR, step: true },
                  { key: "refGear", color: REF_COLOR, step: true },
                ]}
              />
              <div className="chart-title">Steering (°)</div>
              <Chart
                data={chartData}
                height={140}
                lines={[
                  { key: "candSteering", color: CAND_COLOR },
                  { key: "refSteering", color: REF_COLOR },
                ]}
              />
            </details>
          </>
        ) : null}
      </div>
    </div>
  );
}

function SectorDeltaTable({ comparison }: { comparison: LapComparison }) {
  if (comparison.sectorDeltas.length === 0) {
    return <p className="muted">No sector data for these laps.</p>;
  }
  return (
    <table className="data-table" style={{ maxWidth: 480, margin: "8px 0 16px" }}>
      <thead>
        <tr>
          <th>Sector</th>
          <th className="num">Candidate</th>
          <th className="num">Reference</th>
          <th className="num">Δ</th>
        </tr>
      </thead>
      <tbody>
        {comparison.sectorDeltas.map((s) => (
          <tr key={s.sectorNum} style={{ cursor: "default" }}>
            <td>S{s.sectorNum}</td>
            <td className="num">
              {s.candidateMs == null ? "—" : (s.candidateMs / 1000).toFixed(3)}
            </td>
            <td className="num">
              {s.referenceMs == null ? "—" : (s.referenceMs / 1000).toFixed(3)}
            </td>
            <td className={`num ${deltaClass(s.deltaMs)}`}>{formatDelta(s.deltaMs)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

interface ChartLine {
  key: string;
  color: string;
  /** Use a step curve (gear). */
  step?: boolean;
}

function Chart({
  data,
  lines,
  domain,
  height = 220,
  yFormatter,
  zeroLine,
}: {
  data: Record<string, number | null>[];
  lines: ChartLine[];
  domain?: [number, number];
  height?: number;
  yFormatter?: (v: number) => string;
  zeroLine?: boolean;
}) {
  return (
    <div className="chart-wrap" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          syncId={SYNC_ID}
          data={data}
          margin={{ top: 6, right: 12, bottom: 6, left: -8 }}
        >
          <CartesianGrid stroke="#262d3a" strokeDasharray="3 3" />
          <XAxis
            dataKey="x"
            type="number"
            domain={[0, 100]}
            tick={{ fill: "#8b95a5", fontSize: 11 }}
            tickFormatter={(v) => `${v}%`}
          />
          <YAxis
            domain={domain ?? ["auto", "auto"]}
            tick={{ fill: "#8b95a5", fontSize: 11 }}
            width={52}
            allowDecimals={!lines.some((l) => l.step)}
            tickFormatter={yFormatter}
          />
          <Tooltip
            contentStyle={{
              background: "#12161f",
              border: "1px solid #262d3a",
              borderRadius: 6,
              fontSize: 12,
            }}
            labelFormatter={(v) => `${v}% around lap`}
            formatter={(value: number, name: string) => {
              if (name === "deltaMs") {
                return [formatDelta(value), "Δ time"];
              }
              return [value, name];
            }}
          />
          {zeroLine ? <ReferenceLine y={0} stroke="#5b6472" strokeDasharray="4 4" /> : null}
          {lines.map((l) => (
            <Line
              key={l.key}
              type={l.step ? "stepAfter" : "monotone"}
              dataKey={l.key}
              stroke={l.color}
              dot={false}
              strokeWidth={1.6}
              connectNulls={false}
              isAnimationActive={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

function Fact({
  label,
  value,
  className,
}: {
  label: string;
  value: string;
  className?: string;
}) {
  return (
    <div className="fact">
      <span className="fact-label">{label}</span>
      <span className={`fact-value ${className ?? ""}`}>{value}</span>
    </div>
  );
}
