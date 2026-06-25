/// A single playback unit — fixed clip or synthesized speech.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechUnit {
    Clip(String),
    Tts(String),
}

/// Structured speech output from the coach engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechPlan {
    Tts(String),
    Clip(String),
    Sequence(Vec<SpeechUnit>),
}

impl SpeechPlan {
    pub fn tts(text: impl Into<String>) -> Self {
        Self::Tts(text.into())
    }

    pub fn clip(key: impl Into<String>) -> Self {
        Self::Clip(key.into())
    }

    pub fn sequence(units: Vec<SpeechUnit>) -> Self {
        Self::Sequence(units)
    }

    /// Human-readable line for the UI / logs.
    pub fn display_text(&self) -> String {
        match self {
            Self::Tts(s) | Self::Clip(s) => s.clone(),
            Self::Sequence(units) => units
                .iter()
                .map(|u| match u {
                    SpeechUnit::Clip(k) => format!("[{k}]"),
                    SpeechUnit::Tts(t) => t.clone(),
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}
