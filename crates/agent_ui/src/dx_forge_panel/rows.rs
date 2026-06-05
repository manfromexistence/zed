use gpui::{AnyElement, App, SharedString, px};
use ui::{IconName, Tooltip, prelude::*};

use super::snapshot::{DxForgePanelState, DxForgeReceiptRow, DxForgeSourceRow};

pub(super) fn section(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    body: AnyElement,
    cx: &App,
) -> AnyElement {
    v_flex()
        .id(id)
        .min_w_0()
        .gap_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().element_background)
        .px_2()
        .py_1()
        .child(
            h_flex()
                .gap_1()
                .min_w_0()
                .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                .child(Label::new(title).size(LabelSize::Small).truncate()),
        )
        .child(body)
        .into_any_element()
}

pub(super) fn receipt_row(ix: usize, receipt: &DxForgeReceiptRow, cx: &App) -> AnyElement {
    let color = if receipt.blocker_count > 0 {
        Color::Warning
    } else {
        Color::Muted
    };

    let mut stack = v_flex()
        .id(SharedString::from(format!("dx-forge-receipt-{ix}")))
        .min_w_0()
        .gap_0p5()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|style| style.bg(cx.theme().colors().element_hover))
        .child(
            h_flex()
                .gap_1()
                .min_w_0()
                .child(
                    Icon::new(IconName::FileTextOutlined)
                        .size(IconSize::XSmall)
                        .color(color),
                )
                .child(
                    Label::new(receipt.headline.clone())
                        .size(LabelSize::Small)
                        .truncate(),
                ),
        )
        .child(
            Label::new(format!("{} - {}", receipt.kind, receipt.detail))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(
            Label::new(receipt.label.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        );
    let tooltip_label = receipt.headline.clone();
    let tooltip_meta = format!("{} - {}\n{}", receipt.kind, receipt.detail, receipt.label);
    stack = stack.tooltip(move |_, cx| {
        Tooltip::with_meta(tooltip_label.clone(), None, tooltip_meta.clone(), cx)
    });

    if let Some(target_path) = receipt.target_path.as_ref() {
        stack = stack.child(detail_row(IconName::Folder, "Target", target_path.clone()));
    }
    if let Some(destination) = receipt.restore_destination_root.as_ref() {
        stack = stack.child(detail_row(
            IconName::Download,
            "Restore",
            destination.clone(),
        ));
    }

    stack.into_any_element()
}

pub(super) fn source_row(
    id: SharedString,
    icon: IconName,
    source: &DxForgeSourceRow,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex()
        .id(id)
        .min_w_0()
        .gap_0p5()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|style| style.bg(cx.theme().colors().element_hover))
        .child(
            h_flex()
                .gap_1()
                .min_w_0()
                .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                .child(
                    Label::new(source.label.clone())
                        .size(LabelSize::Small)
                        .truncate(),
                ),
        )
        .child(
            Label::new(source.detail.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(detail_row(IconName::Folder, "Path", source.path.clone()));
    let tooltip_label = source.label.clone();
    let tooltip_meta = format!("{}\n{}", source.detail, source.path);
    stack = stack.tooltip(move |_, cx| {
        Tooltip::with_meta(tooltip_label.clone(), None, tooltip_meta.clone(), cx)
    });

    for receipt in source.receipts.iter().take(2) {
        stack = stack.child(detail_row(
            IconName::FileTextOutlined,
            receipt.label.clone(),
            receipt.detail.clone(),
        ));
    }
    for warning in source.warnings.iter().take(2) {
        stack = stack.child(detail_row(IconName::Warning, "Warning", warning.clone()));
    }

    stack.into_any_element()
}

pub(super) fn metric_row(label: &'static str, value: String) -> AnyElement {
    h_flex()
        .justify_between()
        .gap_2()
        .min_w_0()
        .child(
            Label::new(label)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(Label::new(value).size(LabelSize::XSmall).truncate())
        .into_any_element()
}

pub(super) fn status_row(
    icon: IconName,
    color: Color,
    label: &'static str,
    detail: String,
) -> AnyElement {
    v_flex()
        .gap_0p5()
        .child(
            h_flex()
                .gap_1()
                .min_w_0()
                .child(Icon::new(icon).size(IconSize::XSmall).color(color))
                .child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .color(color)
                        .truncate(),
                ),
        )
        .child(
            Label::new(detail)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn empty_row(label: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .min_w_0()
        .gap_1()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().editor_background)
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

fn detail_row(icon: IconName, label: impl Into<String>, value: String) -> AnyElement {
    let label = label.into();
    h_flex()
        .gap_1()
        .min_w_0()
        .pl(px(18.0))
        .child(
            Icon::new(icon)
                .size(IconSize::Indicator)
                .color(Color::Muted),
        )
        .child(
            Label::new(format!("{label}:"))
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .child(
            Label::new(value)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate_start(),
        )
        .into_any_element()
}
