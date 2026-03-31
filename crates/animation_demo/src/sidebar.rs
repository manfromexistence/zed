use std::time::Instant;

use gpui::{Context, Render, Window, hsla, px};
use ui::prelude::*;

const COLLAPSED_WIDTH: f32 = 56.0;
const EXPANDED_WIDTH: f32 = 360.0;

pub struct SidebarPreview {
    started_at: Instant,
}

impl SidebarPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for SidebarPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let t = self.started_at.elapsed().as_secs_f32();
        let wave = ((t * 0.9).sin() * 0.5) + 0.5;
        let width = COLLAPSED_WIDTH + (EXPANDED_WIDTH - COLLAPSED_WIDTH) * wave;
        let content_opacity = ((wave - 0.15) / 0.85).clamp(0.0, 1.0);
        let folder_open = ((t * 1.3).sin() * 0.5) + 0.5;
        let media_offset = (1.0 - wave) * 20.0;

        v_flex()
            .w(px(width))
            .min_h(px(320.0))
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .overflow_hidden()
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_2()
                            .children((0..4).map(|ix| {
                                let glow = (((t * 1.7) + ix as f32).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                                div()
                                    .size(px(30.0 + glow * 7.5))
                                    .rounded_full()
                                    .bg(hsla(ix as f32 / 5.0, 0.7, 0.6, 1.0))
                            })),
                    )
                    .child(
                        v_flex()
                            .opacity(content_opacity)
                            .gap_2()
                            .child(Headline::new("Sidebar").size(HeadlineSize::XSmall))
                            .child(
                                Label::new("Width, fade, folder collapse, and media slide are driven together here.")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        v_flex()
                            .opacity(content_opacity)
                            .gap_1()
                            .child(Label::new("Workspace").size(LabelSize::XSmall).color(Color::Muted))
                            .children((0..3).map(|ix| {
                                div()
                                    .h(px(26.0 + folder_open * 8.0))
                                    .rounded_sm()
                                    .bg(cx.theme().colors().element_background)
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_between()
                                            .px_2()
                                            .py_1()
                                            .child(Label::new(format!("Folder {}", ix + 1)).size(LabelSize::Small))
                                            .child(Label::new(format!("{:.0}px", 120.0 * folder_open)).size(LabelSize::XSmall).color(Color::Muted)),
                                    )
                            })),
                    )
                    .child(
                        div()
                            .mt_auto()
                            .relative()
                            .top(px(media_offset))
                            .opacity(content_opacity)
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border_variant)
                            .bg(cx.theme().colors().element_background)
                            .p_3()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(Label::new("Media Player").size(LabelSize::Small))
                                    .child(Label::new("y: 20 -> 0 preview").size(LabelSize::XSmall).color(Color::Muted)),
                            ),
                    ),
            )
    }
}
