use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, prelude::*};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use self::rows::{
    automation_failure_row, automation_run_row, automation_schedule_row, detail_row, detail_stack,
    history_row,
};
use super::super::{metric_row, muted_card};

mod rows;

pub(super) fn drafts_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let composer = &snapshot.automation_composer;
    let status = if composer.save_draft_available {
        AiSettingItemStatus::Running
    } else if composer.runtime_available {
        AiSettingItemStatus::Starting
    } else {
        AiSettingItemStatus::Stopped
    };
    let field_summary = composer
        .fields
        .iter()
        .take(5)
        .map(|field| {
            if field.required {
                format!("{}*", field.label)
            } else {
                field.label.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    AiSettingItem::new(
        "dx-automation-drafts-composer",
        "Draft composer",
        status,
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Automations))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(match status {
        AiSettingItemStatus::Running => "Ready",
        AiSettingItemStatus::Starting => "Runtime ready",
        _ => "Unavailable",
    })
    .details(detail_stack(vec![
        detail_row(
            "dx-automation-drafts-status".into(),
            IconName::Server,
            "Status",
            composer.status.clone(),
        ),
        detail_row(
            "dx-automation-drafts-fields".into(),
            IconName::TextSnippet,
            "Fields",
            if field_summary.is_empty() {
                "No composer fields in receipt".to_string()
            } else {
                field_summary
            },
        ),
        detail_row(
            "dx-automation-drafts-receipt".into(),
            dx_icon(DxUiIcon::Receipts),
            "Receipt",
            composer.receipt_filename.clone(),
        ),
        detail_row(
            "dx-automation-drafts-action".into(),
            IconName::FileTextOutlined,
            "Next action",
            composer.next_action.clone(),
        ),
    ]))
    .into_any_element()
}

pub(super) fn schedules_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex()
        .gap_1()
        .child(metric_row(
            "Automation source",
            "dx agents automate list --json".to_string(),
        ))
        .child(metric_row("Rows", snapshot.automations.len().to_string()));

    if snapshot.automations.is_empty() {
        return stack
            .child(muted_card("No automation schedule rows from DX Agents", cx))
            .into_any_element();
    }

    for (ix, automation) in snapshot.automations.iter().take(6).enumerate() {
        stack = stack.child(automation_schedule_row(
            SharedString::from(format!("dx-automation-schedule-{ix}")),
            automation,
        ));
    }

    stack.into_any_element()
}

pub(super) fn runs_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let proven_count = snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_successful_execution_proof())
        .count();
    let failed_count = snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_failed_execution_proof())
        .count();
    let mut stack = v_flex()
        .gap_1()
        .child(metric_row(
            "Scheduled execution",
            "pending DX Agents runtime".to_string(),
        ))
        .child(metric_row(
            "Active tasks",
            snapshot.active_task_count.to_string(),
        ))
        .child(metric_row(
            "Execution proof",
            format!("{proven_count} passed / {failed_count} failed"),
        ));

    if snapshot.automations.is_empty() {
        return stack
            .child(muted_card("No automation run rows from DX Agents", cx))
            .into_any_element();
    }

    for (ix, automation) in snapshot.automations.iter().take(6).enumerate() {
        stack = stack.child(automation_run_row(
            SharedString::from(format!("dx-automation-run-{ix}")),
            automation,
        ));
    }

    stack.into_any_element()
}

pub(super) fn history_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let history_count = snapshot
        .automations
        .iter()
        .map(|automation| automation.history.len())
        .sum::<usize>();
    let mut stack = v_flex()
        .gap_1()
        .child(metric_row(
            "Receipt index",
            snapshot.receipt_index.status.clone(),
        ))
        .child(metric_row("Automation receipts", history_count.to_string()));

    let mut rendered = 0;
    for automation in snapshot.automations.iter().take(6) {
        for entry in automation.history.iter().take(3) {
            rendered += 1;
            stack = stack.child(history_row(
                SharedString::from(format!("dx-automation-history-{rendered}")),
                automation,
                entry,
            ));
        }
    }

    if rendered == 0 {
        stack = stack.child(muted_card(
            "No automation history rows in the current DX Agents receipt list",
            cx,
        ));
    }

    stack.into_any_element()
}

pub(super) fn failures_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1().child(metric_row(
        "Blocked tool approvals",
        snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
    ));

    if let Some(error) = &snapshot.action_error.error {
        stack = stack.child(detail_row(
            "dx-automation-failures-action-error".into(),
            IconName::Warning,
            "Action error",
            error.clone(),
        ));
    }

    let mut rendered = 0;
    for automation in snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_failed_execution_proof())
        .take(6)
    {
        rendered += 1;
        stack = stack.child(automation_failure_row(
            SharedString::from(format!("dx-automation-failure-{rendered}")),
            automation,
        ));
    }

    if rendered == 0 && snapshot.action_error.error.is_none() {
        stack = stack.child(muted_card(
            "No failed automation run receipts in the current list",
            cx,
        ));
    }

    stack.into_any_element()
}
