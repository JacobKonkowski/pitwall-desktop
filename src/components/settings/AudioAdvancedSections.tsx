import { useEffect, useState } from "react";
import { listTtsVoices } from "../../lib/api";
import type { AppSettings, TtsVoiceInfo } from "../../lib/types";
import type { UseSettingsResult } from "../../lib/useSettings";
import {
  SettingsRow,
  SettingsSection,
  SettingsSelect,
  SettingsSlider,
  SettingsToggle,
} from "./SettingsFields";

interface Props {
  settings: AppSettings;
  patch: UseSettingsResult["patch"];
}

export function AudioSettingsSection({ settings, patch }: Props) {
  const [voices, setVoices] = useState<TtsVoiceInfo[]>([]);

  useEffect(() => {
    listTtsVoices().then(setVoices).catch(() => setVoices([]));
  }, []);

  return (
    <SettingsSection title="Audio coach" description="Radio callouts during live sessions.">
      <SettingsToggle
        label="Auto-start audio coach with live monitor"
        checked={settings.audioCoachEnabled}
        onChange={(v) => patch({ audioCoachEnabled: v }, true)}
      />
      {voices.length > 0 && (
        <SettingsSelect
          label="Voice"
          value={settings.audioCoachVoice}
          onChange={(v) => patch({ audioCoachVoice: v }, true)}
          options={[
            { value: "", label: "System default" },
            ...voices.map((v) => ({
              value: v.displayName,
              label: `${v.displayName}${v.neural ? " (neural)" : ""}`,
            })),
          ]}
        />
      )}
      <SettingsToggle
        label="Session intro callout"
        checked={settings.audioSessionIntroEnabled}
        onChange={(v) => patch({ audioSessionIntroEnabled: v }, true)}
      />
      <SettingsToggle
        label="Position change callouts"
        checked={settings.audioPositionCalloutsEnabled}
        onChange={(v) => patch({ audioPositionCalloutsEnabled: v }, true)}
      />
      <SettingsToggle
        label="Spotter pack alerts (traffic + clear)"
        checked={settings.audioPackAlertsEnabled}
        onChange={(v) => patch({ audioPackAlertsEnabled: v }, true)}
      />
      <SettingsToggle
        label="Flag callouts"
        checked={settings.audioFlagsEnabled}
        onChange={(v) => patch({ audioFlagsEnabled: v }, true)}
      />
      <SettingsToggle
        label="Incident count callouts"
        checked={settings.audioIncidentsEnabled}
        onChange={(v) => patch({ audioIncidentsEnabled: v }, true)}
      />
      <SettingsToggle
        label="Race fuel-to-finish calls"
        checked={settings.audioFuelRaceEnabled}
        onChange={(v) => patch({ audioFuelRaceEnabled: v }, true)}
      />
      <SettingsToggle
        label="Gap alerts (ahead/behind)"
        checked={settings.audioGapAlertsEnabled}
        onChange={(v) => patch({ audioGapAlertsEnabled: v }, true)}
      />
      <SettingsToggle
        label="Lap and sector pace callouts"
        checked={settings.audioPaceEnabled}
        onChange={(v) => patch({ audioPaceEnabled: v }, true)}
      />
      <SettingsToggle
        label="Strategy calls (fuel, race clock, pits open)"
        checked={settings.audioStrategyEnabled}
        onChange={(v) => patch({ audioStrategyEnabled: v }, true)}
      />
      <SettingsToggle
        label="Race clock milestones"
        checked={settings.audioRaceClockEnabled}
        onChange={(v) => patch({ audioRaceClockEnabled: v }, true)}
      />
      <SettingsToggle
        label="Pits open callout"
        checked={settings.audioPitsOpenEnabled}
        onChange={(v) => patch({ audioPitsOpenEnabled: v }, true)}
      />
      <SettingsSelect
        label="Radio chatter level"
        value={settings.audioCoachChatterLevel}
        onChange={(v) =>
          patch({ audioCoachChatterLevel: v as AppSettings["audioCoachChatterLevel"] }, true)
        }
        options={[
          { value: "minimal", label: "Minimal (safety only)" },
          { value: "normal", label: "Normal" },
          { value: "verbose", label: "Verbose" },
        ]}
      />
      <SettingsSlider
        label="Speech rate"
        value={settings.audioCoachRate}
        min={0.5}
        max={6}
        step={0.1}
        format={(v) => `${v.toFixed(1)}×`}
        onChange={(v) => patch({ audioCoachRate: v }, true)}
      />
      <SettingsSlider
        label="Speech volume"
        value={settings.audioCoachVolume}
        min={0}
        max={1}
        step={0.05}
        format={(v) => `${Math.round(v * 100)}%`}
        onChange={(v) => patch({ audioCoachVolume: v }, true)}
      />
      <SettingsRow label="Fuel warning (liters)">
        <input
          type="number"
          min={0}
          step={0.5}
          value={settings.audioCoachFuelThreshold}
          onChange={(e) =>
            patch({ audioCoachFuelThreshold: parseFloat(e.target.value) || 0 }, true)
          }
        />
      </SettingsRow>
    </SettingsSection>
  );
}

export function AdvancedSettingsSection({ settings, save }: Props & { save: UseSettingsResult["save"] }) {
  const exportSettings = () => {
    const blob = new Blob([JSON.stringify(settings, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "pitwall-settings.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  const importSettings = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const parsed = JSON.parse(text) as AppSettings;
        await save(parsed);
      } catch (e) {
        alert(`Import failed: ${e}`);
      }
    };
    input.click();
  };

  return (
    <SettingsSection title="Advanced" defaultOpen={false}>
      <div className="btn-row">
        <button type="button" onClick={exportSettings}>
          Export settings
        </button>
        <button type="button" onClick={importSettings}>
          Import settings
        </button>
      </div>
      <p className="muted small">
        Legacy VR fields (vrHudOffset, etc.) are kept for migration only; overlay layout is the
        source of truth.
      </p>
    </SettingsSection>
  );
}
