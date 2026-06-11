use gpui::{AnyElement, App, EntityId, IntoElement, WeakEntity, rems};
use ui::{IconName, ListHeader, ListItem, ListItemSpacing, Tab, Tooltip, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::snapshot::DxForgePanelState;

pub(super) fn panel_header(
    workspace: &WeakEntity<Workspace>,
    panel_id: EntityId,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .id("dx-forge-panel-header")
        .h(Tab::container_height(cx))
        .w_full()
        .min_w_0()
        .items_center()
        .justify_between()
        .gap_2()
        .px_2()
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(
            h_flex()
                .min_w_0()
                .items_center()
                .gap_1()
                .child(Icon::new(dx_icon(DxUiIcon::Forge)).size(IconSize::Small))
                .child(
                    Label::new("Forge")
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .child(side_panel_header_controls(
            "dx-forge-panel",
            workspace.clone(),
            panel_id,
            cx,
        ))
}

pub(super) fn status_strip(
    state: DxForgePanelState,
    detail: String,
    workspace_scope: String,
    actions: AnyElement,
    cx: &App,
) -> AnyElement {
    let (icon, color, label) = state_presentation(state);
    let tooltip_title = SharedString::from(label);
    let tooltip_meta = format!("{detail}\n{workspace_scope}");

    div()
        .w_full()
        .border_t_1()
        .border_color(cx.theme().colors().border)
        .child(
            ListItem::new("dx-forge-status")
                .inset(true)
                .selectable(false)
                .height(rems(1.75))
                .spacing(ListItemSpacing::Sparse)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
                .child(
                    h_flex().w_full().min_w_0().gap_1p5().child(
                        Label::new(label)
                            .size(LabelSize::Small)
                            .color(color)
                            .truncate(),
                    ),
                )
                .end_slot(h_flex().flex_none().gap_0p5().child(actions))
                .tooltip(move |_, cx| {
                    Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
                }),
        )
        .into_any_element()
}

pub(super) fn section_header(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    count: usize,
    _cx: &App,
) -> AnyElement {
    let count_tooltip = format!("{count} {}", title.to_ascii_lowercase());
    div()
        .id(id)
        .tooltip(Tooltip::text(count_tooltip))
        .child(
            ListHeader::new(title)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .end_slot(
                    Label::new(count.to_string())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .into_any_element()
}

pub(super) fn empty_row(id: &'static str, label: &'static str, _cx: &App) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Info)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .tooltip(Tooltip::text(label))
        .into_any_element()
}

pub(super) fn state_presentation(state: DxForgePanelState) -> (IconName, Color, &'static str) {
    match state {
        DxForgePanelState::NoWorkspace => (IconName::Folder, Color::Muted, "No workspace"),
        DxForgePanelState::Ready => (IconName::Check, Color::Success, "Ready"),
        DxForgePanelState::Evidence => (IconName::FileTextOutlined, Color::Muted, "Evidence"),
        DxForgePanelState::Attention => (IconName::Warning, Color::Warning, "Review"),
        DxForgePanelState::Empty => (IconName::Circle, Color::Muted, "No receipts"),
        DxForgePanelState::Missing => (IconName::Info, Color::Muted, "Missing"),
    }
}
