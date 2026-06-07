use crate::dx_agent_bridge::DxAgentAutomation;

pub(super) fn automation_schedule_label(automation: &DxAgentAutomation) -> String {
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

pub(super) fn automation_destination_label(automation: &DxAgentAutomation) -> String {
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

pub(super) fn automation_receipt_label(automation: &DxAgentAutomation) -> String {
    if automation.receipts.is_empty() {
        return "Execution proof pending: no automation run receipt yet".to_string();
    }
    let latest = &automation.receipts[0];
    format!(
        "{} receipt: {} {}",
        automation.receipts.len(),
        latest.kind,
        latest.status
    )
}

pub(super) fn automation_history_label(automation: &DxAgentAutomation) -> String {
    if automation.history.is_empty() {
        return "Execution history pending".to_string();
    }
    let latest = &automation.history[0];
    format!(
        "{} history: {} {}",
        automation.history.len(),
        latest.run_id,
        latest.status
    )
}
