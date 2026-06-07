use gpui::{AnyElement, App, SharedString};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, Color, prelude::*};

use crate::dx_agent_bridge::DxAgentAutomation;

use super::super::actions::dx_agent_action_line;
use super::super::connection_rows::{connection_detail_row, connection_detail_stack};
use super::labels::{
    automation_destination_label, automation_history_label, automation_receipt_label,
    automation_schedule_label,
};

pub(super) fn dx_agent_automation_row(
    id: SharedString,
    automation: &DxAgentAutomation,
    _cx: &App,
) -> AnyElement {
    let state = if automation.status.enabled && automation.status.runtime_available {
        automation.status.state.clone()
    } else if automation.status.enabled {
        "runtime pending".to_string()
    } else {
        "paused".to_string()
    };
    let schedule = automation_schedule_label(automation);
    let destination = automation_destination_label(automation);
    let next_label = if automation.status.runtime_available {
        "next"
    } else {
        "requested next"
    };
    let run_window = format!(
        "last {} / {next_label} {}",
        automation.last_run, automation.next_run
    );
    let receipt_summary = automation_receipt_label(automation);
    let history_summary = automation_history_label(automation);
    let mut details = vec![
        connection_detail_row(
            format!("{}-schedule", id).into(),
            dx_icon(DxUiIcon::Automations),
            "Schedule",
            schedule,
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-destination", id).into(),
            dx_icon(DxUiIcon::Channels),
            "Destination",
            destination,
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-run-window", id).into(),
            IconName::Clock,
            "Run window",
            run_window,
        )
        .into_any_element(),
    ];

    if !automation.prompt.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-prompt", id).into(),
                IconName::TextSnippet,
                "Prompt",
                automation.prompt.clone(),
            )
            .into_any_element(),
        );
    }

    if !automation.status.runtime_available {
        details.push(
            connection_detail_row(
                format!("{}-runtime", id).into(),
                IconName::Warning,
                "Runtime",
                automation.status.unavailable_reason.clone(),
            )
            .into_any_element(),
        );
    }

    if !receipt_summary.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-receipt", id).into(),
                dx_icon(DxUiIcon::Receipts),
                "Receipt",
                receipt_summary,
            )
            .into_any_element(),
        );
    }

    if !history_summary.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-history", id).into(),
                IconName::HistoryRerun,
                "History",
                history_summary,
            )
            .into_any_element(),
        );
    }

    if !automation.next_action.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-next-action", id).into(),
                dx_icon(DxUiIcon::Commands),
                "Next",
                automation.next_action.clone(),
            )
            .into_any_element(),
        );
    }

    if let Some(action_line) = dx_agent_action_line(&automation.actions) {
        details.push(
            connection_detail_row(
                format!("{}-actions", id).into(),
                dx_icon(DxUiIcon::Permissions),
                "Actions",
                action_line,
            )
            .into_any_element(),
        );
    }

    AiSettingItem::new(
        id,
        automation.name.clone(),
        automation_status(automation),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Automations))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(state)
    .details(connection_detail_stack(details))
    .into_any_element()
}

fn automation_status(automation: &DxAgentAutomation) -> AiSettingItemStatus {
    let execution_proven = automation.status.enabled
        && automation.status.runtime_available
        && automation.has_successful_execution_proof();
    let execution_failed = automation.has_failed_execution_proof();

    if execution_failed {
        AiSettingItemStatus::Error
    } else if execution_proven {
        AiSettingItemStatus::Running
    } else if automation.status.enabled {
        AiSettingItemStatus::Starting
    } else {
        AiSettingItemStatus::Stopped
    }
}
