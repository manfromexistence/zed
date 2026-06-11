use gpui::{AnyElement, SharedString, prelude::*};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, ListItem, ListItemSpacing,
    prelude::*,
};

use crate::dx_agent_bridge::{DxAgentAutomation, DxAgentAutomationHistoryEntry};

use super::super::super::screen_chrome::{screen_detail_row, screen_detail_stack};

pub(super) fn automation_schedule_row(
    id: SharedString,
    automation: &DxAgentAutomation,
) -> AnyElement {
    AiSettingItem::new(
        id.clone(),
        automation.name.clone(),
        automation_status(automation),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Automations))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(automation.status.state.clone())
    .details(screen_detail_stack(vec![
        screen_detail_row(
            format!("{id}-schedule").into(),
            IconName::Clock,
            "Schedule",
            schedule_label(automation),
        ),
        screen_detail_row(
            format!("{id}-destination").into(),
            dx_icon(DxUiIcon::Channels),
            "Destination",
            destination_label(automation),
        ),
        screen_detail_row(
            format!("{id}-source").into(),
            IconName::FileTextOutlined,
            "Source",
            automation.source.clone(),
        ),
    ]))
    .into_any_element()
}

pub(super) fn automation_run_row(id: SharedString, automation: &DxAgentAutomation) -> AnyElement {
    AiSettingItem::new(
        id.clone(),
        automation.name.clone(),
        automation_status(automation),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(IconName::TodoProgress)
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(if automation.has_successful_execution_proof() {
        "Proof passed"
    } else if automation.has_failed_execution_proof() {
        "Proof failed"
    } else {
        "Proof pending"
    })
    .details(screen_detail_stack(vec![
        screen_detail_row(
            format!("{id}-last").into(),
            IconName::HistoryRerun,
            "Last run",
            automation.last_run.clone(),
        ),
        screen_detail_row(
            format!("{id}-next").into(),
            IconName::Clock,
            "Next run",
            automation.next_run.clone(),
        ),
        screen_detail_row(
            format!("{id}-receipt").into(),
            dx_icon(DxUiIcon::Receipts),
            "Receipts",
            automation.receipts.len().to_string(),
        ),
    ]))
    .into_any_element()
}

pub(super) fn automation_failure_row(
    id: SharedString,
    automation: &DxAgentAutomation,
) -> AnyElement {
    AiSettingItem::new(
        id.clone(),
        automation.name.clone(),
        AiSettingItemStatus::Error,
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(IconName::Warning)
            .size(IconSize::Small)
            .color(Color::Warning),
    )
    .detail_label("Execution proof failed")
    .details(screen_detail_stack(vec![
        screen_detail_row(
            format!("{id}-state").into(),
            IconName::Warning,
            "State",
            automation.status.state.clone(),
        ),
        screen_detail_row(
            format!("{id}-next").into(),
            IconName::FileTextOutlined,
            "Next action",
            automation.next_action.clone(),
        ),
    ]))
    .into_any_element()
}

pub(super) fn history_row(
    id: SharedString,
    automation: &DxAgentAutomation,
    entry: &DxAgentAutomationHistoryEntry,
) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::HistoryRerun)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(automation.name.clone())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(format!(
                        "{} {} {}",
                        entry.run_id, entry.status, entry.finished_at
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Default)
                    .truncate(),
                ),
        )
        .into_any_element()
}

fn automation_status(automation: &DxAgentAutomation) -> AiSettingItemStatus {
    if automation.has_failed_execution_proof() {
        AiSettingItemStatus::Error
    } else if automation.has_successful_execution_proof() {
        AiSettingItemStatus::Running
    } else if automation.status.enabled && automation.status.runtime_available {
        AiSettingItemStatus::Starting
    } else {
        AiSettingItemStatus::Stopped
    }
}

fn schedule_label(automation: &DxAgentAutomation) -> String {
    let mut parts = vec![automation.schedule.kind.clone()];
    if !automation.schedule.summary.is_empty()
        && automation.schedule.summary != automation.schedule.kind
    {
        parts.push(automation.schedule.summary.clone());
    }
    if !automation.schedule.timezone.is_empty() {
        parts.push(automation.schedule.timezone.clone());
    }
    parts.join(" / ")
}

fn destination_label(automation: &DxAgentAutomation) -> String {
    if automation.destination.target.is_empty() {
        format!(
            "{} {}",
            automation.destination.kind, automation.destination.label
        )
    } else {
        format!(
            "{} {} ({})",
            automation.destination.kind,
            automation.destination.label,
            automation.destination.target
        )
    }
}
