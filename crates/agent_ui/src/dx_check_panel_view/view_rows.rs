use gpui::{AnyElement, App, ClickEvent, IntoElement, ParentElement, SharedString, Window};
use ui::{Indicator, ListHeader, ListItem, ListItemSpacing, Tooltip, prelude::*};

use crate::dx_check_panel::{
    DxCheckPanelAdapterPlan, DxCheckPanelNotice, DxCheckPanelQuickFix, DxCheckPanelSection,
    DxCheckPanelSnapshot, DxCheckPanelWebAudit,
};

pub(super) fn section(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    count_label: Option<SharedString>,
    is_open: bool,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    _cx: &App,
) -> gpui::Stateful<gpui::Div> {
    v_flex().id(id).w_full().min_w_0().gap_0p5().child(
        ListHeader::new(title)
            .inset(true)
            .toggle(Some(is_open))
            .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
            .when_some(count_label, |this, count_label| {
                this.end_slot(
                    Label::new(count_label)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                )
            })
            .on_toggle(on_toggle),
    )
}

pub(super) fn detail_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> AnyElement {
    let label = label.into();
    let value = value.into();
    let tooltip = format!("{}: {}", label.as_ref(), value.as_ref());

    ListItem::new(format!("dx-check-detail-{}", stable_id(label.as_ref())))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .end_slot(
            Label::new(value)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn section_row(index: usize, section: &DxCheckPanelSection) -> AnyElement {
    let score = section_score_label(section);
    let tooltip = format!("{}: {score}", section.title);

    ListItem::new(format!(
        "dx-check-section-score-{index}-{}",
        stable_id(&section.title)
    ))
    .inset(true)
    .spacing(ListItemSpacing::Sparse)
    .selectable(false)
    .start_slot(Indicator::dot().color(section_status_color(&section.status)))
    .child(
        Label::new(section.title.clone())
            .size(LabelSize::Small)
            .color(Color::Default)
            .truncate(),
    )
    .end_slot(
        Label::new(score)
            .size(LabelSize::Small)
            .color(Color::Muted)
            .truncate(),
    )
    .tooltip(Tooltip::text(tooltip))
    .into_any_element()
}

pub(super) fn notice_row(
    id: impl Into<SharedString>,
    icon: IconName,
    color: Color,
    message: &str,
    next_action: Option<&str>,
) -> AnyElement {
    let id = id.into();
    let tooltip = next_action
        .map(|next_action| format!("{message}\n{next_action}"))
        .unwrap_or_else(|| message.to_string());
    let next_action = next_action.map(String::from);

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(message.to_string())
                .size(LabelSize::Small)
                .truncate(),
        )
        .when_some(next_action, |this, next_action| {
            this.end_slot(
                Label::new(next_action)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .tooltip(Tooltip::text(tooltip))
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
    let tooltip = format!(
        "{}\n{}\n{}",
        risk,
        fix.next_action,
        fix.command.as_deref().unwrap_or("No command")
    );
    let command = fix.command.clone();

    ListItem::new(SharedString::from(format!("dx-check-quick-fix-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::ListTodo)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(fix.label.clone())
                .size(LabelSize::Small)
                .truncate(),
        )
        .end_slot(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    div().max_w(rems(16.)).overflow_hidden().child(
                        Label::new(fix.next_action.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate_start(),
                    ),
                )
                .when_some(command, |this, command| {
                    this.child(
                        div().max_w(rems(16.)).overflow_hidden().child(
                            Label::new(command)
                                .size(LabelSize::Small)
                                .color(Color::Accent)
                                .truncate_start(),
                        ),
                    )
                }),
        )
        .tooltip(Tooltip::text(tooltip))
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

    let tooltip = format!(
        "{detail}\n{}\n{}",
        plan.command,
        plan.run_command
            .as_deref()
            .unwrap_or("No run command configured")
    );
    let run_command = plan.run_command.clone();

    ListItem::new(SharedString::from(format!("dx-check-adapter-plan-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Terminal)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(plan.label.clone())
                .size(LabelSize::Small)
                .truncate(),
        )
        .end_slot(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    div().max_w(rems(12.)).overflow_hidden().child(
                        Label::new(plan.target.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
                )
                .child(
                    div().max_w(rems(18.)).overflow_hidden().child(
                        Label::new(plan.command.clone())
                            .size(LabelSize::Small)
                            .color(Color::Accent)
                            .truncate_start(),
                    ),
                )
                .when_some(run_command, |this, run_command| {
                    this.child(
                        div().max_w(rems(16.)).overflow_hidden().child(
                            Label::new(run_command)
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate_start(),
                        ),
                    )
                }),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn empty_row(message: &'static str) -> AnyElement {
    ListItem::new(format!("dx-check-empty-{}", stable_id(message)))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Info)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(message)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .tooltip(Tooltip::text(message))
        .into_any_element()
}

pub(super) fn web_audit_row(index: usize, audit: &DxCheckPanelWebAudit, _cx: &App) -> AnyElement {
    let normalized_status = audit.status.to_ascii_lowercase();
    let (icon, color) = match normalized_status.as_str() {
        "ready" => (IconName::Check, Color::Success),
        "blocked" => (IconName::Warning, Color::Error),
        "warning" => (IconName::Warning, Color::Warning),
        _ => (IconName::Info, Color::Muted),
    };
    let source = audit.source.as_deref().unwrap_or(&audit.url);

    let tooltip = format!("{}\n{}\n{}", audit.status, audit.detail, source);

    ListItem::new(SharedString::from(format!("dx-check-web-audit-{index}")))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(audit.label.clone())
                .size(LabelSize::Small)
                .truncate(),
        )
        .end_slot(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(audit.status.clone())
                        .size(LabelSize::Small)
                        .color(color)
                        .truncate(),
                )
                .child(
                    div().max_w(rems(18.)).overflow_hidden().child(
                        Label::new(source.to_string())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate_start(),
                    ),
                ),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn overflow_row(
    id: impl Into<SharedString>,
    hidden_count: usize,
    label: &'static str,
) -> AnyElement {
    let message = format!("{hidden_count} more {label} not shown");
    let tooltip = format!(
        "{message}. This panel shows a capped preview; open the Receipt tab or run the detail command for the full list."
    );

    ListItem::new(id.into())
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Ellipsis)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(message.clone())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn status_color(snapshot: &DxCheckPanelSnapshot) -> Color {
    let status = snapshot.status.to_ascii_lowercase();

    if !snapshot.receipt_present
        || snapshot.receipt_error.is_some()
        || snapshot.fail_count.unwrap_or(0) > 0
        || check_status_is_failure(&status)
    {
        Color::Error
    } else if snapshot.warn_count.unwrap_or(0) > 0
        || !snapshot.warnings.is_empty()
        || check_status_is_warning(&status)
    {
        Color::Warning
    } else if check_status_is_success(&status) && check_snapshot_has_result_signal(snapshot) {
        Color::Success
    } else {
        Color::Muted
    }
}

pub(super) fn outcome_label(
    pass_count: Option<u32>,
    fail_count: Option<u32>,
    warn_count: Option<u32>,
    skipped_count: Option<u32>,
) -> String {
    if [pass_count, fail_count, warn_count, skipped_count]
        .iter()
        .all(Option::is_none)
    {
        return "Counts unavailable".to_string();
    }

    format!(
        "{} pass / {} fail / {} warn / {} skipped",
        check_count_label(pass_count),
        check_count_label(fail_count),
        check_count_label(warn_count),
        check_count_label(skipped_count)
    )
}

fn check_count_label(count: Option<u32>) -> String {
    count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "--".to_string())
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

fn section_status_color(status: &str) -> Color {
    let status = status.to_ascii_lowercase();

    match status.as_str() {
        "pass" | "passed" | "ready" | "ok" => Color::Success,
        "fail" | "failed" | "blocked" | "error" => Color::Error,
        "warn" | "warning" | "review" => Color::Warning,
        _ => Color::Muted,
    }
}

fn check_status_is_success(status: &str) -> bool {
    matches!(status, "pass" | "passed" | "ready" | "ok")
}

fn check_status_is_failure(status: &str) -> bool {
    matches!(status, "fail" | "failed" | "blocked" | "error")
}

fn check_status_is_warning(status: &str) -> bool {
    matches!(status, "warn" | "warning" | "review")
}

fn check_snapshot_has_result_signal(snapshot: &DxCheckPanelSnapshot) -> bool {
    snapshot.score_value.is_some()
        || snapshot.score_percent.is_some()
        || snapshot.pass_count.is_some()
        || snapshot.fail_count.is_some()
        || snapshot.warn_count.is_some()
        || snapshot.skipped_count.is_some()
        || !snapshot.sections.is_empty()
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
