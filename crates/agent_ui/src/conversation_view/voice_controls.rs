use std::time::{Duration, Instant};

use gpui::{AnyElement, App, ClickEvent, IntoElement, Window};
use ui::{
    ButtonCommon, Clickable, Color, Icon, IconButton, IconName, IconSize, Label, LabelSize, Tooltip,
};
use ui::{h_flex, prelude::*, v_flex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComposerVoicePhase {
    Ready,
    Recording,
    Transcribing,
    Speaking,
    Error,
}

#[derive(Clone, Debug)]
pub(super) struct ComposerVoiceState {
    phase: ComposerVoicePhase,
    message: SharedString,
    started_at: Option<Instant>,
}

impl Default for ComposerVoiceState {
    fn default() -> Self {
        Self {
            phase: ComposerVoicePhase::Ready,
            message: "Flow voice ready".into(),
            started_at: None,
        }
    }
}

impl ComposerVoiceState {
    pub(super) fn phase(&self) -> ComposerVoicePhase {
        self.phase
    }

    pub(super) fn is_busy(&self) -> bool {
        matches!(
            self.phase,
            ComposerVoicePhase::Recording
                | ComposerVoicePhase::Transcribing
                | ComposerVoicePhase::Speaking
        )
    }

    pub(super) fn recording_started_at(&self) -> Option<Instant> {
        self.started_at
    }

    pub(super) fn set_ready(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Ready;
        self.message = message.into();
        self.started_at = None;
    }

    pub(super) fn set_recording(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Recording;
        self.message = message.into();
        self.started_at = Some(Instant::now());
    }

    pub(super) fn set_transcribing(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Transcribing;
        self.message = message.into();
        self.started_at = None;
    }

    pub(super) fn set_speaking(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Speaking;
        self.message = message.into();
        self.started_at = None;
    }

    pub(super) fn set_error(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Error;
        self.message = message.into();
        self.started_at = None;
    }

    fn voice_tooltip(&self) -> &'static str {
        match self.phase {
            ComposerVoicePhase::Recording => "Stop Flow voice recording",
            ComposerVoicePhase::Transcribing => "Flow is transcribing with Parakeet",
            ComposerVoicePhase::Speaking => "Flow is reading the composer with Kokoro",
            ComposerVoicePhase::Error => "Retry Flow voice input",
            ComposerVoicePhase::Ready => "Record voice input with Flow",
        }
    }

    fn speak_tooltip(&self) -> &'static str {
        match self.phase {
            ComposerVoicePhase::Speaking => "Kokoro is reading the composer",
            ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing => {
                "Finish voice recording before reading aloud"
            }
            ComposerVoicePhase::Error | ComposerVoicePhase::Ready => {
                "Read the composer aloud with Kokoro"
            }
        }
    }
}

pub(super) fn render_voice_buttons(
    state: &ComposerVoiceState,
    on_voice_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_speak_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Vec<AnyElement> {
    let voice_icon = match state.phase {
        ComposerVoicePhase::Recording => IconName::Stop,
        ComposerVoicePhase::Transcribing => IconName::LoadCircle,
        _ => IconName::Mic,
    };
    let voice_color = match state.phase {
        ComposerVoicePhase::Recording => Color::Error,
        ComposerVoicePhase::Transcribing => Color::Accent,
        ComposerVoicePhase::Error => Color::Warning,
        _ => Color::Muted,
    };
    let speak_disabled = matches!(
        state.phase,
        ComposerVoicePhase::Recording
            | ComposerVoicePhase::Transcribing
            | ComposerVoicePhase::Speaking
    );
    let speak_color = if state.phase == ComposerVoicePhase::Speaking {
        Color::Accent
    } else {
        Color::Muted
    };

    vec![
        IconButton::new("agent-composer-voice-input", voice_icon)
            .icon_size(IconSize::Small)
            .icon_color(voice_color)
            .tooltip(Tooltip::text(state.voice_tooltip()))
            .on_click(on_voice_click)
            .into_any_element(),
        IconButton::new("agent-composer-text-to-speech", IconName::AudioOn)
            .icon_size(IconSize::Small)
            .icon_color(speak_color)
            .disabled(speak_disabled)
            .tooltip(Tooltip::text(state.speak_tooltip()))
            .on_click(on_speak_click)
            .into_any_element(),
    ]
}

pub(super) fn render_voice_recording_panel(
    state: &ComposerVoiceState,
    cx: &App,
) -> Option<AnyElement> {
    if state.phase == ComposerVoicePhase::Ready {
        return None;
    }

    let (title, tone, detail) = match state.phase {
        ComposerVoicePhase::Recording => (
            "Recording with Flow",
            Color::Error,
            state
                .recording_started_at()
                .map(format_elapsed)
                .unwrap_or_else(|| "Listening".into()),
        ),
        ComposerVoicePhase::Transcribing => (
            "Transcribing with Parakeet",
            Color::Accent,
            "Preparing transcript".into(),
        ),
        ComposerVoicePhase::Speaking => (
            "Reading with Kokoro",
            Color::Accent,
            "Synthesizing composer text".into(),
        ),
        ComposerVoicePhase::Error => (
            "Flow voice needs attention",
            Color::Warning,
            state.message.clone(),
        ),
        ComposerVoicePhase::Ready => unreachable!(),
    };

    Some(
        h_flex()
            .id("agent-composer-voice-recording-panel")
            .w_full()
            .justify_between()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(tone.color(cx).alpha(0.08))
            .border_1()
            .border_color(tone.color(cx).alpha(0.18))
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        Icon::new(status_icon(state.phase))
                            .size(IconSize::XSmall)
                            .color(tone),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(Label::new(title).size(LabelSize::Small).color(tone))
                            .child(
                                Label::new(detail)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    ),
            )
            .child(
                Label::new(state.message.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .into_any_element(),
    )
}

fn status_icon(phase: ComposerVoicePhase) -> IconName {
    match phase {
        ComposerVoicePhase::Recording => IconName::Mic,
        ComposerVoicePhase::Transcribing | ComposerVoicePhase::Speaking => IconName::LoadCircle,
        ComposerVoicePhase::Error => IconName::Warning,
        ComposerVoicePhase::Ready => IconName::Mic,
    }
}

fn format_elapsed(started_at: Instant) -> SharedString {
    let elapsed = started_at.elapsed();
    let seconds = elapsed.as_secs().min(Duration::from_secs(599).as_secs());
    format!("{:02}:{:02}", seconds / 60, seconds % 60).into()
}
