use std::ops::Range;

use gpui::{AnyElement, Context, IntoElement, SharedString, Window};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, ListItem, ListItemSpacing,
    prelude::*,
};

use crate::AgentPanel;
use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::{DxLaunchWorkspaceStatus, agents, metric_row, muted_card, section_title};

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
    let body = if let Some(status) = status {
        let snapshot = &status.agent_bridge;
        v_flex()
            .gap_2()
            .child(section_title("Workflow Nodes", dx_icon(DxUiIcon::Plugins)))
            .child(catalog::render_workflow_node_catalog(
                snapshot, state, window, cx,
            ))
            .child(section_title("Browser", dx_icon(DxUiIcon::Browser)))
            .child(trusted_tool_state(
                snapshot,
                "dx-tools-browser",
                dx_icon(DxUiIcon::Browser),
                "Browser tools",
                &["browser", "web", "chrome"],
                "No approved Browser tool receipt is available yet.",
            ))
            .child(section_title("Computer", dx_icon(DxUiIcon::Computer)))
            .child(trusted_tool_state(
                snapshot,
                "dx-tools-computer",
                dx_icon(DxUiIcon::Computer),
                "Computer tools",
                &["computer", "desktop", "screen"],
                "No approved Computer tool receipt is available yet.",
            ))
            .child(section_title("MCP", dx_icon(DxUiIcon::Mcp)))
            .child(mcp_state(snapshot))
            .child(section_title("Receipts", IconName::FileTextOutlined))
            .child(agents::dx_agent_receipt_state(snapshot, cx))
            .child(section_title("Permissions", dx_icon(DxUiIcon::Permissions)))
            .child(permission_state(snapshot))
            .into_any_element()
    } else {
        muted_card("Loading trusted tool bridge receipts", cx)
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
                .child(screen_header(
                    dx_icon(DxUiIcon::Plugins),
                    "Plugins",
                    "Workflow nodes, browser, computer, MCP, receipts, and permission bridge state.",
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
        .details(tool_detail_stack(vec![
            tool_detail_row(
                format!("{id}-bridge").into(),
                IconName::FileTextOutlined,
                "Trusted bridge",
                snapshot.trusted_tool_bridge.status.clone(),
            ),
            tool_detail_row(
                format!("{id}-policy").into(),
                dx_icon(DxUiIcon::Permissions),
                "Policy",
                snapshot.trusted_tool_bridge.trust_policy.clone(),
            ),
            tool_detail_row(
                format!("{id}-ids").into(),
                IconName::ToolWeb,
                "Approved ids",
                if matching_ids.is_empty() {
                    missing_state.to_string()
                } else {
                    matching_ids.join(", ")
                },
            ),
            tool_detail_row(
                format!("{id}-action").into(),
                IconName::FileTextOutlined,
                "Next action",
                snapshot.trusted_tool_bridge.next_action.clone(),
            ),
        ]))
        .into_any_element()
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
    .details(tool_detail_stack(vec![
        tool_detail_row(
            "dx-tools-mcp-zed-route".into(),
            IconName::Server,
            "Dx route",
            "MCP Servers opens the Context Servers extension registry.",
        ),
        tool_detail_row(
            "dx-tools-mcp-contract".into(),
            IconName::FileTextOutlined,
            "DX Agents",
            "MCP tool receipts are pending trusted bridge approval.",
        ),
        tool_detail_row(
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
        .child(metric_row(
            "Bridge contract",
            snapshot.trusted_tool_bridge.bridge_contract_id.clone(),
        ))
        .child(metric_row(
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
            .details(tool_detail_stack(vec![
                tool_detail_row(
                    "dx-tools-permissions-policy".into(),
                    dx_icon(DxUiIcon::Permissions),
                    "Policy",
                    snapshot.trusted_tool_bridge.trust_policy.clone(),
                ),
                tool_detail_row(
                    "dx-tools-permissions-approved-plugin".into(),
                    IconName::ToolWeb,
                    "Approved plugin tools",
                    snapshot
                        .trusted_tool_bridge
                        .approved_plugin_tool_count
                        .to_string(),
                ),
                tool_detail_row(
                    "dx-tools-permissions-approved-automation".into(),
                    dx_icon(DxUiIcon::Automations),
                    "Automation tools",
                    snapshot
                        .trusted_tool_bridge
                        .approved_automation_tool_count
                        .to_string(),
                ),
                tool_detail_row(
                    "dx-tools-permissions-blocked".into(),
                    IconName::Warning,
                    "Blocked tools",
                    snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
                ),
                tool_detail_row(
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

fn screen_header(icon: IconName, title: &'static str, detail: &'static str) -> AnyElement {
    h_flex()
        .items_start()
        .gap_2()
        .child(Icon::new(icon).size(IconSize::Medium).color(Color::Muted))
        .child(
            v_flex()
                .gap_0p5()
                .child(
                    Label::new(title)
                        .size(LabelSize::Default)
                        .color(Color::Default),
                )
                .child(
                    Label::new(detail)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
        )
        .into_any_element()
}

fn tool_detail_stack(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().gap_0p5().pl_4().children(rows).into_any_element()
}

fn tool_detail_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    ListItem::new(id)
        .spacing(ListItemSpacing::ExtraDense)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(label.into())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(detail.into())
                        .size(LabelSize::XSmall)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .into_any_element()
}
