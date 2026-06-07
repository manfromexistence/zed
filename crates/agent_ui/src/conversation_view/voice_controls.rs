use std::time::{Duration, Instant};

use gpui::{AnyElement, App, ClickEvent, IntoElement, Window};
use ui::{
    Button, ButtonCommon, ButtonSize, Clickable, Color, Icon, IconButton, IconName, IconSize,
    Label, LabelSize, Tooltip,
};
use ui::{h_flex, prelude::*, v_flex};

const VOICE_LEVEL_BAR_COUNT: usize = 12;
const MAX_RECORDING_DURATION_LABEL: &str = "90s max";
const VOICE_RECORDING_STOP_TRANSCRIBE_LABEL: &str = "Stop and transcribe";
const VOICE_RECORDING_STOP_TRANSCRIBE_TOOLTIP: &str = "Stop recording and transcribe with Flow";
const VOICE_TRANSCRIPTION_CANCEL_LABEL: &str = "Cancel";
const VOICE_TRANSCRIPTION_CANCEL_TOOLTIP: &str = "Cancel Flow transcription";
const VOICE_RECORDING_DISCARD_LABEL: &str = "Discard";
const VOICE_RECORDING_DISCARD_TOOLTIP: &str = "Discard recording";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComposerVoicePhase {
    Ready,
    Recording,
    Transcribing,
    Synthesizing,
    Speaking,
    Error,
}

#[derive(Clone, Debug)]
pub(super) struct ComposerVoiceState {
    phase: ComposerVoicePhase,
    message: SharedString,
    started_at: Option<Instant>,
    captured_duration: Duration,
    input_level: f32,
}

#[derive(Clone, Debug)]
pub(super) struct ComposerVoiceAvailability {
    pub(super) has_composer_text: bool,
    pub(super) stt_ready: bool,
    pub(super) stt_status: SharedString,
    pub(super) tts_ready: bool,
    pub(super) tts_status: SharedString,
}

impl Default for ComposerVoiceState {
    fn default() -> Self {
        Self {
            phase: ComposerVoicePhase::Ready,
            message: "Flow voice ready".into(),
            started_at: None,
            captured_duration: Duration::ZERO,
            input_level: 0.0,
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
                | ComposerVoicePhase::Synthesizing
                | ComposerVoicePhase::Speaking
        )
    }

    pub(super) fn recording_started_at(&self) -> Option<Instant> {
        self.started_at
    }

    pub(super) fn update_recording_telemetry(
        &mut self,
        captured_duration: Duration,
        input_level: f32,
    ) -> bool {
        if self.phase != ComposerVoicePhase::Recording {
            return false;
        }

        let input_level = input_level.clamp(0.0, 1.0);
        let previous_duration_tick = recording_duration_tick(self.captured_duration);
        let next_duration_tick = recording_duration_tick(captured_duration);
        let previous_level_bars = voice_level_bar_count(self.input_level);
        let next_level_bars = voice_level_bar_count(input_level);

        self.captured_duration = captured_duration;
        self.input_level = input_level;

        previous_duration_tick != next_duration_tick || previous_level_bars != next_level_bars
    }

    pub(super) fn set_ready(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Ready;
        self.message = message.into();
        self.started_at = None;
        self.captured_duration = Duration::ZERO;
        self.input_level = 0.0;
    }

    pub(super) fn set_recording(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Recording;
        self.message = message.into();
        self.started_at = Some(Instant::now());
        self.captured_duration = Duration::ZERO;
        self.input_level = 0.0;
    }

    pub(super) fn set_transcribing(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Transcribing;
        self.message = message.into();
        self.started_at = None;
        self.input_level = 0.0;
    }

    pub(super) fn set_synthesizing(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Synthesizing;
        self.message = message.into();
        self.started_at = None;
        self.input_level = 0.0;
    }

    pub(super) fn set_speaking(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Speaking;
        self.message = message.into();
        self.started_at = None;
        self.input_level = 0.0;
    }

    pub(super) fn set_error(&mut self, message: impl Into<SharedString>) {
        self.phase = ComposerVoicePhase::Error;
        self.message = message.into();
        self.started_at = None;
        self.input_level = 0.0;
    }

    fn voice_tooltip(&self, availability: &ComposerVoiceAvailability) -> SharedString {
        match self.phase {
            ComposerVoicePhase::Recording => VOICE_RECORDING_STOP_TRANSCRIBE_TOOLTIP.into(),
            ComposerVoicePhase::Transcribing => VOICE_TRANSCRIPTION_CANCEL_TOOLTIP.into(),
            ComposerVoicePhase::Synthesizing => "Kokoro is generating audio".into(),
            ComposerVoicePhase::Speaking => "Kokoro read-aloud is active".into(),
            ComposerVoicePhase::Error | ComposerVoicePhase::Ready if !availability.stt_ready => {
                availability.stt_status.clone()
            }
            ComposerVoicePhase::Error => "Retry Flow voice input".into(),
            ComposerVoicePhase::Ready => "Record voice input with Flow".into(),
        }
    }

    fn speak_tooltip(&self, availability: &ComposerVoiceAvailability) -> SharedString {
        match self.phase {
            ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => {
                "Stop Kokoro read-aloud".into()
            }
            ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing => {
                "Finish voice recording before reading aloud".into()
            }
            ComposerVoicePhase::Error | ComposerVoicePhase::Ready if !availability.tts_ready => {
                availability.tts_status.clone()
            }
            ComposerVoicePhase::Error | ComposerVoicePhase::Ready
                if !availability.has_composer_text =>
            {
                "Type text in the composer before reading aloud".into()
            }
            ComposerVoicePhase::Error | ComposerVoicePhase::Ready => {
                "Read the composer aloud with Kokoro".into()
            }
        }
    }
}

pub(super) fn render_voice_buttons(
    state: &ComposerVoiceState,
    availability: ComposerVoiceAvailability,
    on_voice_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_speak_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Vec<AnyElement> {
    let voice_icon = match state.phase {
        ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing => IconName::Stop,
        ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => IconName::Mic,
        ComposerVoicePhase::Ready | ComposerVoicePhase::Error => IconName::Mic,
    };
    let voice_color = match state.phase {
        ComposerVoicePhase::Recording => Color::Error,
        ComposerVoicePhase::Transcribing | ComposerVoicePhase::Synthesizing => Color::Accent,
        ComposerVoicePhase::Error => Color::Warning,
        ComposerVoicePhase::Ready if !availability.stt_ready => Color::Warning,
        ComposerVoicePhase::Speaking | ComposerVoicePhase::Ready => Color::Muted,
    };
    let speak_disabled = match state.phase {
        ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => false,
        ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing => true,
        ComposerVoicePhase::Ready | ComposerVoicePhase::Error => !availability.has_composer_text,
    };
    let voice_disabled = match state.phase {
        ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => true,
        ComposerVoicePhase::Ready | ComposerVoicePhase::Error => false,
        ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing => false,
    };
    let speak_icon = match state.phase {
        ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => IconName::Stop,
        _ => IconName::AudioOn,
    };
    let speak_color = match state.phase {
        ComposerVoicePhase::Synthesizing | ComposerVoicePhase::Speaking => Color::Accent,
        ComposerVoicePhase::Ready | ComposerVoicePhase::Error
            if availability.has_composer_text && !availability.tts_ready =>
        {
            Color::Warning
        }
        _ => Color::Muted,
    };

    vec![
        IconButton::new("agent-composer-voice-input", voice_icon)
            .icon_size(IconSize::Small)
            .icon_color(voice_color)
            .disabled(voice_disabled)
            .tooltip(Tooltip::text(state.voice_tooltip(&availability)))
            .on_click(on_voice_click)
            .into_any_element(),
        IconButton::new("agent-composer-text-to-speech", speak_icon)
            .icon_size(IconSize::Small)
            .icon_color(speak_color)
            .disabled(speak_disabled)
            .tooltip(Tooltip::text(state.speak_tooltip(&availability)))
            .on_click(on_speak_click)
            .into_any_element(),
    ]
}

pub(super) fn render_voice_recording_panel(
    state: &ComposerVoiceState,
    on_stop_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_cancel_recording_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_retry_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_dismiss_error_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Option<AnyElement> {
    if state.phase == ComposerVoicePhase::Ready {
        return None;
    }

    let (title, tone, detail) = match state.phase {
        ComposerVoicePhase::Recording => {
            ("Recording with Flow", Color::Error, recording_detail(state))
        }
        ComposerVoicePhase::Transcribing => (
            "Transcribing with Flow STT",
            Color::Accent,
            "Preparing transcript".into(),
        ),
        ComposerVoicePhase::Synthesizing => (
            "Generating Kokoro audio",
            Color::Accent,
            "Preparing generated audio".into(),
        ),
        ComposerVoicePhase::Speaking => (
            "Reading with Kokoro",
            Color::Accent,
            "Playing generated audio".into(),
        ),
        ComposerVoicePhase::Error => (
            "Flow voice needs attention",
            Color::Warning,
            "Retry or dismiss to continue".into(),
        ),
        ComposerVoicePhase::Ready => unreachable!(),
    };
    let (stop_button_id, stop_button_label, stop_button_tooltip) = match state.phase {
        ComposerVoicePhase::Recording => (
            "agent-composer-stop-voice-recording",
            VOICE_RECORDING_STOP_TRANSCRIBE_LABEL,
            VOICE_RECORDING_STOP_TRANSCRIBE_TOOLTIP,
        ),
        ComposerVoicePhase::Transcribing => (
            "agent-composer-cancel-flow-transcription",
            VOICE_TRANSCRIPTION_CANCEL_LABEL,
            VOICE_TRANSCRIPTION_CANCEL_TOOLTIP,
        ),
        _ => (
            "agent-composer-stop-flow-voice-action",
            "Stop",
            "Stop Flow voice action",
        ),
    };

    Some(
        v_flex()
            .id("agent-composer-voice-recording-panel")
            .w_full()
            .gap_1()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(tone.color(cx).alpha(0.08))
            .border_1()
            .border_color(tone.color(cx).alpha(0.18))
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .gap_2()
                    .child(
                        h_flex()
                            .min_w_0()
                            .flex_1()
                            .gap_1p5()
                            .items_center()
                            .child(
                                Icon::new(status_icon(state.phase))
                                    .size(IconSize::XSmall)
                                    .color(tone),
                            )
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(
                                        Label::new(title)
                                            .size(LabelSize::Small)
                                            .color(tone)
                                            .truncate(),
                                    )
                                    .child(
                                        Label::new(detail)
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted)
                                            .truncate(),
                                    ),
                            ),
                    )
                    .when(
                        matches!(
                            state.phase,
                            ComposerVoicePhase::Recording | ComposerVoicePhase::Transcribing
                        ),
                        |this| {
                            this.child(
                                h_flex()
                                    .flex_wrap()
                                    .gap_1()
                                    .child(
                                        Button::new(stop_button_id, stop_button_label)
                                            .size(ButtonSize::Compact)
                                            .label_size(LabelSize::XSmall)
                                            .color(tone)
                                            .start_icon(
                                                Icon::new(IconName::Stop)
                                                    .size(IconSize::XSmall)
                                                    .color(tone),
                                            )
                                            .tooltip(Tooltip::text(stop_button_tooltip))
                                            .on_click(on_stop_click),
                                    )
                                    .when(state.phase == ComposerVoicePhase::Recording, |this| {
                                        this.child(
                                            Button::new(
                                                "agent-composer-discard-voice-recording",
                                                VOICE_RECORDING_DISCARD_LABEL,
                                            )
                                            .size(ButtonSize::Compact)
                                            .label_size(LabelSize::XSmall)
                                            .color(Color::Error)
                                            .start_icon(
                                                Icon::new(IconName::Trash)
                                                    .size(IconSize::XSmall)
                                                    .color(Color::Error),
                                            )
                                            .tooltip(Tooltip::text(VOICE_RECORDING_DISCARD_TOOLTIP))
                                            .on_click(on_cancel_recording_click),
                                        )
                                    }),
                            )
                        },
                    )
                    .when(state.phase == ComposerVoicePhase::Error, |this| {
                        this.child(
                            h_flex()
                                .gap_1()
                                .child(
                                    IconButton::new(
                                        "agent-composer-retry-voice-input",
                                        IconName::RotateCw,
                                    )
                                    .icon_size(IconSize::XSmall)
                                    .icon_color(tone)
                                    .tooltip(Tooltip::text("Retry Flow voice input"))
                                    .on_click(on_retry_click),
                                )
                                .child(
                                    IconButton::new(
                                        "agent-composer-dismiss-voice-error",
                                        IconName::Close,
                                    )
                                    .icon_size(IconSize::XSmall)
                                    .icon_color(Color::Muted)
                                    .tooltip(Tooltip::text("Dismiss Flow voice error"))
                                    .on_click(on_dismiss_error_click),
                                ),
                        )
                    }),
            )
            .when(state.phase == ComposerVoicePhase::Recording, |this| {
                this.child(render_voice_level_meter(state.input_level, tone, cx))
            })
            .child(
                Label::new(state.message.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
            .into_any_element(),
    )
}

fn render_voice_level_meter(level: f32, tone: Color, cx: &App) -> AnyElement {
    let active_bars = voice_level_bar_count(level);

    h_flex()
        .id("agent-composer-voice-level-meter")
        .w_full()
        .h_2()
        .items_end()
        .gap_0p5()
        .children((0..VOICE_LEVEL_BAR_COUNT).map(|index| {
            let is_active = index < active_bars;
            let height = px(3.0 + (index % 4) as f32 * 2.0);
            div().w(px(3.0)).h(height).rounded_full().bg(if is_active {
                tone.color(cx).alpha(0.70)
            } else {
                Color::Muted.color(cx).alpha(0.18)
            })
        }))
        .into_any_element()
}

fn voice_level_bar_count(level: f32) -> usize {
    (level.clamp(0.0, 1.0) * VOICE_LEVEL_BAR_COUNT as f32).ceil() as usize
}

fn recording_duration_tick(duration: Duration) -> u128 {
    duration.as_millis() / 500
}

fn recording_detail(state: &ComposerVoiceState) -> SharedString {
    let elapsed = state
        .recording_started_at()
        .map(|started_at| format_clock(started_at.elapsed()))
        .unwrap_or_else(|| "00:00".to_string());
    let captured = format_captured_duration(state.captured_duration);
    format!("{elapsed} elapsed / {captured} captured / {MAX_RECORDING_DURATION_LABEL}").into()
}

fn format_clock(duration: Duration) -> String {
    let seconds = duration.as_secs().min(Duration::from_secs(599).as_secs());
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn format_captured_duration(duration: Duration) -> String {
    format!("{:.1}s", duration.as_secs_f32())
}

fn status_icon(phase: ComposerVoicePhase) -> IconName {
    match phase {
        ComposerVoicePhase::Recording => IconName::Mic,
        ComposerVoicePhase::Transcribing
        | ComposerVoicePhase::Synthesizing
        | ComposerVoicePhase::Speaking => IconName::LoadCircle,
        ComposerVoicePhase::Error => IconName::Warning,
        ComposerVoicePhase::Ready => IconName::Mic,
    }
}
