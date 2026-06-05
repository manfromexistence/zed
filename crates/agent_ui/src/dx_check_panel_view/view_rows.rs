use gpui::{
    AnyElement, App, Div, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, px,
};
use theme::ActiveTheme;
use ui::prelude::*;

use crate::dx_check_panel::{
    DxCheckPanelNotice, DxCheckPanelQuickFix, DxCheckPanelSection, DxCheckPanelSnapshot,
    DxCheckPanelWebAudit,
};

pub(super) fn section(title: &'static str, cx: &App) -> gpui::Div {
    v_flex()
        .w_full()
        .min_w_0()
        .gap_0p5()
        .child(section_header(title, cx))
}

fn section_header(title: &'static str, cx: &App) -> AnyElement {
    h_flex()
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
            Label::new(title)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn detail_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> AnyElement {
    row_shell()
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .flex_none(),
        )
        .child(
            Label::new(value.into())
                .size(LabelSize::XSmall)
                .color(Color::Default)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn section_row(section: &DxCheckPanelSection) -> AnyElement {
    detail_row(section.title.clone(), section_score_label(section))
}

pub(super) fn notice_row(
    id: impl Into<SharedString>,
    icon: IconName,
    color: Color,
    message: &str,
    next_action: Option<&str>,
) -> AnyElement {
    let mut stack = row_stack().id(id.into()).child(
        h_flex()
            .min_w_0()
            .gap_1()
            .items_start()
            .child(Icon::new(icon).size(IconSize::XSmall).color(color))
            .child(
                Label::new(message.to_string())
                    .size(LabelSize::XSmall)
                    .color(color)
                    .truncate(),
            ),
    );

    if let Some(next_action) = next_action {
        stack = stack.child(
            Label::new(next_action.to_string())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        );
    }

    stack.into_any_element()
}

pub(super) fn quick_fix_row(index: usize, fix: &DxCheckPanelQuickFix) -> AnyElement {
    let receipt = if fix.writes_receipts {
        "writes receipts"
    } else {
        "no receipt write"
    };
    let risk = format!(
        "{} risk, {}, {receipt}",
        fix.risk_level,
        if fix.requires_user_approval {
            "approval required"
        } else {
            "no approval required"
        }
    );
    let mut stack = row_stack()
        .id(SharedString::from(format!("dx-check-quick-fix-{index}")))
        .child(
            h_flex()
                .min_w_0()
                .gap_2()
                .justify_between()
                .child(
                    Label::new(fix.label.clone())
                        .size(LabelSize::XSmall)
                        .truncate(),
                )
                .child(
                    Label::new(risk)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(
            Label::new(fix.next_action.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        );

    if let Some(command) = fix.command.as_ref() {
        stack = stack.child(
            Label::new(command.clone())
                .size(LabelSize::XSmall)
                .color(Color::Accent)
                .truncate(),
        );
    }

    stack.into_any_element()
}

pub(super) fn empty_row(message: &'static str) -> AnyElement {
    row_shell()
        .child(
            Icon::new(IconName::Info)
                .size(IconSize::XSmall)
                .color(Color::Muted),
        )
        .child(
            Label::new(message)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn web_audit_row(index: usize, audit: &DxCheckPanelWebAudit) -> AnyElement {
    let (icon, color) = match audit.status.as_str() {
        "ready" => (IconName::Check, Color::Success),
        "blocked" => (IconName::Warning, Color::Error),
        "warning" => (IconName::Warning, Color::Warning),
        _ => (IconName::Info, Color::Muted),
    };
    let source = audit.source.as_deref().unwrap_or(&audit.url);

    row_shell()
        .id(SharedString::from(format!("dx-check-web-audit-{index}")))
        .child(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_0p5()
                .child(
                    h_flex()
                        .min_w_0()
                        .gap_2()
                        .justify_between()
                        .child(
                            Label::new(audit.label.clone())
                                .size(LabelSize::Small)
                                .truncate(),
                        )
                        .child(
                            Label::new(audit.status.clone())
                                .size(LabelSize::XSmall)
                                .color(color)
                                .truncate(),
                        ),
                )
                .child(
                    Label::new(audit.detail.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                )
                .child(
                    Label::new(source.to_string())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate_start(),
                ),
        )
        .into_any_element()
}

pub(super) fn status_color(snapshot: &DxCheckPanelSnapshot) -> Color {
    if !snapshot.receipt_present
        || snapshot.receipt_error.is_some()
        || snapshot.fail_count.unwrap_or(0) > 0
    {
        Color::Error
    } else if snapshot.warn_count.unwrap_or(0) > 0 || !snapshot.warnings.is_empty() {
        Color::Warning
    } else {
        Color::Success
    }
}

pub(super) fn outcome_label(
    pass_count: Option<u32>,
    fail_count: Option<u32>,
    warn_count: Option<u32>,
    skipped_count: Option<u32>,
) -> String {
    format!(
        "{} pass / {} fail / {} warn / {} skipped",
        pass_count.unwrap_or(0),
        fail_count.unwrap_or(0),
        warn_count.unwrap_or(0),
        skipped_count.unwrap_or(0)
    )
}

pub(super) fn duration_label(duration_ms: Option<u64>) -> String {
    match duration_ms {
        Some(duration_ms) if duration_ms >= 1_000 => {
            format!("{:.1}s", duration_ms as f64 / 1_000.0)
        }
        Some(duration_ms) => format!("{duration_ms}ms"),
        None => "unknown".to_string(),
    }
}

pub(super) fn count_label(count: usize, noun: &str) -> String {
    let suffix = if count == 1 { "" } else { "s" };
    format!("{count} {noun}{suffix}")
}

pub(super) fn config_label(status: &str, applies_to_score: bool) -> String {
    let applies = if applies_to_score {
        "applied"
    } else {
        "not applied"
    };
    format!("{status}, {applies}")
}

pub(super) fn notice_title(notice: &DxCheckPanelNotice) -> String {
    if notice.code.is_empty() {
        notice.message.clone()
    } else {
        format!("{}: {}", notice.code, notice.message)
    }
}

fn section_score_label(section: &DxCheckPanelSection) -> String {
    match (section.score, section.max_score) {
        (Some(score), Some(max_score)) => {
            let estimated = if section.estimated { ", estimated" } else { "" };
            format!("{score}/{max_score}, {}{estimated}", section.status)
        }
        _ => section.status.clone(),
    }
}

fn row_shell() -> Div {
    h_flex()
        .w_full()
        .min_w_0()
        .gap_1p5()
        .pl_3()
        .pr_1()
        .py_1()
        .border_1()
        .border_r_2()
        .items_start()
}

fn row_stack() -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .gap_1()
        .pl_3()
        .pr_1()
        .py_1()
        .border_1()
        .border_r_2()
}
