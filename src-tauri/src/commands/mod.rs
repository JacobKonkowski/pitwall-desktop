//! Tauri IPC commands and shared [`AppState`].
//!
//! Every `#[tauri::command]` here is registered in [`crate::run`] and wrapped by
//! the frontend API layer. Analyze commands stay available; live / audio / VR
//! are restored for the usable rebuild milestone.
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::analysis::{compare_laps as run_compare, CompareInput, LapComparison};
use crate::audio::AudioCoachService;
use crate::ingest::{check_iracing_config, default_telemetry_dir, run_import};
use crate::live::{LiveService, LiveSnapshot, LiveStatus};
use crate::settings::{load_settings, save_settings, AppSettings};
use crate::storage::{
    Database, ImportStatus, IracingConfigCheck, LapTrace, SessionDetail, SessionSummary,
};
use crate::vr::{
    NativeVrStatus, VrLayerDiagnostics, VrOverlayService, VrOverlayStatus,
};

pub struct AppState {
    pub db: Mutex<Database>,
    pub import_status: Mutex<ImportStatus>,
    /// Serializes imports so DB writes and progress updates stay predictable.
    pub import_gate: tokio::sync::Mutex<()>,
    pub live: Arc<LiveService>,
    pub audio: Arc<AudioCoachService>,
    pub vr: Arc<VrOverlayService>,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Mutex::new(Database::open()?),
            import_status: Mutex::new(ImportStatus::default()),
            import_gate: tokio::sync::Mutex::new(()),
            live: Arc::new(LiveService::new()),
            audio: Arc::new(AudioCoachService::new()),
            vr: Arc::new(VrOverlayService::new()),
            settings: Mutex::new(load_settings()),
        })
    }
}

#[tauri::command]
pub fn list_sessions(state: State<'_, Arc<AppState>>) -> Result<Vec<SessionSummary>, String> {
    state.db.lock().list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(
    state: State<'_, Arc<AppState>>,
    session_id: i64,
) -> Result<Option<SessionDetail>, String> {
    state
        .db
        .lock()
        .get_session(session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_lap_traces(
    state: State<'_, Arc<AppState>>,
    lap_ids: Vec<i64>,
) -> Result<Vec<LapTrace>, String> {
    state
        .db
        .lock()
        .get_lap_traces(&lap_ids)
        .map_err(|e| e.to_string())
}

/// Compare a candidate lap against a reference lap (time, sectors, aligned traces).
#[tauri::command]
pub fn compare_laps(
    state: State<'_, Arc<AppState>>,
    candidate_lap_id: i64,
    reference_lap_id: i64,
) -> Result<LapComparison, String> {
    let db = state.db.lock();
    let (cand_time, cand_sectors, cand_traces) = db
        .get_lap_compare_data(candidate_lap_id)
        .map_err(|e| e.to_string())?;
    let (ref_time, ref_sectors, ref_traces) = db
        .get_lap_compare_data(reference_lap_id)
        .map_err(|e| e.to_string())?;

    let candidate = CompareInput {
        lap_id: candidate_lap_id,
        lap_time_ms: cand_time,
        sectors: &cand_sectors,
        traces: &cand_traces,
    };
    let reference = CompareInput {
        lap_id: reference_lap_id,
        lap_time_ms: ref_time,
        sectors: &ref_sectors,
        traces: &ref_traces,
    };
    Ok(run_compare(&candidate, &reference))
}

#[tauri::command]
pub async fn import_ibt(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<String, String> {
    let path_buf = PathBuf::from(&path);
    run_import(&app, state.inner(), path_buf).await.map_err(|e| {
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
    crate::ingest::import_folder(&app, state.inner(), dir)
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
        Err("Clear database is only available in development builds".into())
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
pub fn delete_session_cmd(
    state: State<'_, Arc<AppState>>,
    session_id: i64,
) -> Result<bool, String> {
    state
        .db
        .lock()
        .delete_session(session_id)
        .map_err(|e| e.to_string())
}

// --- Live -------------------------------------------------------------------

#[tauri::command]
pub fn start_live_monitor(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.live.start(app.clone(), state.inner().clone());
    let settings = state.settings.lock().clone();
    if settings.vr_overlay_enabled {
        state.vr.start(state.live.clone(), settings.clone());
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
pub fn start_demo_clock(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.live.start_demo(app);
    Ok(())
}

#[tauri::command]
pub fn stop_demo_clock(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.live.stop_demo();
    Ok(())
}

// --- Settings ---------------------------------------------------------------

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
pub fn save_settings_cmd(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    save_settings(&settings).map_err(|e| e.to_string())?;
    *state.settings.lock() = settings.clone();
    let _ = app.emit("settings-changed", settings);
    Ok(())
}

// --- Audio ------------------------------------------------------------------

#[tauri::command]
pub fn start_audio_coach(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if !state.live.is_running() {
        return Err("Start live monitor or demo clock first".into());
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

#[tauri::command]
pub fn test_audio_coach(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.audio.speak_test();
    Ok(())
}

// --- VR ---------------------------------------------------------------------

#[tauri::command]
pub fn start_vr_overlay(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let settings = state.settings.lock().clone();
    state.vr.start(state.live.clone(), settings);
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
pub fn get_native_vr_status(state: State<'_, Arc<AppState>>) -> NativeVrStatus {
    state.vr.native_status()
}

fn vr_layer_manifest_path(app: &AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let rel = ["resources", "openxr-layer", crate::vr::MANIFEST_FILE];
    let candidate = app
        .path()
        .resource_dir()
        .ok()
        .map(|dir| rel.iter().fold(dir, |acc, p| acc.join(p)))
        .or_else(|| {
            // Dev fallback: repo openxr-layer build output / source tree.
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("openxr-layer")
                .join(crate::vr::MANIFEST_FILE);
            if manifest.is_file() {
                Some(manifest)
            } else {
                std::env::current_exe().ok().and_then(|exe| {
                    exe.parent()
                        .map(|d| rel.iter().fold(d.to_path_buf(), |acc, p| acc.join(p)))
                })
            }
        })
        .ok_or_else(|| "Could not resolve VR layer manifest path".to_string())?;
    Ok(candidate.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn is_vr_layer_installed() -> bool {
    crate::vr::is_layer_installed()
}

#[tauri::command]
pub fn install_vr_layer(app: AppHandle) -> Result<(), String> {
    let path = vr_layer_manifest_path(&app)?;
    crate::vr::install_layer(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_vr_layer(app: AppHandle) -> Result<(), String> {
    let path = vr_layer_manifest_path(&app)?;
    crate::vr::uninstall_layer(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vr_layer_diagnostics(app: AppHandle) -> Result<VrLayerDiagnostics, String> {
    let path = vr_layer_manifest_path(&app)?;
    Ok(crate::vr::layer_diagnostics(&path))
}

#[tauri::command]
pub fn check_vr_hud_health() -> bool {
    crate::vr::check_hud_health()
}

#[tauri::command]
pub fn open_vr_hud_preview_cmd() -> Result<(), String> {
    crate::vr::open_hud_preview()
}
