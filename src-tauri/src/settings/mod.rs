use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How much non-critical radio traffic the coach produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ChatterLevel {
    Minimal,
    #[default]
    Normal,
    Verbose,
}

/// Overlay widget slots, shared by the desktop pop-out and the VR compositor.
/// The index of each widget in [`OverlayLayout::widgets`] equals its VR overlay
/// slot and kind (0 = coach, 1 = standings, 2 = relative, 3 = radar), so the
/// same configuration drives both surfaces.
pub const WIDGET_COUNT: usize = 4;
pub const WIDGET_COACH: usize = 0;
pub const WIDGET_STANDINGS: usize = 1;
pub const WIDGET_RELATIVE: usize = 2;
pub const WIDGET_RADAR: usize = 3;

/// Per-widget visibility and placement. Desktop coordinates are pixels inside
/// the pop-out window; VR coordinates are meters / multipliers applied on top of
/// the per-kind base pose the compositor uses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WidgetPlacement {
    pub enabled: bool,
    pub desktop_x: f32,
    pub desktop_y: f32,
    pub desktop_w: f32,
    pub desktop_h: f32,
    /// Vertical nudge applied to the widget's VR base pose, in meters.
    pub vr_offset_y: f32,
    /// VR quad scale multiplier.
    pub vr_scale: f32,
    /// VR quad opacity, 0.0–1.0.
    pub vr_opacity: f32,
}

impl Default for WidgetPlacement {
    fn default() -> Self {
        Self {
            enabled: false,
            desktop_x: 24.0,
            desktop_y: 24.0,
            desktop_w: 320.0,
            desktop_h: 180.0,
            vr_offset_y: 0.0,
            vr_scale: 1.0,
            vr_opacity: 1.0,
        }
    }
}

/// Shared catalog of overlay widgets plus the coach field-pace preference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayLayout {
    pub widgets: [WidgetPlacement; WIDGET_COUNT],
    /// Field pace shown on the coach widget: "best", "optimal", or "both".
    pub field_pace_mode: String,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        // Coach is on by default and centered; the rest start disabled but with
        // non-overlapping desktop slots so enabling them lands somewhere sane.
        let mut widgets = [WidgetPlacement::default(); WIDGET_COUNT];
        widgets[WIDGET_COACH] = WidgetPlacement {
            enabled: true,
            desktop_x: 24.0,
            desktop_y: 24.0,
            desktop_w: 360.0,
            desktop_h: 200.0,
            ..WidgetPlacement::default()
        };
        widgets[WIDGET_STANDINGS] = WidgetPlacement {
            desktop_x: 24.0,
            desktop_y: 244.0,
            desktop_w: 320.0,
            desktop_h: 300.0,
            ..WidgetPlacement::default()
        };
        widgets[WIDGET_RELATIVE] = WidgetPlacement {
            desktop_x: 360.0,
            desktop_y: 244.0,
            desktop_w: 300.0,
            desktop_h: 240.0,
            ..WidgetPlacement::default()
        };
        widgets[WIDGET_RADAR] = WidgetPlacement {
            desktop_x: 404.0,
            desktop_y: 24.0,
            desktop_w: 200.0,
            desktop_h: 200.0,
            ..WidgetPlacement::default()
        };
        Self {
            widgets,
            field_pace_mode: "best".into(),
        }
    }
}

impl OverlayLayout {
    /// Seed the layout from the legacy single-overlay `vr_*` settings so users
    /// upgrading from the coach-only build keep their HUD placement.
    fn from_legacy(settings: &AppSettings) -> Self {
        let mut layout = OverlayLayout::default();
        let coach = &mut layout.widgets[WIDGET_COACH];
        coach.enabled = true;
        coach.vr_offset_y = settings.vr_hud_offset;
        coach.vr_scale = settings.vr_overlay_scale;
        coach.vr_opacity = settings.vr_hud_opacity;
        layout.field_pace_mode = settings.vr_field_pace_mode.clone();
        layout
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub ollama_url: String,
    pub ollama_model: String,
    pub overlay_x: i32,
    pub overlay_y: i32,
    pub overlay_width: u32,
    pub overlay_height: u32,
    pub vr_overlay_enabled: bool,
    /// Overlay quad scale for the native VR HUD (also the legacy web scale).
    pub vr_overlay_scale: f32,
    /// "native" (in-headset OpenXR layer) or "web" (OpenKneeboard fallback).
    pub vr_mode: String,
    /// Vertical placement of the native HUD in meters (positive = higher).
    pub vr_hud_offset: f32,
    /// Native HUD opacity, 0.0–1.0.
    pub vr_hud_opacity: f32,
    /// Optional global recenter hotkey (e.g. "Ctrl+F10"); empty = disabled.
    pub vr_recenter_hotkey: String,
    /// Field pace shown on the HUD: "best", "optimal", or "both".
    pub vr_field_pace_mode: String,
    /// Shared widget catalog for the desktop pop-out and the VR compositor.
    pub overlay_layout: OverlayLayout,
    pub audio_coach_enabled: bool,
    /// Speech rate for Windows TTS (0.5 = slow, 1.0 = normal, up to 6.0).
    pub audio_coach_rate: f32,
    /// Speech volume (0.0–1.0).
    pub audio_coach_volume: f32,
    pub audio_coach_fuel_threshold: f32,
    pub audio_pack_alerts_enabled: bool,
    pub audio_flags_enabled: bool,
    pub audio_incidents_enabled: bool,
    pub audio_fuel_race_enabled: bool,
    pub audio_gap_alerts_enabled: bool,
    pub audio_pace_enabled: bool,
    pub audio_strategy_enabled: bool,
    pub audio_race_clock_enabled: bool,
    pub audio_pits_open_enabled: bool,
    #[serde(default)]
    pub audio_coach_chatter_level: ChatterLevel,
    /// WinRT voice display name; empty = system default.
    #[serde(default)]
    pub audio_coach_voice: String,
    #[serde(default = "default_true")]
    pub audio_session_intro_enabled: bool,
    #[serde(default = "default_true")]
    pub audio_position_callouts_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            ollama_model: "llama3.2".into(),
            overlay_x: 100,
            overlay_y: 100,
            overlay_width: 720,
            overlay_height: 520,
            vr_overlay_enabled: false,
            vr_overlay_scale: 1.0,
            vr_mode: "native".into(),
            vr_hud_offset: 0.0,
            vr_hud_opacity: 1.0,
            vr_recenter_hotkey: String::new(),
            vr_field_pace_mode: "best".into(),
            overlay_layout: OverlayLayout::default(),
            audio_coach_enabled: true,
            audio_coach_rate: 1.0,
            audio_coach_volume: 1.0,
            audio_coach_fuel_threshold: 5.0,
            audio_pack_alerts_enabled: true,
            audio_flags_enabled: true,
            audio_incidents_enabled: true,
            audio_fuel_race_enabled: true,
            audio_gap_alerts_enabled: true,
            audio_pace_enabled: true,
            audio_strategy_enabled: true,
            audio_race_clock_enabled: true,
            audio_pits_open_enabled: true,
            audio_coach_chatter_level: ChatterLevel::Normal,
            audio_coach_voice: String::new(),
            audio_session_intro_enabled: true,
            audio_position_callouts_enabled: true,
        }
    }
}

pub fn settings_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pitwall-desktop")
        .join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return AppSettings::default();
    };
    let had_layout = value.get("overlayLayout").is_some();
    let Ok(mut settings) = serde_json::from_value::<AppSettings>(value) else {
        return AppSettings::default();
    };
    // Upgrade a coach-only config to the shared widget layout once.
    if !had_layout {
        settings.overlay_layout = OverlayLayout::from_legacy(&settings);
    }
    settings
}

pub fn save_settings(settings: &AppSettings) -> anyhow::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(path, json)?;
    Ok(())
}
