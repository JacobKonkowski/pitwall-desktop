use pitwall_settings::AppSettings;

use super::super::super::queue::SpeechPriority;
use super::super::super::speech::{SpeechPlan, SpeechUnit};
use super::super::candidate::{Candidate, Mark};
use super::super::context::RaceContext;
use super::super::helpers::wrap_with_radio;
use super::super::rule::Rule;

pub struct IntroRule {
    spoke_session_intro: bool,
}

impl IntroRule {
    pub fn new() -> Self {
        Self {
            spoke_session_intro: false,
        }
    }
}

impl Default for IntroRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for IntroRule {
    fn id(&self) -> &'static str {
        "intro"
    }

    fn on_tick(&mut self, ctx: &RaceContext<'_>, settings: &AppSettings, out: &mut Vec<Candidate>) {
        if !settings.audio_session_intro_enabled
            || self.spoke_session_intro
            || ctx.snap.track.is_empty()
        {
            return;
        }
        let session = if ctx.snap.session_type.is_empty() {
            SpeechUnit::Clip("intro_good_luck".into())
        } else {
            SpeechUnit::Tts(format!("{}. Good luck.", ctx.snap.session_type))
        };
        out.push(Candidate {
            priority: SpeechPriority::CRITICAL,
            plan: wrap_with_radio(
                settings,
                SpeechPlan::sequence(vec![
                    SpeechUnit::Clip("intro_online".into()),
                    SpeechUnit::Tts(ctx.snap.track.clone()),
                    session,
                ]),
            ),
            mark: Mark::Intro,
        });
    }

    fn apply_mark(&mut self, _ctx: &RaceContext<'_>, mark: &Mark) {
        if matches!(mark, Mark::Intro) {
            self.spoke_session_intro = true;
        }
    }

    fn on_session_reset(&mut self) {
        self.spoke_session_intro = false;
    }
}
