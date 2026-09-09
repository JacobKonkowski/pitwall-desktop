import type { SessionSummary } from "../../shared/types";
import { formatDate, formatLapTime } from "../../shared/format";
import type { SessionStats } from "./sessionStats";
import { truncateIbtName } from "./sessionStats";

interface Props {
  session: SessionSummary;
  stats: SessionStats;
}

export function SessionHeader({ session, stats }: Props) {
  const consistency =
    stats.consistencyMs == null
      ? "—"
      : `±${(stats.consistencyMs / 1000).toFixed(3)}s`;

  return (
    <div className="session-header">
      <div>
        <div className="sh-title">{session.track || "Unknown track"}</div>
        <div className="muted">{session.car || "Unknown car"}</div>
        <div className="muted sh-ibt" title={session.ibtPath}>
          {truncateIbtName(session.ibtPath)}
        </div>
      </div>
      <div className="sh-facts">
        <Fact label="Date" value={formatDate(session.sessionDate)} />
        <Fact label="Laps" value={String(session.lapCount)} />
        <Fact
          label="Best (pace)"
          value={formatLapTime(session.bestLapMs)}
          accent={session.bestLapMs != null}
        />
        <Fact
          label="Theo. best"
          value={formatLapTime(stats.theoreticalBestMs)}
          accent={stats.theoreticalBestMs != null}
        />
        <Fact label="Consistency" value={consistency} />
      </div>
    </div>
  );
}

function Fact({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="fact">
      <span className="fact-label">{label}</span>
      <span className={`fact-value${accent ? " accent" : ""}`}>{value}</span>
    </div>
  );
}
