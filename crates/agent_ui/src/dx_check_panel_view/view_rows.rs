use gpui::{AnyElement, App, ClickEvent, IntoElement, ParentElement, SharedString, Window};
use ui::{Indicator, ListHeader, ListItem, ListItemSpacing, prelude::*};

use crate::dx_check_panel::{
    DxCheckPanelAdapterPlan, DxCheckPanelNotice, DxCheckPanelQuickFix, DxCheckPanelSection,
    DxCheckPanelSnapshot, DxCheckPanelWebAudit,
};

pub(super) fn section(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    is_open: bool,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    _cx: &App,
) -> gpui::Stateful<gpui::Div> {
    v_flex().id(id).w_full().min_w_0().gap_0p5().child(
        ListHeader::new(title)
            .inset(true)
            .toggle(Some(is_open))
            .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
            .on_toggle(on_toggle),
    )
}

pub(super) fn status_label(label: impl Into<SharedString>, color: Color, _cx: &App) -> AnyElement {
    h_flex()
        .h_5()
        .min_w_0()
        .items_center()
        .gap_0p5()
        .child(Indicator::dot().color(color))
        .child(
            Label::new(label)
                .size(LabelSize::XSmall)
                .color(color)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn detail_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> AnyElement {
    let label = label.into();
    ListItem::new(format!("dx-check-detail-{}", stable_id(label.as_ref())))
        .inset(true)
        .spacing(ListItemSpacing::ExtraDense)
        .selectable(false)
        .child(
            h_flex()
                .min_w_0()
                .w_full()
                .gap_2()
                .justify_between()
                .child(
                    Label::new(label)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(value.into())
                        .size(LabelSize::XSmall)
                        .color(Color::Default)
                        .truncate(),
                ),
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
    let id = id.into();
    let mut content = v_flex().min_w_0().gap_0p5().child(
        Label::new(message.to_string())
            .size(LabelSize::XSmall)
            .color(color)
            .truncate(),
    );

    if let Some(next_action) = next_action {
        content = content.child(
            Label::new(next_action.to_string())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        );
    }

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .start_slot(Icon::new(icon).size(IconSize::XSmall).color(color))
        .child(content)
        .into_any_element()
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
    let mut content = v_flex()
        .min_w_0()
        .gap_0p5()
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
        content = content.child(
            Label::new(command.clone())
                .size(LabelSize::XSmall)
                .color(Color::Accent)
                .truncate(),
        );
    }

    ListItem::new(SharedString::from(format!("dx-check-quick-fix-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .start_slot(
            Icon::new(IconName::ListTodo)
                .size(IconSize::XSmall)
                .color(Color::Muted),
        )
        .child(content)
        .into_any_element()
}

pub(super) fn adapter_plan_row(index: usize, plan: &DxCheckPanelAdapterPlan) -> AnyElement {
    let configured_from = if plan.configured_from.is_empty() {
        "no config source".to_string()
    } else {
        plan.configured_from.join(", ")
    };
    let detail = format!(
        "{} parser, configured from {}",
        plan.parser, configured_from
    );

    let mut content = v_flex()
        .min_w_0()
        .gap_0p5()
        .child(
            h_flex()
                .min_w_0()
                .gap_2()
                .justify_between()
                .child(
                    Label::new(plan.label.clone())
                        .size(LabelSize::Small)
                        .truncate(),
                )
                .child(
                    Label::new(plan.target.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(
            Label::new(detail)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(
            Label::new(plan.command.clone())
                .size(LabelSize::XSmall)
                .color(Color::Accent)
                .truncate_start(),
        );

    if let Some(run_command) = plan.run_command.as_ref() {
        content = content.child(
            Label::new(run_command.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate_start(),
        );
    }

    ListItem::new(SharedString::from(format!("dx-check-adapter-plan-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .start_slot(
            Icon::new(IconName::Terminal)
                .size(IconSize::XSmall)
                .color(Color::Muted),
        )
        .child(content)
        .into_any_element()
}

pub(super) fn empty_row(message: &'static str) -> AnyElement {
    ListItem::new(format!("dx-check-empty-{}", stable_id(message)))
        .inset(true)
        .spacing(ListItemSpacing::ExtraDense)
        .selectable(false)
        .start_slot(
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

pub(super) fn web_audit_row(index: usize, audit: &DxCheckPanelWebAudit, cx: &App) -> AnyElement {
    let (icon, color) = match audit.status.as_str() {
        "ready" => (IconName::Check, Color::Success),
        "blocked" => (IconName::Warning, Color::Error),
        "warning" => (IconName::Warning, Color::Warning),
        _ => (IconName::Info, Color::Muted),
    };
    let source = audit.source.as_deref().unwrap_or(&audit.url);

    ListItem::new(SharedString::from(format!("dx-check-web-audit-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
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
                        .child(status_label(audit.status.clone(), color, cx)),
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

fn stable_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
