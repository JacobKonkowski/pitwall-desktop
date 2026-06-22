//! In-headset HUD without SteamVR — local web HUD for OpenKneeboard / OpenXR.

mod hud_server;

use std::sync::Arc;
use std::thread;

use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;

use crate::live::LiveService;

pub use hud_server::{hud_url, HUD_PORT};

pub struct VrOverlayService {
    cancel: Mutex<Option<CancellationToken>>,
    status: Mutex<VrOverlayStatus>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VrOverlayStatus {
    pub active: bool,
    pub runtime: String,
    pub message: String,
    pub hud_url: String,
}

impl VrOverlayService {
    pub fn new() -> Self {
        Self {
            cancel: Mutex::new(None),
            status: Mutex::new(VrOverlayStatus {
                hud_url: hud_url(),
                ..Default::default()
            }),
        }
    }

    pub fn is_active(&self) -> bool {
        self.cancel.lock().is_some()
    }

    pub fn status(&self) -> VrOverlayStatus {
        let mut s = self.status.lock().clone();
        s.hud_url = hud_url();
        s
    }

    pub fn stop(&self) {
        if let Some(token) = self.cancel.lock().take() {
            token.cancel();
        }
        *self.status.lock() = VrOverlayStatus {
            active: false,
            runtime: String::new(),
            message: "In-headset HUD stopped".into(),
            hud_url: hud_url(),
        };
    }

    pub fn start(self: &Arc<Self>, live: Arc<LiveService>) {
        if self.is_active() {
            return;
        }

        let token = CancellationToken::new();
        *self.cancel.lock() = Some(token.clone());
        *self.status.lock() = VrOverlayStatus {
            active: true,
            runtime: "OpenXR (Web HUD)".into(),
            message: "Starting in-headset HUD server…".into(),
            hud_url: hud_url(),
        };

        let service = Arc::clone(self);
        thread::spawn(move || {
            if let Err(e) = hud_server::run_hud_server(service.clone(), live, token) {
                service.status.lock().message = format!("HUD server error: {e:#}");
                service.status.lock().active = false;
            }
            *service.cancel.lock() = None;
        });
    }
}
