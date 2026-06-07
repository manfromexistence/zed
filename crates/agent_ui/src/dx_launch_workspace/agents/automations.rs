use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::prelude::*;

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use self::{composer::dx_agent_automation_composer_contract, rows::dx_agent_automation_row};
use super::super::{metric_row, muted_card};

mod composer;
mod labels;
mod rows;

pub(in super::super) fn dx_agent_automation_state(
    snapshot: &DxAgentBridgeSnapshot,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex()
        .gap_1()
        .child(metric_row("Count", snapshot.automation_count.to_string()))
        .child(metric_row("Active", snapshot.active_task_count.to_string()))
        .child(metric_row(
            "Command",
            "dx agents automate list --json".to_string(),
        ))
        .child(metric_row(
            "Trusted bridge",
            snapshot.trusted_tool_bridge.status.clone(),
        ))
        .child(metric_row(
            "Approved tools",
            format!(
                "{} plugin / {} automation",
                snapshot.trusted_tool_bridge.approved_plugin_tool_count,
                snapshot.trusted_tool_bridge.approved_automation_tool_count
            ),
        ))
        .child(metric_row(
            "Blocked tools",
            snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
        ))
        .child(dx_agent_automation_composer_contract(
            &snapshot.automation_composer,
            cx,
        ));

    if snapshot.automations.is_empty() {
        stack = stack.child(muted_card("Run automation list receipt", cx));
    } else {
        for (ix, automation) in snapshot.automations.iter().take(3).enumerate() {
            stack = stack.child(dx_agent_automation_row(
                SharedString::from(format!("dx-agent-automation-{ix}")),
                automation,
                cx,
            ));
        }
    }

    stack.into_any_element()
}
