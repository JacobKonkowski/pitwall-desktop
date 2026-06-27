//! Path B audio coach: WAV clips + WinRT TTS for dynamic numbers.
mod clip_phrases;
mod coach;
mod manifest;
mod phrasing;
mod player;
mod queue;
mod session_mode;
mod speech;
pub mod tts_winrt;

pub use clip_phrases::load_phrases_file;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;

use crate::live::LiveService;
use crate::settings::{load_settings, AppSettings};

use coach::CoachEngine;
use manifest::ClipManifest;
use player::AudioPlayer;
use queue::SpeechQueue;
use speech::SpeechPlan;

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

fn coach_clips_dir() -> PathBuf {
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/audio/coach/default");
    if dev.join("manifest.json").is_file() {
        return dev;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("resources/audio/coach/default");
            if bundled.join("manifest.json").is_file() {
                return bundled;
            }
        }
    }
    dev
}

fn run_audio_loop(
    service: Arc<AudioCoachService>,
    live: Arc<LiveService>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let clips_dir = coach_clips_dir();
    let manifest = ClipManifest::load(clips_dir)?;
    let settings = load_settings();
    let mut player = AudioPlayer::new(manifest, settings.audio_coach_rate, settings.audio_coach_volume)?;
    let mut engine = CoachEngine::new();
    let mut queue = SpeechQueue::new(3);
    let mut applied_rate = f32::NAN;
    let mut applied_volume = f32::NAN;
    let mut applied_voice = String::new();

    while !cancel.is_cancelled() {
        let settings = load_settings();
        apply_voice_settings(
            &mut player,
            &settings,
            &mut applied_rate,
            &mut applied_volume,
            &mut applied_voice,
        );

        let snap = live.snapshot.lock().clone();
        if let Some(plan) = engine.poll(&snap, &settings) {
            queue.push(plan.0, plan.1);
        }

        if let Some(plan) = queue.pop() {
            if cancel.is_cancelled() {
                break;
            }
            play(&player, &service, &plan)?;
            if let Some(plan) = engine.poll(&snap, &settings) {
                queue.push(plan.0, plan.1);
            }
            thread::sleep(Duration::from_millis(200));
            continue;
        }

        thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

fn apply_voice_settings(
    player: &mut AudioPlayer,
    settings: &AppSettings,
    applied_rate: &mut f32,
    applied_volume: &mut f32,
    applied_voice: &mut String,
) {
    let voice = settings.audio_coach_voice.clone();
    if (settings.audio_coach_rate - *applied_rate).abs() > f32::EPSILON
        || (settings.audio_coach_volume - *applied_volume).abs() > f32::EPSILON
        || voice != *applied_voice
    {
        player.set_voice_settings(settings.audio_coach_rate, settings.audio_coach_volume, &voice);
        *applied_rate = settings.audio_coach_rate;
        *applied_volume = settings.audio_coach_volume;
        *applied_voice = voice;
    }
}

fn play(player: &AudioPlayer, service: &AudioCoachService, plan: &SpeechPlan) -> anyhow::Result<()> {
    let line = plan.display_text();
    tracing::info!("Audio coach: {line}");
    *service.last_message.lock() = line;
    player.play_plan(plan)
}

#[cfg(test)]
mod tests {
    use super::speech::SpeechPlan;

    #[test]
    fn display_text_sequence() {
        let plan = SpeechPlan::sequence(vec![]);
        assert_eq!(plan.display_text(), "");
    }
}
