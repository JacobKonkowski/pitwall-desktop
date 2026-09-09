import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { LapSummary } from "../../shared/types";

interface Props {
  laps: LapSummary[];
}

/** Fuel use and tire temps from lap summaries (no separate fuel/tire API). */
export function FuelTirePanel({ laps }: Props) {
  const fuelData = laps
    .filter((l) => l.fuelUsed != null)
    .map((l) => ({ lapNumber: l.lapNumber, fuelUsed: +(l.fuelUsed!).toFixed(3) }));

  const tireData = laps
    .filter((l) => l.lfTemp != null || l.rfTemp != null || l.lrTemp != null || l.rrTemp != null)
    .map((l) => ({
      lapNumber: l.lapNumber,
      lfTemp: l.lfTemp != null ? Math.round(l.lfTemp) : null,
      rfTemp: l.rfTemp != null ? Math.round(l.rfTemp) : null,
      lrTemp: l.lrTemp != null ? Math.round(l.lrTemp) : null,
      rrTemp: l.rrTemp != null ? Math.round(l.rrTemp) : null,
    }));

  return (
    <div className="fuel-tire-grid">
      <div className="panel">
        <div className="panel-header">
          <h2>Fuel</h2>
        </div>
        <div className="panel-body">
          {fuelData.length === 0 ? (
            <p className="muted">No fuel data for this session.</p>
          ) : (
            <ResponsiveContainer width="100%" height={160}>
              <BarChart data={fuelData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#262d3a" />
                <XAxis dataKey="lapNumber" stroke="#8b95a5" tick={{ fontSize: 11 }} />
                <YAxis stroke="#8b95a5" tick={{ fontSize: 11 }} width={40} />
                <Tooltip
                  contentStyle={{
                    background: "#12161f",
                    border: "1px solid #262d3a",
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                />
                <Bar dataKey="fuelUsed" name="Fuel used (L)" fill="#3fb950" />
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>

      <div className="panel">
        <div className="panel-header">
          <h2>Tires</h2>
        </div>
        <div className="panel-body">
          {tireData.length === 0 ? (
            <p className="muted">No tire temperature data for this session.</p>
          ) : (
            <ResponsiveContainer width="100%" height={160}>
              <LineChart data={tireData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#262d3a" />
                <XAxis dataKey="lapNumber" stroke="#8b95a5" tick={{ fontSize: 11 }} />
                <YAxis stroke="#8b95a5" tick={{ fontSize: 11 }} width={40} />
                <Tooltip
                  contentStyle={{
                    background: "#12161f",
                    border: "1px solid #262d3a",
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                />
                <Legend />
                <Line type="monotone" dataKey="lfTemp" name="LF" stroke="#f85149" dot={false} connectNulls />
                <Line type="monotone" dataKey="rfTemp" name="RF" stroke="#4aa3ff" dot={false} connectNulls />
                <Line type="monotone" dataKey="lrTemp" name="LR" stroke="#d29922" dot={false} connectNulls />
                <Line type="monotone" dataKey="rrTemp" name="RR" stroke="#3fb950" dot={false} connectNulls />
              </LineChart>
            </ResponsiveContainer>
          )}
        </div>
      </div>
    </div>
  );
}
