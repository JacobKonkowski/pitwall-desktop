#[cfg(windows)]
mod imp {
    use windows::core::HSTRING;
    use windows::Media::SpeechSynthesis::{SpeechSynthesizer, VoiceInformation};
    use windows::Storage::Streams::{ByteOrder, DataReader};

    pub struct VoiceInfo {
        pub display_name: String,
        pub language: String,
        pub gender: String,
        pub neural: bool,
    }

    pub struct WinRtTts {
        synthesizer: SpeechSynthesizer,
        rate: f32,
        volume: f32,
    }

    impl WinRtTts {
        pub fn new(rate: f32, volume: f32) -> anyhow::Result<Self> {
            Ok(Self {
                synthesizer: SpeechSynthesizer::new()?,
                rate,
                volume,
            })
        }

        pub fn set_rate(&mut self, rate: f32) {
            self.rate = rate;
        }

        pub fn set_volume(&mut self, volume: f32) {
            self.volume = volume;
        }

        fn all_voices() -> anyhow::Result<Vec<VoiceInformation>> {
            let view = SpeechSynthesizer::AllVoices()?;
            let mut out = Vec::new();
            let n = view.Size()?;
            for i in 0..n {
                out.push(view.GetAt(i)?);
            }
            Ok(out)
        }

        pub fn list_voices() -> anyhow::Result<Vec<VoiceInfo>> {
            let mut out = Vec::new();
            for v in Self::all_voices()? {
                out.push(voice_info(&v)?);
            }
            out.sort_by(|a, b| a.display_name.cmp(&b.display_name));
            Ok(out)
        }

        /// Pick a voice by exact display name, or substring match (case-insensitive).
        /// When `hint` is None, prefers the first en-US neural voice.
        pub fn set_voice(&mut self, hint: Option<&str>) -> anyhow::Result<()> {
            let all = Self::all_voices()?;
            let picked = if let Some(hint) = hint {
                let lower = hint.to_ascii_lowercase();
                all.iter()
                    .find(|v| {
                        voice_display(v)
                            .map(|n| n.to_ascii_lowercase().contains(&lower))
                            .unwrap_or(false)
                    })
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("no WinRT voice matching '{hint}'"))?
            } else {
                all.iter()
                    .find(|v| {
                        voice_info(v)
                            .map(|i| i.language.starts_with("en") && i.neural)
                            .unwrap_or(false)
                    })
                    .or_else(|| all.first())
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("no WinRT voices installed"))?
            };
            self.synthesizer.SetVoice(&picked)?;
            if let Ok(name) = picked.DisplayName() {
                println!("WinRT voice: {}", name.to_string());
            }
            Ok(())
        }

        pub fn current_voice_name(&self) -> Option<String> {
            self.synthesizer
                .Voice()
                .ok()
                .and_then(|v| v.DisplayName().ok().map(|s| s.to_string()))
        }

        pub fn synthesize_wav(&self, text: &str) -> anyhow::Result<Vec<u8>> {
            if text.trim().is_empty() {
                return Ok(Vec::new());
            }
            let op = self
                .synthesizer
                .SynthesizeTextToStreamAsync(&HSTRING::from(text))?;
            let stream = op
                .get()
                .map_err(|e| anyhow::anyhow!("WinRT synthesis failed: {e}"))?;
            let size = stream.Size()? as usize;
            if size == 0 {
                return Ok(Vec::new());
            }
            let input = stream.GetInputStreamAt(0)?;
            let reader = DataReader::CreateDataReader(&input)?;
            reader.SetByteOrder(ByteOrder::LittleEndian)?;
            reader
                .LoadAsync(size as u32)?
                .get()
                .map_err(|e| anyhow::anyhow!("WinRT stream read: {e}"))?;
            let mut buf = vec![0u8; size];
            reader.ReadBytes(&mut buf)?;
            let _ = (self.rate, self.volume);
            Ok(buf)
        }
    }

    fn voice_display(v: &VoiceInformation) -> Option<String> {
        v.DisplayName().ok().map(|s| s.to_string())
    }

    fn voice_info(v: &VoiceInformation) -> anyhow::Result<VoiceInfo> {
        Ok(VoiceInfo {
            display_name: v.DisplayName()?.to_string(),
            language: v.Language()?.to_string(),
            gender: format!("{:?}", v.Gender()?),
            neural: v
                .DisplayName()?
                .to_string()
                .to_ascii_lowercase()
                .contains("neural"),
        })
    }
}

#[cfg(not(windows))]
mod imp {
    pub struct VoiceInfo {
        pub display_name: String,
        pub language: String,
        pub gender: String,
        pub neural: bool,
    }

    pub struct WinRtTts;

    impl WinRtTts {
        pub fn new(_rate: f32, _volume: f32) -> anyhow::Result<Self> {
            anyhow::bail!("WinRT TTS is only available on Windows")
        }

        pub fn set_rate(&mut self, _rate: f32) {}

        pub fn set_volume(&mut self, _volume: f32) {}

        pub fn list_voices() -> anyhow::Result<Vec<VoiceInfo>> {
            anyhow::bail!("WinRT TTS is only available on Windows")
        }

        pub fn set_voice(&mut self, _hint: Option<&str>) -> anyhow::Result<()> {
            anyhow::bail!("WinRT TTS is only available on Windows")
        }

        pub fn current_voice_name(&self) -> Option<String> {
            None
        }

        pub fn synthesize_wav(&self, _text: &str) -> anyhow::Result<Vec<u8>> {
            anyhow::bail!("WinRT TTS is only available on Windows")
        }
    }
}

pub use imp::{VoiceInfo, WinRtTts};
