//! Dev-only: batch-export coach WAV clips from `scripts/audio-phrases.txt`.
//!
//! **Not invoked by the PitWall app at runtime.** Use while developing to bake
//! neural WinRT speech into committed WAV files.
//!
//! ```text
//! cargo run --bin gen-audio-clips -- --engine winrt
//! cargo run --bin gen-audio-clips -- --list-voices
//! cargo run --bin gen-audio-clips -- --engine placeholder
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use hound::{SampleFormat, WavSpec, WavWriter};

use pitwall_desktop_lib::audio::{load_phrases_file, tts_winrt::WinRtTts};

#[derive(Debug, Parser)]
#[command(name = "gen-audio-clips")]
struct Args {
    /// `winrt` = Windows neural SpeechSynthesizer (dev machine only).
    /// `placeholder` = short silence for CI / layout tests.
    #[arg(long, default_value = "winrt")]
    engine: String,

    #[arg(long)]
    list_voices: bool,

    /// Substring match on WinRT voice display name (e.g. "Jenny", "Guy").
    /// Default: first en-US neural voice.
    #[arg(long)]
    voice: Option<String>,

    #[arg(
        long,
        default_value = "scripts/audio-phrases.txt",
        value_name = "PATH"
    )]
    phrases: PathBuf,

    #[arg(long, default_value = "resources/audio/coach/default")]
    out_dir: PathBuf,
}

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

    let phrases = load_phrases_file(&phrases_path)?;
    fs::create_dir_all(&out_dir)?;

    let engine = args.engine.to_ascii_lowercase();
    match engine.as_str() {
        "placeholder" => export_placeholder(&phrases, &out_dir)?,
        "winrt" => export_winrt(&phrases, &out_dir, args.voice.as_deref())?,
        other => anyhow::bail!("unknown engine '{other}' (use winrt or placeholder)"),
    }

    println!(
        "Exported {} clips to {}",
        phrases.len(),
        out_dir.display()
    );
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

fn export_placeholder(phrases: &HashMap<String, String>, out_dir: &PathBuf) -> anyhow::Result<()> {
    let mut manifest = HashMap::new();
    for key in phrases.keys() {
        let file = format!("{key}.wav");
        write_placeholder_wav(&out_dir.join(&file))?;
        manifest.insert(key.clone(), file);
        println!("placeholder: {key}");
    }
    write_manifest(out_dir, &manifest)?;
    Ok(())
}

fn export_winrt(
    phrases: &HashMap<String, String>,
    out_dir: &PathBuf,
    voice: Option<&str>,
) -> anyhow::Result<()> {
    let mut tts = WinRtTts::new(1.0, 1.0)?;
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

    write_manifest(out_dir, &manifest)?;
    Ok(())
}

fn write_placeholder_wav(path: &PathBuf) -> anyhow::Result<()> {
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

fn write_manifest(out_dir: &PathBuf, manifest: &HashMap<String, String>) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(out_dir.join("manifest.json"), json)?;
    Ok(())
}
