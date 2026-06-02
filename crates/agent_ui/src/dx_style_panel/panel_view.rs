use gpui::{App, IntoElement, ScrollHandle, SharedString, Window};
use ui::{IconName, Tooltip, WithScrollbar, prelude::*};
use workspace::{CloseActiveSidePanel, SplitActiveSidePanel};

use super::{
    DxStylePanelRow, DxStylePanelSnapshot,
    active_context::ActiveStyleContextSnapshot,
    panel_cards::{generator_host_card, readiness_card, style_context_card},
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
                .child(generator_host_card(
                    snapshot,
                    source_context_json,
                    can_open_generator,
                    cx,
                ))
                .child(style_context_card(snapshot, active_context, cx))
                .child(readiness_card(snapshot, cx))
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
                .flex_1()
                .min_w_0()
                .child(Icon::new(IconName::Sparkle).size(IconSize::Small))
                .child(
                    Label::new("Style Generators")
                        .size(LabelSize::Small)
                        .truncate(),
                ),
        )
        .child(
            h_flex()
                .gap_1()
                .items_center()
                .flex_none()
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
fn style_rows(snapshot: &DxStylePanelSnapshot, cx: &App) -> impl IntoElement + use<> {
    let mut stack = v_flex()
        .id("dx-style-panel-contracts")
        .gap_1()
        .min_w_0()
        .child(section_label("Readiness Contracts"));
    for (ix, row) in snapshot
        .rows
        .iter()
        .filter(|row| row.label != "Native Sidebar")
        .take(STYLE_PANEL_ROW_LIMIT)
        .enumerate()
    {
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
fn section_label(label: &'static str) -> impl IntoElement {
    h_flex()
        .gap_1()
        .items_center()
        .child(
            Icon::new(IconName::FileTextOutlined)
                .size(IconSize::XSmall)
                .color(Color::Muted),
        )
        .child(
            Label::new(label)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
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
