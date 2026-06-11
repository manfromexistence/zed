use std::collections::BTreeMap;
use std::ops::Range;

use editor::{Editor, EditorEvent, MultiBufferOffset};
use gpui::{
    AnyElement, App, Context, Entity, IntoElement, UniformListScrollHandle, Window, point, px,
    uniform_list,
};
use ui::{
    Button, ButtonStyle, Color, Headline, HeadlineSize, Label, LabelSize, ScrollableHandle,
    ToggleButtonGroup, ToggleButtonGroupSize, ToggleButtonGroupStyle, ToggleButtonSimple, Tooltip,
    WithScrollbar, prelude::*,
};

use crate::AgentPanel;
use crate::dx_agent_bridge::{
    DxAgentBridgeSnapshot, DxWorkflowNodeCatalogSummary, DxWorkflowNodeSummary,
    catalog_receipt_status_label,
};
use crate::workflow_node_icons::workflow_node_element_id;

use super::super::catalog_chrome::render_catalog_search;
use super::super::screen_chrome::{screen_empty_state, screen_section, workspace_stat};
use super::workflow_nodes;

const MAX_PLUGIN_CATALOG_SEARCH_QUERY_CHARS: usize = 256;
const MAX_FILTERED_WORKFLOW_NODE_RESULTS: usize = 1_000;
const MAX_DISPLAYED_WORKFLOW_NODE_RESULTS: usize = MAX_FILTERED_WORKFLOW_NODE_RESULTS;
const MAX_PLUGIN_CATEGORY_FILTER_CHARS: usize = 26;
const MAX_PLUGIN_CATALOG_SOURCE_CHARS: usize = 42;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PluginCatalogFilter {
    All,
    Configured,
    NeedsSetup,
}

pub(crate) struct DxPluginsCatalogState {
    pub(crate) list: UniformListScrollHandle,
    pub(crate) query_editor: Entity<Editor>,
    filter: PluginCatalogFilter,
    category_filter: Option<String>,
    selected_node_id: Option<String>,
}

impl DxPluginsCatalogState {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<AgentPanel>) -> Self {
        let query_editor = cx.new(|cx| {
            let mut input = Editor::single_line(window, cx);
            input.set_placeholder_text("Search plugins...", window, cx);
            input
        });

        Self {
            list: UniformListScrollHandle::new(),
            query_editor,
            filter: PluginCatalogFilter::All,
            category_filter: None,
            selected_node_id: None,
        }
    }

    pub(crate) fn query_editor(&self) -> Entity<Editor> {
        self.query_editor.clone()
    }

    pub(crate) fn on_query_editor_event(
        &mut self,
        event: &EditorEvent,
        cx: &mut Context<AgentPanel>,
    ) {
        if matches!(event, EditorEvent::Edited { .. }) {
            self.scroll_to_top();
            cx.notify();
        }
    }

    pub(crate) fn set_filter(&mut self, filter: PluginCatalogFilter, cx: &mut Context<AgentPanel>) {
        if self.filter != filter {
            self.filter = filter;
            self.scroll_to_top();
            cx.notify();
        }
    }

    pub(crate) fn set_category_filter(
        &mut self,
        category: Option<String>,
        cx: &mut Context<AgentPanel>,
    ) {
        if self.category_filter != category {
            self.category_filter = category;
            self.scroll_to_top();
            cx.notify();
        }
    }

    pub(crate) fn set_selected_node(&mut self, node_id: String, cx: &mut Context<AgentPanel>) {
        if self.selected_node_id.as_deref() != Some(node_id.as_str()) {
            self.selected_node_id = Some(node_id);
            cx.notify();
        }
    }

    fn selected_node<'a>(
        &self,
        catalog: &'a DxWorkflowNodeCatalogSummary,
        cx: &mut App,
    ) -> Option<&'a DxWorkflowNodeSummary> {
        if let Some(selected_node_id) = self.selected_node_id.as_deref() {
            if let Some(node) = catalog
                .nodes
                .iter()
                .find(|node| node.id.as_str() == selected_node_id)
            {
                return Some(node);
            }
        }

        self.filtered_node_indices(catalog, cx)
            .first()
            .and_then(|index| catalog.nodes.get(*index))
    }

    pub(crate) fn filtered_node_indices(
        &self,
        catalog: &DxWorkflowNodeCatalogSummary,
        cx: &mut App,
    ) -> Vec<usize> {
        let search = self.search_query(cx);
        catalog
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                search
                    .as_deref()
                    .is_none_or(|query| workflow_node_matches_search(node, query))
            })
            .filter(|(_, node)| {
                self.category_filter
                    .as_deref()
                    .is_none_or(|category| node.category.eq_ignore_ascii_case(category))
            })
            .filter(|(_, node)| match self.filter {
                PluginCatalogFilter::All => true,
                PluginCatalogFilter::Configured => node.configured,
                PluginCatalogFilter::NeedsSetup => {
                    !node.configured && node.credential_status != "not_required"
                }
            })
            .map(|(index, _)| index)
            .take(MAX_FILTERED_WORKFLOW_NODE_RESULTS)
            .collect()
    }

    fn search_query(&self, cx: &mut App) -> Option<String> {
        let buffer = self.query_editor.read(cx).buffer().clone();
        let snapshot = buffer.read(cx).snapshot(cx);
        let search = snapshot
            .text_for_range(MultiBufferOffset(0)..snapshot.len())
            .flat_map(str::chars)
            .take(MAX_PLUGIN_CATALOG_SEARCH_QUERY_CHARS)
            .collect::<String>();
        let search = search.trim();
        if search.is_empty() {
            None
        } else {
            Some(search.to_lowercase())
        }
    }

    fn scroll_to_top(&mut self) {
        self.list.set_offset(point(px(0.), px(0.)));
    }
}

pub(crate) fn render_workflow_node_catalog(
    snapshot: &DxAgentBridgeSnapshot,
    state: &mut DxPluginsCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let catalog = &snapshot.workflow_node_catalog;
    let filtered_node_indices = state.filtered_node_indices(catalog, cx);
    let count = filtered_node_indices
        .len()
        .min(MAX_DISPLAYED_WORKFLOW_NODE_RESULTS);

    v_flex()
        .gap_0()
        .child(
            v_flex()
                .gap_4()
                .pb_3()
                .child(render_catalog_summary(catalog, cx))
                .child(render_catalog_controls(state, cx)),
        )
        .when(!catalog.nodes.is_empty(), |this| {
            this.child(render_category_filter_row(catalog, state, cx))
        })
        .child(
            h_flex()
                .w_full()
                .items_start()
                .gap_3()
                .pt_1()
                .child(
                    v_flex()
                        .flex_1()
                        .min_w(rems_from_px(360.))
                        .min_h(rems_from_px(420.))
                        .overflow_y_hidden()
                        .map(|this| {
                            if catalog.nodes.is_empty() {
                                return this
                                    .child(screen_empty_state(
                                        "dx-workflow-node-catalog-missing",
                                        dx_icon(DxUiIcon::Plugins),
                                        catalog_empty_message(catalog),
                                        cx,
                                    ))
                                    .into_any_element();
                            }
                            if count == 0 {
                                return this
                                    .child(render_empty_state(state, cx))
                                    .into_any_element();
                            }

                            this.child(
                                uniform_list(
                                    "dx-workflow-node-plugins",
                                    count,
                                    cx.processor(AgentPanel::render_tools_workflow_node_rows),
                                )
                                .flex_grow_1()
                                .pb_4()
                                .track_scroll(&state.list),
                            )
                            .vertical_scrollbar_for(&state.list, window, cx)
                            .into_any_element()
                        }),
                )
                .child(render_selected_workflow_node_detail(catalog, state, cx)),
        )
        .when(!catalog.configured_plugins.is_empty(), |this| {
            this.child(screen_section(
                "dx-configured-plugins",
                "Configured Plugins",
                dx_icon(DxUiIcon::Plugins),
                format!("{} configured", catalog.configured_plugins.len()),
                v_flex()
                    .gap_1()
                    .children(
                        catalog
                            .configured_plugins
                            .iter()
                            .map(workflow_nodes::configured_plugin_row),
                    )
                    .into_any_element(),
                cx,
            ))
        })
        .into_any_element()
}

pub(crate) fn render_workflow_node_rows(
    state: &DxPluginsCatalogState,
    snapshot: Option<&DxAgentBridgeSnapshot>,
    range: Range<usize>,
    cx: &mut Context<AgentPanel>,
) -> Vec<AnyElement> {
    let Some(snapshot) = snapshot else {
        return range
            .map(|_| workflow_nodes::missing_workflow_node_card("Missing plugin catalog."))
            .collect();
    };
    let catalog = &snapshot.workflow_node_catalog;
    let indices = state.filtered_node_indices(catalog, cx);
    let selected_node_id = state.selected_node(catalog, cx).map(|node| node.id.clone());
    range
        .map(|row_index| {
            let Some(node_index) = indices.get(row_index).copied() else {
                return workflow_nodes::missing_workflow_node_card("Missing plugin row.");
            };
            let Some(node) = catalog.nodes.get(node_index) else {
                return workflow_nodes::missing_workflow_node_card("Missing plugin metadata.");
            };
            let node_id = node.id.clone();
            workflow_nodes::workflow_node_card(
                node,
                selected_node_id.as_deref() == Some(node.id.as_str()),
                cx.weak_entity(),
                cx.listener(move |this, _event, _window, cx| {
                    this.set_plugin_catalog_selected_node(node_id.clone(), cx);
                }),
                cx,
            )
        })
        .collect()
}

fn render_selected_workflow_node_detail(
    catalog: &DxWorkflowNodeCatalogSummary,
    state: &DxPluginsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    super::details::render_selected_workflow_node_detail(
        catalog,
        state.selected_node(catalog, cx),
        cx,
    )
}

fn render_catalog_summary(
    catalog: &DxWorkflowNodeCatalogSummary,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    h_flex()
        .w_full()
        .justify_between()
        .items_start()
        .gap_3()
        .child(
            Label::new(catalog_summary_label(catalog))
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .child(
            h_flex()
                .gap_2()
                .flex_none()
                .child(workspace_stat(
                    "dx-plugin-summary-indexed",
                    "Indexed",
                    catalog.nodes.len().to_string(),
                    cx,
                ))
                .child(workspace_stat(
                    "dx-plugin-summary-configured",
                    "Configured",
                    catalog.configured_plugin_count.to_string(),
                    cx,
                ))
                .child(workspace_stat(
                    "dx-plugin-summary-source",
                    "Catalog",
                    catalog_receipt_status_label(&catalog.status),
                    cx,
                )),
        )
        .when(!catalog.present, |this| {
            this.child(screen_empty_state(
                "dx-workflow-node-catalog-source-missing",
                dx_icon(DxUiIcon::Receipts),
                catalog.next_action.clone(),
                cx,
            ))
        })
        .into_any_element()
}

fn render_catalog_controls(
    state: &DxPluginsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    h_flex()
        .w_full()
        .min_w_0()
        .flex_wrap()
        .gap_2()
        .child(render_catalog_search(
            "dx-plugin-catalog-search",
            &state.query_editor,
            cx,
        ))
        .child(
            ToggleButtonGroup::single_row(
                "dx-plugin-filter-buttons",
                [
                    ToggleButtonSimple::new(
                        "All",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_plugin_catalog_filter(PluginCatalogFilter::All, cx);
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Configured",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_plugin_catalog_filter(PluginCatalogFilter::Configured, cx);
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Needs Setup",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_plugin_catalog_filter(PluginCatalogFilter::NeedsSetup, cx);
                        }),
                    ),
                ],
            )
            .style(ToggleButtonGroupStyle::Outlined)
            .size(ToggleButtonGroupSize::Custom(rems_from_px(30.)))
            .label_size(LabelSize::Default)
            .auto_width()
            .selected_index(match state.filter {
                PluginCatalogFilter::All => 0,
                PluginCatalogFilter::Configured => 1,
                PluginCatalogFilter::NeedsSetup => 2,
            }),
        )
        .into_any_element()
}

fn render_category_filter_row(
    catalog: &DxWorkflowNodeCatalogSummary,
    state: &DxPluginsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let category_counts = indexed_category_counts(catalog);

    let all_tooltip = format!("All indexed categories: {} plugins", catalog.nodes.len());

    h_flex()
        .id("dx-plugin-category-filter-row")
        .gap_2()
        .py_2p5()
        .border_t_1()
        .border_b_1()
        .border_color(cx.theme().colors().border_variant)
        .overflow_x_scroll()
        .child(
            Button::new("filter-all-categories", "All")
                .style(if state.category_filter.is_none() {
                    ButtonStyle::Filled
                } else {
                    ButtonStyle::Subtle
                })
                .toggle_state(state.category_filter.is_none())
                .tooltip(Tooltip::text(all_tooltip))
                .on_click(cx.listener(|this, _event, _window, cx| {
                    this.set_plugin_catalog_category_filter(None, cx);
                })),
        )
        .children(category_counts.into_iter().map(|(category, count)| {
            let active = state.category_filter.as_deref() == Some(category.as_str());
            let button_id = workflow_node_element_id("filter-category", &category);
            Button::new(
                button_id,
                bounded_plugin_category_filter_label(&category, count),
            )
            .style(if active {
                ButtonStyle::Filled
            } else {
                ButtonStyle::Subtle
            })
            .toggle_state(active)
            .tooltip({
                let category = category.clone();
                move |_, cx| {
                    Tooltip::with_meta(
                        "Indexed category",
                        None,
                        format!("{category}: {count} plugins"),
                        cx,
                    )
                }
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.set_plugin_catalog_category_filter(Some(category.clone()), cx);
            }))
        }))
        .into_any_element()
}

fn indexed_category_counts(catalog: &DxWorkflowNodeCatalogSummary) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::<String, usize>::new();
    for node in &catalog.nodes {
        *counts.entry(node.category.clone()).or_default() += 1;
    }
    counts.into_iter().collect()
}

fn render_empty_state(state: &DxPluginsCatalogState, cx: &mut Context<AgentPanel>) -> AnyElement {
    let message = if state.search_query(cx).is_some() {
        "No workflow-node plugins match this search."
    } else {
        match state.filter {
            PluginCatalogFilter::All => "No workflow-node plugins are indexed.",
            PluginCatalogFilter::Configured => "No configured plugins match this filter.",
            PluginCatalogFilter::NeedsSetup => "No plugins need credential setup.",
        }
    };

    v_flex()
        .gap_1()
        .child(Headline::new("No Plugins").size(HeadlineSize::Small))
        .child(screen_empty_state(
            "dx-workflow-node-catalog-empty",
            dx_icon(DxUiIcon::Plugins),
            message,
            cx,
        ))
        .into_any_element()
}

fn catalog_summary_label(catalog: &DxWorkflowNodeCatalogSummary) -> String {
    format!(
        "{} visible / {} source total from {}",
        catalog.nodes.len(),
        catalog.node_count,
        catalog_source_label(catalog)
    )
}

fn catalog_source_label(catalog: &DxWorkflowNodeCatalogSummary) -> String {
    catalog
        .source_packages
        .iter()
        .find_map(|source| sanitized_catalog_source_package(source))
        .unwrap_or_else(|| "DX serializer catalog receipt".to_string())
}

fn sanitized_catalog_source_package(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("missing_")
        || trimmed.starts_with("unknown_")
        || trimmed.starts_with('.')
        || trimmed.starts_with('/')
        || trimmed.contains('\\')
        || trimmed.contains(":\\")
        || trimmed.contains(":/")
        || trimmed.contains(".dx/")
        || trimmed.contains("crates/")
        || trimmed.contains("receipts/")
        || trimmed.contains("tools/")
        || (trimmed.contains('/') && !trimmed.starts_with('@'))
    {
        return None;
    }

    Some(bounded_plugin_category_text(
        trimmed,
        MAX_PLUGIN_CATALOG_SOURCE_CHARS,
    ))
}

fn catalog_empty_message(catalog: &DxWorkflowNodeCatalogSummary) -> String {
    if catalog.present {
        "No workflow-node plugins are indexed.".to_string()
    } else {
        catalog.next_action.clone()
    }
}

fn bounded_plugin_category_filter_label(category: &str, count: usize) -> String {
    let suffix = format!(" ({count})");
    let max_category_chars = MAX_PLUGIN_CATEGORY_FILTER_CHARS.saturating_sub(suffix.len());
    format!(
        "{}{}",
        bounded_plugin_category_text(category, max_category_chars),
        suffix
    )
}

fn bounded_plugin_category_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut bounded = trimmed.chars().take(keep).collect::<String>();
    bounded.push_str("...");
    bounded
}

fn workflow_node_matches_search(
    node: &crate::dx_agent_bridge::DxWorkflowNodeSummary,
    query: &str,
) -> bool {
    node.id.to_lowercase().contains(query)
        || node.display_name.to_lowercase().contains(query)
        || node.category.to_lowercase().contains(query)
        || node.description.to_lowercase().contains(query)
        || node.runtime.to_lowercase().contains(query)
        || node.trust_status.to_lowercase().contains(query)
        || node.source_package.to_lowercase().contains(query)
        || node
            .credential_types
            .iter()
            .any(|credential| credential.to_lowercase().contains(query))
}
