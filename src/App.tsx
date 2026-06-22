import { useCallback, useEffect, useState } from "react";

import {

  checkIracingConfig,

  clearDatabase,

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

import { ConfigBanner } from "./components/ConfigBanner";

import { FuelTirePanel } from "./components/FuelTirePanel";

import { LapCompareChart } from "./components/LapCompareChart";

import { LapTable } from "./components/LapTable";

import { LivePanel } from "./components/LivePanel";

import { SessionBrowser } from "./components/SessionBrowser";

import "./App.css";



type AppTab = "analyze" | "live";



function App() {

  const [tab, setTab] = useState<AppTab>("analyze");

  const [sessions, setSessions] = useState<SessionSummary[]>([]);

  const [selectedId, setSelectedId] = useState<number | null>(null);

  const [detail, setDetail] = useState<SessionDetail | null>(null);

  const [selectedLaps, setSelectedLaps] = useState<number[]>([]);

  const [highlightedLaps, setHighlightedLaps] = useState<number[]>([]);

  const [traces, setTraces] = useState<LapTrace[]>([]);

  const [fuel, setFuel] = useState<FuelSummary | null>(null);

  const [tires, setTires] = useState<TireSummary | null>(null);

  const [config, setConfig] = useState<IracingConfigCheck | null>(null);

  const [importStatus, setImportStatus] = useState<ImportStatus>({

    active: false,

    currentFile: null,

    progressPct: 0,

    message: "Idle",

  });

  const [loading, setLoading] = useState(false);



  const refreshSessions = useCallback(async () => {

    const list = await listSessions();

    setSessions(list);

  }, []);



  const loadSession = useCallback(async (sessionId: number) => {

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

  }, []);



  useEffect(() => {

    refreshSessions();

    checkIracingConfig().then(setConfig);



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

    if (selectedId != null) {

      loadSession(selectedId);

    }

  }, [selectedId, loadSession]);



  useEffect(() => {

    if (selectedLaps.length === 0) {

      setTraces([]);

      return;

    }

    getLapTraces(selectedLaps).then(setTraces);

  }, [selectedLaps]);



  const handleToggleLap = (lapId: number) => {

    setSelectedLaps((prev) => {

      if (prev.includes(lapId)) return prev.filter((id) => id !== lapId);

      if (prev.length >= 2) return [prev[1], lapId];

      return [...prev, lapId];

    });

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



  const handleClearDatabase = async () => {

    if (

      !window.confirm(

        "Delete all imported sessions from the local database? This cannot be undone."

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

    setTab("live");

    await startLiveMonitor();

  };



  return (

    <div className="app">

      <header className="app-header">

        <h1>PitWall Desktop</h1>

        <nav className="app-tabs">

          <button

            type="button"

            className={tab === "analyze" ? "tab active" : "tab"}

            onClick={() => setTab("analyze")}

          >

            Analyze

          </button>

          <button

            type="button"

            className={tab === "live" ? "tab active" : "tab"}

            onClick={() => setTab("live")}

          >

            Live

          </button>

        </nav>

        <span className="subtitle">iRacing telemetry — post-session & live</span>

        {importStatus.active && (

          <div className="import-progress">

            <div className="import-progress-bar" style={{ width: `${importStatus.progressPct}%` }} />

            <span className="import-progress-label">{importStatus.message}</span>

          </div>

        )}

        {importStatus.message !== "Idle" && !importStatus.active && (

          <span className="status-pill">{importStatus.message}</span>

        )}

      </header>



      <ConfigBanner

        config={config}

        importStatus={importStatus}

        onStartLive={handleStartLive}

      />



      {tab === "live" ? (

        <main className="live-main">

          <LivePanel />

        </main>

      ) : (

        <main className="layout">

          <aside className="sidebar">

            <SessionBrowser

              sessions={sessions}

              selectedId={selectedId}

              onSelect={setSelectedId}

              onImportFile={handleImportFile}

              onImportFolder={handleImportFolder}

              onClearDatabase={handleClearDatabase}

              loading={loading}

            />

          </aside>



          <section className="main-content">

            {!detail ? (

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

                />

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

