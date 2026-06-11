use gpui::{AnyElement, SharedString};
use ui::{AiSettingItemStatus, IconName, prelude::*};

use crate::dx_agent_bridge::{DxAgentAutomation, DxAgentBridgeSnapshot};

use super::super::super::screen_chrome::{screen_detail_row, screen_detail_stack};

pub(super) fn composer_contract_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let composer = &snapshot.automation_composer;
    screen_detail_stack(vec![
        detail(
            "composer-schema",
            IconName::FileTextOutlined,
            "Schema",
            composer.schema_version.clone(),
        ),
        detail(
            "composer-receipt",
            dx_icon(DxUiIcon::Receipts),
            "Receipt present",
            yes_no(composer.receipt_present),
        ),
        detail(
            "composer-runtime",
            dx_icon(DxUiIcon::Gateway),
            "Runtime",
            yes_no(composer.runtime_available),
        ),
        detail(
            "composer-save",
            dx_icon(DxUiIcon::Automations),
            "Save draft",
            yes_no(composer.save_draft_available),
        ),
        detail(
            "composer-enable",
            dx_icon(DxUiIcon::Automations),
            "Enable",
            yes_no(composer.enable_available),
        ),
        detail(
            "composer-fields",
            IconName::FileTextOutlined,
            "Fields",
            composer.field_summary(5),
        ),
        detail(
            "composer-backed",
            dx_icon(DxUiIcon::Receipts),
            "Receipt backed",
            yes_no(composer.fields_receipt_backed),
        ),
        detail(
            "composer-command",
            IconName::Terminal,
            "Command",
            composer.command.clone(),
        ),
        detail(
            "composer-action",
            IconName::ArrowRight,
            "Next action",
            composer.next_action.clone(),
        ),
    ])
}

pub(super) fn runtime_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    screen_detail_stack(vec![
        detail(
            "runtime-count",
            dx_icon(DxUiIcon::Automations),
            "Automation rows",
            snapshot.automations.len().to_string(),
        ),
        detail(
            "runtime-active",
            dx_icon(DxUiIcon::Gateway),
            "Active tasks",
            snapshot.active_task_count.to_string(),
        ),
        detail(
            "runtime-ready",
            IconName::Check,
            "Runtime-ready rows",
            snapshot
                .automations
                .iter()
                .filter(|automation| automation.status.runtime_available)
                .count()
                .to_string(),
        ),
        detail(
            "runtime-proof",
            dx_icon(DxUiIcon::Receipts),
            "Execution proof",
            execution_proof_summary(snapshot),
        ),
        detail(
            "runtime-blocked",
            IconName::Warning,
            "Blocked tools",
            snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
        ),
        detail(
            "runtime-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.automation_composer.next_action.clone(),
        ),
    ])
}

pub(super) fn failure_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    screen_detail_stack(vec![
        detail(
            "failure-proof",
            IconName::Warning,
            "Execution proof",
            execution_proof_summary(snapshot),
        ),
        detail(
            "failure-action-error",
            IconName::Warning,
            "Action error",
            snapshot.action_error.status.clone(),
        ),
        detail(
            "failure-last-error",
            IconName::Warning,
            "Bridge error",
            snapshot
                .last_error
                .clone()
                .unwrap_or_else(|| "None".to_string()),
        ),
        detail(
            "failure-next-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.action_error.next_action.clone(),
        ),
    ])
}

pub(super) fn automation_details(automation: &DxAgentAutomation) -> AnyElement {
    screen_detail_stack(vec![
        automation_detail(
            automation,
            "prompt",
            IconName::FileTextOutlined,
            "Prompt",
            automation.prompt.clone(),
        ),
        automation_detail(
            automation,
            "schedule",
            dx_icon(DxUiIcon::Automations),
            "Schedule",
            automation.schedule.summary.clone(),
        ),
        automation_detail(
            automation,
            "timezone",
            IconName::Clock,
            "Timezone",
            automation.schedule.timezone.clone(),
        ),
        automation_detail(
            automation,
            "state",
            dx_icon(DxUiIcon::Gateway),
            "State",
            automation.status.state.clone(),
        ),
        automation_detail(
            automation,
            "enabled",
            IconName::Check,
            "Enabled",
            yes_no(automation.status.enabled),
        ),
        automation_detail(
            automation,
            "runtime",
            dx_icon(DxUiIcon::Gateway),
            "Runtime",
            yes_no(automation.status.runtime_available),
        ),
        automation_detail(
            automation,
            "destination",
            dx_icon(DxUiIcon::Channels),
            "Destination",
            automation.destination.label.clone(),
        ),
        automation_detail(
            automation,
            "last-run",
            IconName::Clock,
            "Last run",
            automation.last_run.clone(),
        ),
        automation_detail(
            automation,
            "next-run",
            IconName::Clock,
            "Requested next",
            automation.next_run.clone(),
        ),
        automation_detail(
            automation,
            "proof",
            dx_icon(DxUiIcon::Receipts),
            "Execution proof",
            automation_proof_label(automation),
        ),
        automation_detail(
            automation,
            "receipts",
            dx_icon(DxUiIcon::Receipts),
            "Receipts",
            automation.receipts.len().to_string(),
        ),
        automation_detail(
            automation,
            "history",
            IconName::FileTextOutlined,
            "History",
            automation.history.len().to_string(),
        ),
        automation_detail(
            automation,
            "action",
            IconName::ArrowRight,
            "Next action",
            automation.next_action.clone(),
        ),
    ])
}

pub(super) fn automation_status(automation: &DxAgentAutomation) -> AiSettingItemStatus {
    if automation.has_failed_execution_proof() {
        AiSettingItemStatus::Error
    } else if automation.status.enabled && automation.status.runtime_available {
        AiSettingItemStatus::Running
    } else if !automation.status.runtime_available {
        AiSettingItemStatus::AuthRequired
    } else {
        AiSettingItemStatus::Stopped
    }
}

fn execution_proof_summary(snapshot: &DxAgentBridgeSnapshot) -> String {
    let passed = snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_successful_execution_proof())
        .count();
    let failed = snapshot
        .automations
        .iter()
        .filter(|automation| automation.has_failed_execution_proof())
        .count();
    format!("{passed} passed / {failed} failed")
}

fn automation_proof_label(automation: &DxAgentAutomation) -> &'static str {
    if automation.has_failed_execution_proof() {
        "Execution proof failed"
    } else if automation.has_successful_execution_proof() {
        "Execution proof passed"
    } else {
        "Execution proof pending"
    }
}

fn detail(
    suffix: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
) -> AnyElement {
    screen_detail_row(
        format!("dx-automation-detail-{suffix}").into(),
        icon,
        label,
        value,
    )
}

fn automation_detail(
    automation: &DxAgentAutomation,
    suffix: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
) -> AnyElement {
    screen_detail_row(
        format!("dx-automation-detail-{}-{suffix}", automation.id).into(),
        icon,
        label,
        value,
    )
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
