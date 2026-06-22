pub mod analysis;
pub mod audio;
pub mod coach;
pub mod commands;
pub mod ingest;
pub mod live;
pub mod overlay;
pub mod settings;
pub mod storage;
pub mod vr;

use std::sync::Arc;

use commands::AppState;
use ingest::start_watcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pitwall_desktop_lib=info,pitwall=warn")),
        )
        .try_init();

    let state = Arc::new(AppState::new().expect("failed to initialize database"));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state.clone())
        .setup(move |app| {
            start_watcher(app.handle().clone(), state.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_sessions,
            commands::get_session,
            commands::get_lap_traces,
            commands::get_fuel_summary,
            commands::get_tire_summary,
            commands::import_ibt,
            commands::import_folder_cmd,
            commands::check_iracing_config_cmd,
            commands::get_import_status,
            commands::pick_ibt_file,
            commands::clear_database_cmd,
            commands::start_live_monitor,
            commands::stop_live_monitor,
            commands::get_live_status,
            commands::get_live_snapshot,
            commands::get_coach_report,
            commands::generate_coach_summary,
            commands::get_settings,
            commands::save_settings_cmd,
            commands::open_desktop_overlay_cmd,
            commands::close_desktop_overlay_cmd,
            commands::is_desktop_overlay_open_cmd,
            commands::start_vr_overlay,
            commands::stop_vr_overlay,
            commands::get_vr_overlay_status,
            commands::check_vr_hud_health,
            commands::open_vr_hud_preview_cmd,
            commands::start_audio_coach,
            commands::stop_audio_coach,
            commands::get_audio_coach_status,
            commands::get_audio_coach_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
