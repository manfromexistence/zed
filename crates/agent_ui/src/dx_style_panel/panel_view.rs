use gpui::{App, EntityId, IntoElement, ScrollHandle, WeakEntity, Window};
use ui::{IconName, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::{DxStylePanelSnapshot, active_context::ActiveStyleContextSnapshot};

pub(super) fn render_panel(
    _snapshot: &DxStylePanelSnapshot,
    _active_context: &ActiveStyleContextSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel_id: EntityId,
    _scroll_handle: &ScrollHandle,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    v_flex()
        .id("dx-style-panel")
        .size_full()
        .min_h_0()
        .min_w_0()
        .bg(cx.theme().colors().panel_background)
        .child(panel_header(workspace, panel_id, cx))
        .child(
            div()
                .id("dx-style-panel-web-preview-host")
                .flex_1()
                .min_h_0()
                .min_w_0()
                .p_2()
                .child(
                    v_flex()
                        .id("dx-style-panel-web-preview")
                        .size_full()
                        .rounded_sm()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .bg(cx.theme().colors().editor_background)
                        .overflow_hidden()
                        .child(
                            h_flex()
                                .h(px(28.0))
                                .flex_none()
                                .px_2()
                                .border_b_1()
                                .border_color(cx.theme().colors().border)
                                .child(
                                    Label::new("Web Preview")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_h_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Label::new("Styles").size(LabelSize::Small)),
                        ),
                ),
        )
}
fn panel_header(
    workspace: &WeakEntity<Workspace>,
    panel_id: EntityId,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .justify_between()
        .gap_2()
        .px_2()
        .py_1()
        .child(
            h_flex()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(Icon::new(IconName::Sparkle).size(IconSize::Small))
                .child(Label::new("Style").size(LabelSize::Small).truncate()),
        )
        .child(side_panel_header_controls(
            "dx-style-panel",
            workspace.clone(),
            panel_id,
            cx,
        ))
}
