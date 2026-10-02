//! Dev-only: batch-export coach WAV clips from `scripts/audio-phrases.txt`.
//!
//! **Not invoked by the Race Refinery app at runtime.** Use while developing to bake
//! the bundled Piper voice into committed WAV files, so fixed callouts match the
//! live numbers the app synthesizes with the same voice.
//!
//! ```text
//! cargo run --bin gen-audio-clips
//! cargo run --bin gen-audio-clips -- --engine winrt --voice "Guy"
//! cargo run --bin gen-audio-clips -- --list-voices
//! cargo run --bin gen-audio-clips -- --engine placeholder
//! cargo run --bin gen-audio-clips -- --only tyre_hot,lap_invalid
//! ```
//!
//! `radio_beep` is not speech: it is always written as a synthesized two-tone
//! chirp, independent of the engine.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use hound::{SampleFormat, WavSpec, WavWriter};

use race_refinery_desktop_lib::audio::{
    load_phrases_file, tts_piper::PiperTts, tts_winrt::WinRtTts, PIPER_VOICE_REL,
};

#[derive(Debug, Parser)]
#[command(name = "gen-audio-clips")]
struct Args {
    /// `piper` = bundled neural voice (run scripts/fetch-piper-voice.ps1 first).
    /// `winrt` = Windows SpeechSynthesizer.
    /// `placeholder` = short silence for CI / layout tests.
    #[arg(long, default_value = "piper")]
    engine: String,

    #[arg(long)]
    list_voices: bool,

    /// Substring match on WinRT voice display name (e.g. "Jenny", "Guy").
    /// Default: first en-US neural voice. WinRT engine only.
    #[arg(long)]
    voice: Option<String>,

    /// Piper voice folder (`*.onnx`, `tokens.txt`, `espeak-ng-data/`).
    #[arg(long, default_value = PIPER_VOICE_REL, value_name = "DIR")]
    voice_dir: PathBuf,

    #[arg(long, default_value = "scripts/audio-phrases.txt", value_name = "PATH")]
    phrases: PathBuf,

    #[arg(long, default_value = "resources/audio/coach/default")]
    out_dir: PathBuf,

    /// Comma-separated clip keys to (re)generate; other clips and their
    /// manifest entries are left untouched. Default: every phrase.
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
}

const RADIO_BEEP_KEY: &str = "radio_beep";

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let phrases_path = if args.phrases.is_absolute() {
        args.phrases
    } else {
        manifest_dir.join("..").join(&args.phrases)
    };
    let out_dir = if args.out_dir.is_absolute() {
        args.out_dir
    } else {
        manifest_dir.join(&args.out_dir)
    };

    if args.list_voices {
        list_voices()?;
        return Ok(());
    }

    let mut phrases = load_phrases_file(&phrases_path)?;
    fs::create_dir_all(&out_dir)?;

    let only_beep = args.only.iter().any(|k| k == RADIO_BEEP_KEY);
    if !args.only.is_empty() {
        for key in &args.only {
            if key != RADIO_BEEP_KEY && !phrases.contains_key(key) {
                anyhow::bail!("'{key}' is not in {}", phrases_path.display());
            }
        }
        phrases.retain(|k, _| args.only.contains(k));
    }

    let voice_dir = if args.voice_dir.is_absolute() {
        args.voice_dir
    } else {
        manifest_dir.join(&args.voice_dir)
    };

    let engine = args.engine.to_ascii_lowercase();
    let mut manifest = match engine.as_str() {
        "placeholder" => export_placeholder(&phrases, &out_dir)?,
        "piper" | "winrt" if phrases.is_empty() => HashMap::new(),
        "piper" => export_piper(&phrases, &out_dir, &voice_dir)?,
        "winrt" => export_winrt(&phrases, &out_dir, args.voice.as_deref())?,
        other => anyhow::bail!("unknown engine '{other}' (use piper, winrt or placeholder)"),
    };

    let mut count = phrases.len();
    if args.only.is_empty() || only_beep {
        let file = format!("{RADIO_BEEP_KEY}.wav");
        write_radio_beep(&out_dir.join(&file))?;
        manifest.insert(RADIO_BEEP_KEY.into(), file);
        println!("tone: {RADIO_BEEP_KEY}");
        count += 1;
    }

    if args.only.is_empty() {
        write_manifest(&out_dir, &manifest)?;
    } else {
        let mut merged = read_manifest(&out_dir)?;
        merged.extend(manifest);
        write_manifest(&out_dir, &merged)?;
    }

    println!("Exported {count} clips to {}", out_dir.display());
    Ok(())
}

fn list_voices() -> anyhow::Result<()> {
    let voices = WinRtTts::list_voices()?;
    if voices.is_empty() {
        println!("No WinRT voices found.");
        return Ok(());
    }
    println!("Installed WinRT voices:\n");
    for v in voices {
        let tag = if v.neural { "neural" } else { "standard" };
        println!(
            "  {}  [{}] {} ({})",
            v.display_name, tag, v.language, v.gender
        );
    }
    println!("\nRe-run with: --engine winrt --voice \"<substring>\"");
    Ok(())
}

fn export_placeholder(
    phrases: &HashMap<String, String>,
    out_dir: &Path,
) -> anyhow::Result<HashMap<String, String>> {
    let mut manifest = HashMap::new();
    for key in phrases.keys() {
        let file = format!("{key}.wav");
        write_placeholder_wav(&out_dir.join(&file))?;
        manifest.insert(key.clone(), file);
        println!("placeholder: {key}");
    }
    Ok(manifest)
}

fn export_piper(
    phrases: &HashMap<String, String>,
    out_dir: &Path,
    voice_dir: &Path,
) -> anyhow::Result<HashMap<String, String>> {
    let tts = PiperTts::load(voice_dir).map_err(|e| {
        e.context(format!(
            "Piper voice not found in {} (run scripts/fetch-piper-voice.ps1)",
            voice_dir.display()
        ))
    })?;
    println!("Using Piper voice: {}", tts.model_name());

    let mut manifest = HashMap::new();
    let mut keys: Vec<_> = phrases.keys().collect();
    keys.sort();

    for key in keys {
        let text = &phrases[key];
        let file = format!("{key}.wav");
        let audio = tts.synthesize(text, 1.0)?;
        if audio.samples.is_empty() {
            anyhow::bail!("Piper returned empty audio for '{key}'");
        }
        let spec = WavSpec {
            channels: 1,
            sample_rate: audio.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(out_dir.join(&file), spec)?;
        for s in &audio.samples {
            writer.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
        }
        writer.finalize()?;
        manifest.insert(key.clone(), file);
        println!("piper: {key}  ({text})");
    }

    Ok(manifest)
}

fn export_winrt(
    phrases: &HashMap<String, String>,
    out_dir: &Path,
    voice: Option<&str>,
) -> anyhow::Result<HashMap<String, String>> {
    let mut tts = WinRtTts::new(1.0)?;
    tts.set_voice(voice)?;
    if let Some(name) = tts.current_voice_name() {
        println!("Using voice: {name}");
    }

    let mut manifest = HashMap::new();
    let mut keys: Vec<_> = phrases.keys().collect();
    keys.sort();

    for key in keys {
        let text = &phrases[key];
        let file = format!("{key}.wav");
        let path = out_dir.join(&file);
        let bytes = tts.synthesize_wav(text)?;
        if bytes.is_empty() {
            anyhow::bail!("WinRT returned empty audio for '{key}'");
        }
        fs::write(&path, &bytes)?;
        manifest.insert(key.clone(), file);
        println!("winrt: {key}  ({text})");
    }

    Ok(manifest)
}

/// Short two-tone radio chirp (1.2 kHz then 1.8 kHz, ~140 ms) with a soft
/// envelope so it does not click.
fn write_radio_beep(path: &Path) -> anyhow::Result<()> {
    // Matches the Piper speech clips.
    const RATE: u32 = 22050;
    let spec = WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    let tone_len = (RATE as f32 * 0.07) as usize;
    let fade = (RATE as f32 * 0.008) as usize;
    for (freq, gap_after) in [(1200.0_f32, true), (1800.0_f32, false)] {
        for i in 0..tone_len {
            let env = (i.min(tone_len - 1 - i).min(fade) as f32) / fade as f32;
            let t = i as f32 / RATE as f32;
            let s = (t * freq * std::f32::consts::TAU).sin() * env * 0.35;
            writer.write_sample((s * i16::MAX as f32) as i16)?;
        }
        if gap_after {
            for _ in 0..(RATE as usize / 100) {
                writer.write_sample(0i16)?;
            }
        }
    }
    writer.finalize()?;
    Ok(())
}

fn read_manifest(out_dir: &Path) -> anyhow::Result<HashMap<String, String>> {
    let path = out_dir.join("manifest.json");
    match fs::read_to_string(&path) {
        Ok(json) => Ok(serde_json::from_str(&json)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e.into()),
    }
}

fn write_placeholder_wav(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let spec = WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)?;
    for _ in 0..2205 {
        writer.write_sample(0i16)?;
    }
    writer.finalize()?;
    Ok(())
}

fn write_manifest(out_dir: &Path, manifest: &HashMap<String, String>) -> anyhow::Result<()> {
    let sorted: BTreeMap<_, _> = manifest.iter().collect();
    let json = serde_json::to_string_pretty(&sorted)?;
    fs::write(out_dir.join("manifest.json"), json)?;
    Ok(())
}
