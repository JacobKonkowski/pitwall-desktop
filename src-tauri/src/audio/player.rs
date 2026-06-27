use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use super::manifest::ClipManifest;
use super::speech::{SpeechPlan, SpeechUnit};
use super::tts_winrt::WinRtTts;

pub struct AudioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    manifest: ClipManifest,
    tts: WinRtTts,
}

impl AudioPlayer {
    pub fn new(manifest: ClipManifest, rate: f32, volume: f32) -> anyhow::Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| anyhow::anyhow!("audio output: {e}"))?;
        Ok(Self {
            _stream: stream,
            handle,
            manifest,
            tts: WinRtTts::new(rate, volume)?,
        })
    }

    pub fn set_voice_settings(&mut self, rate: f32, volume: f32, voice: &str) {
        self.tts.set_rate(rate);
        self.tts.set_volume(volume);
        if !voice.is_empty() {
            if let Err(e) = self.tts.set_voice(Some(voice)) {
                tracing::warn!("TTS voice selection failed: {e:#}");
            }
        }
    }

    pub fn play_plan(&self, plan: &SpeechPlan) -> anyhow::Result<()> {
        match plan {
            SpeechPlan::Clip(key) => self.play_clip(key),
            SpeechPlan::Sequence(units) => {
                for unit in units {
                    match unit {
                        SpeechUnit::Tts(text) => self.play_tts(text)?,
                        SpeechUnit::Clip(key) => {
                            if let Err(e) = self.play_clip(key) {
                                tracing::warn!("clip {key}: {e:#}");
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn play_clip(&self, key: &str) -> anyhow::Result<()> {
        let Some(path) = self.manifest.path(key) else {
            tracing::warn!("missing coach clip: {key}");
            return Ok(());
        };
        self.play_wav_file(&path)
    }

    fn play_tts(&self, text: &str) -> anyhow::Result<()> {
        let bytes = self.tts.synthesize_wav(text)?;
        if bytes.is_empty() {
            return Ok(());
        }
        self.play_wav_bytes(&bytes)
    }

    fn play_wav_file(&self, path: &Path) -> anyhow::Result<()> {
        let file = File::open(path)?;
        let decoder = Decoder::new(BufReader::new(file))?;
        self.play_decoder(decoder)
    }

    fn play_wav_bytes(&self, bytes: &[u8]) -> anyhow::Result<()> {
        let decoder = Decoder::new(Cursor::new(bytes.to_vec()))?;
        self.play_decoder(decoder)
    }

    fn play_decoder<R: std::io::Read + std::io::Seek + Send + 'static>(
        &self,
        decoder: Decoder<R>,
    ) -> anyhow::Result<()> {
        let sink = Sink::try_new(&self.handle)?;
        sink.append(decoder);
        while !sink.empty() {
            std::thread::sleep(Duration::from_millis(20));
        }
        sink.sleep_until_end();
        Ok(())
    }
}
