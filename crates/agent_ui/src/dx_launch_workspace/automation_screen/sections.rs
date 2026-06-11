use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::prelude::*;

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use self::rows::{
    automation_failure_row, automation_run_row, automation_schedule_row, history_row,
};
use super::super::screen_chrome::{screen_detail_row, screen_empty_state};

mod drafts;
mod rows;

pub(super) use drafts::drafts_state;

pub(super) fn schedules_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex()
        .gap_1()
        .child(screen_detail_row(
            "dx-automation-schedules-source".into(),
            dx_icon(DxUiIcon::Automations),
            "Automation source",
            "dx agents automate list --json".to_string(),
        ))
        .child(screen_detail_row(
            "dx-automation-schedules-count".into(),
            dx_icon(DxUiIcon::Receipts),
            "Rows",
            snapshot.automations.len().to_string(),
        ));

    if snapshot.automations.is_empty() {
        return stack
            .child(screen_empty_state(
                "dx-automation-schedules-empty",
                dx_icon(DxUiIcon::Automations),
                "No automation schedule rows from DX Agents",
                cx,
            ))
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
        .child(screen_detail_row(
            "dx-automation-runs-runtime".into(),
            IconName::TodoProgress,
            "Scheduled execution",
            "pending DX Agents runtime".to_string(),
        ))
        .child(screen_detail_row(
            "dx-automation-runs-active-tasks".into(),
            dx_icon(DxUiIcon::Automations),
            "Active tasks",
            snapshot.active_task_count.to_string(),
        ))
        .child(screen_detail_row(
            "dx-automation-runs-execution-proof".into(),
            dx_icon(DxUiIcon::Receipts),
            "Execution proof",
            format!("{proven_count} passed / {failed_count} failed"),
        ));

    if snapshot.automations.is_empty() {
        return stack
            .child(screen_empty_state(
                "dx-automation-runs-empty",
                IconName::TodoProgress,
                "No automation run rows from DX Agents",
                cx,
            ))
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
        .child(screen_detail_row(
            "dx-automation-history-index".into(),
            dx_icon(DxUiIcon::Receipts),
            "Receipt index",
            snapshot.receipt_index.status.clone(),
        ))
        .child(screen_detail_row(
            "dx-automation-history-count".into(),
            IconName::HistoryRerun,
            "Automation receipts",
            history_count.to_string(),
        ));

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
        stack = stack.child(screen_empty_state(
            "dx-automation-history-empty",
            IconName::HistoryRerun,
            "No automation history rows in the current DX Agents receipt list",
            cx,
        ));
    }

    stack.into_any_element()
}

pub(super) fn failures_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1().child(screen_detail_row(
        "dx-automation-failures-blocked-tools".into(),
        dx_icon(DxUiIcon::Permissions),
        "Blocked tool approvals",
        snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
    ));

    if let Some(error) = &snapshot.action_error.error {
        stack = stack.child(screen_detail_row(
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
        stack = stack.child(screen_empty_state(
            "dx-automation-failures-empty",
            IconName::Warning,
            "No failed automation run receipts in the current list",
            cx,
        ));
    }

    stack.into_any_element()
}

pub(super) fn history_summary(snapshot: &DxAgentBridgeSnapshot) -> String {
    let history_count = snapshot
        .automations
        .iter()
        .map(|automation| automation.history.len())
        .sum::<usize>();
    format!("{history_count} receipt rows")
}

pub(super) fn failure_summary(snapshot: &DxAgentBridgeSnapshot) -> String {
    let failed_count = snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_failed_execution_proof())
        .count();
    format!(
        "{failed_count} failed / {} blocked approvals",
        snapshot.trusted_tool_bridge.blocked_tool_count
    )
}
