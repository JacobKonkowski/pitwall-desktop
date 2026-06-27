import { useState } from "react";
import type { SettingsSectionId } from "../lib/types";
import { useSettings } from "../lib/useSettings";
import {
  AdvancedSettingsSection,
  AudioSettingsSection,
} from "./settings/AudioAdvancedSections";
import { AiSettingsSection, OverlaySettingsSection } from "./settings/AiOverlaySections";

const SECTIONS: { id: SettingsSectionId; label: string }[] = [
  { id: "ai", label: "AI" },
  { id: "overlay", label: "Overlay & VR" },
  { id: "audio", label: "Audio coach" },
  { id: "advanced", label: "Advanced" },
];

export function SettingsPage() {
  const { settings, loading, error, patch, save } = useSettings();
  const [section, setSection] = useState<SettingsSectionId>("ai");

  if (loading || !settings) {
    return (
      <div className="settings-page panel">
        <p className="muted">Loading settings…</p>
      </div>
    );
  }

  return (
    <div className="settings-page">
      <nav className="settings-nav panel" aria-label="Settings sections">
        <h2>Settings</h2>
        {SECTIONS.map((s) => (
          <button
            key={s.id}
            type="button"
            className={section === s.id ? "tab active" : "tab"}
            onClick={() => setSection(s.id)}
          >
            {s.label}
          </button>
        ))}
      </nav>
      <div className="settings-content panel">
        {error && <p className="live-error">{error}</p>}
        {section === "ai" && <AiSettingsSection settings={settings} patch={patch} />}
        {section === "overlay" && <OverlaySettingsSection settings={settings} patch={patch} />}
        {section === "audio" && <AudioSettingsSection settings={settings} patch={patch} />}
        {section === "advanced" && (
          <AdvancedSettingsSection settings={settings} patch={patch} save={save} />
        )}
      </div>
    </div>
  );
}
