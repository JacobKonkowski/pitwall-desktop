use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use tauri::AppHandle;

use crate::commands::AppState;
use crate::ingest::{default_telemetry_dir, run_import, scan_ibt_files};

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
                    // Only new files — Modify fires when OneDrive syncs or files are read,
                    // which would queue dozens of parallel imports and block the DB.
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

pub async fn import_folder(app: &AppHandle, state: &Arc<AppState>, dir: PathBuf) -> anyhow::Result<usize> {
    let files = scan_ibt_files(&dir)?;
    let count = files.len();
    for path in files {
        run_import(app, state, path).await?;
    }
    Ok(count)
}
