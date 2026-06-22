import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { LapTrace } from "../lib/types";

interface Props {
  traces: LapTrace[];
}

const COLORS = ["#4fc3f7", "#ffb74d", "#ef5350"];

function mergeTraces(traces: LapTrace[]) {
  if (traces.length === 0) return [];

  const maxLen = Math.max(...traces.map((t) => t.points.length));
  const rows: Record<string, number | string>[] = [];

  for (let i = 0; i < maxLen; i++) {
    const row: Record<string, number | string> = {};
    traces.forEach((trace, idx) => {
      const pt = trace.points[i];
      if (!pt) return;
      row.distPct = Math.round(pt.distPct * 1000) / 10;
      row[`speed_${idx}`] = pt.speed;
      row[`throttle_${idx}`] = pt.throttle * 100;
      row[`brake_${idx}`] = pt.brake * 100;
    });
    if (row.distPct != null) rows.push(row);
  }
  return rows;
}

export function LapCompareChart({ traces }: Props) {
  if (traces.length === 0) {
    return (
      <div className="panel chart-panel empty-chart">
        <p className="muted">Select valid laps to compare speed, throttle, and brake traces.</p>
      </div>
    );
  }

  const data = mergeTraces(traces);

  return (
    <div className="panel chart-panel">
      <div className="panel-header">
        <h2>Lap Compare</h2>
        <span className="muted">
          {traces.map((t) => `Lap ${t.lapNumber}`).join(" vs ")}
        </span>
      </div>

      <div className="chart-block">
        <h3>Speed (m/s)</h3>
        <ResponsiveContainer width="100%" height={180}>
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#333" />
            <XAxis dataKey="distPct" unit="%" stroke="#888" />
            <YAxis stroke="#888" />
            <Tooltip contentStyle={{ background: "#1a1a1a", border: "1px solid #333" }} />
            <Legend />
            {traces.map((t, i) => (
              <Line
                key={t.lapId}
                type="monotone"
                dataKey={`speed_${i}`}
                name={`Lap ${t.lapNumber}`}
                stroke={COLORS[i % COLORS.length]}
                dot={false}
                strokeWidth={2}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="chart-block">
        <h3>Throttle / Brake (%)</h3>
        <ResponsiveContainer width="100%" height={180}>
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#333" />
            <XAxis dataKey="distPct" unit="%" stroke="#888" />
            <YAxis domain={[0, 100]} stroke="#888" />
            <Tooltip contentStyle={{ background: "#1a1a1a", border: "1px solid #333" }} />
            <Legend />
            {traces.map((t, i) => (
              <Line
                key={`th-${t.lapId}`}
                type="monotone"
                dataKey={`throttle_${i}`}
                name={`Throttle L${t.lapNumber}`}
                stroke={COLORS[i % COLORS.length]}
                dot={false}
                strokeWidth={1.5}
              />
            ))}
            {traces.map((t, i) => (
              <Line
                key={`br-${t.lapId}`}
                type="monotone"
                dataKey={`brake_${i}`}
                name={`Brake L${t.lapNumber}`}
                stroke={COLORS[i % COLORS.length]}
                strokeDasharray="4 4"
                dot={false}
                strokeWidth={1.5}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
