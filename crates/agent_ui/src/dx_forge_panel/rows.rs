use gpui::{AnyElement, App, SharedString, px};
use ui::{IconName, Tooltip, prelude::*};

use super::snapshot::{DxForgePanelState, DxForgeReceiptRow, DxForgeSourceRow};

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

pub(super) fn receipt_row(
    ix: usize,
    receipt: &DxForgeReceiptRow,
    open_button: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let color = if receipt.blocker_count > 0 {
        Color::Warning
    } else {
        Color::Muted
    };
    let tooltip_label = receipt.headline.clone();
    let tooltip_meta = receipt_tooltip(receipt);

    row_shell(
        SharedString::from(format!("dx-forge-receipt-{ix}")),
        IconName::FileTextOutlined,
        color,
        receipt.headline.clone(),
        receipt.detail.clone(),
        receipt.label.clone(),
        open_button,
        cx,
    )
    .tooltip(move |_, cx| Tooltip::with_meta(tooltip_label.clone(), None, tooltip_meta.clone(), cx))
    .into_any_element()
}

pub(super) fn source_row(
    id: SharedString,
    icon: IconName,
    source: &DxForgeSourceRow,
    open_button: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let color = if source.warnings.is_empty() {
        Color::Muted
    } else {
        Color::Warning
    };
    let tooltip_label = source.label.clone();
    let tooltip_meta = source_tooltip(source);

    row_shell(
        id,
        icon,
        color,
        source.label.clone(),
        source.detail.clone(),
        source.path.clone(),
        open_button,
        cx,
    )
    .tooltip(move |_, cx| Tooltip::with_meta(tooltip_label.clone(), None, tooltip_meta.clone(), cx))
    .into_any_element()
}

pub(super) fn empty_row(id: &'static str, label: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .id(id)
        .h(px(28.0))
        .w_full()
        .min_w_0()
        .gap_1()
        .pl_3()
        .pr_1()
        .border_1()
        .border_r_2()
        .child(
            Icon::new(IconName::Info)
                .size(IconSize::XSmall)
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

fn row_shell(
    id: SharedString,
    icon: IconName,
    icon_color: Color,
    title: String,
    detail: String,
    path: String,
    open_button: Option<AnyElement>,
    cx: &App,
) -> Div {
    let mut row = h_flex()
        .id(id)
        .w_full()
        .min_w_0()
        .gap_1p5()
        .pl_3()
        .pr_1()
        .py_1()
        .border_1()
        .border_r_2()
        .bg(cx.theme().colors().ghost_element_background)
        .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
        .active(|style| style.bg(cx.theme().colors().ghost_element_active))
        .child(Icon::new(icon).size(IconSize::Small).color(icon_color))
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_0p5()
                .child(Label::new(title).size(LabelSize::Small).truncate())
                .child(
                    Label::new(detail)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                )
                .child(
                    Label::new(path)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate_start(),
                ),
        );

    if let Some(open_button) = open_button {
        row = row.child(open_button);
    }

    row
}

fn receipt_tooltip(receipt: &DxForgeReceiptRow) -> String {
    let mut lines = vec![
        format!("{} - {}", receipt.kind, receipt.detail),
        receipt.source_path.clone(),
    ];
    if let Some(target_path) = receipt.target_path.as_ref() {
        lines.push(format!("Target: {target_path}"));
    }
    if let Some(destination) = receipt.restore_destination_root.as_ref() {
        lines.push(format!("Restore: {destination}"));
    }
    lines.join("\n")
}

fn source_tooltip(source: &DxForgeSourceRow) -> String {
    let mut lines = vec![source.detail.clone(), source.path.clone()];
    for receipt in source.receipts.iter().take(2) {
        lines.push(format!("{}: {}", receipt.label, receipt.detail));
    }
    for warning in source.warnings.iter().take(2) {
        lines.push(format!("Warning: {warning}"));
    }
    lines.join("\n")
}
