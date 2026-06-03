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
                .id("dx-style-panel-empty-state")
                .flex_1()
                .min_h_0()
                .min_w_0()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .child(
                    div().max_w(px(240.)).child(
                        Label::new("Open an HTML, CSS, or TSX file to view Style controls.")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
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
