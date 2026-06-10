use gpui::{AnyElement, App, ClickEvent, IntoElement, SharedString, Window};
use ui::{Button, ButtonStyle, ContextMenu, IconName, PopoverMenu, Tooltip, prelude::*};

use crate::dx_agent_bridge::{DxConfiguredPluginSummary, DxWorkflowNodeSummary};
use crate::workflow_node_icons::{workflow_node_element_id, workflow_node_icon_asset_for};

pub(super) fn workflow_node_card(
    node: &DxWorkflowNodeSummary,
    selected: bool,
    on_select: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> AnyElement {
    let icon = workflow_node_icon_asset_for(
        node.icon.as_deref(),
        Some(node.category.as_str()),
        node.display_name.as_str(),
    );
    div()
        .w_full()
        .pb_2()
        .child(
            v_flex()
                .w_full()
                .p_3()
                .gap_2()
                .rounded_md()
                .border_1()
                .border_color(if selected {
                    cx.theme().colors().border
                } else {
                    cx.theme().colors().border_variant
                })
                .bg(if selected {
                    cx.theme().colors().element_selected
                } else {
                    cx.theme().colors().elevated_surface_background.opacity(0.5)
                })
                .hover(|this| this.bg(cx.theme().colors().element_hover))
                .on_click(on_select)
                .child(
                    h_flex()
                        .gap_2()
                        .items_start()
                        .child(icon.render(IconSize::Small, Color::Muted))
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
                            workflow_node_element_id("dx-workflow-node-runtime", &node.id),
                            IconName::ToolHammer,
                            "Runtime",
                            format!("{} / {}", node.runtime, node.trust_status),
                        ))
                        .child(detail_row(
                            workflow_node_element_id("dx-workflow-node-ports", &node.id),
                            IconName::ArrowRightLeft,
                            "Ports",
                            format!("{} in / {} out", node.input_count, node.output_count),
                        ))
                        .child(detail_row(
                            workflow_node_element_id("dx-workflow-node-options", &node.id),
                            dx_icon(DxUiIcon::Settings),
                            "Parameters",
                            format!(
                                "{} fields / {} dynamic options",
                                node.parameter_count, node.dynamic_option_count
                            ),
                        ))
                        .child(detail_row(
                            workflow_node_element_id("dx-workflow-node-credentials", &node.id),
                            dx_icon(DxUiIcon::Credentials),
                            "Credentials",
                            credential_detail(node),
                        )),
                ),
        )
        .into_any_element()
}

pub(super) fn missing_workflow_node_card(message: &'static str) -> AnyElement {
    ListItem::new(SharedString::from(message))
        .spacing(ui::ListItemSpacing::Dense)
        .selectable(false)
        .start_slot(
            Icon::new(dx_icon(DxUiIcon::Plugins))
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(message)
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .into_any_element()
}

fn render_plugin_config_menu(node: DxWorkflowNodeSummary) -> AnyElement {
    let trigger_id = workflow_node_element_id("dx-workflow-node-configure", &node.id);
    PopoverMenu::new(workflow_node_element_id(
        "dx-workflow-node-config-menu",
        &node.id,
    ))
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

pub(super) fn configured_plugin_row(plugin: &DxConfiguredPluginSummary) -> AnyElement {
    let icon = workflow_node_icon_asset_for(
        plugin.icon.as_deref(),
        Some(plugin.node_id.as_str()),
        plugin.display_name.as_str(),
    );
    ListItem::new(workflow_node_element_id("dx-configured-plugin", &plugin.id))
        .spacing(ui::ListItemSpacing::Dense)
        .selectable(false)
        .start_slot(icon.render(IconSize::Small, Color::Muted))
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
