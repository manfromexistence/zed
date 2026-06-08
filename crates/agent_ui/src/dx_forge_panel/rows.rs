use gpui::{AnyElement, App, px};
use ui::{IconName, ListItem, ListItemSpacing, prelude::*};

use super::snapshot::DxForgePanelState;

pub(super) fn status_strip(
    state: DxForgePanelState,
    detail: String,
    workspace_scope: String,
    cx: &App,
) -> AnyElement {
    let (icon, color, label) = state_presentation(state);

    h_flex()
        .id("dx-forge-status")
        .h(px(32.0))
        .w_full()
        .min_w_0()
        .gap_2()
        .px_2()
        .border_y_1()
        .border_color(cx.theme().colors().border)
        .child(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(color)
                .truncate(),
        )
        .child(
            Label::new(detail)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(div().flex_1())
        .child(
            Label::new(workspace_scope)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn section_header(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    count: usize,
    cx: &App,
) -> AnyElement {
    h_flex()
        .id(id)
        .h(px(28.0))
        .w_full()
        .min_w_0()
        .pl_3()
        .pr_1()
        .gap_2()
        .justify_between()
        .border_1()
        .border_r_2()
        .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                .child(
                    Label::new(title)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(
            Label::new(count.to_string())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
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
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn state_presentation(state: DxForgePanelState) -> (IconName, Color, &'static str) {
    match state {
        DxForgePanelState::NoWorkspace => (IconName::Folder, Color::Muted, "No workspace"),
        DxForgePanelState::Ready => (IconName::Check, Color::Success, "Ready"),
        DxForgePanelState::Attention => (IconName::Warning, Color::Warning, "Needs attention"),
        DxForgePanelState::Empty => (IconName::Circle, Color::Muted, "Waiting for receipts"),
        DxForgePanelState::Missing => (IconName::Info, Color::Muted, "Not configured"),
    }
}
