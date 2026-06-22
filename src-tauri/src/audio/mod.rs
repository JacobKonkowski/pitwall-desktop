mod coach;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;
use tts::Tts;

use crate::live::LiveService;
use crate::settings::load_settings;

pub struct AudioCoachService {
    cancel: Mutex<Option<CancellationToken>>,
    active: Mutex<bool>,
    last_message: Mutex<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioCoachStatus {
    pub active: bool,
    pub last_message: String,
}

impl AudioCoachService {
    pub fn new() -> Self {
        Self {
            cancel: Mutex::new(None),
            active: Mutex::new(false),
            last_message: Mutex::new(String::new()),
        }
    }

    pub fn is_active(&self) -> bool {
        self.cancel.lock().is_some()
    }

    pub fn status(&self) -> AudioCoachStatus {
        AudioCoachStatus {
            active: *self.active.lock(),
            last_message: self.last_message.lock().clone(),
        }
    }

    pub fn stop(&self) {
        if let Some(token) = self.cancel.lock().take() {
            token.cancel();
        }
        *self.active.lock() = false;
    }

    /// Start the audio coach. Ignores `audio_coach_enabled` — use that flag only for auto-start.
    pub fn start(self: &Arc<Self>, live: Arc<LiveService>) {
        if self.is_active() {
            return;
        }
        let token = CancellationToken::new();
        *self.cancel.lock() = Some(token.clone());
        *self.active.lock() = true;

        let service = Arc::clone(self);
        thread::spawn(move || {
            if let Err(e) = run_audio_loop(service.clone(), live, token) {
                tracing::warn!("Audio coach stopped: {e:#}");
            }
            *service.active.lock() = false;
            *service.cancel.lock() = None;
        });
    }

    pub fn last_message(&self) -> String {
        self.last_message.lock().clone()
    }
}

fn run_audio_loop(
    service: Arc<AudioCoachService>,
    live: Arc<LiveService>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let mut tts = Tts::default()?;
    let _ = tts.set_rate(2.0);

    let mut engine = coach::CoachEngine::new();

    while !cancel.is_cancelled() {
        let settings = load_settings();
        let snap = live.snapshot.lock().clone();
        let messages = engine.poll(&snap, &settings);

        for msg in messages {
            if cancel.is_cancelled() {
                break;
            }
            speak(&mut tts, &service, &msg)?;
            thread::sleep(Duration::from_millis(200));
        }

        thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

fn speak(tts: &mut Tts, service: &AudioCoachService, msg: &str) -> anyhow::Result<()> {
    tracing::info!("Audio coach: {msg}");
    *service.last_message.lock() = msg.to_string();
    tts.speak(msg, true)?;
    Ok(())
}
