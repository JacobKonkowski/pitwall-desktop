mod snapshot;
mod tracker;

pub use snapshot::{LiveConnectionState, LiveSnapshot, LiveStatus};

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use parking_lot::Mutex;
use pitwall::{Pitwall, SessionInfo, UpdateRate};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::analysis::types::SectorBoundary;
use crate::analysis::sector_splitter::normalize_sector_boundaries;
use crate::ingest::frame::AnalysisFrame;

use self::tracker::LiveTracker;

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

    pub fn start(self: &Arc<Self>, app: AppHandle) {
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
            let svc = Arc::clone(&service);
            if let Err(e) = svc.run_loop(app, token).await {
                error!("Live monitor error: {e:#}");
                service.status.lock().state = LiveConnectionState::Error;
                service.status.lock().message = format!("Live error: {e:#}");
            }
            *service.cancel.lock() = None;
        });
    }

    async fn run_loop(self: Arc<Self>, app: AppHandle, cancel: CancellationToken) -> anyhow::Result<()> {
        info!("Starting live telemetry monitor");
        let connection = Pitwall::connect().await?;
        let mut frame_stream = connection.subscribe::<AnalysisFrame>(UpdateRate::Max(10));
        let mut session_stream = Box::pin(connection.session_updates());

        let mut tracker = LiveTracker::new();
        let mut sector_bounds: Vec<SectorBoundary> = Vec::new();
        let mut got_frame = false;

        let mut emit_tick = tokio::time::interval(Duration::from_millis(100));
        emit_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    info!("Live monitor cancelled");
                    break;
                }
                session = session_stream.next() => {
                    if let Some(session) = session {
                        sector_bounds = extract_sector_boundaries(&session);
                        tracker.set_session_meta(&session);
                    }
                }
                frame = frame_stream.next() => {
                    match frame {
                        Some(f) => {
                            got_frame = true;
                            self.status.lock().state = LiveConnectionState::Connected;
                            self.status.lock().message = "Receiving telemetry".into();
                            let bounds = normalize_sector_boundaries(&sector_bounds);
                            *self.snapshot.lock() = tracker.snapshot_from_frame(&f, &bounds);
                        }
                        None => {
                            warn!("Live frame stream ended");
                            self.status.lock().state = LiveConnectionState::WaitingForSession;
                            self.status.lock().message = "Waiting for iRacing session...".into();
                            got_frame = false;
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
        Ok(())
    }
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
