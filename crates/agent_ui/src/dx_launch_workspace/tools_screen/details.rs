use gpui::{AnyElement, Context, IntoElement, SharedString};
use ui::{Color, Headline, HeadlineSize, Icon, IconName, Label, LabelSize, prelude::*};

use crate::AgentPanel;
use crate::dx_agent_bridge::DxWorkflowNodeSummary;
use crate::workflow_node_icons::{workflow_node_element_id, workflow_node_icon_asset_for};

pub(super) fn render_selected_workflow_node_detail(
    node: Option<&DxWorkflowNodeSummary>,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let Some(node) = node else {
        return div()
            .w(rems_from_px(340.))
            .flex_none()
            .child(super::muted_card(
                "Select a plugin to inspect dx.serializer.machine metadata.",
                cx,
            ))
            .into_any_element();
    };
    let icon = workflow_node_icon_asset_for(
        node.icon.as_deref(),
        Some(node.category.as_str()),
        node.display_name.as_str(),
    );

    v_flex()
        .w(rems_from_px(340.))
        .flex_none()
        .gap_3()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().elevated_surface_background.opacity(0.5))
        .child(
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
                ),
        )
        .child(render_workflow_node_configuration(node))
        .child(render_workflow_node_contract(node))
        .child(render_workflow_node_permissions(node))
        .child(render_workflow_node_ports(node))
        .child(render_workflow_node_dynamic_options(node))
        .child(render_workflow_node_receipts(node))
        .child(render_workflow_node_actions(node))
        .child(render_workflow_node_trust(node))
        .into_any_element()
}

fn render_workflow_node_configuration(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Credential Setup").size(HeadlineSize::XSmall))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-credential-state", &node.id),
            dx_icon(DxUiIcon::Credentials),
            "Status",
            credential_setup_state(node),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-credential-types", &node.id),
            IconName::FileTextOutlined,
            "Types",
            credential_type_summary(node),
        ))
        .children(
            node.credentials
                .iter()
                .enumerate()
                .map(|(index, credential)| {
                    detail_row(
                        workflow_node_element_id(
                            format!("dx-workflow-node-detail-credential-{index}"),
                            &node.id,
                        ),
                        dx_icon(DxUiIcon::Credentials),
                        "Credential",
                        format!(
                            "{} / {} / receipt {}",
                            credential.credential_type, credential.status, credential.receipt_id
                        ),
                    )
                }),
        )
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-configure-action", &node.id),
            IconName::PlayOutlined,
            "Action",
            node.configure_action.clone(),
        ))
        .into_any_element()
}

fn render_workflow_node_contract(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Plugin Contract").size(HeadlineSize::XSmall))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-serializer", &node.id),
            IconName::FileTextOutlined,
            "Serializer",
            "dx.serializer.machine",
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-runtime", &node.id),
            IconName::ToolHammer,
            "Runtime",
            format!("{} / {}", node.runtime, node.trust_status),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-source", &node.id),
            dx_icon(DxUiIcon::Source),
            "Source",
            node.source_package.clone(),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-ports", &node.id),
            IconName::ArrowRightLeft,
            "Ports",
            format!("{} in / {} out", node.input_count, node.output_count),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-options", &node.id),
            dx_icon(DxUiIcon::Settings),
            "Parameters",
            format!(
                "{} fields / {} dynamic options",
                node.parameter_count, node.dynamic_option_count
            ),
        ))
        .into_any_element()
}

fn render_workflow_node_permissions(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Permissions").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.permissions,
            "No permission metadata declared.",
            |permission, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-permission-{index}"),
                        &node.id,
                    ),
                    dx_icon(DxUiIcon::Permissions),
                    "Permission",
                    format!(
                        "{} / {} / receipt_required={}",
                        permission.level, permission.status, permission.receipt_required
                    ),
                )
            },
        ))
        .into_any_element()
}

fn render_workflow_node_ports(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Inputs").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.inputs,
            "No input metadata declared.",
            |port, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-input-{index}"),
                        &node.id,
                    ),
                    IconName::ArrowRightLeft,
                    "Input",
                    format!("{} / {} / required={}", port.name, port.kind, port.required),
                )
            },
        ))
        .child(Headline::new("Outputs").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.outputs,
            "No output metadata declared.",
            |port, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-output-{index}"),
                        &node.id,
                    ),
                    IconName::ArrowRightLeft,
                    "Output",
                    format!("{} / {} / required={}", port.name, port.kind, port.required),
                )
            },
        ))
        .into_any_element()
}

fn render_workflow_node_dynamic_options(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Dynamic Options").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.dynamic_options,
            "No dynamic option metadata declared.",
            |option, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-dynamic-option-{index}"),
                        &node.id,
                    ),
                    dx_icon(DxUiIcon::Settings),
                    "Option",
                    format!(
                        "{} / {} / action {} / receipt {}",
                        option.label, option.status, option.action_id, option.receipt_id
                    ),
                )
            },
        ))
        .into_any_element()
}

fn render_workflow_node_receipts(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Receipts").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.receipts,
            "No receipt metadata declared.",
            |receipt, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-receipt-{index}"),
                        &node.id,
                    ),
                    IconName::FileTextOutlined,
                    "Receipt",
                    format!(
                        "{} / {} / {}",
                        receipt.schema, receipt.status, receipt.required_for
                    ),
                )
            },
        ))
        .into_any_element()
}

fn render_workflow_node_actions(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Actions").size(HeadlineSize::XSmall))
        .children(metadata_or_empty(
            &node.actions,
            "No action metadata declared.",
            |action, index| {
                detail_row(
                    workflow_node_element_id(
                        format!("dx-workflow-node-detail-action-{index}"),
                        &node.id,
                    ),
                    IconName::PlayOutlined,
                    "Action",
                    format!(
                        "{} / {} / approval={} / receipt {}",
                        action.label, action.risk, action.requires_approval, action.receipt_id
                    ),
                )
            },
        ))
        .into_any_element()
}

fn render_workflow_node_trust(node: &DxWorkflowNodeSummary) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Headline::new("Trust").size(HeadlineSize::XSmall))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-trust-policy", &node.id),
            dx_icon(DxUiIcon::Permissions),
            "Policy",
            format!("{} / {}", node.trust.status, node.trust.trust_policy),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-trust-source", &node.id),
            dx_icon(DxUiIcon::Source),
            "Source",
            format!(
                "owned={} / first_party={} / approved={}",
                node.trust.source_owned,
                node.trust.first_party,
                node.trust.approved_by_trusted_bridge
            ),
        ))
        .child(detail_row(
            workflow_node_element_id("dx-workflow-node-detail-trust-enable", &node.id),
            dx_icon(DxUiIcon::Plugins),
            "Enablement",
            format!(
                "default={} / user_enablement_required={}",
                node.trust.enabled_by_default, node.trust.requires_user_enablement_for_input
            ),
        ))
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

fn metadata_or_empty<T>(
    rows: &[T],
    empty: &'static str,
    render: impl Fn(&T, usize) -> AnyElement,
) -> Vec<AnyElement> {
    if rows.is_empty() {
        vec![detail_row(
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
