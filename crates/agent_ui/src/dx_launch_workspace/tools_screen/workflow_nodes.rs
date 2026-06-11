use gpui::{AnyElement, App, ClickEvent, IntoElement, SharedString, WeakEntity, Window};
use ui::{
    Button, ButtonStyle, Chip, ContextMenu, IconName, ListItem, PopoverMenu, Tooltip, prelude::*,
};

use crate::AgentPanel;
use crate::dx_agent_bridge::{DxConfiguredPluginSummary, DxWorkflowNodeSummary};
use crate::workflow_node_icons::{workflow_node_element_id, workflow_node_icon_asset_for};

const MAX_PLUGIN_CARD_TITLE_CHARS: usize = 48;
const MAX_PLUGIN_CARD_CATEGORY_CHARS: usize = 28;

pub(super) fn workflow_node_card(
    node: &DxWorkflowNodeSummary,
    selected: bool,
    panel: WeakEntity<AgentPanel>,
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
        .mt_4()
        .child(
            v_flex()
                .w_full()
                .h(rems_from_px(110.))
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
                        .justify_between()
                        .gap_2()
                        .items_start()
                        .child(
                            h_flex()
                                .min_w_0()
                                .flex_1()
                                .gap_2()
                                .items_start()
                                .child(icon.render(IconSize::Medium, Color::Muted))
                                .child(plugin_title_block(node)),
                        )
                        .child(plugin_action_stack(node.clone(), panel)),
                )
                .child(
                    h_flex()
                        .min_w_0()
                        .w_full()
                        .justify_between()
                        .gap_3()
                        .child(
                            Label::new(node.description.clone())
                                .size(LabelSize::Small)
                                .color(Color::Default)
                                .truncate(),
                        )
                        .child(
                            Label::new(format!(
                                "{} in / {} out / {} parameters",
                                node.input_count, node.output_count, node.parameter_count
                            ))
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                        ),
                )
                .child(
                    h_flex()
                        .min_w_0()
                        .w_full()
                        .justify_between()
                        .gap_2()
                        .child(plugin_source_row(node))
                        .child(plugin_status_chips(node)),
                ),
        )
        .into_any_element()
}

fn plugin_title_block(node: &DxWorkflowNodeSummary) -> AnyElement {
    h_flex()
        .min_w_0()
        .gap_2()
        .child(
            Headline::new(bounded_plugin_card_text(
                &node.display_name,
                MAX_PLUGIN_CARD_TITLE_CHARS,
            ))
            .size(HeadlineSize::Small),
        )
        .child(
            Label::new(bounded_plugin_card_text(
                &node.category,
                MAX_PLUGIN_CARD_CATEGORY_CHARS,
            ))
            .size(LabelSize::Small)
            .color(Color::Muted)
            .truncate(),
        )
        .into_any_element()
}

fn plugin_action_stack(node: DxWorkflowNodeSummary, panel: WeakEntity<AgentPanel>) -> AnyElement {
    h_flex()
        .gap_1()
        .flex_none()
        .child(render_plugin_config_menu(node, panel))
        .into_any_element()
}

fn plugin_source_row(node: &DxWorkflowNodeSummary) -> AnyElement {
    h_flex()
        .min_w_0()
        .gap_1()
        .child(
            Icon::new(dx_icon(DxUiIcon::Source))
                .size(IconSize::XSmall)
                .color(Color::Muted),
        )
        .child(
            Label::new(format!("{} / {}", node.source_package, node.source_path))
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn plugin_status_chips(node: &DxWorkflowNodeSummary) -> AnyElement {
    h_flex()
        .gap_1()
        .flex_none()
        .child(Chip::new(plugin_configured_state_label(node)).truncate())
        .child(Chip::new(node.runtime.clone()).truncate())
        .child(Chip::new(node.trust_status.clone()).truncate())
        .child(Chip::new(node.credential_status.clone()).truncate())
        .child(Chip::new(format!("{} dynamic", node.dynamic_option_count)).truncate())
        .into_any_element()
}

fn plugin_configured_state_label(node: &DxWorkflowNodeSummary) -> &'static str {
    if node.configured {
        "Configured"
    } else if node.credential_status == "not_required" {
        "Ready"
    } else {
        "Needs Setup"
    }
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

fn render_plugin_config_menu(
    node: DxWorkflowNodeSummary,
    panel: WeakEntity<AgentPanel>,
) -> AnyElement {
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
        let panel = panel.clone();
        Some(ContextMenu::build(window, cx, move |menu, _window, _cx| {
            let action_label = plugin_config_menu_action_label(&node);
            menu.header(node.display_name.clone())
                .custom_row({
                    let node = node.clone();
                    move |_window, _cx| plugin_config_status_row(&node)
                })
                .custom_row({
                    let node = node.clone();
                    move |_window, _cx| plugin_config_requirements_row(&node)
                })
                .custom_row({
                    let node = node.clone();
                    move |_window, _cx| plugin_config_next_action_row(&node)
                })
                .entry(action_label, None, {
                    let node = node.clone();
                    let panel = panel.clone();
                    move |window, cx| {
                        if let Some(panel) = panel.upgrade() {
                            panel.update(cx, |this, cx| {
                                this.draft_dx_workflow_node_configuration_prompt(
                                    node.clone(),
                                    window,
                                    cx,
                                );
                            });
                        }
                    }
                })
        }))
    })
    .into_any_element()
}

fn plugin_config_menu_action_label(node: &DxWorkflowNodeSummary) -> &'static str {
    if node.configured {
        "Review configuration"
    } else if node.credential_status == "not_required" {
        "Review plugin contract"
    } else {
        "Draft setup request"
    }
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

fn bounded_plugin_card_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut bounded = trimmed.chars().take(keep).collect::<String>();
    bounded.push_str("...");
    bounded
}
