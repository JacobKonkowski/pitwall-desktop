import { useMemo, useState } from "react";
import type { SessionSummary } from "../../shared/types";
import { formatDate, formatLapTime } from "../../shared/format";

interface Props {
  sessions: SessionSummary[];
  selectedId: number | null;
  onSelect: (id: number) => void;
  onDelete: (id: number) => void;
}

export function SessionBrowser({ sessions, selectedId, onSelect, onDelete }: Props) {
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return sessions;
    return sessions.filter(
      (s) =>
        s.track.toLowerCase().includes(q) ||
        s.car.toLowerCase().includes(q) ||
        s.sessionDate.toLowerCase().includes(q),
    );
  }, [sessions, query]);

  return (
    <aside className="session-sidebar">
      <div className="sidebar-search">
        <input
          type="text"
          placeholder="Search track or car…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>
      <div className="session-list">
        {filtered.length === 0 ? (
          <div className="muted" style={{ padding: 16, fontSize: 13 }}>
            {sessions.length === 0 ? "No sessions imported." : "No matches."}
          </div>
        ) : (
          filtered.map((s) => (
            <div
              key={s.id}
              className={`session-item${s.id === selectedId ? " active" : ""}`}
              onClick={() => onSelect(s.id)}
            >
              <div className="si-track">{s.track || "Unknown track"}</div>
              <div className="si-car">{s.car || "Unknown car"}</div>
              <div className="si-meta">
                <span>{formatDate(s.sessionDate)}</span>
                <span className="si-best">{formatLapTime(s.bestLapMs)}</span>
              </div>
              <button
                className="btn btn-ghost btn-danger si-delete"
                title="Delete session"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(s.id);
                }}
              >
                Delete
              </button>
            </div>
          ))
        )}
      </div>
    </aside>
  );
}
