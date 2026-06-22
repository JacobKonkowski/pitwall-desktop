mod snapshot;
mod tracker;

pub use snapshot::{LiveConnectionState, LiveSnapshot, LiveStatus};

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use parking_lot::Mutex;
use pitwall::{LiveConnection, Pitwall, SessionInfo, UpdateRate};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::analysis::types::SectorBoundary;
use crate::analysis::sector_splitter::normalize_sector_boundaries;
use crate::commands::AppState;
use crate::ingest::frame::AnalysisFrame;

use self::tracker::LiveTracker;

/// How far back we look for IBT files to auto-import when a live session ends.
const POST_SESSION_IBT_MAX_AGE_SECS: u64 = 600;

pub struct LiveService {
    pub status: Mutex<LiveStatus>,
    pub snapshot: Mutex<LiveSnapshot>,
    cancel: Mutex<Option<CancellationToken>>,
}

impl LiveService {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(LiveStatus::default()),
            snapshot: Mutex::new(LiveSnapshot::default()),
            cancel: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.cancel.lock().is_some()
    }

    pub fn stop(&self) {
        if let Some(token) = self.cancel.lock().take() {
            token.cancel();
        }
        *self.status.lock() = LiveStatus {
            state: LiveConnectionState::Disconnected,
            message: "Live monitor stopped".into(),
        };
    }

    pub fn start(self: &Arc<Self>, app: AppHandle, state: Arc<AppState>) {
        if self.is_running() {
            return;
        }
        let token = CancellationToken::new();
        *self.cancel.lock() = Some(token.clone());
        *self.status.lock() = LiveStatus {
            state: LiveConnectionState::WaitingForSession,
            message: "Connecting to iRacing...".into(),
        };

        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            Arc::clone(&service).run_loop(app, state, token).await;
            *service.cancel.lock() = None;
        });
    }

    fn set_status(&self, state: LiveConnectionState, message: impl Into<String>) {
        *self.status.lock() = LiveStatus {
            state,
            message: message.into(),
        };
    }

    /// Sleep for `dur` unless cancelled. Returns true if cancelled.
    async fn sleep_or_cancel(cancel: &CancellationToken, dur: Duration) -> bool {
        tokio::select! {
            _ = cancel.cancelled() => true,
            _ = tokio::time::sleep(dur) => false,
        }
    }

    /// Outer reconnect loop: keeps trying to connect to iRacing with exponential
    /// backoff and recovers from dropped frame streams without manual restart.
    async fn run_loop(self: Arc<Self>, app: AppHandle, state: Arc<AppState>, cancel: CancellationToken) {
        info!("Starting live telemetry monitor");
        let min_backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(5);
        let mut backoff = min_backoff;
        let mut attempted = false;

        while !cancel.is_cancelled() {
            if attempted {
                self.set_status(
                    LiveConnectionState::Reconnecting,
                    "Reconnecting to iRacing...",
                );
            }

            let connection = match Pitwall::connect().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Live connect failed: {e:#}");
                    attempted = true;
                    if Self::sleep_or_cancel(&cancel, backoff).await {
                        break;
                    }
                    backoff = (backoff * 2).min(max_backoff);
                    continue;
                }
            };

            attempted = true;
            let received = self.run_session(&app, &state, &connection, &cancel).await;
            if cancel.is_cancelled() {
                break;
            }

            // A clean session (frames seen) means iRacing was healthy; reset backoff
            // so the next reconnect is quick.
            if received {
                backoff = min_backoff;
            }

            self.set_status(
                LiveConnectionState::Reconnecting,
                "iRacing session ended — waiting to reconnect...",
            );
            if Self::sleep_or_cancel(&cancel, backoff).await {
                break;
            }
            backoff = (backoff * 2).min(max_backoff);
        }

        info!("Live monitor stopped");
    }

    /// Run a single connection until the frame stream ends or we are cancelled.
    /// Returns true if at least one frame was received during this session.
    async fn run_session(
        &self,
        app: &AppHandle,
        state: &Arc<AppState>,
        connection: &LiveConnection,
        cancel: &CancellationToken,
    ) -> bool {
        let mut frame_stream = connection.subscribe::<AnalysisFrame>(UpdateRate::Max(10));
        let mut session_stream = Box::pin(connection.session_updates());

        let mut tracker = LiveTracker::new();
        let mut sector_bounds: Vec<SectorBoundary> = Vec::new();
        let mut got_frame = false;
        let mut ever_got_frame = false;

        let mut emit_tick = tokio::time::interval(Duration::from_millis(100));
        emit_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("Live monitor cancelled");
                    return ever_got_frame;
                }
                session = session_stream.next() => {
                    if let Some(session) = session {
                        sector_bounds = extract_sector_boundaries(&session);
                        let prev_track = tracker.track().to_string();
                        tracker.set_session_meta(&session);
                        // New track means a new session: drop carried-over deltas/bests.
                        if !prev_track.is_empty() && prev_track != tracker.track() {
                            info!("Track changed ({prev_track} -> {}), resetting tracker", tracker.track());
                            tracker.reset_session();
                            tracker.set_session_meta(&session);
                        }
                    }
                }
                frame = frame_stream.next() => {
                    match frame {
                        Some(f) => {
                            got_frame = true;
                            ever_got_frame = true;
                            self.set_status(LiveConnectionState::Connected, "Receiving telemetry");
                            let bounds = normalize_sector_boundaries(&sector_bounds);
                            *self.snapshot.lock() = tracker.snapshot_from_frame(&f, &bounds);
                        }
                        None => {
                            warn!("Live frame stream ended");
                            if ever_got_frame {
                                spawn_post_session_import(app.clone(), Arc::clone(state));
                            }
                            return ever_got_frame;
                        }
                    }
                }
                _ = emit_tick.tick() => {
                    if got_frame {
                        let snap = self.snapshot.lock().clone();
                        let status = self.status.lock().clone();
                        let _ = app.emit("live-telemetry", &snap);
                        let _ = app.emit("live-status", &status);
                    }
                }
            }
        }
    }
}

/// After a live session ends, scan the telemetry folder for recently written IBT
/// files and import any not yet in the database. Complements the filesystem
/// watcher, which can miss `Create` events when iRacing writes via OneDrive.
fn spawn_post_session_import(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let dir = crate::ingest::default_telemetry_dir();
        let files = match crate::ingest::scan_ibt_files(&dir) {
            Ok(files) => files,
            Err(e) => {
                warn!("Post-session IBT scan failed: {e:#}");
                return;
            }
        };

        let cutoff = std::time::SystemTime::now()
            .checked_sub(Duration::from_secs(POST_SESSION_IBT_MAX_AGE_SECS));
        for path in files {
            let recent = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .zip(cutoff)
                .map(|(modified, cutoff)| modified >= cutoff)
                .unwrap_or(false);
            if !recent {
                continue;
            }
            // run_import skips files already imported (hash/path check), so this is
            // safe to call on every recent file.
            if let Err(e) = crate::ingest::run_import(&app, &state, path.clone()).await {
                warn!("Post-session import of {} failed: {e:#}", path.display());
            }
        }
    });
}

fn extract_sector_boundaries(session: &SessionInfo) -> Vec<SectorBoundary> {
    let Some(split) = &session.split_time_info else {
        return Vec::new();
    };
    let Some(sectors) = &split.sectors else {
        return Vec::new();
    };
    sectors
        .iter()
        .filter_map(|s| {
            Some(SectorBoundary {
                sector_num: s.sector_num?,
                start_pct: s.sector_start_pct?,
            })
        })
        .collect()
}
