use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter};
use tracing::info;

use crate::commands::AppState;
use crate::ingest::{file_identity_hash, parse_ibt_file_with_progress, save_parsed_ibt, ProgressCallback};
use crate::ingest::ibt_importer::ImportResult;
use crate::storage::ImportStatus;

/// Run a single IBT import. Only one import holds the gate at a time so DB writes
/// and progress updates stay predictable (watcher + manual import cannot overlap).
pub async fn run_import(app: &AppHandle, state: &Arc<AppState>, path: PathBuf) -> Result<ImportResult> {
    let path_label = path.to_string_lossy().to_string();

    if let Some(result) = try_skip_import(state, &path)? {
        finish_status(app, state, &result);
        return Ok(result);
    }

    let _gate = state.import_gate.lock().await;

    if let Some(result) = try_skip_import(state, &path)? {
        finish_status(app, state, &result);
        return Ok(result);
    }

    set_status(
        app,
        state,
        Some(path_label.clone()),
        true,
        1.0,
        "Opening IBT file...",
    );

    let state_progress = state.clone();
    let app_progress = app.clone();
    let progress_path = path_label.clone();
    let progress = Some(Box::new(move |pct: f64, message: String| {
        {
            let mut status = state_progress.import_status.lock();
            status.active = true;
            status.current_file = Some(progress_path.clone());
            status.progress_pct = pct;
            status.message = message;
        }
        let _ = app_progress.emit("import-status", state_progress.import_status.lock().clone());
    }) as ProgressCallback);

    let (parsed, hash, elapsed_ms) = parse_ibt_file_with_progress(&path, progress)
        .await
        .context("parse IBT")?;

    let trace_points: usize = parsed.laps.iter().map(|l| l.traces.len()).sum();
    info!(
        "Parsed {} — {} laps, {} trace points; saving to database",
        path.display(),
        parsed.laps.len(),
        trace_points
    );
    set_status(
        app,
        state,
        Some(path_label.clone()),
        true,
        92.0,
        format!("Saving {} laps to database...", parsed.laps.len()),
    );

    let state_save = state.clone();
    let path_save = path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db = state_save.db.lock();
        save_parsed_ibt(&db, &path_save, parsed, &hash, elapsed_ms)
    })
    .await
    .context("save task join")??;

    finish_status(app, state, &result);
    if !result.skipped {
        // Best-effort: attach a matching live standings snapshot to this session.
        if let Err(e) = state.db.lock().link_standings_to_session(result.session_id) {
            info!("Standings link skipped: {e:#}");
        }
        let _ = app.emit("import-complete", result.session_id);
    }
    Ok(result)
}

fn try_skip_import(state: &Arc<AppState>, path: &Path) -> Result<Option<ImportResult>> {
    let hash = file_identity_hash(path)?;
    let path_str = path.to_string_lossy().to_string();
    let db = state.db.lock();
    if db.hash_exists(&hash)? || db.path_exists(&path_str)? {
        info!("Skipping already-imported IBT: {}", path.display());
        return Ok(Some(ImportResult {
            session_id: 0,
            lap_count: 0,
            elapsed_ms: 0,
            skipped: true,
        }));
    }
    Ok(None)
}

fn set_status(
    app: &AppHandle,
    state: &Arc<AppState>,
    current_file: Option<String>,
    active: bool,
    pct: f64,
    message: impl Into<String>,
) {
    {
        let mut status = state.import_status.lock();
        status.active = active;
        status.current_file = current_file;
        status.progress_pct = pct;
        status.message = message.into();
    }
    let _ = app.emit("import-status", state.import_status.lock().clone());
}

fn finish_status(app: &AppHandle, state: &Arc<AppState>, result: &ImportResult) {
    let message = if result.skipped {
        "Already imported".into()
    } else {
        format!(
            "Imported {} laps in {} ms",
            result.lap_count, result.elapsed_ms
        )
    };
    {
        let mut status = state.import_status.lock();
        *status = ImportStatus {
            active: false,
            current_file: None,
            progress_pct: 100.0,
            message,
            ..Default::default()
        };
    }
    let _ = app.emit("import-status", state.import_status.lock().clone());
}
