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
