import { useMemo, useState } from "react";
import { formatDate, formatLapTime } from "../lib/api";
import type { ImportStatus, SessionSummary } from "../lib/types";

type SortKey = "date" | "track" | "best";

interface Props {
  sessions: SessionSummary[];
  selectedId: number | null;
  importStatus: ImportStatus;
  onSelect: (id: number) => void;
  onImportFile: () => void;
  onImportFolder: () => void;
  onDeleteSession: (id: number) => void;
  onClearDatabase?: () => void;
  loading: boolean;
}

export function SessionBrowser({
  sessions,
  selectedId,
  importStatus,
  onSelect,
  onImportFile,
  onImportFolder,
  onDeleteSession,
  onClearDatabase,
  loading,
}: Props) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortKey>("date");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    let list = sessions;
    if (q) {
      list = list.filter(
        (s) =>
          s.track.toLowerCase().includes(q) ||
          s.car.toLowerCase().includes(q) ||
          s.ibtPath.toLowerCase().includes(q),
      );
    }
    return list.slice().sort((a, b) => {
      if (sort === "track") return a.track.localeCompare(b.track);
      if (sort === "best") {
        const ab = a.bestLapMs ?? Number.MAX_SAFE_INTEGER;
        const bb = b.bestLapMs ?? Number.MAX_SAFE_INTEGER;
        return ab - bb;
      }
      return b.sessionDate.localeCompare(a.sessionDate);
    });
  }, [sessions, query, sort]);

  const scanSummary =
    importStatus.batchElapsedMs != null && importStatus.batchFileCount != null
      ? `Last scan: ${importStatus.batchFileCount} files in ${(importStatus.batchElapsedMs / 1000).toFixed(1)}s`
      : null;

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
      {scanSummary && <p className="muted small scan-summary">{scanSummary}</p>}
      {sessions.length > 0 && (
        <div className="session-browser-filters">
          <input
            type="search"
            placeholder="Search track, car, path…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            aria-label="Search sessions"
          />
          <select value={sort} onChange={(e) => setSort(e.target.value as SortKey)} aria-label="Sort">
            <option value="date">Sort: date</option>
            <option value="track">Sort: track</option>
            <option value="best">Sort: best lap</option>
          </select>
        </div>
      )}

      {sessions.length === 0 ? (
        <div className="empty-state">
          <p>No sessions imported yet.</p>
          <p className="muted">
            Record telemetry in iRacing with Alt+L, then click Scan Folder to import from
            Documents/iRacing/telemetry.
          </p>
        </div>
      ) : filtered.length === 0 ? (
        <p className="muted empty-state">No sessions match your search.</p>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              <th>Track</th>
              <th>Car</th>
              <th>Date</th>
              <th>Laps</th>
              <th>Best</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {filtered.map((s) => (
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
                <td>
                  <button
                    type="button"
                    className="btn-danger btn-small"
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteSession(s.id);
                    }}
                    disabled={loading}
                    aria-label={`Delete session ${s.track}`}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {import.meta.env.DEV && onClearDatabase && (
        <div className="dev-tools">
          <button type="button" className="btn-danger" onClick={onClearDatabase} disabled={loading}>
            Clear database (dev)
          </button>
        </div>
      )}
    </div>
  );
}
