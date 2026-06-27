import { useCallback, useEffect, useState } from "react";
import {
  buildOpenKneeboardUrl,
  checkVrHudHealth,
  closeDesktopOverlay,
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
  VrLayerDiagnostics,
  VrOverlayStatus,
} from "../lib/types";
import { LiveMetricsGrid, LiveRaceContext } from "./live/LiveMetricsGrid";
import { LiveProximityPanel } from "./live/LiveProximityPanel";
import {
  LiveSectorsPanel,
  LiveTiresPanel,
  LiveTrafficPanel,
} from "./live/LiveTrafficPanel";
import { SessionLeaderboard } from "./SessionLeaderboard";

interface Props {
  onOpenSettings?: () => void;
}

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

export function LivePanel({ onOpenSettings }: Props) {
  const [status, setStatus] = useState<LiveStatus>({
    state: "disconnected",
    message: "Live monitor stopped",
  });
  const [snap, setSnap] = useState<LiveSnapshot | null>(null);
  const [running, setRunning] = useState(false);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [vrStatus, setVrStatus] = useState<VrOverlayStatus | null>(null);
  const [layerDiag, setLayerDiag] = useState<VrLayerDiagnostics | null>(null);
  const [vrHudHealthy, setVrHudHealthy] = useState<boolean | null>(null);
  const [audioStatus, setAudioStatus] = useState<AudioCoachStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
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
    const audioPoll = setInterval(() => {
      getAudioCoachStatus().then(setAudioStatus).catch(() => {});
    }, 2000);
    return () => {
      clearInterval(audioPoll);
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
      return;
    }
    const id = setInterval(() => {
      getNativeVrStatus().catch(() => {});
    }, 2000);
    return () => clearInterval(id);
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

  const hudUrl =
    settings && vrStatus?.hudUrl
      ? buildOpenKneeboardUrl(settings, vrStatus.hudUrl.split("?")[0])
      : vrStatus?.hudUrl;

  const handlers = {
    start: async () => {
      setError(null);
      try {
        await startLiveMonitor();
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    stop: async () => {
      setError(null);
      try {
        await stopLiveMonitor();
        setSnap(null);
        await refresh();
      } catch (e) {
        setError(String(e));
      }
    },
    toggleOverlay: async () => {
      setError(null);
      try {
        if (overlayOpen) await closeDesktopOverlay();
        else await openDesktopOverlay();
        setOverlayOpen(await isDesktopOverlayOpen());
      } catch (e) {
        setError(String(e));
      }
    },
    toggleVr: async () => {
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
    },
    toggleAudio: async () => {
      setError(null);
      try {
        if (audioStatus?.active) await stopAudioCoach();
        else await startAudioCoach();
        setAudioStatus(await getAudioCoachStatus());
      } catch (e) {
        setError(String(e));
      }
    },
    installLayer: async () => {
      setError(null);
      try {
        await installVrLayer();
        setVrStatus(await getVrOverlayStatus());
        await refreshLayerDiag();
      } catch (e) {
        setError(String(e));
      }
    },
    uninstallLayer: async () => {
      setError(null);
      try {
        await uninstallVrLayer();
        setVrStatus(await getVrOverlayStatus());
        await refreshLayerDiag();
      } catch (e) {
        setError(String(e));
      }
    },
    previewVr: async () => {
      setError(null);
      try {
        await openVrHudPreview();
      } catch (e) {
        setError(String(e));
      }
    },
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
            <button type="button" onClick={handlers.start}>
              Start live monitor
            </button>
          ) : (
            <button type="button" onClick={handlers.stop}>
              Stop
            </button>
          )}
          <button type="button" onClick={handlers.toggleOverlay} disabled={!running}>
            {overlayOpen ? "Close desktop overlay" : "Pop out overlay"}
          </button>
          <button type="button" onClick={handlers.toggleVr} disabled={!running}>
            {vrStatus?.active ? "Stop in-headset HUD" : "Start in-headset HUD"}
          </button>
          <button
            type="button"
            onClick={handlers.toggleAudio}
            disabled={!running}
            className={audioStatus?.active ? "tab active" : ""}
          >
            {audioStatus?.active ? "Stop audio coach" : "Start audio coach"}
          </button>
          {onOpenSettings && (
            <button type="button" onClick={onOpenSettings}>
              Settings
            </button>
          )}
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
              Install the OpenXR layer once, restart iRacing in OpenXR mode, then enable widgets in
              Settings.
            </p>
            {layerDiag && layerDiag.issues.length > 0 && (
              <ul className="vr-steps muted small">
                {layerDiag.issues.map((issue) => (
                  <li key={issue}>{issue}</li>
                ))}
              </ul>
            )}
            <div className="btn-row">
              <button type="button" onClick={handlers.installLayer}>
                {layerDiag?.registered ? "Reinstall VR layer" : "Install VR layer"}
              </button>
              {layerDiag?.registered && (
                <button type="button" className="tab" onClick={handlers.uninstallLayer}>
                  Uninstall layer
                </button>
              )}
            </div>
          </div>
        )}
        {vrStatus?.active && vrStatus.mode !== "native" && hudUrl && (
          <div className="vr-hud-help panel">
            <div className="vr-hud-help-header">
              <h3>In-headset setup (OpenKneeboard)</h3>
              {vrHudHealthy === true && (
                <span className="vr-health-pill vr-health-ok">HUD server ready</span>
              )}
              {vrHudHealthy === false && (
                <span className="vr-health-pill vr-health-err">HUD server not responding</span>
              )}
            </div>
            <p className="muted small">
              Add this URL as a Web Dashboard tab in OpenKneeboard. Layout and field pace follow your
              Settings.
            </p>
            <div className="vr-url-row">
              <code>{hudUrl}</code>
              <button type="button" onClick={() => navigator.clipboard.writeText(hudUrl)}>
                Copy URL
              </button>
              <button type="button" onClick={handlers.previewVr}>
                Preview in browser
              </button>
            </div>
          </div>
        )}
        {audioStatus?.active && audioStatus.lastMessage && (
          <p className="audio-coach-last muted small">
            <strong>Last spoken:</strong> {audioStatus.lastMessage}
          </p>
        )}
        {error && <p className="live-error">{error}</p>}
      </div>

      {!snap ? (
        <div className="empty-state center panel">
          <p>
            {running
              ? "Waiting for iRacing session telemetry…"
              : "Start the live monitor while iRacing is running (irsdkEnableMem=1)."}
          </p>
        </div>
      ) : (
        <div className="live-dashboard">
          <div className="live-dashboard-col">
            <div className="panel live-meta">
              <div>
                <strong>{snap.track || "Live session"}</strong>
                <span className="muted"> · {snap.car || "—"}</span>
              </div>
              <span className="muted">{snap.sessionType}</span>
            </div>
            <LiveRaceContext snap={snap} />
            <LiveMetricsGrid snap={snap} />
            <LiveSectorsPanel snap={snap} />
            <LiveTiresPanel snap={snap} />
          </div>
          <div className="live-dashboard-col">
            <LiveTrafficPanel snap={snap} />
            <LiveProximityPanel snap={snap} />
          </div>
          <SessionLeaderboard competitors={snap.competitors} />
        </div>
      )}
    </div>
  );
}
