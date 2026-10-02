//! Bundled Piper (VITS) neural voice, run in-process on the CPU through sherpa-onnx.
//!
//! A voice folder holds one `*.onnx` model, its `tokens.txt`, and the
//! `espeak-ng-data/` phonemizer tables (see `scripts/fetch-piper-voice.ps1`).

use std::path::{Path, PathBuf};

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig,
    OfflineTtsVitsModelConfig,
};

/// Piper voice folder relative to a resource root.
pub const PIPER_VOICE_REL: &str = "resources/audio/coach/piper";

/// Two threads keep a short line well under real time without competing with the sim.
const NUM_THREADS: i32 = 2;

/// Samples quieter than this at either end count as padding (about -38 dBFS).
const SILENCE_THRESHOLD: f32 = 0.012;
/// Padding kept around speech so chained clips and numbers do not clip consonants.
const EDGE_PAD_MS: u32 = 40;

/// Mono samples in `[-1, 1]` at `sample_rate` Hz.
pub struct PiperAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct PiperTts {
    tts: OfflineTts,
    model_name: String,
}

impl PiperTts {
    /// True when `dir` holds a complete voice (model, tokens, phonemizer data).
    pub fn is_voice_dir(dir: &Path) -> bool {
        find_model(dir).is_some()
            && dir.join("tokens.txt").is_file()
            && dir.join("espeak-ng-data").is_dir()
    }

    pub fn load(dir: &Path) -> anyhow::Result<Self> {
        let model = find_model(dir)
            .ok_or_else(|| anyhow::anyhow!("no .onnx voice model in {}", dir.display()))?;
        let tokens = dir.join("tokens.txt");
        let data_dir = dir.join("espeak-ng-data");
        if !tokens.is_file() || !data_dir.is_dir() {
            anyhow::bail!(
                "Piper voice in {} is missing tokens.txt or espeak-ng-data",
                dir.display()
            );
        }
        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                vits: OfflineTtsVitsModelConfig {
                    model: Some(path_string(&model)),
                    tokens: Some(path_string(&tokens)),
                    data_dir: Some(path_string(&data_dir)),
                    ..Default::default()
                },
                num_threads: NUM_THREADS,
                provider: Some("cpu".into()),
                ..Default::default()
            },
            max_num_sentences: 1,
            ..Default::default()
        };
        let tts = OfflineTts::create(&config)
            .ok_or_else(|| anyhow::anyhow!("failed to load Piper voice {}", model.display()))?;
        let model_name = model
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Ok(Self { tts, model_name })
    }

    /// Model file stem, e.g. `en_US-lessac-medium`.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Speak `text`; `speed` > 1 talks faster (inverse of Piper's length scale).
    pub fn synthesize(&self, text: &str, speed: f32) -> anyhow::Result<PiperAudio> {
        let text: String = text.chars().filter(|c| *c != '\0').collect();
        if text.trim().is_empty() {
            return Ok(PiperAudio {
                samples: Vec::new(),
                sample_rate: self.tts.sample_rate().max(1) as u32,
            });
        }
        let config = GenerationConfig {
            speed: speed.clamp(0.5, 2.0),
            ..Default::default()
        };
        let audio = self
            .tts
            .generate_with_config(&text, &config, None::<fn(&[f32], f32) -> bool>)
            .ok_or_else(|| anyhow::anyhow!("Piper synthesis failed for '{text}'"))?;
        let sample_rate = audio.sample_rate().max(1) as u32;
        Ok(PiperAudio {
            samples: trim_silence(audio.samples(), sample_rate).to_vec(),
            sample_rate,
        })
    }
}

/// Drop leading/trailing padding beyond [`EDGE_PAD_MS`].
fn trim_silence(samples: &[f32], sample_rate: u32) -> &[f32] {
    let loud = |s: &f32| s.abs() >= SILENCE_THRESHOLD;
    let (Some(first), Some(last)) = (
        samples.iter().position(loud),
        samples.iter().rposition(loud),
    ) else {
        return &[];
    };
    let pad = (sample_rate * EDGE_PAD_MS / 1000) as usize;
    &samples[first.saturating_sub(pad)..(last + 1 + pad).min(samples.len())]
}

fn find_model(dir: &Path) -> Option<PathBuf> {
    let mut models: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "onnx"))
        .collect();
    models.sort();
    models.into_iter().next()
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Instant;

    use super::{trim_silence, PiperTts, PIPER_VOICE_REL};

    #[test]
    fn trims_padding_but_keeps_margin() {
        // 1 kHz rate: 40 ms pad = 40 samples.
        let mut samples = vec![0.0; 200];
        samples.extend([0.5; 10]);
        samples.extend(vec![0.0; 200]);
        let trimmed = trim_silence(&samples, 1000);
        assert_eq!(trimmed.len(), 40 + 10 + 40);
        assert_eq!(trimmed[40], 0.5);
    }

    #[test]
    fn silent_audio_trims_to_nothing() {
        assert!(trim_silence(&[0.0; 50], 1000).is_empty());
    }

    #[test]
    #[ignore = "needs the voice from scripts/fetch-piper-voice.ps1"]
    fn synthesizes_a_number_line_faster_than_real_time() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../src-tauri")
            .join(PIPER_VOICE_REL);
        let started = Instant::now();
        let tts = PiperTts::load(&dir).expect("load voice");
        let load_ms = started.elapsed().as_millis();

        let started = Instant::now();
        let audio = tts
            .synthesize("12, one twenty-nine point four. 3 tenths faster.", 1.0)
            .expect("synthesize");
        let synth = started.elapsed().as_secs_f64();
        let spoken = audio.samples.len() as f64 / f64::from(audio.sample_rate);
        println!(
            "load {load_ms} ms, synth {:.0} ms for {spoken:.2} s of audio",
            synth * 1000.0
        );
        assert!(spoken > 1.0);
        assert!(synth < spoken, "synthesis slower than real time");
    }
}
