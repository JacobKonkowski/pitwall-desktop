use std::cell::Cell;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::Arc;

use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use race_refinery_settings::AppSettings;

use super::manifest::ClipManifest;
use super::speech::{SpeechPlan, SpeechUnit};
use super::tts_piper::PiperTts;
use super::tts_winrt::WinRtTts;

type BoxedSource = Box<dyn Source<Item = f32> + Send>;

pub struct AudioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    manifest: ClipManifest,
    piper: Option<Arc<PiperTts>>,
    winrt: Option<WinRtTts>,
    rate: f32,
    volume: f32,
    /// Windows voice picked for numbers; empty selects the bundled Piper voice.
    voice: String,
    piper_failed: Cell<bool>,
}

impl AudioPlayer {
    pub fn new(
        manifest: ClipManifest,
        piper: Option<Arc<PiperTts>>,
        settings: &AppSettings,
    ) -> anyhow::Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| anyhow::anyhow!("audio output: {e}"))?;
        let winrt = match WinRtTts::new(settings.audio_coach_rate) {
            Ok(tts) => Some(tts),
            Err(e) => {
                tracing::warn!("Windows speech unavailable: {e:#}");
                None
            }
        };
        let mut player = Self {
            _stream: stream,
            handle,
            manifest,
            piper,
            winrt,
            rate: settings.audio_coach_rate,
            volume: settings.audio_coach_volume,
            voice: settings.audio_coach_voice.clone(),
            piper_failed: Cell::new(false),
        };
        player.select_winrt_voice();
        Ok(player)
    }

    /// Pick up rate / volume / voice changes; cheap when nothing changed.
    pub fn apply_settings(&mut self, settings: &AppSettings) {
        if (settings.audio_coach_rate - self.rate).abs() > f32::EPSILON {
            self.rate = settings.audio_coach_rate;
            if let Some(winrt) = &mut self.winrt {
                winrt.set_rate(self.rate);
            }
        }
        self.volume = settings.audio_coach_volume;
        if settings.audio_coach_voice != self.voice {
            self.voice = settings.audio_coach_voice.clone();
            self.select_winrt_voice();
        }
    }

    fn select_winrt_voice(&mut self) {
        let Some(winrt) = &mut self.winrt else {
            return;
        };
        let hint = (!self.voice.is_empty()).then_some(self.voice.as_str());
        if let Err(e) = winrt.set_voice(hint) {
            tracing::warn!("TTS voice selection failed: {e:#}");
        }
    }

    /// Play every unit back to back on one sink. Units are appended in order, so
    /// synthesis of a later number overlaps playback of the clips before it.
    pub fn play_plan(&self, plan: &SpeechPlan) -> anyhow::Result<()> {
        let single;
        let units: &[SpeechUnit] = match plan {
            SpeechPlan::Clip(key) => {
                single = [SpeechUnit::Clip(key.clone())];
                &single
            }
            SpeechPlan::Sequence(units) => units,
        };
        let sink = Sink::try_new(&self.handle)?;
        sink.set_volume(self.volume.max(0.0));
        for unit in units {
            let source = match unit {
                SpeechUnit::Clip(key) => self.clip_source(key),
                SpeechUnit::Tts(text) => self.tts_source(text),
            };
            if let Some(source) = source {
                sink.append(source);
            }
        }
        sink.sleep_until_end();
        Ok(())
    }

    fn clip_source(&self, key: &str) -> Option<BoxedSource> {
        let Some(path) = self.manifest.path(key) else {
            tracing::warn!("missing coach clip: {key}");
            return None;
        };
        let decoded = File::open(&path)
            .map_err(anyhow::Error::from)
            .and_then(|file| Ok(Decoder::new(BufReader::new(file))?));
        match decoded {
            Ok(decoder) => Some(Box::new(decoder.convert_samples())),
            Err(e) => {
                tracing::warn!("clip {key}: {e:#}");
                None
            }
        }
    }

    fn tts_source(&self, text: &str) -> Option<BoxedSource> {
        if text.trim().is_empty() {
            return None;
        }
        if self.voice.is_empty() {
            if let Some(piper) = &self.piper {
                match piper.synthesize(text, self.rate) {
                    Ok(audio) if audio.samples.is_empty() => return None,
                    Ok(audio) => {
                        return Some(Box::new(SamplesBuffer::new(
                            1,
                            audio.sample_rate,
                            audio.samples,
                        )))
                    }
                    Err(e) => {
                        if !self.piper_failed.replace(true) {
                            tracing::warn!("Piper TTS failed, using Windows speech: {e:#}");
                        }
                    }
                }
            }
        }
        let winrt = self.winrt.as_ref()?;
        let decoded = winrt
            .synthesize_wav(text)
            .and_then(|bytes| Ok(Decoder::new(Cursor::new(bytes))?));
        match decoded {
            Ok(decoder) => Some(Box::new(decoder.convert_samples())),
            Err(e) => {
                tracing::warn!("TTS '{text}': {e:#}");
                None
            }
        }
    }
}
