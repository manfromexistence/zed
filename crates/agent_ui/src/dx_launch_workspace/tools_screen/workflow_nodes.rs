use gpui::{AnyElement, App, IntoElement, SharedString};
use ui::{Button, ButtonStyle, ContextMenu, IconName, PopoverMenu, Tooltip, prelude::*};

use crate::dx_agent_bridge::{
    DxAgentBridgeSnapshot, DxConfiguredPluginSummary, DxWorkflowNodeSummary,
};
use crate::workflow_node_icons::workflow_node_icon_for;

pub(super) const MAX_WORKFLOW_NODE_PLUGIN_CARDS: usize = 12;

pub(super) fn render_workflow_node_plugins(
    snapshot: &DxAgentBridgeSnapshot,
    cx: &mut App,
) -> AnyElement {
    let catalog = &snapshot.workflow_node_catalog;
    let mut stack = v_flex().gap_2().child(catalog_summary_row(snapshot, cx));

    if catalog.nodes.is_empty() {
        return stack
            .child(
                super::muted_card(
                    "Run DX JS workflow-node catalog generation to load dx.serializer.machine node metadata.",
                    cx,
                ),
            )
            .into_any_element();
    }

    for node in catalog.nodes.iter().take(MAX_WORKFLOW_NODE_PLUGIN_CARDS) {
        stack = stack.child(workflow_node_card(node, cx));
    }

    if catalog.nodes.len() > MAX_WORKFLOW_NODE_PLUGIN_CARDS {
        stack = stack.child(super::metric_row(
            "More nodes",
            format!(
                "{} additional workflow node(s) are available in the serialized catalog.",
                catalog
                    .nodes
                    .len()
                    .saturating_sub(MAX_WORKFLOW_NODE_PLUGIN_CARDS)
            ),
        ));
    }

    if !catalog.configured_plugins.is_empty() {
        stack = stack
            .child(super::section_title(
                "Configured Plugins",
                dx_icon(DxUiIcon::Plugins),
            ))
            .children(catalog.configured_plugins.iter().map(configured_plugin_row));
    }

    stack.into_any_element()
}

fn catalog_summary_row(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    let catalog = &snapshot.workflow_node_catalog;
    v_flex()
        .gap_1()
        .child(super::metric_row("Catalog", catalog.status.clone()))
        .child(super::metric_row(
            "Serializer",
            catalog.serializer_format.clone(),
        ))
        .child(super::metric_row(
            "Nodes",
            format!(
                "{} indexed / {} configured",
                catalog.node_count, catalog.configured_plugin_count
            ),
        ))
        .child(super::metric_row(
            "Source",
            catalog
                .source_packages
                .first()
                .cloned()
                .unwrap_or_else(|| catalog.catalog_path.display().to_string()),
        ))
        .when(!catalog.present, |this| {
            this.child(super::muted_card(catalog.next_action.clone(), cx))
        })
        .into_any_element()
}

fn workflow_node_card(node: &DxWorkflowNodeSummary, cx: &App) -> AnyElement {
    let icon = workflow_node_icon(node);
    div()
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
                        .gap_2()
                        .items_start()
                        .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                        .child(
                            v_flex()
                                .min_w_0()
                                .flex_1()
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .min_w_0()
                                        .gap_1()
                                        .child(
                                            Label::new(node.display_name.clone())
                                                .size(LabelSize::Small)
                                                .color(Color::Default)
                                                .truncate(),
                                        )
                                        .child(
                                            Label::new(node.category.clone())
                                                .size(LabelSize::XSmall)
                                                .color(Color::Muted)
                                                .truncate(),
                                        ),
                                )
                                .child(
                                    Label::new(node.description.clone())
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted)
                                        .truncate(),
                                ),
                        )
                        .child(render_plugin_config_menu(node.clone())),
                )
                .child(
                    v_flex()
                        .gap_0p5()
                        .child(detail_row(
                            format!("dx-workflow-node-runtime-{}", node.id).into(),
                            IconName::ToolHammer,
                            "Runtime",
                            format!("{} / {}", node.runtime, node.trust_status),
                        ))
                        .child(detail_row(
                            format!("dx-workflow-node-ports-{}", node.id).into(),
                            IconName::ArrowRightLeft,
                            "Ports",
                            format!("{} in / {} out", node.input_count, node.output_count),
                        ))
                        .child(detail_row(
                            format!("dx-workflow-node-options-{}", node.id).into(),
                            dx_icon(DxUiIcon::Settings),
                            "Parameters",
                            format!(
                                "{} fields / {} dynamic options",
                                node.parameter_count, node.dynamic_option_count
                            ),
                        ))
                        .child(detail_row(
                            format!("dx-workflow-node-credentials-{}", node.id).into(),
                            dx_icon(DxUiIcon::Credentials),
                            "Credentials",
                            credential_detail(node),
                        )),
                ),
        )
        .into_any_element()
}

fn render_plugin_config_menu(node: DxWorkflowNodeSummary) -> AnyElement {
    let trigger_id = format!("dx-workflow-node-configure-{}", node.id);
    PopoverMenu::new(format!("dx-workflow-node-config-menu-{}", node.id))
        .trigger_with_tooltip(
            Button::new(trigger_id, "Configure")
                .style(ButtonStyle::Subtle)
                .label_size(LabelSize::Small)
                .start_icon(Icon::new(dx_icon(DxUiIcon::Credentials)).size(IconSize::Small)),
            Tooltip::text("Configure plugin credentials"),
        )
        .anchor(gpui::Anchor::BottomRight)
        .menu(move |window, cx| {
            let node = node.clone();
            Some(ContextMenu::build(window, cx, move |menu, _window, _cx| {
                menu.header(node.display_name.clone())
                    .custom_row({
                        let node = node.clone();
                        move |_window, _cx| plugin_config_status_row(&node)
                    })
                    .custom_row({
                        let node = node.clone();
                        move |_window, _cx| plugin_config_requirements_row(&node)
                    })
                    .custom_row(move |_window, _cx| plugin_config_next_action_row(&node))
            }))
        })
        .into_any_element()
}

fn plugin_config_status_row(node: &DxWorkflowNodeSummary) -> AnyElement {
    detail_menu_row(
        dx_icon(DxUiIcon::Credentials),
        "Credential status",
        node.credential_status.clone(),
    )
}

fn plugin_config_requirements_row(node: &DxWorkflowNodeSummary) -> AnyElement {
    detail_menu_row(
        IconName::FileTextOutlined,
        "Required types",
        if node.credential_types.is_empty() {
            "None".to_string()
        } else {
            node.credential_types.join(", ")
        },
    )
}

fn plugin_config_next_action_row(node: &DxWorkflowNodeSummary) -> AnyElement {
    detail_menu_row(
        IconName::PlayOutlined,
        "Action",
        node.configure_action.clone(),
    )
}

fn configured_plugin_row(plugin: &DxConfiguredPluginSummary) -> AnyElement {
    let icon = workflow_node_icon_for(
        plugin.icon.as_deref(),
        Some(plugin.node_id.as_str()),
        plugin.display_name.as_str(),
    );
    ListItem::new(SharedString::from(format!(
        "dx-configured-plugin-{}",
        plugin.id
    )))
    .spacing(ui::ListItemSpacing::Dense)
    .selectable(false)
    .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
    .child(
        h_flex()
            .min_w_0()
            .gap_1()
            .child(
                Label::new(plugin.display_name.clone())
                    .size(LabelSize::Small)
                    .color(Color::Default)
                    .truncate(),
            )
            .child(
                Label::new(format!("{} / {}", plugin.status, plugin.credential_status))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            ),
    )
    .into_any_element()
}

fn detail_row(
    id: SharedString,
    icon: IconName,
    label: &'static str,
    detail: impl Into<SharedString>,
) -> AnyElement {
    ListItem::new(id)
        .spacing(ui::ListItemSpacing::ExtraDense)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(label)
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

fn detail_menu_row(icon: IconName, label: &'static str, detail: String) -> AnyElement {
    h_flex()
        .min_w(rems(18.))
        .max_w(rems(34.))
        .gap_2()
        .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            v_flex()
                .min_w_0()
                .gap_0p5()
                .child(
                    Label::new(label)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(detail)
                        .size(LabelSize::XSmall)
                        .color(Color::Default)
                        .line_height_style(LineHeightStyle::UiLabel),
                ),
        )
        .into_any_element()
}

fn credential_detail(node: &DxWorkflowNodeSummary) -> String {
    if node.credential_types.is_empty() {
        node.credential_status.clone()
    } else {
        format!(
            "{}: {}",
            node.credential_status,
            node.credential_types.join(", ")
        )
    }
}

fn workflow_node_icon(node: &DxWorkflowNodeSummary) -> IconName {
    workflow_node_icon_for(
        node.icon.as_deref(),
        Some(node.category.as_str()),
        node.display_name.as_str(),
    )
}
