use std::time::Instant;

use gpui::{BoxShadow, Context, Render, Window, hsla, point, px};
use ui::prelude::*;

pub struct MacOsDockPreview {
    started_at: Instant,
}

impl MacOsDockPreview {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Render for MacOsDockPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let active_ix = ((elapsed / 2.2) as usize) % 5;
        let hover_center = ((elapsed * 1.2).sin() * 0.5) + 0.5;

        v_flex()
            .w_full()
            .min_h(px(150.0))
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .p_4()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(Headline::new("macOS Dock").size(HeadlineSize::XSmall))
                    .child(
                        Label::new("Hover magnification, active glow, and tab switching rhythm preview.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                h_flex()
                    .items_end()
                    .justify_center()
                    .gap_3()
                    .children((0..5).map(|ix| {
                        let distance = ((ix as f32 / 4.0) - hover_center).abs();
                        let influence = (1.0 - distance * 2.4).clamp(0.0, 1.0);
                        let scale = 1.0 + influence * 0.3;
                        let is_active = ix == active_ix;
                        div()
                            .w(px(44.0))
                            .h(px(44.0))
                            .rounded_lg()
                            .bg(hsla(ix as f32 / 5.0, 0.7, 0.62, 1.0))
                            .scale(scale)
                            .shadow(if is_active {
                                vec![BoxShadow {
                                    color: hsla(ix as f32 / 5.0, 0.8, 0.62, 0.45),
                                    offset: point(px(0.0), px(0.0)),
                                    blur_radius: px(12.0),
                                    spread_radius: px(4.0),
                                }]
                            } else {
                                Vec::new()
                            })
                    })),
            )
    }
}
