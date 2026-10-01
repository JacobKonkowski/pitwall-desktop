//! Path B audio coach: WAV clips + WinRT TTS for dynamic numbers.
mod clip_phrases;
mod coach;
pub mod engine;
mod manifest;
mod phrasing;
mod player;
mod queue;
mod session_mode;
mod speech;
pub mod tts_winrt;

pub use clip_phrases::load_phrases_file;
pub use engine::{RaceContext, RaceEngine, RuleSet, SessionMeta};
pub use speech::SpeechPlan;

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;

use pitwall_live::LiveService;
use pitwall_settings::{load_settings, AppSettings};

use coach::CoachEngine;
use manifest::ClipManifest;
use player::AudioPlayer;
use queue::SpeechQueue;
use speech::SpeechUnit;

/// Coach clip folder relative to a resource root (`manifest.json` + `*.wav`).
pub const COACH_CLIPS_REL: &str = "resources/audio/coach/default";

pub struct AudioCoachService {
    cancel: Mutex<Option<CancellationToken>>,
    active: Mutex<bool>,
    last_message: Mutex<String>,
    clips_dir: Mutex<Option<PathBuf>>,
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
            clips_dir: Mutex::new(None),
        }
    }

    /// Directory the host resolved for coach clips; tried before workspace fallbacks.
    pub fn set_clips_dir(&self, dir: PathBuf) {
        *self.clips_dir.lock() = Some(dir);
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

    /// Speak a fixed test line without requiring live iRacing.
    pub fn speak_test(self: &Arc<Self>) {
        let service = Arc::clone(self);
        thread::spawn(move || {
            if let Err(e) = run_speak_test(service) {
                tracing::warn!("Audio coach test failed: {e:#}");
            }
        });
    }
}

impl Default for AudioCoachService {
    fn default() -> Self {
        Self::new()
    }
}

/// The clip set committed in the repo, which `tauri dev` and tests run against.
fn workspace_clips_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../src-tauri")
        .join(COACH_CLIPS_REL)
}

/// Clip directories in lookup order: host override, repo `src-tauri`, this crate,
/// then beside the running executable.
fn clip_dir_candidates(override_dir: Option<PathBuf>) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = override_dir.into_iter().collect();
    candidates.push(workspace_clips_dir());
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(COACH_CLIPS_REL));
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
    {
        candidates.push(dir.join(COACH_CLIPS_REL));
    }
    candidates
}

fn coach_clips_dir(service: &AudioCoachService) -> PathBuf {
    let override_dir = service.clips_dir.lock().clone();
    clip_dir_candidates(override_dir)
        .into_iter()
        .find(|dir| dir.join("manifest.json").is_file())
        .unwrap_or_else(workspace_clips_dir)
}

/// Load the clip manifest, falling back to an empty one so TTS lines still play.
fn load_manifest(service: &AudioCoachService) -> ClipManifest {
    let dir = coach_clips_dir(service);
    ClipManifest::load(dir.clone()).unwrap_or_else(|e| {
        tracing::warn!("Coach clips unavailable, continuing TTS-only: {e:#}");
        ClipManifest::empty(dir)
    })
}

fn run_speak_test(service: Arc<AudioCoachService>) -> anyhow::Result<()> {
    let manifest = load_manifest(&service);
    let settings = load_settings();
    let player = AudioPlayer::new(
        manifest,
        settings.audio_coach_rate,
        settings.audio_coach_volume,
    )?;
    let plan = SpeechPlan::sequence(vec![
        SpeechUnit::Tts("PitWall coach online.".into()),
        SpeechUnit::Tts("Last lap, one minute twenty nine point four five two.".into()),
    ]);
    play(&player, &service, &plan, &settings)
}

fn run_audio_loop(
    service: Arc<AudioCoachService>,
    live: Arc<LiveService>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let manifest = load_manifest(&service);
    let settings = load_settings();
    let mut player = AudioPlayer::new(
        manifest,
        settings.audio_coach_rate,
        settings.audio_coach_volume,
    )?;
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

        if let Some(meta) = live.session_meta.lock().clone() {
            engine.set_session_meta(meta);
        }

        let snap = live.snapshot.lock().clone();
        if let Some(plan) = engine.poll(&snap, &settings) {
            queue.push(plan.0, plan.1);
        }

        if let Some(plan) = queue.pop() {
            if cancel.is_cancelled() {
                break;
            }
            play(&player, &service, &plan, &settings)?;
            if let Some(plan) = engine.poll(&snap, &settings) {
                queue.push(plan.0, plan.1);
            }
            let gap_ms = settings.audio_inter_message_gap_ms.max(0) as u64;
            thread::sleep(Duration::from_millis(200 + gap_ms));
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
        player.set_voice_settings(
            settings.audio_coach_rate,
            settings.audio_coach_volume,
            &voice,
        );
        *applied_rate = settings.audio_coach_rate;
        *applied_volume = settings.audio_coach_volume;
        *applied_voice = voice;
    }
}

fn play(
    player: &AudioPlayer,
    service: &AudioCoachService,
    plan: &SpeechPlan,
    _settings: &AppSettings,
) -> anyhow::Result<()> {
    let line = plan.display_text();
    tracing::info!("Audio coach: {line}");
    *service.last_message.lock() = line;
    player.play_plan(plan)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::speech::SpeechPlan;
    use super::{clip_dir_candidates, coach_clips_dir, workspace_clips_dir, AudioCoachService};

    #[test]
    fn display_text_sequence() {
        let plan = SpeechPlan::sequence(vec![]);
        assert_eq!(plan.display_text(), "");
    }

    #[test]
    fn workspace_clips_dir_holds_manifest() {
        assert!(workspace_clips_dir().join("manifest.json").is_file());
    }

    #[test]
    fn resolves_workspace_clips_without_override() {
        let service = AudioCoachService::new();
        assert_eq!(coach_clips_dir(&service), workspace_clips_dir());
    }

    #[test]
    fn override_is_tried_first() {
        let custom = PathBuf::from("custom-clips");
        let candidates = clip_dir_candidates(Some(custom.clone()));
        assert_eq!(candidates[0], custom);
        assert_eq!(candidates[1], workspace_clips_dir());
    }

    #[test]
    fn missing_override_falls_through() {
        let service = AudioCoachService::new();
        service.set_clips_dir(PathBuf::from("does-not-exist"));
        assert_eq!(coach_clips_dir(&service), workspace_clips_dir());
    }
}
