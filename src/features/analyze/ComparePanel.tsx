import { useEffect, useMemo, useState } from "react";
import {
  CartesianGrid,
  Line,
  LineChart,
  ReferenceArea,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { compareLaps } from "../../shared/api";
import type {
  AssistKind,
  AssistSpan,
  CornerDelta,
  LapComparison,
  LapSummary,
} from "../../shared/types";
import { deltaClass, formatDelta, formatLapTime } from "../../shared/format";
import { CornerTable } from "./CornerTable";
import { CAND_COLOR, DELTA_COLOR, REF_COLOR } from "./compareColors";

const SYNC_ID = "compare-dist";

interface Props {
  laps: LapSummary[];
  candidate: LapSummary | null;
  reference: LapSummary | null;
  onChangeReference: (id: number) => void;
  /** Lap fraction (0..1) under the cursor, or `null` when the cursor leaves. */
  onHoverDistPct?: (pct: number | null) => void;
  /** Corner row click: zoom the track map to that lap fraction. */
  onFocusDistPct?: (pct: number) => void;
}

const ASSIST_NAMES: Record<AssistKind, string> = { abs: "ABS", tc: "TC" };

/** A shaded x range on a chart, in percent around the lap. */
interface ChartBand {
  x1: number;
  x2: number;
  color: string;
  label: string;
}

function assistBands(spans: AssistSpan[], kind: AssistKind): ChartBand[] {
  return spans
    .filter((s) => s.kind === kind)
    .map((s) => ({
      x1: +(s.startPct * 100).toFixed(2),
      x2: +(s.endPct * 100).toFixed(2),
      color: s.lap === "candidate" ? CAND_COLOR : REF_COLOR,
      label: `${ASSIST_NAMES[kind]} (${s.lap})`,
    }));
}

function lapLabel(lap: LapSummary): string {
  const time = lap.lapTimeMs != null ? ` · ${formatLapTime(lap.lapTimeMs)}` : "";
  const pace = lap.paceEligible ? " ✓" : "";
  return `Lap ${lap.lapNumber}${time}${pace}`;
}

export function ComparePanel({
  laps,
  candidate,
  reference,
  onChangeReference,
  onHoverDistPct,
  onFocusDistPct,
}: Props) {
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
    const toDeg = (rad: number) => +(rad * (180 / Math.PI)).toFixed(1);
    return comparison.series.map((p) => ({
      x: +(p.distPct * 100).toFixed(2),
      deltaMs: p.cumulativeDeltaMs == null ? null : +p.cumulativeDeltaMs.toFixed(1),
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

  const absBands = useMemo(() => assistBands(comparison?.assists ?? [], "abs"), [comparison]);
  const tcBands = useMemo(() => assistBands(comparison?.assists ?? [], "tc"), [comparison]);

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

            {comparison.timing == null ? (
              <p className="muted">
                Not enough trace data on these laps to time them against each other.
              </p>
            ) : (
              <CornerTable
                corners={comparison.corners}
                estimated={comparison.timing === "estimated"}
                laps={laps}
                candidate={candidate}
                reference={reference}
                onHoverDistPct={onHoverDistPct}
                onFocusDistPct={onFocusDistPct}
              />
            )}

            <div className="chart-title">
              Time gain / loss vs distance
              {comparison.timing === "estimated" ? " (estimated)" : ""}
            </div>
            <Chart
              data={chartData}
              onHoverDistPct={onHoverDistPct}
              lines={[{ key: "deltaMs", color: DELTA_COLOR }]}
              yFormatter={(v) => `${v >= 0 ? "+" : ""}${(v / 1000).toFixed(3)}s`}
              zeroLine
              corners={comparison.corners}
              cornerLabels
            />

            <div className="chart-title">Speed (km/h)</div>
            <Chart
              data={chartData}
              onHoverDistPct={onHoverDistPct}
              corners={comparison.corners}
              lines={[
                { key: "candSpeed", color: CAND_COLOR },
                { key: "refSpeed", color: REF_COLOR },
              ]}
            />

            <div className="chart-title">Throttle (%)</div>
            {tcBands.length > 0 || absBands.length > 0 ? (
              <p className="muted chart-note">
                Shaded: traction control cutting throttle, and ABS releasing the brake, in
                each lap's color.
              </p>
            ) : null}
            <Chart
              data={chartData}
              onHoverDistPct={onHoverDistPct}
              corners={comparison.corners}
              domain={[0, 100]}
              bands={tcBands}
              lines={[
                { key: "candThrottle", color: CAND_COLOR },
                { key: "refThrottle", color: REF_COLOR },
              ]}
            />

            <div className="chart-title">Brake (%)</div>
            <Chart
              data={chartData}
              onHoverDistPct={onHoverDistPct}
              corners={comparison.corners}
              domain={[0, 100]}
              bands={absBands}
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
                onHoverDistPct={onHoverDistPct}
                height={140}
                lines={[
                  { key: "candGear", color: CAND_COLOR, step: true },
                  { key: "refGear", color: REF_COLOR, step: true },
                ]}
              />
              <div className="chart-title">Steering (°)</div>
              <Chart
                data={chartData}
                onHoverDistPct={onHoverDistPct}
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
  onHoverDistPct,
  corners,
  cornerLabels,
  bands,
}: {
  data: Record<string, number | null>[];
  lines: ChartLine[];
  domain?: [number, number];
  height?: number;
  yFormatter?: (v: number) => string;
  zeroLine?: boolean;
  onHoverDistPct?: (pct: number | null) => void;
  /** Draw a faint marker at each corner's slowest point. */
  corners?: CornerDelta[];
  cornerLabels?: boolean;
  /** Shaded x ranges, named in the tooltip when hovered. */
  bands?: ChartBand[];
}) {
  return (
    <div className="chart-wrap" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          syncId={SYNC_ID}
          data={data}
          margin={{ top: 6, right: 12, bottom: 6, left: -8 }}
          onMouseMove={(state) => {
            // `activeLabel` is the x axis value: percent around the lap.
            const label = Number(state?.activeLabel);
            onHoverDistPct?.(Number.isFinite(label) ? label / 100 : null);
          }}
          onMouseLeave={() => onHoverDistPct?.(null)}
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
            labelFormatter={(v) => {
              const active = bands?.filter((b) => v >= b.x1 && v <= b.x2).map((b) => b.label);
              return active?.length
                ? `${v}% around lap · ${active.join(", ")}`
                : `${v}% around lap`;
            }}
            formatter={(value: number, name: string) => {
              if (name === "deltaMs") {
                return [formatDelta(value), "Δ time"];
              }
              return [value, name];
            }}
          />
          {zeroLine ? <ReferenceLine y={0} stroke="#5b6472" strokeDasharray="4 4" /> : null}
          {bands?.map((b) => (
            <ReferenceArea
              key={`${b.label}-${b.x1}`}
              x1={b.x1}
              x2={b.x2}
              fill={b.color}
              fillOpacity={0.15}
              stroke="none"
              ifOverflow="hidden"
            />
          ))}
          {corners?.map((c) => (
            <ReferenceLine
              key={c.number}
              x={+(c.apexPct * 100).toFixed(2)}
              stroke="#3a4352"
              strokeDasharray="2 4"
              label={
                cornerLabels
                  ? { value: `C${c.number}`, position: "insideTop", fill: "#8b95a5", fontSize: 10 }
                  : undefined
              }
            />
          ))}
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
