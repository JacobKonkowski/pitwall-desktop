use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

use crate::commands::AppState;
use crate::ingest::{default_telemetry_dir, run_import, scan_ibt_files};
use crate::storage::ImportStatus;

pub fn start_watcher(app: AppHandle, state: Arc<AppState>) {
    let telemetry_dir = default_telemetry_dir();
    if !telemetry_dir.exists() {
        let _ = std::fs::create_dir_all(&telemetry_dir);
    }

    let app_clone = app.clone();
    let state_clone = state.clone();
    let watch_path = telemetry_dir.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("watcher runtime");

        let pending: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let pending_timer = pending.clone();
        let app_timer = app_clone.clone();
        let state_timer = state_clone.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(3));
            let batch: Vec<PathBuf> = {
                let mut guard = pending_timer.lock();
                if guard.is_empty() {
                    continue;
                }
                guard.drain().collect()
            };
            for path in batch {
                let _ = rt.block_on(run_import(&app_timer, &state_timer, path));
            }
        });

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Create(_)) {
                        for path in event.paths {
                            if path
                                .extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e.eq_ignore_ascii_case("ibt"))
                                == Some(true)
                            {
                                pending.lock().insert(path);
                            }
                        }
                    }
                }
            },
            notify::Config::default(),
        )
        .expect("create file watcher");

        if watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .is_err()
        {
            return;
        }

        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
}

fn set_batch_progress(
    app: &AppHandle,
    state: &Arc<AppState>,
    index: usize,
    total: usize,
    path: &PathBuf,
) {
    let label = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    {
        let mut status = state.import_status.lock();
        status.active = true;
        status.current_file = Some(path.to_string_lossy().into_owned());
        status.message = format!("Importing file {index} of {total}: {label}");
    }
    let _ = app.emit("import-status", state.import_status.lock().clone());
}

fn finish_batch(
    app: &AppHandle,
    state: &Arc<AppState>,
    total: usize,
    imported: usize,
    skipped: usize,
    elapsed_ms: u64,
) {
    let secs = elapsed_ms as f64 / 1000.0;
    let message = if total == 0 {
        "Folder scan complete: no IBT files found".into()
    } else {
        format!(
            "Folder scan complete: {imported} imported, {skipped} skipped ({total} files) in {secs:.1}s"
        )
    };
    {
        let mut status = state.import_status.lock();
        *status = ImportStatus {
            active: false,
            current_file: None,
            progress_pct: 100.0,
            message,
            batch_elapsed_ms: Some(elapsed_ms),
            batch_file_count: Some(total),
            batch_imported_count: Some(imported),
            batch_skipped_count: Some(skipped),
        };
    }
    let _ = app.emit("import-status", state.import_status.lock().clone());
}

pub async fn import_folder(app: &AppHandle, state: &Arc<AppState>, dir: PathBuf) -> anyhow::Result<usize> {
    let started = Instant::now();
    let files = scan_ibt_files(&dir)?;
    let total = files.len();
    let mut imported = 0usize;
    let mut skipped = 0usize;

    if total == 0 {
        finish_batch(app, state, 0, 0, 0, started.elapsed().as_millis() as u64);
        return Ok(0);
    }

    for (i, path) in files.iter().enumerate() {
        set_batch_progress(app, state, i + 1, total, path);
        let result = run_import(app, state, path.clone()).await?;
        if result.skipped {
            skipped += 1;
        } else {
            imported += 1;
        }
    }

    finish_batch(
        app,
        state,
        total,
        imported,
        skipped,
        started.elapsed().as_millis() as u64,
    );
    Ok(total)
}
