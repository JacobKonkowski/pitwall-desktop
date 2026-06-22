use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub ollama_url: String,
    pub ollama_model: String,
    pub overlay_x: i32,
    pub overlay_y: i32,
    pub overlay_width: u32,
    pub overlay_height: u32,
    pub vr_overlay_enabled: bool,
    pub vr_overlay_scale: f32,
    pub audio_coach_enabled: bool,
    pub audio_coach_fuel_threshold: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            ollama_model: "llama3.2".into(),
            overlay_x: 100,
            overlay_y: 100,
            overlay_width: 320,
            overlay_height: 180,
            vr_overlay_enabled: false,
            vr_overlay_scale: 1.0,
            audio_coach_enabled: true,
            audio_coach_fuel_threshold: 5.0,
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
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    AppSettings::default()
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
