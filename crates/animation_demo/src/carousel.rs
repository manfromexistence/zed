use std::time::Instant;

use gpui::{App, Context, Render, Window, px};
use ui::prelude::*;

pub struct ScreenCarouselPreview {
    started_at: Instant,
}

impl ScreenCarouselPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for ScreenCarouselPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let phase = ((elapsed * 0.45).sin() * 0.5) + 0.5;
        let left_width = 160.0 + phase * 70.0;
        let center_width = 280.0 - phase * 40.0;
        let right_width = 150.0 + (1.0 - phase) * 90.0;
        let active_scale = 0.95 + phase * 0.05;

        v_flex()
            .flex_1()
            .min_h(px(320.0))
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .p_4()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("Screen Carousel").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("Directional gravity, live resize bias, and center/neighbor reveal are represented by the shifting panel widths below.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .flex_1()
                    .items_end()
                    .children([
                        screen_card(left_width, 0.96, "Left", cx),
                        screen_card(center_width, active_scale, "Center", cx),
                        screen_card(right_width, 0.96, "Right", cx),
                    ]),
            )
    }
}

fn screen_card(width: f32, scale: f32, label: &'static str, cx: &App) -> impl IntoElement + use<> {
    div()
        .w(px(width))
        .h(px(220.0))
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().element_background)
        .scale(scale)
        .opacity(scale)
        .child(
            v_flex()
                .size_full()
                .justify_between()
                .p_4()
                .child(Headline::new(label).size(HeadlineSize::XSmall))
                .child(Label::new(format!("{width:.0}px")).size(LabelSize::Small).color(Color::Muted)),
        )
}
