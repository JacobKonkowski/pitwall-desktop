import { useCallback, useEffect, useRef, useState } from "react";
import { getLiveSnapshot, getSettings, onLiveTelemetry, saveSettings } from "../lib/api";
import type { AppSettings, LiveSnapshot, WidgetPlacement } from "../lib/types";
import { WIDGET_KINDS } from "../lib/types";
import { Widget } from "../widgets";
import { hasLiveData } from "../widgets/format";
import "../overlay.css";

const MIN_W = 160;
const MIN_H = 90;

export function OverlayView() {
  const [snap, setSnap] = useState<LiveSnapshot | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const draggingRef = useRef(false);

  useEffect(() => {
    getLiveSnapshot().then((s) => {
      if (s.track) setSnap(s);
    });
    getSettings().then(setSettings);
    let unlisten: (() => void) | undefined;
    onLiveTelemetry((payload) => setSnap(payload)).then((fn) => {
      unlisten = fn;
    });
    // Pick up enable/disable and field-pace changes made in the main window.
    const poll = setInterval(() => {
      if (draggingRef.current) return;
      getSettings().then(setSettings).catch(() => {});
    }, 2000);
    return () => {
      unlisten?.();
      clearInterval(poll);
    };
  }, []);

  const persist = useCallback(async (index: number, patch: Partial<WidgetPlacement>) => {
    // Re-read so concurrent main-window edits are not clobbered by a drag.
    const latest = await getSettings();
    const widgets = latest.overlayLayout.widgets.map((w, i) =>
      i === index ? { ...w, ...patch } : w,
    );
    const next = { ...latest, overlayLayout: { ...latest.overlayLayout, widgets } };
    setSettings(next);
    await saveSettings(next);
  }, []);

  if (!settings) {
    return (
      <div className="overlay-shell">
        <span className="overlay-wait">Loading overlay…</span>
      </div>
    );
  }

  const enabled = settings.overlayLayout.widgets
    .map((w, i) => ({ w, i }))
    .filter(({ w }) => w.enabled);

  return (
    <div className="overlay-shell">
      {!hasLiveData(snap) ? (
        <span className="overlay-wait">Waiting for live telemetry…</span>
      ) : enabled.length === 0 ? (
        <span className="overlay-wait">No widgets enabled — turn some on in Settings.</span>
      ) : (
        enabled.map(({ w, i }) => (
          <DraggableWidget
            key={WIDGET_KINDS[i]}
            placement={w}
            onChange={(patch) => persist(i, patch)}
            onDragState={(active) => (draggingRef.current = active)}
          >
            <div className="pw-widget">
              <Widget
                kind={WIDGET_KINDS[i]}
                snap={snap!}
                fieldPaceMode={settings.overlayLayout.fieldPaceMode}
              />
            </div>
          </DraggableWidget>
        ))
      )}
    </div>
  );
}

interface DraggableProps {
  placement: WidgetPlacement;
  onChange: (patch: Partial<WidgetPlacement>) => void;
  onDragState: (active: boolean) => void;
  children: React.ReactNode;
}

function DraggableWidget({ placement, onChange, onDragState, children }: DraggableProps) {
  const [rect, setRect] = useState({
    x: placement.desktopX,
    y: placement.desktopY,
    w: placement.desktopW,
    h: placement.desktopH,
  });

  // Sync from settings unless the user is mid-gesture on this widget.
  const gestureRef = useRef(false);
  useEffect(() => {
    if (!gestureRef.current) {
      setRect({
        x: placement.desktopX,
        y: placement.desktopY,
        w: placement.desktopW,
        h: placement.desktopH,
      });
    }
  }, [placement.desktopX, placement.desktopY, placement.desktopW, placement.desktopH]);

  const startMove = (e: React.PointerEvent) => {
    e.preventDefault();
    gestureRef.current = true;
    onDragState(true);
    const startX = e.clientX;
    const startY = e.clientY;
    const origin = { ...rect };
    const move = (ev: PointerEvent) => {
      setRect({
        ...origin,
        x: Math.max(0, origin.x + (ev.clientX - startX)),
        y: Math.max(0, origin.y + (ev.clientY - startY)),
      });
    };
    const up = (ev: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      gestureRef.current = false;
      onDragState(false);
      onChange({
        desktopX: Math.max(0, origin.x + (ev.clientX - startX)),
        desktopY: Math.max(0, origin.y + (ev.clientY - startY)),
      });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  const startResize = (e: React.PointerEvent) => {
    e.preventDefault();
    e.stopPropagation();
    gestureRef.current = true;
    onDragState(true);
    const startX = e.clientX;
    const startY = e.clientY;
    const origin = { ...rect };
    const move = (ev: PointerEvent) => {
      setRect({
        ...origin,
        w: Math.max(MIN_W, origin.w + (ev.clientX - startX)),
        h: Math.max(MIN_H, origin.h + (ev.clientY - startY)),
      });
    };
    const up = (ev: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      gestureRef.current = false;
      onDragState(false);
      onChange({
        desktopW: Math.max(MIN_W, origin.w + (ev.clientX - startX)),
        desktopH: Math.max(MIN_H, origin.h + (ev.clientY - startY)),
      });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div
      className="overlay-widget"
      style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}
    >
      <div className="overlay-widget-grip" onPointerDown={startMove} title="Drag to move" />
      {children}
      <div className="overlay-widget-resize" onPointerDown={startResize} title="Drag to resize" />
    </div>
  );
}
