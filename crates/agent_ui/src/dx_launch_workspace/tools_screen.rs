use std::ops::Range;

use gpui::{AnyElement, Context, IntoElement, Window};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, prelude::*};

use crate::AgentPanel;
use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::screen_chrome::{
    screen_detail_row, screen_detail_stack, screen_empty_state, screen_section,
    workspace_page_header, workspace_stat,
};
use super::{DxLaunchWorkspaceStatus, agents};

mod catalog;
mod details;
mod workflow_nodes;

pub(crate) use catalog::{DxPluginsCatalogState, PluginCatalogFilter};

pub(crate) fn render_tools_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    state: &mut DxPluginsCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let (header_stats, body) = if let Some(status) = status {
        let snapshot = &status.agent_bridge;
        (
            vec![
                workspace_stat(
                    "dx-tools-stat-indexed",
                    "Indexed",
                    snapshot.workflow_node_catalog.nodes.len().to_string(),
                    cx,
                ),
                workspace_stat(
                    "dx-tools-stat-configured",
                    "Configured",
                    snapshot
                        .workflow_node_catalog
                        .configured_plugin_count
                        .to_string(),
                    cx,
                ),
                workspace_stat(
                    "dx-tools-stat-trusted",
                    "Trusted tools",
                    snapshot
                        .trusted_tool_bridge
                        .trusted_tool_ids
                        .len()
                        .to_string(),
                    cx,
                ),
                workspace_stat(
                    "dx-tools-stat-blocked",
                    "Blocked",
                    snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
                    cx,
                ),
            ],
            v_flex()
                .gap_3()
                .child(screen_section(
                    "dx-tools-workflow-nodes",
                    "Workflow Nodes",
                    dx_icon(DxUiIcon::Plugins),
                    workflow_node_catalog_summary(snapshot),
                    catalog::render_workflow_node_catalog(snapshot, state, window, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-tools-browser",
                    "Browser",
                    dx_icon(DxUiIcon::Browser),
                    trusted_tool_summary(snapshot, &["browser", "web", "chrome"]),
                    trusted_tool_state(
                        snapshot,
                        "dx-tools-browser-state",
                        dx_icon(DxUiIcon::Browser),
                        "Browser tools",
                        &["browser", "web", "chrome"],
                        "No approved Browser tool receipt is available yet.",
                    ),
                    cx,
                ))
                .child(screen_section(
                    "dx-tools-computer",
                    "Computer",
                    dx_icon(DxUiIcon::Computer),
                    trusted_tool_summary(snapshot, &["computer", "desktop", "screen"]),
                    trusted_tool_state(
                        snapshot,
                        "dx-tools-computer-state",
                        dx_icon(DxUiIcon::Computer),
                        "Computer tools",
                        &["computer", "desktop", "screen"],
                        "No approved Computer tool receipt is available yet.",
                    ),
                    cx,
                ))
                .child(screen_section(
                    "dx-tools-mcp",
                    "MCP",
                    dx_icon(DxUiIcon::Mcp),
                    "Context Servers",
                    mcp_state(snapshot),
                    cx,
                ))
                .child(screen_section(
                    "dx-tools-receipts",
                    "Receipts",
                    IconName::FileTextOutlined,
                    format!(
                        "{} bridge receipts / {} catalog receipts",
                        snapshot.trusted_tool_bridge.receipt_count,
                        workflow_node_receipt_count(snapshot)
                    ),
                    agents::dx_agent_receipt_state(snapshot, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-tools-permissions",
                    "Permissions",
                    dx_icon(DxUiIcon::Permissions),
                    permission_summary(snapshot),
                    permission_state(snapshot),
                    cx,
                ))
                .into_any_element(),
        )
    } else {
        (
            vec![workspace_stat(
                "dx-tools-stat-loading",
                "State",
                "Loading",
                cx,
            )],
            screen_empty_state(
                "dx-tools-loading",
                dx_icon(DxUiIcon::Receipts),
                "Loading trusted tool bridge receipts",
                cx,
            ),
        )
    };

    div()
        .id("dx-tools-screen")
        .size_full()
        .min_w_0()
        .overflow_y_scroll()
        .bg(cx.theme().colors().panel_background)
        .child(
            v_flex()
                .gap_3()
                .p_4()
                .child(workspace_page_header(
                    dx_icon(DxUiIcon::Plugins),
                    "Plugins",
                    "Workflow nodes, browser, computer, MCP, receipts, and permission bridge state.",
                    header_stats,
                    cx,
                ))
                .child(body),
        )
        .into_any_element()
}

pub(crate) fn render_workflow_node_catalog_rows(
    state: &DxPluginsCatalogState,
    snapshot: Option<&DxAgentBridgeSnapshot>,
    range: Range<usize>,
    cx: &mut Context<AgentPanel>,
) -> Vec<AnyElement> {
    catalog::render_workflow_node_rows(state, snapshot, range, cx)
}

fn trusted_tool_state(
    snapshot: &DxAgentBridgeSnapshot,
    id: &'static str,
    icon: IconName,
    label: &'static str,
    search_terms: &[&'static str],
    missing_state: &'static str,
) -> AnyElement {
    let matching_ids = trusted_tool_ids(snapshot, search_terms);
    let status = if !snapshot.trusted_tool_bridge.present {
        AiSettingItemStatus::Stopped
    } else if !matching_ids.is_empty() {
        AiSettingItemStatus::Running
    } else {
        AiSettingItemStatus::Starting
    };

    AiSettingItem::new(id, label, status, AiSettingItemSource::Custom)
        .icon(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .detail_label(match status {
            AiSettingItemStatus::Running => "Approved",
            AiSettingItemStatus::Starting => "Pending approval",
            _ => "Unavailable",
        })
        .details(screen_detail_stack(vec![
            screen_detail_row(
                format!("{id}-bridge").into(),
                IconName::FileTextOutlined,
                "Trusted bridge",
                snapshot.trusted_tool_bridge.status.clone(),
            ),
            screen_detail_row(
                format!("{id}-policy").into(),
                dx_icon(DxUiIcon::Permissions),
                "Policy",
                snapshot.trusted_tool_bridge.trust_policy.clone(),
            ),
            screen_detail_row(
                format!("{id}-ids").into(),
                IconName::ToolWeb,
                "Approved tools",
                if matching_ids.is_empty() {
                    missing_state.to_string()
                } else {
                    approved_tool_count_label(matching_ids.len())
                },
            ),
            screen_detail_row(
                format!("{id}-action").into(),
                IconName::FileTextOutlined,
                "Next action",
                snapshot.trusted_tool_bridge.next_action.clone(),
            ),
        ]))
        .into_any_element()
}

fn approved_tool_count_label(count: usize) -> String {
    match count {
        0 => "No approved tools".to_string(),
        1 => "1 approved tool".to_string(),
        count => format!("{count} approved tools"),
    }
}

fn mcp_state(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    AiSettingItem::new(
        "dx-tools-mcp",
        "MCP bridge",
        if snapshot.trusted_tool_bridge.present {
            AiSettingItemStatus::Starting
        } else {
            AiSettingItemStatus::Stopped
        },
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Mcp))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label("Context Servers")
    .details(screen_detail_stack(vec![
        screen_detail_row(
            "dx-tools-mcp-zed-route".into(),
            IconName::Server,
            "Dx route",
            "MCP Servers opens the Context Servers extension registry.",
        ),
        screen_detail_row(
            "dx-tools-mcp-contract".into(),
            IconName::FileTextOutlined,
            "DX Agents",
            "MCP tool receipts are pending trusted bridge approval.",
        ),
        screen_detail_row(
            "dx-tools-mcp-action".into(),
            IconName::FileTextOutlined,
            "Next action",
            snapshot.trusted_tool_bridge.next_action.clone(),
        ),
    ]))
    .into_any_element()
}

fn permission_state(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let status = if snapshot.trusted_tool_bridge.blocked_tool_count > 0 {
        AiSettingItemStatus::Error
    } else if snapshot.trusted_tool_bridge.present {
        AiSettingItemStatus::Running
    } else {
        AiSettingItemStatus::Stopped
    };

    v_flex()
        .gap_1()
        .child(screen_detail_row(
            "dx-tools-permissions-bridge-contract".into(),
            IconName::FileTextOutlined,
            "Bridge contract",
            snapshot.trusted_tool_bridge.bridge_contract_id.clone(),
        ))
        .child(screen_detail_row(
            "dx-tools-permissions-receipt-count".into(),
            dx_icon(DxUiIcon::Receipts),
            "Receipt count",
            snapshot.trusted_tool_bridge.receipt_count.to_string(),
        ))
        .child(
            AiSettingItem::new(
                "dx-tools-permissions",
                "Tool permissions",
                status,
                AiSettingItemSource::Custom,
            )
            .icon(
                Icon::new(dx_icon(DxUiIcon::Permissions))
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .detail_label(match status {
                AiSettingItemStatus::Error => "Blocked tools",
                AiSettingItemStatus::Running => "Policy loaded",
                _ => "Receipt required",
            })
            .details(screen_detail_stack(vec![
                screen_detail_row(
                    "dx-tools-permissions-policy".into(),
                    dx_icon(DxUiIcon::Permissions),
                    "Policy",
                    snapshot.trusted_tool_bridge.trust_policy.clone(),
                ),
                screen_detail_row(
                    "dx-tools-permissions-approved-plugin".into(),
                    IconName::ToolWeb,
                    "Approved plugin tools",
                    snapshot
                        .trusted_tool_bridge
                        .approved_plugin_tool_count
                        .to_string(),
                ),
                screen_detail_row(
                    "dx-tools-permissions-approved-automation".into(),
                    dx_icon(DxUiIcon::Automations),
                    "Automation tools",
                    snapshot
                        .trusted_tool_bridge
                        .approved_automation_tool_count
                        .to_string(),
                ),
                screen_detail_row(
                    "dx-tools-permissions-blocked".into(),
                    IconName::Warning,
                    "Blocked tools",
                    snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
                ),
                screen_detail_row(
                    "dx-tools-permissions-next".into(),
                    IconName::FileTextOutlined,
                    "Next action",
                    snapshot.trusted_tool_bridge.next_action.clone(),
                ),
            ])),
        )
        .into_any_element()
}

fn trusted_tool_ids(
    snapshot: &DxAgentBridgeSnapshot,
    search_terms: &[&'static str],
) -> Vec<String> {
    snapshot
        .trusted_tool_bridge
        .trusted_tool_ids
        .iter()
        .filter(|tool_id| {
            let normalized = tool_id.to_lowercase();
            search_terms.iter().any(|term| normalized.contains(term))
        })
        .cloned()
        .collect()
}

fn workflow_node_catalog_summary(snapshot: &DxAgentBridgeSnapshot) -> String {
    format!(
        "{} indexed / {} configured",
        snapshot.workflow_node_catalog.nodes.len(),
        snapshot.workflow_node_catalog.configured_plugin_count
    )
}

fn workflow_node_receipt_count(snapshot: &DxAgentBridgeSnapshot) -> usize {
    snapshot
        .workflow_node_catalog
        .nodes
        .iter()
        .map(|node| node.receipts.len())
        .sum()
}

fn trusted_tool_summary(snapshot: &DxAgentBridgeSnapshot, search_terms: &[&'static str]) -> String {
    let approved = trusted_tool_ids(snapshot, search_terms).len();
    if approved == 0 {
        snapshot.trusted_tool_bridge.status.clone()
    } else {
        format!("{approved} approved ids")
    }
}

fn permission_summary(snapshot: &DxAgentBridgeSnapshot) -> String {
    format!(
        "{} approved / {} blocked",
        snapshot.trusted_tool_bridge.approved_plugin_tool_count
            + snapshot.trusted_tool_bridge.approved_automation_tool_count,
        snapshot.trusted_tool_bridge.blocked_tool_count
    )
}
