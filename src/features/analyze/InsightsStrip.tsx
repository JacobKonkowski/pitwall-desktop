import type { SessionStats } from "./sessionStats";

interface Props {
  stats: SessionStats;
}

/** Light deterministic insights from lap data (no LLM). */
export function InsightsStrip({ stats }: Props) {
  const bullets: string[] = [];

  if (stats.paceLapCount >= 2 && stats.consistencyMs != null) {
    const sec = (stats.consistencyMs / 1000).toFixed(3);
    if (stats.consistencyMs < 200) {
      bullets.push(`Consistency is tight (±${sec}s stdev across ${stats.paceLapCount} pace laps).`);
    } else if (stats.consistencyMs < 500) {
      bullets.push(`Consistency is moderate (±${sec}s stdev across ${stats.paceLapCount} pace laps).`);
    } else {
      bullets.push(`Consistency is loose (±${sec}s stdev) — look for outlier laps.`);
    }
  }

  if (stats.weakSector != null && stats.weakSectorLossMs != null) {
    bullets.push(
      `Weakest sector is S${stats.weakSector} (avg +${(stats.weakSectorLossMs / 1000).toFixed(3)}s vs best).`,
    );
  }

  if (stats.fuelOutlierLap != null && stats.fuelOutlierUsed != null) {
    bullets.push(
      `Fuel outlier on lap ${stats.fuelOutlierLap} (${stats.fuelOutlierUsed.toFixed(2)} L used).`,
    );
  }

  if (bullets.length === 0) return null;

  return (
    <div className="insights-strip panel">
      <div className="insights-label">Insights</div>
      <ul>
        {bullets.map((b) => (
          <li key={b}>{b}</li>
        ))}
      </ul>
    </div>
  );
}
