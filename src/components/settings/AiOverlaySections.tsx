import type { AppSettings, WidgetPlacement } from "../../lib/types";
import { defaultOverlayLayout, WIDGET_KINDS, WIDGET_LABELS } from "../../lib/types";
import type { UseSettingsResult } from "../../lib/useSettings";
import {
  SettingsSection,
  SettingsSelect,
  SettingsSlider,
  SettingsTextInput,
  SettingsToggle,
} from "./SettingsFields";

interface Props {
  settings: AppSettings;
  patch: UseSettingsResult["patch"];
}

export function AiSettingsSection({ settings, patch }: Props) {
  return (
    <SettingsSection
      title="AI (Ollama)"
      description="Used by the Analyze tab to generate AI session summaries."
    >
      <SettingsTextInput
        label="Ollama URL"
        value={settings.ollamaUrl}
        onChange={(v) => patch({ ollamaUrl: v })}
      />
      <SettingsTextInput
        label="Ollama model"
        value={settings.ollamaModel}
        onChange={(v) => patch({ ollamaModel: v })}
      />
    </SettingsSection>
  );
}

export function OverlaySettingsSection({ settings, patch }: Props) {
  const patchWidget = (index: number, partial: Partial<WidgetPlacement>) => {
    const widgets = settings.overlayLayout.widgets.map((w, i) =>
      i === index ? { ...w, ...partial } : w,
    );
    patch({ overlayLayout: { ...settings.overlayLayout, widgets } }, true);
  };

  const patchLayout = (partial: Partial<AppSettings["overlayLayout"]>) => {
    patch({ overlayLayout: { ...settings.overlayLayout, ...partial } }, true);
  };

  return (
    <SettingsSection
      title="Overlay & VR"
      description="Desktop pop-out and in-headset HUD widgets share this layout."
    >
      <SettingsToggle
        label="Auto-start in-headset HUD with live monitor"
        checked={settings.vrOverlayEnabled}
        onChange={(v) => patch({ vrOverlayEnabled: v }, true)}
      />
      <SettingsSelect
        label="VR mode"
        value={settings.vrMode}
        onChange={(v) => patch({ vrMode: v }, true)}
        options={[
          { value: "native", label: "Native (in-headset, no OpenKneeboard)" },
          { value: "web", label: "Web fallback (OpenKneeboard)" },
        ]}
      />
      <SettingsTextInput
        label="VR recenter hotkey"
        value={settings.vrRecenterHotkey}
        placeholder="e.g. Ctrl+F10"
        onChange={(v) => patch({ vrRecenterHotkey: v }, true)}
      />
      <SettingsSelect
        label="Field pace (coach)"
        value={settings.overlayLayout.fieldPaceMode}
        onChange={(v) => patchLayout({ fieldPaceMode: v })}
        options={[
          { value: "best", label: "Session best (FLD)" },
          { value: "optimal", label: "Session optimal (OPT)" },
          { value: "both", label: "Both" },
        ]}
      />
      <p className="muted small">
        Overlay window: {settings.overlayWidth}×{settings.overlayHeight} at ({settings.overlayX},{" "}
        {settings.overlayY}) — resize the pop-out to change.
      </p>
      <div className="overlay-widgets-settings">
        <div className="overlay-widgets-header">
          <h4>Overlay widgets</h4>
          <button
            type="button"
            className="tab"
            onClick={() => patch({ overlayLayout: defaultOverlayLayout() }, true)}
          >
            Reset layout
          </button>
        </div>
        {settings.overlayLayout.widgets.map((w, i) => (
          <div key={WIDGET_KINDS[i]} className="overlay-widget-settings">
            <SettingsToggle
              label={WIDGET_LABELS[WIDGET_KINDS[i]]}
              checked={w.enabled}
              onChange={(v) => patchWidget(i, { enabled: v })}
            />
            {w.enabled && (
              <div className="overlay-widget-vr">
                <SettingsSlider
                  label="VR height"
                  value={w.vrOffsetY}
                  min={-0.5}
                  max={0.5}
                  step={0.02}
                  format={(v) => `${v.toFixed(2)} m`}
                  onChange={(v) => patchWidget(i, { vrOffsetY: v })}
                />
                <SettingsSlider
                  label="VR scale"
                  value={w.vrScale}
                  min={0.5}
                  max={2}
                  step={0.1}
                  format={(v) => `${v.toFixed(1)}×`}
                  onChange={(v) => patchWidget(i, { vrScale: v })}
                />
                <SettingsSlider
                  label="VR opacity"
                  value={w.vrOpacity}
                  min={0.2}
                  max={1}
                  step={0.05}
                  format={(v) => `${Math.round(v * 100)}%`}
                  onChange={(v) => patchWidget(i, { vrOpacity: v })}
                />
              </div>
            )}
          </div>
        ))}
      </div>
    </SettingsSection>
  );
}
