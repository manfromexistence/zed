use std::time::Instant;

use gpui::{BoxShadow, Context, Render, Window, hsla, point, px};
use ui::prelude::*;

const SPAN_COUNT: usize = 25;
const CORNER_RADIUS: f32 = 12.0;

pub struct HelloGlowPreview {
    started_at: Instant,
}

impl HelloGlowPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for HelloGlowPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let shift = ((self.started_at.elapsed().as_secs_f32() * 4.0) as usize) % SPAN_COUNT;
        let accent = rainbow_color((shift + (SPAN_COUNT / 3)) % SPAN_COUNT).opacity(0.45);

        v_flex()
            .flex_1()
            .h_full()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .p_4()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("Hello Glow").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("Animated rainbow glow card with a 6-second shift cycle and dual glow layers.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex_1()
                    .min_h(px(200.0))
                    .rounded(px(CORNER_RADIUS))
                    .bg(cx.theme().colors().editor_background)
                    .shadow(vec![
                        BoxShadow {
                            color: accent,
                            offset: point(px(0.0), px(0.0)),
                            blur_radius: px(6.0),
                            spread_radius: px(2.0),
                        },
                        BoxShadow {
                            color: accent.opacity(0.7),
                            offset: point(px(0.0), px(0.0)),
                            blur_radius: px(18.0),
                            spread_radius: px(8.0),
                        },
                    ])
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .absolute()
                            .inset_0()
                            .size_full()
                            .children((0..SPAN_COUNT).map(move |ix| {
                                div()
                                    .flex_1()
                                    .h_full()
                                    .bg(rainbow_color((ix + shift) % SPAN_COUNT).opacity(0.92))
                            })),
                    )
                    .child(
                        div()
                            .absolute()
                            .inset(px(10.0))
                            .rounded(px(CORNER_RADIUS - 2.0))
                            .bg(cx.theme().colors().elevated_surface_background.opacity(0.72)),
                    )
                    .child(
                        v_flex()
                            .relative()
                            .size_full()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                Headline::new("Hello")
                                    .size(HeadlineSize::Small),
                            )
                            .child(
                                Label::new("6s gradient loop • 6px inner glow • 18px outer glow")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(
                                Label::new(format!(
                                    "radius {}px • span count {}",
                                    CORNER_RADIUS as i32, SPAN_COUNT
                                ))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                            ),
                    ),
            )
    }
}

fn rainbow_color(index: usize) -> gpui::Hsla {
    hsla(index as f32 / SPAN_COUNT as f32, 0.8, 0.6, 1.0)
}
