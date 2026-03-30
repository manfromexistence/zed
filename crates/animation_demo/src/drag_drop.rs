use std::time::Instant;

use gpui::{BoxShadow, Context, Render, Window, hsla, point, px};
use ui::prelude::*;

pub struct DragDropPreview {
    started_at: Instant,
}

impl DragDropPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for DragDropPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let motion = ((elapsed * 1.4).sin() * 0.5) + 0.5;
        let indicator_x = 24.0 + motion * 180.0;

        v_flex()
            .w_full()
            .min_h(px(220.0))
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .p_4()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("Drag and Drop").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("Ghost card, blue drop line, and before/after indicator rhythm preview.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .rounded_md()
                    .bg(cx.theme().colors().element_background)
                    .border_1()
                    .border_color(cx.theme().colors().border_variant)
                    .child(
                        h_flex()
                            .gap_2()
                            .p_4()
                            .children((0..4).map(|ix| {
                                div()
                                    .w(px(92.0))
                                    .h(px(44.0))
                                    .rounded_md()
                                    .bg(cx.theme().colors().surface_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border)
                                    .child(
                                        h_flex()
                                            .size_full()
                                            .justify_center()
                                            .items_center()
                                            .child(Label::new(format!("Tab {}", ix + 1)).size(LabelSize::Small)),
                                    )
                            })),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(58.0))
                            .left(px(indicator_x))
                            .w(px(64.0))
                            .h(px(2.0))
                            .bg(hsla(0.58, 0.9, 0.62, 1.0)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(53.0))
                            .left(px(indicator_x + 26.0))
                            .size(px(8.0))
                            .rounded_full()
                            .bg(hsla(0.58, 0.9, 0.62, 1.0)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(82.0))
                            .left(px(48.0 + motion * 120.0))
                            .w(px(92.0))
                            .h(px(44.0))
                            .rounded_md()
                            .bg(hsla(0.62, 0.7, 0.62, 1.0))
                            .opacity(0.5)
                            .shadow(vec![BoxShadow {
                                color: hsla(0.0, 0.0, 0.0, 0.25),
                                offset: point(px(0.0), px(10.0)),
                                blur_radius: px(20.0),
                                spread_radius: px(3.0),
                            }])
                            .child(
                                h_flex()
                                    .size_full()
                                    .justify_center()
                                    .items_center()
                                    .child(Label::new("Dragging").size(LabelSize::Small)),
                            ),
                    ),
            )
    }
}
