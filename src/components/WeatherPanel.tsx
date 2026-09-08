import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { WeatherSummary } from "../lib/types";

interface Props {
  weather: WeatherSummary | null;
}

const TOOLTIP_STYLE = { background: "#1a1a1a", border: "1px solid #333" };

function WeatherChart({
  title,
  data,
  dataKey,
  color,
  decimals,
  emptyMessage,
  valueLabels,
}: {
  title: string;
  data: object[];
  dataKey: string;
  color: string;
  decimals: number;
  emptyMessage: string;
  valueLabels?: Record<number, string>;
}) {
  const hasData = data.some((row) => (row as Record<string, unknown>)[dataKey] != null);
  const formatValue = (v: number) => valueLabels?.[v] ?? v.toFixed(decimals);
  return (
    <div className="panel">
      <div className="panel-header">
        <h2>{title}</h2>
      </div>
      {!hasData ? (
        <p className="muted">{emptyMessage}</p>
      ) : (
        <ResponsiveContainer width="100%" height={160}>
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#333" />
            <XAxis dataKey="lapNumber" stroke="#888" />
            <YAxis
              stroke="#888"
              domain={valueLabels ? [1, 7] : ["auto", "auto"]}
              ticks={valueLabels ? Object.keys(valueLabels).map(Number) : undefined}
              tickFormatter={valueLabels ? (v: number) => formatValue(v) : undefined}
              width={valueLabels ? 110 : 60}
            />
            <Tooltip
              contentStyle={TOOLTIP_STYLE}
              formatter={(v: number) => formatValue(v)}
            />
            <Line
              type="monotone"
              dataKey={dataKey}
              name={title}
              stroke={color}
              dot={false}
              strokeWidth={2}
              connectNulls
            />
          </LineChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}

const WETNESS_LABELS: Record<number, string> = {
  0: "Unknown",
  1: "Dry",
  2: "Mostly Dry",
  3: "Very Lightly Wet",
  4: "Lightly Wet",
  5: "Moderately Wet",
  6: "Very Wet",
  7: "Extremely Wet",
};

export function WeatherPanel({ weather }: Props) {
  if (!weather || weather.laps.length === 0) {
    return (
      <div className="panel">
        <div className="panel-header"><h2>Weather</h2></div>
        <p className="muted">No weather data for this session.</p>
      </div>
    );
  }

  const data = weather.laps;

  return (
    <div className="fuel-tire-grid">
      <WeatherChart title="Track Temp (°C)"    data={data} dataKey="trackTemp" color="#ef5350" decimals={1} emptyMessage="No track temperature data." />
      <WeatherChart title="Air Temp (°C)"      data={data} dataKey="airTemp"   color="#ffb74d" decimals={1} emptyMessage="No air temperature data." />
      <WeatherChart title="Air Pressure (Pa)"  data={data} dataKey="airPres"   color="#42a5f5" decimals={0} emptyMessage="No air pressure data." />
      <WeatherChart title="Air Density (kg/m³)" data={data} dataKey="airDens"  color="#ce93d8" decimals={4} emptyMessage="No air density data." />
      <WeatherChart title="Humidity (%)"       data={data} dataKey="relHumid"  color="#4dd0e1" decimals={1} emptyMessage="No humidity data." />
      <WeatherChart title="Wind Speed (m/s)"   data={data} dataKey="windVel"   color="#a5d6a7" decimals={2} emptyMessage="No wind speed data." />
      <WeatherChart title="Wind Dir (rad)"     data={data} dataKey="windDir"   color="#fff176" decimals={2} emptyMessage="No wind direction data." />
      <WeatherChart title="Track Wetness"      data={data} dataKey="trackWetn" color="#80cbc4" decimals={1} emptyMessage="No track wetness data." valueLabels={WETNESS_LABELS} />
    </div>
  );
}
