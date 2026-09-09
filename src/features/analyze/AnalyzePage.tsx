import { useCallback, useEffect, useMemo, useState } from "react";
import {
  checkIracingConfig,
  confirmDialog,
  deleteSession,
  getSession,
  listSessions,
  onImportComplete,
} from "../../shared/api";
import { showToast } from "../../shared/toast";
import type {
  IracingConfigCheck,
  LapSummary,
  SessionDetail,
  SessionSummary,
} from "../../shared/types";
import { ComparePanel } from "./ComparePanel";
import { ConfigBanner } from "./ConfigBanner";
import { FuelTirePanel } from "./FuelTirePanel";
import { InsightsStrip } from "./InsightsStrip";
import { LapTable } from "./LapTable";
import { SessionBrowser } from "./SessionBrowser";
import { SessionHeader } from "./SessionHeader";
import { computeSessionStats } from "./sessionStats";
import { useImportActions } from "./useImportActions";

const LAST_SESSION_KEY = "pitwall.lastSessionId";

/** Fastest pace-eligible lap in the session (the default compare reference). */
function defaultReferenceLap(laps: LapSummary[]): LapSummary | null {
  return laps
    .filter((l) => l.paceEligible && l.lapTimeMs != null)
    .reduce<LapSummary | null>((best, l) => {
      if (!best || (l.lapTimeMs ?? Infinity) < (best.lapTimeMs ?? Infinity)) return l;
      return best;
    }, null);
}

function readLastSessionId(): number | null {
  try {
    const raw = localStorage.getItem(LAST_SESSION_KEY);
    if (!raw) return null;
    const n = Number(raw);
    return Number.isFinite(n) ? n : null;
  } catch {
    return null;
  }
}

function writeLastSessionId(id: number | null) {
  try {
    if (id == null) localStorage.removeItem(LAST_SESSION_KEY);
    else localStorage.setItem(LAST_SESSION_KEY, String(id));
  } catch {
    /* ignore quota / private mode */
  }
}

export function AnalyzePage() {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [candidateLapId, setCandidateLapId] = useState<number | null>(null);
  const [referenceLapId, setReferenceLapId] = useState<number | null>(null);
  const [config, setConfig] = useState<IracingConfigCheck | null>(null);
  const importActions = useImportActions();

  const selectSession = useCallback((id: number | null) => {
    setSelectedId(id);
    writeLastSessionId(id);
  }, []);

  const refreshSessions = useCallback(async (): Promise<SessionSummary[]> => {
    const list = await listSessions();
    setSessions(list);
    return list;
  }, []);

  // Initial load + config check.
  useEffect(() => {
    refreshSessions()
      .then((list) => {
        setSelectedId((prev) => {
          if (prev != null && list.some((s) => s.id === prev)) return prev;
          const stored = readLastSessionId();
          if (stored != null && list.some((s) => s.id === stored)) return stored;
          return list[0]?.id ?? null;
        });
      })
      .catch((e) => {
        console.error("listSessions failed", e);
        showToast(`Failed to list sessions: ${String(e)}`, "error");
      });
    checkIracingConfig().then(setConfig).catch(() => undefined);
  }, [refreshSessions]);

  // Refresh (and auto-select) when an import completes.
  useEffect(() => {
    const unlisten = onImportComplete(async (sessionId) => {
      const list = await refreshSessions();
      if (sessionId && list.some((s) => s.id === sessionId)) {
        selectSession(sessionId);
      }
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => undefined);
    };
  }, [refreshSessions, selectSession]);

  // Load detail when the selected session changes.
  useEffect(() => {
    if (selectedId == null) {
      setDetail(null);
      return;
    }
    writeLastSessionId(selectedId);
    let cancelled = false;
    setLoadingDetail(true);
    getSession(selectedId)
      .then((d) => {
        if (cancelled) return;
        setDetail(d);
        const ref = d ? defaultReferenceLap(d.laps) : null;
        setReferenceLapId(ref?.id ?? null);
        setCandidateLapId(null);
      })
      .catch((e) => {
        console.error("getSession failed", e);
        showToast(`Failed to load session: ${String(e)}`, "error");
      })
      .finally(() => !cancelled && setLoadingDetail(false));
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  const handleDelete = useCallback(
    async (sessionId: number) => {
      const ok = await confirmDialog(
        "Delete this session and its laps from the local database? This cannot be undone.",
        "Delete session",
      );
      if (!ok) return;
      try {
        await deleteSession(sessionId);
        const list = await refreshSessions();
        setSelectedId((prev) => {
          const next = prev === sessionId ? list[0]?.id ?? null : prev;
          writeLastSessionId(next);
          return next;
        });
      } catch (e) {
        showToast(`Delete failed: ${String(e)}`, "error");
      }
    },
    [refreshSessions],
  );

  const laps = detail?.laps ?? [];
  const stats = useMemo(() => computeSessionStats(laps), [laps]);
  const hasEligible = useMemo(() => laps.some((l) => l.paceEligible), [laps]);
  const okChannelPresent = useMemo(
    () => laps.some((l) => l.deltaBestOk !== null),
    [laps],
  );
  const candidate = laps.find((l) => l.id === candidateLapId) ?? null;
  const reference = laps.find((l) => l.id === referenceLapId) ?? null;

  return (
    <div className="analyze">
      <SessionBrowser
        sessions={sessions}
        selectedId={selectedId}
        onSelect={selectSession}
        onDelete={handleDelete}
      />
      <div className="analyze-workspace">
        {selectedId == null ? (
          <EmptyState config={config} importActions={importActions} />
        ) : loadingDetail ? (
          <div className="loading">Loading session…</div>
        ) : !detail ? (
          <div className="loading">Session not found.</div>
        ) : (
          <>
            <div className="panel">
              <SessionHeader session={detail.session} stats={stats} />
            </div>

            <InsightsStrip stats={stats} />

            {!hasEligible && laps.length > 0 && !okChannelPresent ? (
              <ConfigBanner />
            ) : null}

            <div className="panel">
              <div className="panel-header">
                <h2>Laps</h2>
                <div className="lap-legend muted" style={{ marginLeft: "auto" }}>
                  <span>
                    <span className="swatch cand" /> Candidate — click
                  </span>
                  <span>
                    <span className="swatch ref" /> Reference — Shift/right-click
                  </span>
                </div>
              </div>
              <div className="panel-body" style={{ padding: 0 }}>
                <LapTable
                  laps={laps}
                  candidateLapId={candidateLapId}
                  referenceLapId={referenceLapId}
                  onSelectCandidate={setCandidateLapId}
                  onSelectReference={setReferenceLapId}
                />
              </div>
            </div>

            <ComparePanel
              laps={laps}
              candidate={candidate}
              reference={reference}
              onChangeReference={setReferenceLapId}
            />

            <FuelTirePanel laps={laps} />
          </>
        )}
      </div>
    </div>
  );
}

function EmptyState({
  config,
  importActions,
}: {
  config: IracingConfigCheck | null;
  importActions: ReturnType<typeof useImportActions>;
}) {
  const { busy, handleImport, handleScan } = importActions;
  return (
    <div className="empty-state">
      <h2>No sessions yet</h2>
      <p className="muted">
        Import an iRacing <code>.ibt</code> file, or scan your telemetry folder.
        <br />
        Record telemetry in the sim with <strong>Alt+L</strong>.
      </p>
      {config && !config.diskEnabled ? (
        <p className="muted">
          Tip: set <code>irsdkEnableDisk=1</code> in{" "}
          <code>Documents\iRacing\app.ini</code> to record IBT files.
        </p>
      ) : null}
      <div className="empty-actions">
        <button className="btn" onClick={handleScan} disabled={busy}>
          Scan folder
        </button>
        <button className="btn btn-primary" onClick={handleImport} disabled={busy}>
          Import IBT
        </button>
      </div>
    </div>
  );
}
