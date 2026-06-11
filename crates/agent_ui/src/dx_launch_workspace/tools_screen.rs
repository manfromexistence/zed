use gpui::{AnyElement, App, IntoElement, SharedString};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, ListHeader, ListItem,
    ListItemSpacing, Tooltip, prelude::*,
};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::{DxLaunchWorkspaceStatus, agents, muted_card};

mod catalog;
use catalog::{PluginCatalogEntry, first_party_plugin_catalog};

pub(crate) fn render_tools_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    cx: &mut App,
) -> AnyElement {
    let body = if let Some(status) = status {
        let snapshot = &status.agent_bridge;
        v_flex()
            .gap_3()
            .child(plugin_catalog_summary(snapshot))
            .child(plugin_catalog_cards(snapshot, cx))
            .child(tools_section(
                "dx-plugins-bridge-section",
                "Bridge Status",
                dx_icon(DxUiIcon::Permissions),
            ))
            .child(permission_state(snapshot))
            .child(tools_section(
                "dx-plugins-mcp-section",
                "MCP",
                dx_icon(DxUiIcon::Mcp),
            ))
            .child(mcp_state(snapshot))
            .child(tools_section(
                "dx-plugins-receipts-section",
                "Receipts",
                IconName::FileTextOutlined,
            ))
            .child(agents::dx_agent_receipt_state(snapshot, cx))
            .into_any_element()
    } else {
        muted_card("Loading DX plugin catalog and bridge receipts", cx)
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
                    "First-party DX plugins, credentials, receipts, and agent bridge readiness.",
                ))
                .child(body),
        )
        .into_any_element()
}

fn plugin_catalog_summary(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let entries = first_party_plugin_catalog();

    ListItem::new("dx-plugins-catalog-summary")
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(dx_icon(DxUiIcon::Plugins))
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            h_flex()
                .min_w_0()
                .gap_2()
                .child(
                    v_flex()
                        .min_w_0()
                        .gap_0p5()
                        .child(
                            Label::new("DX first-party plugin catalog")
                                .size(LabelSize::Small)
                                .color(Color::Default),
                        )
                        .child(
                            Label::new("Browser, Computer, and Driven are loaded from DX-owned manifest entries.")
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                )
                .child(
                    Label::new(format!("{} plugins", entries.len()))
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .flex_none(),
                ),
        )
        .end_slot(
            Label::new(if snapshot.trusted_tool_bridge.present {
                "Bridge receipts loaded"
            } else {
                "Bridge receipt pending"
            })
            .size(LabelSize::Small)
            .color(if snapshot.trusted_tool_bridge.present {
                Color::Success
            } else {
                Color::Muted
            }),
        )
        .into_any_element()
}

fn plugin_catalog_cards(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    v_flex()
        .gap_2()
        .children(
            first_party_plugin_catalog()
                .iter()
                .map(|entry| plugin_catalog_card(entry, snapshot, cx)),
        )
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

fn plugin_catalog_card(
    entry: &PluginCatalogEntry,
    snapshot: &DxAgentBridgeSnapshot,
    cx: &App,
) -> AnyElement {
    let approved_tool_ids = trusted_tool_ids(snapshot, plugin_trust_terms(entry));
    let bridge_present = snapshot.trusted_tool_bridge.present;
    let status = if !approved_tool_ids.is_empty() {
        ("Enabled for Agents", Color::Success)
    } else if bridge_present {
        ("Approval pending", Color::Muted)
    } else {
        ("Bridge receipt pending", Color::Muted)
    };
    let credential_label = if entry.credentials.is_empty() {
        "No credentials required".to_string()
    } else {
        format!("Credentials: {}", entry.credentials.join(", "))
    };
    let approved_label = if approved_tool_ids.is_empty() {
        "No approved tool ids yet".to_string()
    } else {
        approved_tool_ids.join(", ")
    };

    div()
        .id(SharedString::from(format!("dx-plugin-card-{}", entry.id)))
        .w_full()
        .child(
            v_flex()
                .w_full()
                .p_3()
                .gap_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border_variant)
                .bg(cx.theme().colors().elevated_surface_background.opacity(0.5))
                .child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .child(
                            Icon::new(entry.icon)
                                .size(IconSize::Medium)
                                .color(Color::Default),
                        )
                        .child(
                            v_flex()
                                .min_w_0()
                                .flex_1()
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .min_w_0()
                                        .gap_2()
                                        .child(
                                            Label::new(entry.name)
                                                .size(LabelSize::Default)
                                                .color(Color::Default)
                                                .truncate(),
                                        )
                                        .child(
                                            Label::new(entry.id)
                                                .size(LabelSize::Small)
                                                .color(Color::Muted)
                                                .truncate(),
                                        ),
                                )
                                .child(
                                    Label::new(entry.description)
                                        .size(LabelSize::Small)
                                        .color(Color::Muted)
                                        .truncate(),
                                ),
                        )
                        .child(
                            Label::new(status.0)
                                .size(LabelSize::Small)
                                .color(status.1)
                                .flex_none(),
                        ),
                )
                .child(
                    h_flex()
                        .min_w_0()
                        .gap_1()
                        .flex_wrap()
                        .child(plugin_meta_pill(entry.category, cx))
                        .child(plugin_meta_pill(entry.runtime, cx))
                        .child(plugin_meta_pill(entry.engine, cx))
                        .child(plugin_meta_pill(entry.trust, cx)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(plugin_card_row(
                            format!("{}-permissions", entry.id).into(),
                            dx_icon(DxUiIcon::Permissions),
                            "Permissions",
                            entry.permissions.join(", "),
                        ))
                        .child(plugin_card_row(
                            format!("{}-io", entry.id).into(),
                            IconName::ArrowRightLeft,
                            "Inputs / outputs",
                            format!(
                                "{} -> {}",
                                entry.inputs.join(", "),
                                entry.outputs.join(", ")
                            ),
                        ))
                        .child(plugin_card_row(
                            format!("{}-credentials", entry.id).into(),
                            dx_icon(DxUiIcon::Credentials),
                            "Credentials",
                            credential_label,
                        ))
                        .child(plugin_card_row(
                            format!("{}-receipts", entry.id).into(),
                            dx_icon(DxUiIcon::Receipts),
                            "Receipts",
                            entry.receipts.join(", "),
                        ))
                        .child(plugin_card_row(
                            format!("{}-approved", entry.id).into(),
                            IconName::Check,
                            "Approved tools",
                            approved_label,
                        ))
                        .child(plugin_card_row(
                            format!("{}-source", entry.id).into(),
                            dx_icon(DxUiIcon::Source),
                            "Source",
                            entry.source_root,
                        )),
                ),
        )
        .into_any_element()
}

fn plugin_meta_pill(label: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .child(
            Label::new(label.into())
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .into_any_element()
}

fn plugin_card_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    tool_detail_row(id, icon, label, detail)
}

fn plugin_trust_terms(entry: &PluginCatalogEntry) -> &'static [&'static str] {
    match entry.id {
        "dx.browser" => &["dx.browser", "browser", "web_preview"],
        "dx.computer" => &["dx.computer", "computer", "managed_browser", "chrome"],
        "dx.driven" => &["dx.driven", "driven", "workflow", "source_guard"],
        _ => &[],
    }
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
        .child(tool_detail_row(
            "dx-tools-permissions-contract".into(),
            IconName::FileTextOutlined,
            "Bridge contract",
            snapshot.trusted_tool_bridge.bridge_contract_id.clone(),
        ))
        .child(tool_detail_row(
            "dx-tools-permissions-receipts".into(),
            IconName::FileTextOutlined,
            "Receipt count",
            snapshot.trusted_tool_bridge.receipt_count.to_string(),
        ))
        .child(tool_detail_row(
            "dx-tools-permissions-approved-plugins".into(),
            dx_icon(DxUiIcon::Plugins),
            "Approved plugins",
            snapshot
                .trusted_tool_bridge
                .approved_plugin_tool_count
                .to_string(),
        ))
        .child(tool_detail_row(
            "dx-tools-permissions-approved-automations".into(),
            dx_icon(DxUiIcon::Automations),
            "Automation tools",
            snapshot
                .trusted_tool_bridge
                .approved_automation_tool_count
                .to_string(),
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
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
        .into_any_element()
}

fn tools_section(id: &'static str, title: &'static str, icon: IconName) -> AnyElement {
    div()
        .id(id)
        .child(
            ListHeader::new(title)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted)),
        )
        .into_any_element()
}

fn tool_detail_stack(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().gap_1().children(rows).into_any_element()
}

fn tool_detail_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    let label = label.into();
    let detail = detail.into();
    let tooltip = format!("{}: {}", label.as_ref(), detail.as_ref());

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(detail)
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}
