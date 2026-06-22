use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::analysis::coach::build_coach_report;
use crate::audio::AudioCoachService;
use crate::coach::generate_summary;
use crate::ingest::{
    check_iracing_config, default_telemetry_dir, run_import,
};
use crate::live::{LiveService, LiveSnapshot, LiveStatus};
use crate::overlay::{close_desktop_overlay, is_desktop_overlay_open, open_desktop_overlay};
use crate::settings::{load_settings, save_settings, AppSettings};
use crate::storage::{
    Database, FuelSummary, ImportStatus, IracingConfigCheck, LapTrace, SessionDetail,
    SessionSummary, TireSummary,
};
use crate::vr::{VrOverlayService, VrOverlayStatus};

pub struct AppState {
    pub db: Mutex<Database>,
    pub import_status: Mutex<ImportStatus>,
    pub import_gate: tokio::sync::Mutex<()>,
    pub live: Arc<LiveService>,
    pub vr: Arc<VrOverlayService>,
    pub audio: Arc<AudioCoachService>,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Mutex::new(Database::open()?),
            import_status: Mutex::new(ImportStatus {
                active: false,
                current_file: None,
                progress_pct: 0.0,
                message: "Idle".into(),
            }),
            import_gate: tokio::sync::Mutex::new(()),
            live: Arc::new(LiveService::new()),
            vr: Arc::new(VrOverlayService::new()),
            audio: Arc::new(AudioCoachService::new()),
            settings: Mutex::new(load_settings()),
        })
    }
}

#[tauri::command]
pub fn list_sessions(state: State<'_, Arc<AppState>>) -> Result<Vec<SessionSummary>, String> {
    state.db.lock().list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(state: State<'_, Arc<AppState>>, session_id: i64) -> Result<Option<SessionDetail>, String> {
    state
        .db
        .lock()
        .get_session(session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_lap_traces(state: State<'_, Arc<AppState>>, lap_ids: Vec<i64>) -> Result<Vec<LapTrace>, String> {
    state
        .db
        .lock()
        .get_lap_traces(&lap_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_fuel_summary(state: State<'_, Arc<AppState>>, session_id: i64) -> Result<FuelSummary, String> {
    state
        .db
        .lock()
        .get_fuel_summary(session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tire_summary(state: State<'_, Arc<AppState>>, session_id: i64) -> Result<TireSummary, String> {
    state
        .db
        .lock()
        .get_tire_summary(session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_ibt(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    run_import(&app, state.inner(), path_buf)
        .await
        .map_err(|e| {
            let msg = format!("Import failed: {e:#}");
            {
                let mut status = state.import_status.lock();
                status.active = false;
                status.message = msg.clone();
            }
            let _ = app.emit("import-status", state.import_status.lock().clone());
            msg
        })?;
    Ok(state.import_status.lock().message.clone())
}

#[tauri::command]
pub async fn import_folder_cmd(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    let dir = default_telemetry_dir();
    crate::ingest::watcher::import_folder(&app, state.inner(), dir)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_iracing_config_cmd() -> IracingConfigCheck {
    check_iracing_config()
}

#[tauri::command]
pub fn get_import_status(state: State<'_, Arc<AppState>>) -> ImportStatus {
    state.import_status.lock().clone()
}

#[tauri::command]
pub fn pick_ibt_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("iRacing Telemetry", &["ibt"])
        .blocking_pick_file();
    Ok(file.map(|f| f.to_string()))
}

#[tauri::command]
pub fn clear_database_cmd(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    #[cfg(not(debug_assertions))]
    {
        let _ = state;
        return Err("Clear database is only available in development builds".into());
    }
    #[cfg(debug_assertions)]
    {
        let removed = state.db.lock().clear_all().map_err(|e| e.to_string())?;
        let mut status = state.import_status.lock();
        *status = ImportStatus {
            active: false,
            current_file: None,
            progress_pct: 0.0,
            message: if removed > 0 {
                format!("Cleared {removed} session(s) from database")
            } else {
                "Database already empty".into()
            },
        };
        Ok(removed)
    }
}

#[tauri::command]
pub fn start_live_monitor(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.live.start(app.clone());
    let settings = state.settings.lock().clone();
    if settings.vr_overlay_enabled {
        state.vr.start(state.live.clone());
    }
    if settings.audio_coach_enabled {
        state.audio.start(state.live.clone());
    }
    Ok(())
}

#[tauri::command]
pub fn stop_live_monitor(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.live.stop();
    state.vr.stop();
    state.audio.stop();
    Ok(())
}

#[tauri::command]
pub fn get_live_status(state: State<'_, Arc<AppState>>) -> LiveStatus {
    state.live.status.lock().clone()
}

#[tauri::command]
pub fn get_live_snapshot(state: State<'_, Arc<AppState>>) -> LiveSnapshot {
    state.live.snapshot.lock().clone()
}

#[tauri::command]
pub fn get_coach_report(
    state: State<'_, Arc<AppState>>,
    session_id: i64,
) -> Result<crate::analysis::coach::CoachReport, String> {
    let detail = state
        .db
        .lock()
        .get_session(session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;
    Ok(build_coach_report(session_id, &detail))
}

#[tauri::command]
pub async fn generate_coach_summary(
    state: State<'_, Arc<AppState>>,
    session_id: i64,
) -> Result<crate::coach::CoachSummaryResult, String> {
    let detail = state
        .db
        .lock()
        .get_session(session_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;
    let report = build_coach_report(session_id, &detail);
    generate_summary(&report)
        .await
        .map_err(|e| format!("AI summary failed: {e:#}"))
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
pub fn save_settings_cmd(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    save_settings(&settings).map_err(|e| e.to_string())?;
    *state.settings.lock() = settings;
    Ok(())
}

#[tauri::command]
pub fn open_desktop_overlay_cmd(app: AppHandle) -> Result<(), String> {
    open_desktop_overlay(&app)
}

#[tauri::command]
pub fn close_desktop_overlay_cmd(app: AppHandle) -> Result<(), String> {
    close_desktop_overlay(&app)
}

#[tauri::command]
pub fn is_desktop_overlay_open_cmd(app: AppHandle) -> bool {
    is_desktop_overlay_open(&app)
}

#[tauri::command]
pub fn start_vr_overlay(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if !state.live.is_running() {
        return Err("Start live monitor first".into());
    }
    state.vr.start(state.live.clone());
    Ok(())
}

#[tauri::command]
pub fn stop_vr_overlay(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.vr.stop();
    Ok(())
}

#[tauri::command]
pub fn get_vr_overlay_status(state: State<'_, Arc<AppState>>) -> VrOverlayStatus {
    state.vr.status()
}

#[tauri::command]
pub fn start_audio_coach(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if !state.live.is_running() {
        return Err("Start live monitor first".into());
    }
    state.audio.start(state.live.clone());
    Ok(())
}

#[tauri::command]
pub fn stop_audio_coach(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.audio.stop();
    Ok(())
}

#[tauri::command]
pub fn get_audio_coach_status(state: State<'_, Arc<AppState>>) -> crate::audio::AudioCoachStatus {
    state.audio.status()
}

#[tauri::command]
pub fn get_audio_coach_message(state: State<'_, Arc<AppState>>) -> String {
    state.audio.last_message()
}
