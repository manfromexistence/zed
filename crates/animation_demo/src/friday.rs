use std::time::Instant;

use gpui::{BoxShadow, Context, Render, Window, hsla, point, px};
use ui::prelude::*;

const PALETTE_SPAN_COUNT: usize = 25;
const RENDER_SPAN_COUNT: usize = 120;
const BORDER_THICKNESS: f32 = 10.0;
const GLOW_SPREAD: f32 = 8.0;
const GLOW_INTENSITY: f32 = 12.0;
const SLIDE_DURATION: f32 = 0.75;
const ACTIVE_DURATION: f32 = 2.4;
const EXIT_DURATION: f32 = 0.45;
const CYCLE_DURATION: f32 = SLIDE_DURATION + ACTIVE_DURATION + EXIT_DURATION;

pub struct FridayBorderPreview {
    started_at: Instant,
}

impl FridayBorderPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for FridayBorderPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();

        let elapsed = self.started_at.elapsed().as_secs_f32();
        let t = elapsed % CYCLE_DURATION;
        let active_t = ((t - SLIDE_DURATION) / ACTIVE_DURATION).clamp(0.0, 1.0);
        let exiting = ((t - SLIDE_DURATION - ACTIVE_DURATION) / EXIT_DURATION).clamp(0.0, 1.0);
        let fade = if t < SLIDE_DURATION + ACTIVE_DURATION {
            1.0
        } else {
            1.0 - exiting
        };
        let show_top = if t < SLIDE_DURATION { 0.0 } else { fade };
        let show_bottom = fade;
        let show_right = fade;
        let show_left = if t < SLIDE_DURATION {
            ((t - 0.3) / (SLIDE_DURATION - 0.3)).clamp(0.0, 1.0)
        } else {
            fade
        };
        let bounce_offset = if t < SLIDE_DURATION {
            0.0
        } else {
            (active_t * std::f32::consts::PI * 5.0).sin() * 14.0 * (1.0 - active_t * 0.65)
        };
        let shift = (elapsed * 0.42).fract();

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
                    .child(Headline::new("Friday Border").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("25-color segmented border, dual glow layers, 750ms entry, and bounce-backed center content.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex_1()
                    .min_h(px(240.0))
                    .rounded_lg()
                    .bg(cx.theme().colors().element_background)
                    .overflow_hidden()
                    .child(border_strip(shift, show_top, Edge::Top))
                    .child(border_strip(shift, show_bottom, Edge::Bottom))
                    .child(border_strip(shift, show_left, Edge::Left))
                    .child(border_strip(shift, show_right, Edge::Right))
                    .child(
                        div()
                            .absolute()
                            .inset_6()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .bg(cx.theme().colors().editor_background)
                            .child(
                                v_flex()
                                    .relative()
                                    .top(px(bounce_offset))
                                    .size_full()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .child(
                                        Headline::new("Active Phase")
                                            .size(HeadlineSize::Small),
                                    )
                                    .child(
                                        Label::new("Scroll bounce placeholder uses the same spring vocabulary that will drive the final workspace effect.")
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(
                                        Label::new(format!(
                                            "top {:.0}% • right {:.0}% • left {:.0}% • glow {}px + {}px",
                                            show_top * 100.0,
                                            show_right * 100.0,
                                            show_left * 100.0,
                                            GLOW_SPREAD as i32,
                                            GLOW_INTENSITY as i32
                                        ))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                    ),
                            ),
                    ),
            )
    }
}

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

fn border_strip(shift: f32, opacity: f32, edge: Edge) -> impl IntoElement + use<> {
    let is_horizontal = matches!(edge, Edge::Top | Edge::Bottom);
    let shadow_color = rainbow_color(shift, 0.92, 0.60).opacity(0.32 * opacity);

    let base = div()
        .absolute()
        .when(is_horizontal, |this| {
            this.left_0()
                .right_0()
                .h(px(BORDER_THICKNESS))
                .when(matches!(edge, Edge::Top), |this| this.top_0())
                .when(matches!(edge, Edge::Bottom), |this| this.bottom_0())
        })
        .when(!is_horizontal, |this| {
            this.top_0()
                .bottom_0()
                .w(px(BORDER_THICKNESS))
                .when(matches!(edge, Edge::Left), |this| this.left_0())
                .when(matches!(edge, Edge::Right), |this| this.right_0())
        })
        .opacity(opacity)
        .shadow(vec![
            BoxShadow {
                color: shadow_color,
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(GLOW_SPREAD),
                spread_radius: px(2.0),
            },
            BoxShadow {
                color: shadow_color.opacity(0.65),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(GLOW_INTENSITY + 4.0),
                spread_radius: px(4.0),
            },
        ]);

    if is_horizontal {
        base.child(
            h_flex().size_full().children(
                (0..RENDER_SPAN_COUNT).map(move |ix| {
                    let hue_offset = (ix as f32 / RENDER_SPAN_COUNT as f32 + shift).fract();
                    div()
                        .flex_1()
                        .h_full()
                        .bg(rainbow_color(hue_offset, 0.92, 0.60))
                }),
            ),
        )
    } else {
        base.child(
            v_flex().size_full().children(
                (0..RENDER_SPAN_COUNT).map(move |ix| {
                    let hue_offset = (ix as f32 / RENDER_SPAN_COUNT as f32 + shift).fract();
                    div()
                        .flex_1()
                        .w_full()
                        .bg(rainbow_color(hue_offset, 0.92, 0.60))
                }),
            ),
        )
    }
}

fn rainbow_color(hue_offset: f32, saturation: f32, lightness: f32) -> gpui::Hsla {
    let quantized = (hue_offset * PALETTE_SPAN_COUNT as f32).round() / PALETTE_SPAN_COUNT as f32;
    let hue = ((hue_offset * 0.7) + (quantized * 0.3)).fract();
    hsla(hue, saturation, lightness, 1.0)
}
