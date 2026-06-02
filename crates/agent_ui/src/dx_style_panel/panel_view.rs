use gpui::{Action, App, IntoElement, ScrollHandle, SharedString, Window};
use ui::{IconName, Tooltip, WithScrollbar, prelude::*};
use workspace::{CloseActiveSidePanel, SplitActiveSidePanel};
use zed_actions::dx_style::OpenGeneratorPreviewForContext;

use super::{
    DxStylePanelRow, DxStylePanelSnapshot, active_context::ActiveStyleContextSnapshot,
    panel_metric::metric,
};
const STYLE_PANEL_ROW_LIMIT: usize = 13;
pub(super) fn render_panel(
    snapshot: &DxStylePanelSnapshot,
    active_context: &ActiveStyleContextSnapshot,
    scroll_handle: &ScrollHandle,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let source_context_json = active_context.web_preview_context_json();
    let can_open_generator =
        snapshot.web_preview_bridge_ready && active_context.can_open_generator();
    v_flex()
        .id("dx-style-panel")
        .size_full()
        .min_h_0()
        .min_w_0()
        .gap_2()
        .p_2()
        .bg(cx.theme().colors().panel_background)
        .child(panel_header())
        .child(
            v_flex()
                .id("dx-style-panel-scroll")
                .flex_1()
                .min_h_0()
                .min_w_0()
                .gap_2()
                .overflow_y_scroll()
                .track_scroll(scroll_handle)
                .child(style_summary(snapshot, active_context, cx))
                .child(
                    Button::new(
                        "dx-style-panel-open-generator-preview",
                        "Open Web Preview Generators",
                    )
                    .full_width()
                    .label_size(LabelSize::Small)
                    .color(Color::Muted)
                    .start_icon(Icon::new(IconName::Sparkle).size(IconSize::Small))
                    .disabled(!can_open_generator)
                    .on_click(move |_, window, cx| {
                        window.dispatch_action(
                            OpenGeneratorPreviewForContext {
                                source_context_json: source_context_json.clone(),
                            }
                            .boxed_clone(),
                            cx,
                        );
                    }),
                )
                .child(style_rows(snapshot, cx))
                .vertical_scrollbar_for(scroll_handle, window, cx),
        )
}
fn panel_header() -> impl IntoElement {
    h_flex()
        .justify_between()
        .gap_2()
        .child(
            h_flex()
                .gap_1()
                .child(Icon::new(IconName::Sparkle).size(IconSize::Small))
                .child(Label::new("Style").size(LabelSize::Small)),
        )
        .child(
            h_flex()
                .gap_1()
                .items_center()
                .child(
                    IconButton::new("dx-style-panel-split-side-panel", IconName::SplitAlt)
                        .shape(ui::IconButtonShape::Square)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Split Panel"))
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(SplitActiveSidePanel), cx);
                        }),
                )
                .child(
                    IconButton::new("dx-style-panel-close-side-panel", IconName::Close)
                        .shape(ui::IconButtonShape::Square)
                        .icon_size(IconSize::Small)
                        .tooltip(Tooltip::text("Close Panel"))
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(CloseActiveSidePanel), cx);
                        }),
                ),
        )
}
fn style_summary(
    snapshot: &DxStylePanelSnapshot,
    active_context: &ActiveStyleContextSnapshot,
    cx: &App,
) -> impl IntoElement + use<> {
    let gate = &active_context.apply_gate;
    let generator_count = format!("{} planned", snapshot.visual_generator_count);
    let active_style_target = active_context
        .css_property
        .clone()
        .or_else(|| active_context.token.clone())
        .or_else(|| active_context.group_context.summary())
        .unwrap_or_else(|| active_context.status.clone());

    v_flex()
        .gap_1()
        .rounded_sm()
        .p_2()
        .bg(cx.theme().colors().element_background)
        .child(metric("Status", snapshot.status.clone()))
        .child(metric("Target", active_style_target))
        .child(metric(
            "Web Preview",
            if snapshot.web_preview_bridge_ready {
                "ready".to_string()
            } else if snapshot.web_preview_host_present {
                "host present".to_string()
            } else {
                "host missing".to_string()
            },
        ))
        .child(metric("Generators", generator_count))
        .when_some(active_context.css_generator.clone(), |this, generator| {
            this.child(metric("Generator", generator))
        })
        .child(metric("Apply", gate.state.clone()))
        .child(metric("Gate", gate.reason.clone()))
        .child(
            Label::new(snapshot.next_action.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
}
fn style_rows(snapshot: &DxStylePanelSnapshot, cx: &App) -> impl IntoElement + use<> {
    let mut stack = v_flex().gap_1().min_w_0();
    for (ix, row) in snapshot.rows.iter().take(STYLE_PANEL_ROW_LIMIT).enumerate() {
        stack = stack.child(style_row(
            SharedString::from(format!("dx-style-panel-row-{ix}")),
            row,
            cx,
        ));
    }
    for (ix, warning) in snapshot.warnings.iter().take(3).enumerate() {
        stack = stack.child(
            h_flex()
                .id(SharedString::from(format!("dx-style-panel-warning-{ix}")))
                .gap_1()
                .rounded_sm()
                .p_1()
                .bg(cx.theme().colors().element_background)
                .child(
                    Icon::new(IconName::Info)
                        .size(IconSize::XSmall)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(warning.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        );
    }

    stack
}
fn style_row(id: SharedString, row: &DxStylePanelRow, cx: &App) -> impl IntoElement + use<> {
    v_flex()
        .id(id)
        .gap_0p5()
        .rounded_sm()
        .p_1()
        .bg(cx.theme().colors().element_background)
        .child(metric(row.label.clone(), row.state.clone()))
        .child(
            Label::new(row.detail.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
}
