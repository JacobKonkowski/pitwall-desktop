import { useCallback, useEffect, useState } from "react";
import {
  checkVrHudHealth,
  closeDesktopOverlay,
  formatDelta,
  formatLapTime,
  getAudioCoachStatus,
  getLiveSnapshot,
  getLiveStatus,
  getSettings,
  getVrOverlayStatus,
  isDesktopOverlayOpen,
  onLiveStatus,
  onLiveTelemetry,
  openDesktopOverlay,
  openVrHudPreview,
  saveSettings,
  startAudioCoach,
  startLiveMonitor,
  startVrOverlay,
  stopAudioCoach,
  stopLiveMonitor,
  stopVrOverlay,
} from "../lib/api";
import type { AppSettings, AudioCoachStatus, LiveSnapshot, LiveStatus, VrOverlayStatus } from "../lib/types";
import { SessionLeaderboard } from "./SessionLeaderboard";

function stateClass(state: LiveStatus["state"]): string {
  switch (state) {
    case "connected":
      return "live-pill live-pill-ok";
    case "waitingForSession":
    case "reconnecting":
      return "live-pill live-pill-wait";
    case "error":
      return "live-pill live-pill-err";
    default:
      return "live-pill";
  }
}

function stateLabel(state: LiveStatus["state"]): string {
  switch (state) {
    case "connected":
      return "Connected";
    case "waitingForSession":
      return "Waiting";
    case "reconnecting":
      return "Reconnecting";
    case "error":
      return "Error";
    default:
      return "Disconnected";
  }
}

function formatPosition(snap: LiveSnapshot): string {
  const cls = snap.playerClassPosition;
  const overall = snap.playerPosition;
  if (cls != null && overall != null && cls !== overall) {
    return `P${cls} (P${overall})`;
  }
  return `P${cls ?? overall}`;
}

export function LivePanel() {
  const [status, setStatus] = useState<LiveStatus>({
    state: "disconnected",
    message: "Live monitor stopped",
  });
  const [snap, setSnap] = useState<LiveSnapshot | null>(null);
  const [running, setRunning] = useState(false);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [vrStatus, setVrStatus] = useState<VrOverlayStatus | null>(null);
  const [vrHudHealthy, setVrHudHealthy] = useState<boolean | null>(null);
  const [audioStatus, setAudioStatus] = useState<AudioCoachStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [liveStatus, snapshot, overlay, vr, audio, cfg] = await Promise.all([
      getLiveStatus(),
      getLiveSnapshot(),
      isDesktopOverlayOpen(),
      getVrOverlayStatus(),
      getAudioCoachStatus(),
      getSettings(),
    ]);
    setStatus(liveStatus);
    setSnap(snapshot.track ? snapshot : null);
    setRunning(liveStatus.state !== "disconnected");
    setOverlayOpen(overlay);
    setVrStatus(vr);
    setAudioStatus(audio);
    setSettings(cfg);
  }, []);

  useEffect(() => {
    refresh();
    let audioPoll: ReturnType<typeof setInterval> | undefined;
    const unsubs = Promise.all([
      onLiveTelemetry((payload) => {
        setSnap(payload);
        setRunning(true);
      }),
      onLiveStatus((payload) => {
        setStatus(payload);
        setRunning(payload.state !== "disconnected");
      }),
    ]);
    audioPoll = setInterval(() => {
      getAudioCoachStatus().then(setAudioStatus).catch(() => {});
    }, 2000);
    return () => {
      if (audioPoll) clearInterval(audioPoll);
      unsubs.then((fns) => fns.forEach((fn) => fn()));
    };
  }, [refresh]);

  useEffect(() => {
    if (!vrStatus?.active) {
      setVrHudHealthy(null);
      return;
    }
    let cancelled = false;
    const poll = () => {
      checkVrHudHealth()
        .then((ok) => {
          if (!cancelled) setVrHudHealthy(ok);
        })
        .catch(() => {
          if (!cancelled) setVrHudHealthy(false);
        });
    };
    poll();
    const id = setInterval(poll, 3000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [vrStatus?.active]);

  const handleStart = async () => {
    setError(null);
    try {
      await startLiveMonitor();
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleStop = async () => {
    setError(null);
    try {
      await stopLiveMonitor();
      setSnap(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleToggleOverlay = async () => {
    setError(null);
    try {
      if (overlayOpen) {
        await closeDesktopOverlay();
      } else {
        await openDesktopOverlay();
      }
      setOverlayOpen(await isDesktopOverlayOpen());
    } catch (e) {
      setError(String(e));
    }
  };

  const handleToggleVr = async () => {
    setError(null);
    try {
      if (vrStatus?.active) {
        await stopVrOverlay();
        setVrHudHealthy(null);
      } else {
        await startVrOverlay();
      }
      setVrStatus(await getVrOverlayStatus());
    } catch (e) {
      setError(String(e));
    }
  };

  const handlePreviewVr = async () => {
    setError(null);
    try {
      await openVrHudPreview();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleToggleAudio = async () => {
    setError(null);
    try {
      if (audioStatus?.active) {
        await stopAudioCoach();
      } else {
        await startAudioCoach();
      }
      setAudioStatus(await getAudioCoachStatus());
      const cfg = await getSettings();
      setSettings(cfg);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSaveSettings = async (patch: Partial<AppSettings>) => {
    if (!settings) return;
    const next = { ...settings, ...patch };
    await saveSettings(next);
    setSettings(next);
  };

  const sectorProgress = (sectorNum: number): number => {
    if (!snap) return 0;
    const sector = snap.sectors.find((s) => s.sectorNum === sectorNum);
    if (sector?.completed) return 100;
    if (snap.currentSector === sectorNum) {
      const bounds = [0, 0.33, 0.66, 1];
      const start = bounds[sectorNum - 1] ?? 0;
      const end = bounds[sectorNum] ?? 1;
      const span = end - start;
      if (span <= 0) return 0;
      return Math.min(100, Math.max(0, ((snap.lapDistPct - start) / span) * 100));
    }
    return 0;
  };

  return (
    <div className="live-panel">
      <div className="panel live-header-panel">
        <div className="live-header-row">
          <div>
            <h2>Live telemetry</h2>
            <span className="muted">Real-time data from iRacing shared memory</span>
          </div>
          <span className={stateClass(status.state)}>{stateLabel(status.state)}</span>
        </div>
        <p className="live-status-msg">{status.message}</p>
        <div className="btn-row live-actions">
          {!running ? (
            <button onClick={handleStart}>Start live monitor</button>
          ) : (
            <button onClick={handleStop}>Stop</button>
          )}
          <button onClick={handleToggleOverlay} disabled={!running}>
            {overlayOpen ? "Close desktop overlay" : "Pop out overlay (desktop)"}
          </button>
          <button onClick={handleToggleVr} disabled={!running}>
            {vrStatus?.active ? "Stop in-headset HUD" : "Start in-headset HUD"}
          </button>
          <button
            onClick={handleToggleAudio}
            disabled={!running}
            className={audioStatus?.active ? "tab active" : ""}
          >
            {audioStatus?.active ? "Stop audio coach" : "Start audio coach"}
          </button>
          <button onClick={() => setSettingsOpen((v) => !v)}>Settings</button>
        </div>
        {vrStatus?.message && (
          <p className="muted small">
            {vrStatus.runtime || "VR"} — {vrStatus.message}
          </p>
        )}
        {vrStatus?.active && vrStatus.hudUrl && (
          <div className="vr-hud-help panel">
            <div className="vr-hud-help-header">
              <h3>In-headset setup (no SteamVR)</h3>
              {vrHudHealthy === true && (
                <span className="vr-health-pill vr-health-ok">HUD server ready</span>
              )}
              {vrHudHealthy === false && (
                <span className="vr-health-pill vr-health-err">HUD server not responding</span>
              )}
            </div>
            <p className="muted small">
              Use iRacing in <strong>OpenXR</strong> mode. Add this URL as a{" "}
              <strong>Web Dashboard</strong> tab in{" "}
              <a href="https://openkneeboard.com/" target="_blank" rel="noreferrer">
                OpenKneeboard
              </a>{" "}
              (same approach as RaceLab/iOverlay):
            </p>
            <div className="vr-url-row">
              <code>{vrStatus.hudUrl}</code>
              <button
                type="button"
                onClick={() => navigator.clipboard.writeText(vrStatus.hudUrl)}
              >
                Copy URL
              </button>
              <button type="button" onClick={handlePreviewVr}>
                Preview in browser
              </button>
            </div>
            <ol className="vr-steps muted small">
              <li>
                Install{" "}
                <a href="https://openkneeboard.com/" target="_blank" rel="noreferrer">
                  OpenKneeboard
                </a>
              </li>
              <li>Settings → Tabs → Add tab → Web Dashboard → paste URL above</li>
              <li>Preview in browser first to confirm data appears, then use in VR</li>
              <li>Start iRacing in VR (OpenXR), join session, bind recenter in OpenKneeboard</li>
            </ol>
          </div>
        )}
        {audioStatus?.active && audioStatus.lastMessage && (
          <p className="audio-coach-last muted small">
            <strong>Last spoken:</strong> {audioStatus.lastMessage}
          </p>
        )}
        {error && <p className="live-error">{error}</p>}
      </div>

      {settingsOpen && settings && (
        <div className="panel live-settings">
          <h3>Live / overlay settings</h3>
          <label className="settings-row">
            <span>Ollama URL</span>
            <input
              value={settings.ollamaUrl}
              onChange={(e) => handleSaveSettings({ ollamaUrl: e.target.value })}
            />
          </label>
          <label className="settings-row">
            <span>Ollama model</span>
            <input
              value={settings.ollamaModel}
              onChange={(e) => handleSaveSettings({ ollamaModel: e.target.value })}
            />
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.vrOverlayEnabled}
              onChange={(e) => handleSaveSettings({ vrOverlayEnabled: e.target.checked })}
            />
            <span>Auto-start in-headset HUD server with live monitor</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioCoachEnabled}
              onChange={(e) => handleSaveSettings({ audioCoachEnabled: e.target.checked })}
            />
            <span>Auto-start audio coach with live monitor</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioPackAlertsEnabled}
              onChange={(e) => handleSaveSettings({ audioPackAlertsEnabled: e.target.checked })}
            />
            <span>Spotter pack alerts (car left/right, 3-wide)</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioFlagsEnabled}
              onChange={(e) => handleSaveSettings({ audioFlagsEnabled: e.target.checked })}
            />
            <span>Flag callouts (yellow, green, blue, checkered)</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioIncidentsEnabled}
              onChange={(e) => handleSaveSettings({ audioIncidentsEnabled: e.target.checked })}
            />
            <span>Incident count callouts</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioFuelRaceEnabled}
              onChange={(e) => handleSaveSettings({ audioFuelRaceEnabled: e.target.checked })}
            />
            <span>Race fuel-to-finish calls</span>
          </label>
          <label className="settings-row">
            <span>Fuel warning (liters)</span>
            <input
              type="number"
              min={0}
              step={0.5}
              value={settings.audioCoachFuelThreshold}
              onChange={(e) =>
                handleSaveSettings({ audioCoachFuelThreshold: parseFloat(e.target.value) || 0 })
              }
            />
          </label>
        </div>
      )}

      {!snap ? (
        <div className="empty-state center panel">
          <p>
            {running
              ? "Waiting for iRacing session telemetry…"
              : "Start the live monitor while iRacing is running (irsdkEnableMem=1)."}
          </p>
        </div>
      ) : (
        <>
          <div className="panel live-meta">
            <div>
              <strong>{snap.track || "Unknown track"}</strong>
              <span className="muted"> · {snap.car}</span>
            </div>
            <span className="muted">{snap.sessionType}</span>
          </div>

          <div className="live-metrics">
            <div className="live-metric-card">
              <span className="label">Lap</span>
              <span className="value">{snap.lap}</span>
            </div>
            <div className="live-metric-card">
              <span className="label">Lap time</span>
              <span className="value">{formatLapTime(snap.lapTimeMs)}</span>
            </div>
            <div className="live-metric-card">
              <span className="label">Δ best</span>
              <span className={`value ${snap.deltaToBestMs != null && snap.deltaToBestMs > 0 ? "slow" : "fast"}`}>
                {formatDelta(snap.deltaToBestMs)}
              </span>
            </div>
            {(snap.playerPosition != null || snap.playerClassPosition != null) && (
              <div className="live-metric-card">
                <span className="label">Position</span>
                <span className="value">{formatPosition(snap)}</span>
              </div>
            )}
            {snap.deltaToSessionBestMs != null && (
              <div className="live-metric-card">
                <span className="label">Δ session best</span>
                <span className={`value ${snap.deltaToSessionBestMs > 0 ? "slow" : "fast"}`}>
                  {formatDelta(snap.deltaToSessionBestMs)}
                </span>
              </div>
            )}
            {snap.deltaToSessionOptimalMs != null && (
              <div className="live-metric-card">
                <span className="label">Δ session optimal</span>
                <span className={`value ${snap.deltaToSessionOptimalMs > 0 ? "slow" : "fast"}`}>
                  {formatDelta(snap.deltaToSessionOptimalMs)}
                </span>
              </div>
            )}
            <div className="live-metric-card">
              <span className="label">Last lap</span>
              <span className="value">{formatLapTime(snap.lastLapMs)}</span>
            </div>
            <div className="live-metric-card">
              <span className="label">Fuel</span>
              <span className="value">{snap.fuelLevel.toFixed(1)} L</span>
            </div>
            <div className="live-metric-card">
              <span className="label">Speed</span>
              <span className="value">{snap.speed.toFixed(0)}</span>
            </div>
          </div>

          <SessionLeaderboard competitors={snap.competitors} />

          <div className="panel">
            <h3>Sector progress</h3>
            <div className="sector-bars">
              {[1, 2, 3].map((n) => {
                const sector = snap.sectors.find((s) => s.sectorNum === n);
                const pct = sector?.completed ? 100 : sectorProgress(n);
                return (
                  <div key={n} className="sector-bar-row">
                    <span className="sector-label">S{n}</span>
                    <div className="sector-bar-track">
                      <div className="sector-bar-fill" style={{ width: `${pct}%` }} />
                    </div>
                    <span className="sector-time">{formatLapTime(sector?.timeMs ?? null)}</span>
                  </div>
                );
              })}
            </div>
          </div>

          <div className="panel">
            <h3>Tire temps (°C)</h3>
            <div className="tire-temps">
              <span>LF {snap.lfTemp.toFixed(0)}</span>
              <span>RF {snap.rfTemp.toFixed(0)}</span>
              <span>LR {snap.lrTemp.toFixed(0)}</span>
              <span>RR {snap.rrTemp.toFixed(0)}</span>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
