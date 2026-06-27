import { useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import {
  checkIracingConfig,
  clearDatabase,
  deleteSession,
  getFuelSummary,
  getLapTraces,
  getSession,
  getTireSummary,
  importFolder,
  importIbt,
  listSessions,
  onImportComplete,
  onImportStatus,
  pickIbtFile,
  startLiveMonitor,
} from "./lib/api";
import type {
  FuelSummary,
  ImportStatus,
  IracingConfigCheck,
  LapTrace,
  SessionDetail,
  SessionSummary,
  TireSummary,
} from "./lib/types";
import { CoachPanel } from "./components/CoachPanel";
import { SessionStandingsPanel } from "./components/SessionStandingsPanel";
import { ConfigBanner } from "./components/ConfigBanner";
import { FuelTirePanel } from "./components/FuelTirePanel";
import { LapCompareChart } from "./components/LapCompareChart";
import { LapTable } from "./components/LapTable";
import { LivePanel } from "./components/LivePanel";
import { SessionBrowser } from "./components/SessionBrowser";
import { SettingsPage } from "./components/SettingsPage";
import "./App.css";

type AppTab = "analyze" | "live" | "settings";
const TAB_KEY = "pitwall-tab";

function loadTab(): AppTab {
  const saved = localStorage.getItem(TAB_KEY);
  if (saved === "live" || saved === "settings" || saved === "analyze") return saved;
  return "analyze";
}

function App() {
  const [tab, setTab] = useState<AppTab>(loadTab);
  const [version, setVersion] = useState<string | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [selectedLaps, setSelectedLaps] = useState<number[]>([]);
  const [highlightedLaps, setHighlightedLaps] = useState<number[]>([]);
  const [traces, setTraces] = useState<LapTrace[]>([]);
  const [fuel, setFuel] = useState<FuelSummary | null>(null);
  const [tires, setTires] = useState<TireSummary | null>(null);
  const [config, setConfig] = useState<IracingConfigCheck | null>(null);
  const [configLoading, setConfigLoading] = useState(true);
  const [importStatus, setImportStatus] = useState<ImportStatus>({
    active: false,
    currentFile: null,
    progressPct: 0,
    message: "Idle",
  });
  const [loading, setLoading] = useState(false);
  const [sessionLoading, setSessionLoading] = useState(false);
  const [sessionError, setSessionError] = useState<string | null>(null);

  const setAppTab = (next: AppTab) => {
    setTab(next);
    localStorage.setItem(TAB_KEY, next);
  };

  const refreshSessions = useCallback(async () => {
    try {
      const list = await listSessions();
      setSessions(list);
      setSessionError(null);
    } catch (e) {
      setSessionError(String(e));
    }
  }, []);

  const loadSession = useCallback(async (sessionId: number) => {
    setSessionLoading(true);
    setSessionError(null);
    try {
      const data = await getSession(sessionId);
      setDetail(data);
      setSelectedLaps([]);
      setHighlightedLaps([]);
      setTraces([]);
      if (data) {
        const [fuelData, tireData] = await Promise.all([
          getFuelSummary(sessionId),
          getTireSummary(sessionId),
        ]);
        setFuel(fuelData);
        setTires(tireData);
      } else {
        setFuel(null);
        setTires(null);
      }
    } catch (e) {
      setSessionError(String(e));
      setDetail(null);
    } finally {
      setSessionLoading(false);
    }
  }, []);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion(null));
  }, []);

  useEffect(() => {
    refreshSessions();
    checkIracingConfig()
      .then(setConfig)
      .catch(() => setConfig(null))
      .finally(() => setConfigLoading(false));

    const unsubs: Promise<() => void>[] = [
      onImportComplete((sessionId) => {
        refreshSessions().then(() => {
          setSelectedId(sessionId);
          loadSession(sessionId);
        });
      }),
      onImportStatus(setImportStatus),
    ];

    return () => {
      Promise.all(unsubs).then((fns) => fns.forEach((fn) => fn()));
    };
  }, [refreshSessions, loadSession]);

  useEffect(() => {
    if (selectedId != null) loadSession(selectedId);
  }, [selectedId, loadSession]);

  useEffect(() => {
    if (selectedLaps.length === 0) {
      setTraces([]);
      return;
    }
    getLapTraces(selectedLaps).then(setTraces).catch(() => setTraces([]));
  }, [selectedLaps]);

  const handleToggleLap = (lapId: number) => {
    setSelectedLaps((prev) => {
      if (prev.includes(lapId)) return prev.filter((id) => id !== lapId);
      if (prev.length >= 2) return [prev[1], lapId];
      return [...prev, lapId];
    });
  };

  const handleCompareLaps = (lapNumbers: number[]) => {
    if (!detail || lapNumbers.length === 0) return;
    const ids = detail.laps.filter((l) => lapNumbers.includes(l.lapNumber)).map((l) => l.id);
    setSelectedLaps(ids.slice(0, 2));
    setHighlightedLaps(lapNumbers);
  };

  const handleImportFile = async () => {
    const path = await pickIbtFile();
    if (!path) return;
    setLoading(true);
    try {
      await importIbt(path);
      await refreshSessions();
    } finally {
      setLoading(false);
    }
  };

  const handleImportFolder = async () => {
    setLoading(true);
    try {
      await importFolder();
      await refreshSessions();
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteSession = async (sessionId: number) => {
    if (!window.confirm("Delete this session and all lap data? This cannot be undone.")) return;
    setLoading(true);
    try {
      await deleteSession(sessionId);
      if (selectedId === sessionId) {
        setSelectedId(null);
        setDetail(null);
        setFuel(null);
        setTires(null);
      }
      await refreshSessions();
    } finally {
      setLoading(false);
    }
  };

  const handleClearDatabase = async () => {
    if (
      !window.confirm(
        "Delete all imported sessions from the local database? This cannot be undone.",
      )
    ) {
      return;
    }
    setLoading(true);
    try {
      await clearDatabase();
      setSelectedId(null);
      setDetail(null);
      setSelectedLaps([]);
      setHighlightedLaps([]);
      setTraces([]);
      setFuel(null);
      setTires(null);
      await refreshSessions();
    } finally {
      setLoading(false);
    }
  };

  const handleStartLive = async () => {
    setAppTab("live");
    await startLiveMonitor();
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>
          PitWall Desktop
          {version && <span className="app-version">v{version}</span>}
        </h1>
        <nav className="app-tabs">
          <button
            type="button"
            className={tab === "analyze" ? "tab active" : "tab"}
            onClick={() => setAppTab("analyze")}
          >
            Analyze
          </button>
          <button
            type="button"
            className={tab === "live" ? "tab active" : "tab"}
            onClick={() => setAppTab("live")}
          >
            Live
          </button>
          <button
            type="button"
            className={tab === "settings" ? "tab active" : "tab"}
            onClick={() => setAppTab("settings")}
            aria-label="Settings"
          >
            Settings
          </button>
        </nav>
        <span className="subtitle">iRacing telemetry — post-session & live</span>
        {importStatus.active && (
          <div className="import-progress">
            <div
              className="import-progress-bar"
              style={{ width: `${importStatus.progressPct}%` }}
            />
            <span className="import-progress-label">{importStatus.message}</span>
          </div>
        )}
        {importStatus.message !== "Idle" && !importStatus.active && (
          <span className="status-pill">{importStatus.message}</span>
        )}
      </header>

      <ConfigBanner
        config={config}
        configLoading={configLoading}
        importStatus={importStatus}
        onStartLive={handleStartLive}
      />

      {sessionError && tab === "analyze" && (
        <div className="error-banner panel">{sessionError}</div>
      )}

      {tab === "settings" ? (
        <main className="settings-main">
          <SettingsPage />
        </main>
      ) : tab === "live" ? (
        <main className="live-main">
          <LivePanel onOpenSettings={() => setAppTab("settings")} />
        </main>
      ) : (
        <main className="layout">
          <aside className="sidebar">
            <SessionBrowser
              sessions={sessions}
              selectedId={selectedId}
              importStatus={importStatus}
              onSelect={setSelectedId}
              onImportFile={handleImportFile}
              onImportFolder={handleImportFolder}
              onDeleteSession={handleDeleteSession}
              onClearDatabase={handleClearDatabase}
              loading={loading}
            />
          </aside>
          <section className="main-content">
            {sessionLoading && selectedId != null ? (
              <div className="empty-state center panel">
                <p className="muted">Loading session…</p>
              </div>
            ) : !detail ? (
              <div className="empty-state center">
                <p>Select a session or import telemetry to get started.</p>
              </div>
            ) : (
              <>
                <div className="session-title">
                  <h2>{detail.session.track}</h2>
                  <span>{detail.session.car}</span>
                </div>
                <LapTable
                  laps={detail.laps}
                  selectedLaps={selectedLaps}
                  highlightedLaps={highlightedLaps}
                  onToggleLap={handleToggleLap}
                />
                <CoachPanel
                  sessionId={detail.session.id}
                  highlightedLaps={highlightedLaps}
                  onHighlightLaps={setHighlightedLaps}
                  onCompareLaps={handleCompareLaps}
                />
                <SessionStandingsPanel sessionId={detail.session.id} />
                <LapCompareChart traces={traces} />
                <FuelTirePanel fuel={fuel} tires={tires} />
              </>
            )}
          </section>
        </main>
      )}
    </div>
  );
}

export default App;
