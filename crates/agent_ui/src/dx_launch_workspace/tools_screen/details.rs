use gpui::{AnyElement, Context, IntoElement, SharedString};
use ui::{Color, Headline, HeadlineSize, IconName, Label, LabelSize, prelude::*};

use crate::AgentPanel;
use crate::dx_agent_bridge::{DxWorkflowNodeCatalogSummary, DxWorkflowNodeSummary};
use crate::workflow_node_icons::{workflow_node_element_id, workflow_node_icon_asset_for};

use super::super::screen_chrome::{
    screen_detail_row, screen_detail_stack, screen_empty_state, screen_section,
};

pub(super) fn render_selected_workflow_node_detail(
    catalog: &DxWorkflowNodeCatalogSummary,
    node: Option<&DxWorkflowNodeSummary>,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let Some(node) = node else {
        return v_flex()
            .w(rems_from_px(340.))
            .flex_none()
            .child(screen_empty_state(
                "dx-workflow-node-detail-empty",
                dx_icon(DxUiIcon::Plugins),
                "Select a plugin to inspect serializer metadata.",
                cx,
            ))
            .into_any_element();
    };

    v_flex()
        .w(rems_from_px(340.))
        .flex_none()
        .gap_3()
        .child(render_workflow_node_summary(node, cx))
        .child(render_workflow_node_configuration(node, cx))
        .child(render_workflow_node_contract(catalog, node, cx))
        .child(render_workflow_node_permissions(node, cx))
        .child(render_workflow_node_ports(node, cx))
        .child(render_workflow_node_dynamic_options(node, cx))
        .child(render_workflow_node_receipts(node, cx))
        .child(render_workflow_node_actions(node, cx))
        .child(render_workflow_node_trust(node, cx))
        .into_any_element()
}

fn render_workflow_node_summary(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let icon = workflow_node_icon_asset_for(
        node.icon.as_deref(),
        Some(node.category.as_str()),
        node.display_name.as_str(),
    );

    screen_section(
        "dx-workflow-node-detail-summary",
        "Plugin Detail",
        dx_icon(DxUiIcon::Plugins),
        node.category.clone(),
        h_flex()
            .items_start()
            .gap_2()
            .child(icon.render(IconSize::Medium, Color::Default))
            .child(
                v_flex()
                    .min_w_0()
                    .gap_0p5()
                    .child(Headline::new(node.display_name.clone()).size(HeadlineSize::Small))
                    .child(
                        Label::new(node.category.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .truncate(),
                    )
                    .child(
                        Label::new(node.description.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .line_height_style(LineHeightStyle::UiLabel),
                    ),
            )
            .into_any_element(),
        cx,
    )
}

fn render_workflow_node_configuration(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-configuration",
        "Credential Setup",
        dx_icon(DxUiIcon::Credentials),
        node.credential_status.clone(),
        screen_detail_stack(
            vec![
                screen_detail_row(
                    workflow_node_element_id("dx-workflow-node-detail-credential-state", &node.id),
                    dx_icon(DxUiIcon::Credentials),
                    "Status",
                    credential_setup_state(node),
                ),
                screen_detail_row(
                    workflow_node_element_id("dx-workflow-node-detail-credential-types", &node.id),
                    IconName::FileTextOutlined,
                    "Types",
                    credential_type_summary(node),
                ),
            ]
            .into_iter()
            .chain(
                node.credentials
                    .iter()
                    .enumerate()
                    .map(|(index, credential)| {
                        screen_detail_row(
                            workflow_node_element_id(
                                &format!("dx-workflow-node-detail-credential-{index}"),
                                &node.id,
                            ),
                            dx_icon(DxUiIcon::Credentials),
                            "Credential",
                            format!(
                                "{} status {}",
                                credential.credential_type, credential.status
                            ),
                        )
                    }),
            )
            .chain(std::iter::once(screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-configure-action", &node.id),
                IconName::PlayOutlined,
                "Bridge action",
                "Resolved privately from trusted DX receipts".to_string(),
            )))
            .collect(),
        ),
        cx,
    )
}

fn render_workflow_node_contract(
    catalog: &DxWorkflowNodeCatalogSummary,
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-contract",
        "Plugin Contract",
        IconName::FileTextOutlined,
        node.runtime.clone(),
        screen_detail_stack(vec![
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-serializer", &node.id),
                IconName::FileTextOutlined,
                "Serializer",
                catalog.serializer_format.clone(),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-schema", &node.id),
                IconName::FileTextOutlined,
                "Schema",
                catalog.schema_version.clone(),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-runtime", &node.id),
                IconName::ToolHammer,
                "Runtime",
                format!("{} runtime, {} trust", node.runtime, node.trust_status),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-source", &node.id),
                dx_icon(DxUiIcon::Source),
                "Source",
                node.source_package.clone(),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-ports", &node.id),
                IconName::ArrowRightLeft,
                "Ports",
                format!("{} inputs, {} outputs", node.input_count, node.output_count),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-options", &node.id),
                dx_icon(DxUiIcon::Settings),
                "Parameters",
                format!(
                    "{} fields, {} dynamic options",
                    node.parameter_count, node.dynamic_option_count
                ),
            ),
        ]),
        cx,
    )
}

fn render_workflow_node_permissions(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-permissions",
        "Permissions",
        dx_icon(DxUiIcon::Permissions),
        format!("{} rows", node.permissions.len()),
        screen_detail_stack(metadata_or_empty(
            &node.permissions,
            "No permission metadata declared.",
            |permission, index| {
                screen_detail_row(
                    workflow_node_element_id(
                        &format!("dx-workflow-node-detail-permission-{index}"),
                        &node.id,
                    ),
                    dx_icon(DxUiIcon::Permissions),
                    "Permission",
                    format!(
                        "{} level, {} status, receipt required {}",
                        permission.level,
                        permission.status,
                        yes_no(permission.receipt_required)
                    ),
                )
            },
        )),
        cx,
    )
}

fn render_workflow_node_ports(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-ports",
        "Ports",
        IconName::ArrowRightLeft,
        format!(
            "{} inputs, {} outputs",
            node.inputs.len(),
            node.outputs.len()
        ),
        v_flex()
            .gap_2()
            .child(Headline::new("Inputs").size(HeadlineSize::XSmall))
            .child(screen_detail_stack(metadata_or_empty(
                &node.inputs,
                "No input metadata declared.",
                |port, index| {
                    screen_detail_row(
                        workflow_node_element_id(
                            &format!("dx-workflow-node-detail-input-{index}"),
                            &node.id,
                        ),
                        IconName::ArrowRightLeft,
                        "Input",
                        format!(
                            "{} kind {}, required {}",
                            port.name,
                            port.kind,
                            yes_no(port.required)
                        ),
                    )
                },
            )))
            .child(Headline::new("Outputs").size(HeadlineSize::XSmall))
            .child(screen_detail_stack(metadata_or_empty(
                &node.outputs,
                "No output metadata declared.",
                |port, index| {
                    screen_detail_row(
                        workflow_node_element_id(
                            &format!("dx-workflow-node-detail-output-{index}"),
                            &node.id,
                        ),
                        IconName::ArrowRightLeft,
                        "Output",
                        format!(
                            "{} kind {}, required {}",
                            port.name,
                            port.kind,
                            yes_no(port.required)
                        ),
                    )
                },
            )))
            .into_any_element(),
        cx,
    )
}

fn render_workflow_node_dynamic_options(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-dynamic-options",
        "Dynamic Options",
        dx_icon(DxUiIcon::Settings),
        format!("{} rows", node.dynamic_options.len()),
        screen_detail_stack(metadata_or_empty(
            &node.dynamic_options,
            "No dynamic option metadata declared.",
            |option, index| {
                screen_detail_row(
                    workflow_node_element_id(
                        &format!("dx-workflow-node-detail-dynamic-option-{index}"),
                        &node.id,
                    ),
                    dx_icon(DxUiIcon::Settings),
                    "Option",
                    format!("{} status {}", option.label, option.status),
                )
            },
        )),
        cx,
    )
}

fn render_workflow_node_receipts(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-receipts",
        "Receipts",
        IconName::FileTextOutlined,
        format!("{} rows", node.receipts.len()),
        screen_detail_stack(metadata_or_empty(
            &node.receipts,
            "No receipt metadata declared.",
            |receipt, index| {
                screen_detail_row(
                    workflow_node_element_id(
                        &format!("dx-workflow-node-detail-receipt-{index}"),
                        &node.id,
                    ),
                    IconName::FileTextOutlined,
                    "Receipt",
                    format!(
                        "{} schema, {} status, required for {}",
                        receipt.schema, receipt.status, receipt.required_for
                    ),
                )
            },
        )),
        cx,
    )
}

fn render_workflow_node_actions(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-actions",
        "Actions",
        IconName::PlayOutlined,
        format!("{} rows", node.actions.len()),
        screen_detail_stack(metadata_or_empty(
            &node.actions,
            "No action metadata declared.",
            |action, index| {
                screen_detail_row(
                    workflow_node_element_id(
                        &format!("dx-workflow-node-detail-action-{index}"),
                        &node.id,
                    ),
                    IconName::PlayOutlined,
                    "Action",
                    format!(
                        "{} risk {}, approval required {}",
                        action.label,
                        action.risk,
                        yes_no(action.requires_approval)
                    ),
                )
            },
        )),
        cx,
    )
}

fn render_workflow_node_trust(
    node: &DxWorkflowNodeSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    screen_section(
        "dx-workflow-node-detail-trust",
        "Trust",
        dx_icon(DxUiIcon::Permissions),
        node.trust.status.clone(),
        screen_detail_stack(vec![
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-trust-policy", &node.id),
                dx_icon(DxUiIcon::Permissions),
                "Policy",
                format!(
                    "{} status, {} policy",
                    node.trust.status, node.trust.trust_policy
                ),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-trust-source", &node.id),
                dx_icon(DxUiIcon::Source),
                "Source",
                format!(
                    "source owned {}, first-party {}, bridge approved {}",
                    yes_no(node.trust.source_owned),
                    yes_no(node.trust.first_party),
                    yes_no(node.trust.approved_by_trusted_bridge)
                ),
            ),
            screen_detail_row(
                workflow_node_element_id("dx-workflow-node-detail-trust-enable", &node.id),
                dx_icon(DxUiIcon::Plugins),
                "Enablement",
                format!(
                    "enabled by default {}, user enablement required {}",
                    yes_no(node.trust.enabled_by_default),
                    yes_no(node.trust.requires_user_enablement_for_input)
                ),
            ),
        ]),
        cx,
    )
}

fn metadata_or_empty<T>(
    rows: &[T],
    empty: &'static str,
    render: impl Fn(&T, usize) -> AnyElement,
) -> Vec<AnyElement> {
    if rows.is_empty() {
        vec![screen_detail_row(
            SharedString::from(empty),
            IconName::Info,
            "Status",
            empty,
        )]
    } else {
        rows.iter()
            .enumerate()
            .map(|(index, row)| render(row, index))
            .collect()
    }
}

fn credential_setup_state(node: &DxWorkflowNodeSummary) -> String {
    if node.credential_status == "not_required" {
        "No credentials required by this plugin receipt.".to_string()
    } else if node.configured {
        format!(
            "Configured through DX Agents credential bridge ({})",
            node.credential_status
        )
    } else {
        format!(
            "DX Agents credential bridge receipt required before configuration ({})",
            node.credential_status
        )
    }
}

fn credential_type_summary(node: &DxWorkflowNodeSummary) -> String {
    if node.credential_types.is_empty() {
        "None declared".to_string()
    } else {
        node.credential_types.join(", ")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
