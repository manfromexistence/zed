use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::{IconName, prelude::*};

use super::{
    DxLaunchWorkspaceStatus, check, compact_status_row, proof, signal_row, style_panel,
    subagent_summary,
};

pub(super) fn agent_overview_section(
    status: &DxLaunchWorkspaceStatus,
    guided_cards: AnyElement,
    cx: &App,
) -> AnyElement {
    let bridge_state = if status.agent_bridge.enabled {
        status.agent_bridge.status.clone()
    } else {
        "disabled".to_string()
    };

    v_flex()
        .gap_1()
        .child(guided_cards)
        .child(compact_status_row(
            "dx-agent-overview-active-thread",
            dx_icon(DxUiIcon::Agent),
            "Active",
            status.active_status.clone(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-overview-bridge",
            dx_icon(DxUiIcon::Gateway),
            "Bridge",
            bridge_state,
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-overview-active-tasks",
            dx_icon(DxUiIcon::Receipts),
            "Active tasks",
            status.agent_bridge.active_task_count.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-overview-release-gate",
            IconName::Check,
            "Release gate",
            status.agent_bridge.release_gate.status.clone(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-overview-quality",
            IconName::Check,
            "Quality",
            format!("{}/100", status.check_score.score),
            cx,
        ))
        .child(style_panel::dx_style_panel_state(&status.style_panel, cx))
        .child(check::check_score_state(&status.check_score, cx))
        .child(proof::proof_freshness_state(&status.proof_freshness, cx))
        .child(proof::runtime_proof_status_state(
            &status.runtime_proof_status,
            cx,
        ))
        .into_any_element()
}

pub(super) fn agent_environment_section(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let bridge = &status.agent_bridge.trusted_tool_bridge;

    v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-agent-environment-worktrees",
            dx_icon(DxUiIcon::Project),
            "Worktrees",
            status.visible_worktree_count.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-environment-background",
            IconName::HistoryRerun,
            "Background threads",
            status.background_thread_count.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-environment-accounts",
            dx_icon(DxUiIcon::Connections),
            "Accounts",
            format!(
                "{} connected / {} need auth",
                status.agent_bridge.connected_accounts_summary.connected,
                status.agent_bridge.connected_accounts_summary.needs_auth
            ),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-environment-automations",
            dx_icon(DxUiIcon::Automations),
            "Automations",
            status.agent_bridge.automation_count.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-environment-tools",
            IconName::ToolWeb,
            "Trusted tools",
            format!(
                "{} approved / {} blocked",
                bridge.approved_plugin_tool_count + bridge.approved_automation_tool_count,
                bridge.blocked_tool_count
            ),
            cx,
        ))
        .into_any_element()
}

pub(super) fn agent_sources_section(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let summary = status.source_sets.attachment_summary();
    let stack = v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-agent-sources-total",
            dx_icon(DxUiIcon::Source),
            "Total sources",
            status.source_sets.total_sources.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-sources-roots",
            dx_icon(DxUiIcon::Project),
            "Workspace roots",
            summary.workspace_roots.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-sources-attachable",
            IconName::Paperclip,
            "Attachable",
            summary.attachable_sources.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-sources-receipts",
            dx_icon(DxUiIcon::Receipts),
            "Managed receipts",
            summary.managed_receipts.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-sources-media",
            dx_icon(DxUiIcon::Media),
            "Media outputs",
            summary.produced_files.to_string(),
            cx,
        ));

    stack.into_any_element()
}

pub(super) fn agent_subagents_section(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    subagent_summary(status, cx)
}

pub(super) fn agent_approvals_section(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let bridge = &status.agent_bridge.trusted_tool_bridge;
    let mut stack = v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-agent-approvals-bridge",
            dx_icon(DxUiIcon::Permissions),
            "Trusted bridge",
            bridge.status.clone(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-approvals-policy",
            dx_icon(DxUiIcon::Permissions),
            "Policy",
            bridge.trust_policy.clone(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-approvals-approved",
            IconName::ToolWeb,
            "Approved tools",
            format!(
                "{} plugin / {} automation",
                bridge.approved_plugin_tool_count, bridge.approved_automation_tool_count
            ),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-approvals-blocked",
            IconName::Warning,
            "Blocked tools",
            bridge.blocked_tool_count.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-agent-approvals-gate",
            IconName::Check,
            "Gate review",
            status.agent_bridge.release_gate.status.clone(),
            cx,
        ));

    if !bridge.present {
        stack = stack.child(signal_row(
            SharedString::from("dx-agent-approvals-missing-bridge"),
            IconName::Warning,
            Color::Warning,
            bridge.next_action.clone(),
        ));
    }

    if bridge.blocked_tool_count > 0 {
        stack = stack.child(signal_row(
            SharedString::from("dx-agent-approvals-blocked-tools"),
            IconName::Warning,
            Color::Error,
            "Blocked trusted tool approval receipts need review.",
        ));
    }

    if status.agent_bridge.action_error.redaction_requires_review {
        stack = stack.child(signal_row(
            SharedString::from("dx-agent-approvals-redaction"),
            IconName::Warning,
            Color::Warning,
            status.agent_bridge.action_error.redaction_summary.clone(),
        ));
    }

    if let Some(error) = &status.agent_bridge.action_error.error {
        stack = stack.child(signal_row(
            SharedString::from("dx-agent-approvals-action-error"),
            IconName::Close,
            Color::Error,
            error.clone(),
        ));
    }

    stack.into_any_element()
}
