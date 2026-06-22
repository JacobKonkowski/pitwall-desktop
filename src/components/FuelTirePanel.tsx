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
import type { FuelSummary, TireSummary } from "../lib/types";

interface Props {
  fuel: FuelSummary | null;
  tires: TireSummary | null;
}

export function FuelTirePanel({ fuel, tires }: Props) {
  return (
    <div className="fuel-tire-grid">
      <div className="panel">
        <div className="panel-header">
          <h2>Fuel</h2>
          {fuel?.tankCapacity != null && (
            <span className="muted">Tank ~{fuel.tankCapacity.toFixed(1)} L</span>
          )}
        </div>
        {!fuel || fuel.laps.length === 0 ? (
          <p className="muted">No fuel data for this session.</p>
        ) : (
          <>
            <ResponsiveContainer width="100%" height={160}>
              <BarChart data={fuel.laps}>
                <CartesianGrid strokeDasharray="3 3" stroke="#333" />
                <XAxis dataKey="lapNumber" stroke="#888" />
                <YAxis stroke="#888" />
                <Tooltip contentStyle={{ background: "#1a1a1a", border: "1px solid #333" }} />
                <Bar dataKey="fuelUsed" name="Fuel used (L)" fill="#66bb6a" />
              </BarChart>
            </ResponsiveContainer>
            {fuel.laps.length > 0 && fuel.laps[fuel.laps.length - 1].lapsRemainingEstimate != null && (
              <p className="fuel-estimate">
                Est. laps remaining:{" "}
                <strong>
                  {fuel.laps[fuel.laps.length - 1].lapsRemainingEstimate!.toFixed(1)}
                </strong>
              </p>
            )}
          </>
        )}
      </div>

      <div className="panel">
        <div className="panel-header">
          <h2>Tires</h2>
        </div>
        {!tires || tires.laps.length === 0 ? (
          <p className="muted">No tire temperature data for this session.</p>
        ) : (
          <>
            <ResponsiveContainer width="100%" height={160}>
              <LineChart data={tires.laps}>
                <CartesianGrid strokeDasharray="3 3" stroke="#333" />
                <XAxis dataKey="lapNumber" stroke="#888" />
                <YAxis stroke="#888" />
                <Tooltip contentStyle={{ background: "#1a1a1a", border: "1px solid #333" }} />
                <Legend />
                <Line type="monotone" dataKey="lfTemp" name="LF" stroke="#ef5350" dot={false} />
                <Line type="monotone" dataKey="rfTemp" name="RF" stroke="#42a5f5" dot={false} />
                <Line type="monotone" dataKey="lrTemp" name="LR" stroke="#ffb74d" dot={false} />
                <Line type="monotone" dataKey="rrTemp" name="RR" stroke="#66bb6a" dot={false} />
              </LineChart>
            </ResponsiveContainer>
            <p className="muted small">{tires.note}</p>
          </>
        )}
      </div>
    </div>
  );
}
