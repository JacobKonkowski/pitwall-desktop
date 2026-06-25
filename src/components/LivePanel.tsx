import { useCallback, useEffect, useState } from "react";
import {
  checkVrHudHealth,
  closeDesktopOverlay,
  formatDelta,
  formatLapTime,
  getAudioCoachStatus,
  getLiveSnapshot,
  getLiveStatus,
  getNativeVrStatus,
  getSettings,
  getVrLayerDiagnostics,
  getVrOverlayStatus,
  installVrLayer,
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
  uninstallVrLayer,
} from "../lib/api";
import type {
  AppSettings,
  AudioCoachStatus,
  LiveSnapshot,
  LiveStatus,
  NativeVrStatus,
  VrLayerDiagnostics,
  VrOverlayStatus,
  WidgetPlacement,
} from "../lib/types";
import { defaultOverlayLayout, WIDGET_KINDS, WIDGET_LABELS } from "../lib/types";
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
  const [nativeVr, setNativeVr] = useState<NativeVrStatus | null>(null);
  const [layerDiag, setLayerDiag] = useState<VrLayerDiagnostics | null>(null);
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
    setSnap(liveStatus.state === "connected" ? snapshot : null);
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
    if (!vrStatus?.active || vrStatus.mode === "native") {
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
  }, [vrStatus?.active, vrStatus?.mode]);

  useEffect(() => {
    if (!vrStatus?.active || vrStatus.mode !== "native") {
      setNativeVr(null);
      return;
    }
    let cancelled = false;
    const poll = () => {
      getNativeVrStatus()
        .then((s) => {
          if (!cancelled) setNativeVr(s);
        })
        .catch(() => {});
    };
    poll();
    const id = setInterval(poll, 2000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [vrStatus?.active, vrStatus?.mode]);

  const refreshLayerDiag = useCallback(async () => {
    if (settings?.vrMode !== "native") {
      setLayerDiag(null);
      return;
    }
    try {
      setLayerDiag(await getVrLayerDiagnostics());
    } catch {
      setLayerDiag(null);
    }
  }, [settings?.vrMode]);

  useEffect(() => {
    refreshLayerDiag();
  }, [refreshLayerDiag, vrStatus?.layerInstalled, settings?.vrMode]);

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

  const handleInstallLayer = async () => {
    setError(null);
    try {
      await installVrLayer();
      setVrStatus(await getVrOverlayStatus());
      setNativeVr(await getNativeVrStatus());
      await refreshLayerDiag();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleUninstallLayer = async () => {
    setError(null);
    try {
      await uninstallVrLayer();
      setVrStatus(await getVrOverlayStatus());
      setNativeVr(await getNativeVrStatus());
      await refreshLayerDiag();
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

  const handlePatchWidget = async (index: number, patch: Partial<WidgetPlacement>) => {
    if (!settings) return;
    const widgets = settings.overlayLayout.widgets.map((w, i) =>
      i === index ? { ...w, ...patch } : w,
    );
    await handleSaveSettings({ overlayLayout: { ...settings.overlayLayout, widgets } });
  };

  const handlePatchLayout = async (patch: Partial<AppSettings["overlayLayout"]>) => {
    if (!settings) return;
    await handleSaveSettings({ overlayLayout: { ...settings.overlayLayout, ...patch } });
  };

  const handleResetLayout = async () => {
    await handleSaveSettings({ overlayLayout: defaultOverlayLayout() });
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
        {settings?.vrMode === "native" && (
          <div className="vr-hud-help panel">
            <div className="vr-hud-help-header">
              <h3>In-headset HUD (native)</h3>
              {layerDiag?.ready ? (
                <span className="vr-health-pill vr-health-ok">Layer ready</span>
              ) : layerDiag?.registered ? (
                <span className="vr-health-pill vr-health-err">Layer needs attention</span>
              ) : (
                <span className="vr-health-pill vr-health-err">Layer not installed</span>
              )}
            </div>
            <p className="muted small">
              PitWall composites the HUD in your headset through its own OpenXR layer — no
              OpenKneeboard required. Install once, restart iRacing in OpenXR mode, then start the
              in-headset HUD while the live monitor is running.
            </p>
            {vrStatus?.active && (
              <p className="muted small">
                {nativeVr?.compositorActive
                  ? "OpenXR layer is compositing in your headset."
                  : nativeVr?.telemetryPublishing
                    ? "Telemetry is publishing, but the OpenXR layer is not active. iRacing must be running in OpenXR VR (not desktop or OpenVR)."
                    : "Start iRacing on track in OpenXR VR — the layer reads telemetry once the sim is in your headset."}
              </p>
            )}
            {layerDiag && layerDiag.issues.length > 0 && (
              <ul className="vr-steps muted small">
                {layerDiag.issues.map((issue) => (
                  <li key={issue}>{issue}</li>
                ))}
              </ul>
            )}
            {layerDiag?.ready && (
              <p className="muted small">
                After reinstalling, fully quit and restart iRacing. Enable widgets under Settings →
                Overlay widgets.
              </p>
            )}
            <div className="btn-row">
              <button type="button" onClick={handleInstallLayer}>
                {layerDiag?.registered ? "Reinstall VR layer" : "Install VR layer"}
              </button>
              {layerDiag?.registered && (
                <button type="button" className="tab" onClick={handleUninstallLayer}>
                  Uninstall layer
                </button>
              )}
            </div>
          </div>
        )}
        {vrStatus?.active && vrStatus.mode !== "native" && vrStatus.hudUrl && (
          <div className="vr-hud-help panel">
            <div className="vr-hud-help-header">
              <h3>In-headset setup (OpenKneeboard fallback)</h3>
              {vrHudHealthy === true && (
                <span className="vr-health-pill vr-health-ok">HUD server ready</span>
              )}
              {vrHudHealthy === false && (
                <span className="vr-health-pill vr-health-err">HUD server not responding</span>
              )}
            </div>
            <p className="muted small">
              Web fallback mode. Use iRacing in <strong>OpenXR</strong> mode and add this URL as a{" "}
              <strong>Web Dashboard</strong> tab in{" "}
              <a href="https://openkneeboard.com/" target="_blank" rel="noreferrer">
                OpenKneeboard
              </a>
              . Switch to native mode in Settings to drop the OpenKneeboard dependency.
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
              <li>
                OpenKneeboard → <strong>Advanced</strong> → turn off header/footer (cleaner HUD)
              </li>
              <li>
                Join iRacing, get <strong>in the car</strong>, toggle OpenKneeboard, then{" "}
                <strong>Recenter</strong> while seated — loading-screen position is usually too low
              </li>
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
            <span>Auto-start in-headset HUD with live monitor</span>
          </label>
          <label className="settings-row">
            <span>VR mode</span>
            <select
              value={settings.vrMode}
              onChange={(e) => handleSaveSettings({ vrMode: e.target.value })}
            >
              <option value="native">Native (in-headset, no OpenKneeboard)</option>
              <option value="web">Web fallback (OpenKneeboard)</option>
            </select>
          </label>
          <label className="settings-row">
            <span>Field pace (coach)</span>
            <select
              value={settings.overlayLayout.fieldPaceMode}
              onChange={(e) => handlePatchLayout({ fieldPaceMode: e.target.value })}
            >
              <option value="best">Session best (FLD)</option>
              <option value="optimal">Session optimal (OPT)</option>
              <option value="both">Both</option>
            </select>
          </label>

          <div className="overlay-widgets-settings">
            <div className="overlay-widgets-header">
              <h4>Overlay widgets</h4>
              <button type="button" className="tab" onClick={handleResetLayout}>
                Reset layout
              </button>
            </div>
            <p className="muted small">
              Enable widgets to show them on both the desktop pop-out and the in-headset HUD.
              Drag and resize them on the desktop overlay; tune VR height, scale, and opacity here.
            </p>
            {settings.overlayLayout.widgets.map((w, i) => (
              <div key={WIDGET_KINDS[i]} className="overlay-widget-settings">
                <label className="settings-row checkbox">
                  <input
                    type="checkbox"
                    checked={w.enabled}
                    onChange={(e) => handlePatchWidget(i, { enabled: e.target.checked })}
                  />
                  <span>{WIDGET_LABELS[WIDGET_KINDS[i]]}</span>
                </label>
                {w.enabled && (
                  <div className="overlay-widget-vr">
                    <label className="settings-row">
                      <span>VR height ({w.vrOffsetY.toFixed(2)} m)</span>
                      <input
                        type="range"
                        min={-0.5}
                        max={0.5}
                        step={0.02}
                        value={w.vrOffsetY}
                        onChange={(e) =>
                          handlePatchWidget(i, { vrOffsetY: parseFloat(e.target.value) })
                        }
                      />
                    </label>
                    <label className="settings-row">
                      <span>VR scale ({w.vrScale.toFixed(1)}×)</span>
                      <input
                        type="range"
                        min={0.5}
                        max={2}
                        step={0.1}
                        value={w.vrScale}
                        onChange={(e) =>
                          handlePatchWidget(i, { vrScale: parseFloat(e.target.value) })
                        }
                      />
                    </label>
                    <label className="settings-row">
                      <span>VR opacity ({Math.round(w.vrOpacity * 100)}%)</span>
                      <input
                        type="range"
                        min={0.2}
                        max={1}
                        step={0.05}
                        value={w.vrOpacity}
                        onChange={(e) =>
                          handlePatchWidget(i, { vrOpacity: parseFloat(e.target.value) })
                        }
                      />
                    </label>
                  </div>
                )}
              </div>
            ))}
          </div>
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
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioGapAlertsEnabled ?? true}
              onChange={(e) => handleSaveSettings({ audioGapAlertsEnabled: e.target.checked })}
            />
            <span>Gap alerts (ahead/behind on lap and when closing)</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioPaceEnabled ?? true}
              onChange={(e) => handleSaveSettings({ audioPaceEnabled: e.target.checked })}
            />
            <span>Lap and sector pace callouts</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioStrategyEnabled ?? true}
              onChange={(e) => handleSaveSettings({ audioStrategyEnabled: e.target.checked })}
            />
            <span>Strategy calls (fuel, race clock, pits open)</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioRaceClockEnabled ?? true}
              onChange={(e) => handleSaveSettings({ audioRaceClockEnabled: e.target.checked })}
            />
            <span>Race clock milestones (5 laps, 5 minutes, etc.)</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioPitsOpenEnabled ?? true}
              onChange={(e) => handleSaveSettings({ audioPitsOpenEnabled: e.target.checked })}
            />
            <span>Pits open callout</span>
          </label>
          <label className="settings-row checkbox">
            <input
              type="checkbox"
              checked={settings.audioPackClearEnabled ?? false}
              onChange={(e) => handleSaveSettings({ audioPackClearEnabled: e.target.checked })}
            />
            <span>Spotter clear callout</span>
          </label>
          <label className="settings-row">
            <span>Radio chatter level</span>
            <select
              value={settings.audioCoachChatterLevel ?? "normal"}
              onChange={(e) =>
                handleSaveSettings({
                  audioCoachChatterLevel: e.target.value as "minimal" | "normal" | "verbose",
                })
              }
            >
              <option value="minimal">Minimal (safety only)</option>
              <option value="normal">Normal</option>
              <option value="verbose">Verbose</option>
            </select>
          </label>
          <label className="settings-row">
            <span>Speech rate ({settings.audioCoachRate.toFixed(1)}×)</span>
            <input
              type="range"
              min={0.5}
              max={2}
              step={0.1}
              value={settings.audioCoachRate}
              onChange={(e) =>
                handleSaveSettings({ audioCoachRate: parseFloat(e.target.value) })
              }
            />
          </label>
          <label className="settings-row">
            <span>Speech volume ({Math.round(settings.audioCoachVolume * 100)}%)</span>
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.audioCoachVolume}
              onChange={(e) =>
                handleSaveSettings({ audioCoachVolume: parseFloat(e.target.value) })
              }
            />
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
              <strong>{snap.track || "Live session"}</strong>
              <span className="muted"> · {snap.car || "—"}</span>
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
