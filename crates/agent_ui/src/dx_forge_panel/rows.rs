use gpui::{AnyElement, App, SharedString};
use ui::{IconName, ListHeader, ListItem, ListItemSpacing, prelude::*};

use super::snapshot::DxForgePanelState;

pub(super) fn status_strip(
    state: DxForgePanelState,
    detail: String,
    workspace_scope: String,
    actions: AnyElement,
    cx: &App,
) -> AnyElement {
    let (icon, color, label) = state_presentation(state);

    ListItem::new("dx-forge-status")
        .selectable(false)
        .spacing(ListItemSpacing::Dense)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            h_flex()
                .w_full()
                .min_w_0()
                .gap_1p5()
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
                        .truncate()
                        .flex_1(),
                ),
        )
        .end_slot(
            h_flex()
                .flex_none()
                .gap_1()
                .child(status_chip(workspace_scope, Color::Muted, cx))
                .child(actions),
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
    div()
        .id(id)
        .child(
            ListHeader::new(title)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                .end_slot(count_chip(count, Color::Muted, cx)),
        )
        .into_any_element()
}

pub(super) fn status_chip(label: impl Into<SharedString>, color: Color, _cx: &App) -> AnyElement {
    h_flex()
        .h_5()
        .min_w_0()
        .items_center()
        .gap_0p5()
        .px_0p5()
        .child(
            Icon::new(IconName::Circle)
                .size(IconSize::XSmall)
                .color(color),
        )
        .child(
            Label::new(label)
                .size(LabelSize::XSmall)
                .color(color)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn count_chip(count: usize, color: Color, _cx: &App) -> AnyElement {
    h_flex()
        .h_5()
        .min_w_5()
        .items_center()
        .justify_center()
        .px_0p5()
        .child(
            Label::new(count.to_string())
                .size(LabelSize::XSmall)
                .color(color)
                .truncate(),
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
        DxForgePanelState::Attention => (IconName::Warning, Color::Warning, "Review"),
        DxForgePanelState::Empty => (IconName::Circle, Color::Muted, "No receipts"),
        DxForgePanelState::Missing => (IconName::Info, Color::Muted, "Missing"),
    }
}
