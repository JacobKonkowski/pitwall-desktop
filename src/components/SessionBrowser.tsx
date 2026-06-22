import { formatDate, formatLapTime } from "../lib/api";
import type { SessionSummary } from "../lib/types";

interface Props {
  sessions: SessionSummary[];
  selectedId: number | null;
  onSelect: (id: number) => void;
  onImportFile: () => void;
  onImportFolder: () => void;
  onClearDatabase?: () => void;
  loading: boolean;
}

export function SessionBrowser({
  sessions,
  selectedId,
  onSelect,
  onImportFile,
  onImportFolder,
  onClearDatabase,
  loading,
}: Props) {
  return (
    <div className="panel session-browser">
      <div className="panel-header">
        <h2>Sessions</h2>
        <div className="btn-row">
          <button type="button" onClick={onImportFile} disabled={loading}>
            Import IBT
          </button>
          <button type="button" onClick={onImportFolder} disabled={loading}>
            Scan Folder
          </button>
        </div>
      </div>

      {sessions.length === 0 ? (
        <div className="empty-state">
          <p>No sessions imported yet.</p>
          <p className="muted">
            Record telemetry in iRacing with Alt+L, then click Scan Folder to import from
            Documents/iRacing/telemetry.
          </p>
        </div>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Track</th>
              <th>Car</th>
              <th>Date</th>
              <th>Laps</th>
              <th>Best</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((s) => (
              <tr
                key={s.id}
                className={selectedId === s.id ? "selected" : ""}
                onClick={() => onSelect(s.id)}
              >
                <td>{s.track}</td>
                <td>{s.car}</td>
                <td>{formatDate(s.sessionDate)}</td>
                <td>{s.lapCount}</td>
                <td>{formatLapTime(s.bestLapMs)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {import.meta.env.DEV && onClearDatabase && (
        <div className="dev-tools">
          <button
            type="button"
            className="btn-danger"
            onClick={onClearDatabase}
            disabled={loading}
          >
            Clear database (dev)
          </button>
        </div>
      )}
    </div>
  );
}
