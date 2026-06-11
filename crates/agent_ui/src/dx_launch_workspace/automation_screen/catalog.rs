use editor::{Editor, EditorEvent, MultiBufferOffset};
use gpui::{
    AnyElement, App, Context, Entity, IntoElement, SharedString, UniformListScrollHandle, Window,
    point, px, uniform_list,
};
use ui::{
    ListItem, ListItemSpacing, ScrollableHandle, ToggleButtonGroup, ToggleButtonGroupSize,
    ToggleButtonGroupStyle, ToggleButtonSimple, Tooltip, WithScrollbar, prelude::*,
};

use crate::AgentPanel;
use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::super::catalog_chrome::{
    render_catalog_row_labels, render_catalog_search, render_catalog_status_chip,
};
use super::super::screen_chrome::{screen_empty_state, screen_section};

mod details;
mod entry;

use entry::{AutomationCatalogEntry, all_automation_entries};

const MAX_AUTOMATION_SEARCH_QUERY_CHARS: usize = 256;
const MAX_AUTOMATION_CATALOG_RESULTS: usize = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutomationCatalogFilter {
    All,
    Enabled,
    NeedsRuntime,
    Failed,
}

pub(crate) struct DxAutomationCatalogState {
    pub(crate) list: UniformListScrollHandle,
    query_editor: Entity<Editor>,
    filter: AutomationCatalogFilter,
    selected_entry_id: Option<String>,
}

impl DxAutomationCatalogState {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<AgentPanel>) -> Self {
        let query_editor = cx.new(|cx| {
            let mut input = Editor::single_line(window, cx);
            input.set_placeholder_text("Search automations...", window, cx);
            input
        });

        Self {
            list: UniformListScrollHandle::new(),
            query_editor,
            filter: AutomationCatalogFilter::All,
            selected_entry_id: None,
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

    pub(crate) fn set_filter(
        &mut self,
        filter: AutomationCatalogFilter,
        cx: &mut Context<AgentPanel>,
    ) {
        if self.filter != filter {
            self.filter = filter;
            self.scroll_to_top();
            cx.notify();
        }
    }

    pub(crate) fn set_selected_entry(&mut self, entry_id: String, cx: &mut Context<AgentPanel>) {
        if self.selected_entry_id.as_deref() != Some(entry_id.as_str()) {
            self.selected_entry_id = Some(entry_id);
            cx.notify();
        }
    }

    fn selected_entry(
        &self,
        snapshot: &DxAgentBridgeSnapshot,
        cx: &mut App,
    ) -> Option<AutomationCatalogEntry> {
        let entries = self.filtered_entries(snapshot, cx);
        if let Some(selected_entry_id) = self.selected_entry_id.as_deref() {
            if let Some(entry) = entries
                .iter()
                .copied()
                .find(|entry| entry.id(snapshot).as_str() == selected_entry_id)
            {
                return Some(entry);
            }
        }
        entries.first().copied()
    }

    fn filtered_entries(
        &self,
        snapshot: &DxAgentBridgeSnapshot,
        cx: &mut App,
    ) -> Vec<AutomationCatalogEntry> {
        let search = self.search_query(cx);
        all_automation_entries(snapshot)
            .into_iter()
            .filter(|entry| entry.matches_filter(snapshot, self.filter))
            .filter(|entry| {
                search
                    .as_deref()
                    .is_none_or(|query| entry.matches_search(snapshot, query))
            })
            .take(MAX_AUTOMATION_CATALOG_RESULTS)
            .collect()
    }

    fn search_query(&self, cx: &mut App) -> Option<String> {
        let buffer = self.query_editor.read(cx).buffer().clone();
        let snapshot = buffer.read(cx).snapshot(cx);
        let query = snapshot
            .text_for_range(MultiBufferOffset(0)..snapshot.len())
            .flat_map(str::chars)
            .take(MAX_AUTOMATION_SEARCH_QUERY_CHARS)
            .collect::<String>();
        let query = query.trim();
        if query.is_empty() {
            None
        } else {
            Some(query.to_lowercase())
        }
    }

    fn scroll_to_top(&mut self) {
        self.list.set_offset(point(px(0.), px(0.)));
    }
}

pub(crate) fn render_automation_catalog(
    snapshot: &DxAgentBridgeSnapshot,
    state: &mut DxAutomationCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let count = state.filtered_entries(snapshot, cx).len();

    v_flex()
        .gap_3()
        .child(render_automation_controls(state, cx))
        .child(
            h_flex()
                .w_full()
                .items_start()
                .gap_3()
                .child(render_automation_list(count, state, window, cx))
                .child(render_selected_automation_detail(snapshot, state, cx)),
        )
        .into_any_element()
}

pub(crate) fn render_automation_catalog_rows(
    state: &DxAutomationCatalogState,
    snapshot: Option<&DxAgentBridgeSnapshot>,
    range: std::ops::Range<usize>,
    cx: &mut Context<AgentPanel>,
) -> Vec<AnyElement> {
    let Some(snapshot) = snapshot else {
        return range
            .map(|_| {
                screen_empty_state(
                    "dx-automation-catalog-missing",
                    dx_icon(DxUiIcon::Receipts),
                    "Missing DX Agents automation snapshot.",
                    cx,
                )
            })
            .collect();
    };

    let entries = state.filtered_entries(snapshot, cx);
    let selected_entry_id = state
        .selected_entry(snapshot, cx)
        .map(|entry| entry.id(snapshot));

    range
        .filter_map(|row_index| entries.get(row_index).copied())
        .map(|entry| {
            let entry_id = entry.id(snapshot);
            render_automation_catalog_row(
                snapshot,
                entry,
                selected_entry_id.as_deref() == Some(entry_id.as_str()),
                entry_id,
                cx,
            )
        })
        .collect()
}

fn render_automation_list(
    count: usize,
    state: &DxAutomationCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let body = v_flex()
        .min_h(rems_from_px(360.))
        .overflow_y_hidden()
        .map(|this| {
            if count == 0 {
                return this
                    .child(render_empty_automation_catalog(state, cx))
                    .into_any_element();
            }

            this.child(
                uniform_list(
                    "dx-automation-catalog",
                    count,
                    cx.processor(AgentPanel::render_automation_catalog_rows),
                )
                .flex_grow_1()
                .track_scroll(&state.list),
            )
            .vertical_scrollbar_for(&state.list, window, cx)
            .into_any_element()
        });

    v_flex()
        .flex_1()
        .min_w(rems_from_px(360.))
        .child(screen_section(
            "dx-automation-catalog-list",
            "Automation Catalog",
            dx_icon(DxUiIcon::Automations),
            format!("{count} entries"),
            body.into_any_element(),
            cx,
        ))
        .into_any_element()
}

fn render_automation_controls(
    state: &DxAutomationCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    h_flex()
        .w_full()
        .flex_wrap()
        .gap_2()
        .child(render_catalog_search(
            "dx-automation-catalog-search",
            &state.query_editor,
            cx,
        ))
        .child(
            ToggleButtonGroup::single_row(
                "dx-automation-filter-buttons",
                [
                    ToggleButtonSimple::new(
                        "All",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_automation_catalog_filter(AutomationCatalogFilter::All, cx);
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Enabled",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_automation_catalog_filter(
                                AutomationCatalogFilter::Enabled,
                                cx,
                            );
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Needs Runtime",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_automation_catalog_filter(
                                AutomationCatalogFilter::NeedsRuntime,
                                cx,
                            );
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Failed",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_automation_catalog_filter(AutomationCatalogFilter::Failed, cx);
                        }),
                    ),
                ],
            )
            .style(ToggleButtonGroupStyle::Outlined)
            .size(ToggleButtonGroupSize::Custom(rems_from_px(30.)))
            .label_size(LabelSize::Default)
            .auto_width()
            .selected_index(match state.filter {
                AutomationCatalogFilter::All => 0,
                AutomationCatalogFilter::Enabled => 1,
                AutomationCatalogFilter::NeedsRuntime => 2,
                AutomationCatalogFilter::Failed => 3,
            }),
        )
        .into_any_element()
}

fn render_automation_catalog_row(
    snapshot: &DxAgentBridgeSnapshot,
    entry: AutomationCatalogEntry,
    selected: bool,
    entry_id: String,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let panel = cx.weak_entity();
    let click_entry_id = entry_id.clone();
    let title = entry.title(snapshot);
    let detail_label = entry.detail_label(snapshot);
    let tooltip = format!("{title}: {detail_label}");

    ListItem::new(format!("dx-automation-catalog-row-{entry_id}"))
        .spacing(ListItemSpacing::Sparse)
        .rounded()
        .toggle_state(selected)
        .start_slot(
            Icon::new(entry.icon())
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .end_slot(render_catalog_status_chip(entry.status(snapshot)))
        .tooltip(Tooltip::text(tooltip))
        .on_click(move |_event, _window, cx| {
            panel
                .update(cx, |panel, cx| {
                    panel.set_automation_catalog_selected_entry(click_entry_id.clone(), cx);
                })
                .ok();
        })
        .child(render_catalog_row_labels(title, detail_label))
        .into_any_element()
}

fn render_selected_automation_detail(
    snapshot: &DxAgentBridgeSnapshot,
    state: &DxAutomationCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let body = if let Some(entry) = state.selected_entry(snapshot, cx) {
        entry.detail_body(snapshot)
    } else {
        screen_empty_state(
            "dx-automation-detail-empty",
            dx_icon(DxUiIcon::Automations),
            "Select an automation entry to inspect receipt-backed details.",
            cx,
        )
    };

    v_flex()
        .w(rems_from_px(344.))
        .min_w(rems_from_px(304.))
        .flex_none()
        .child(screen_section(
            "dx-automation-selected-detail",
            "Automation Detail",
            dx_icon(DxUiIcon::Automations),
            state
                .selected_entry(snapshot, cx)
                .map(|entry| SharedString::from(entry.detail_label(snapshot)))
                .unwrap_or_else(|| "No selection".into()),
            body,
            cx,
        ))
        .into_any_element()
}

fn render_empty_automation_catalog(
    state: &DxAutomationCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let message = if state.search_query(cx).is_some() {
        "No automations match this search."
    } else {
        match state.filter {
            AutomationCatalogFilter::All => "No automation entries are available.",
            AutomationCatalogFilter::Enabled => "No enabled automation rows.",
            AutomationCatalogFilter::NeedsRuntime => "No automation rows are blocked on runtime.",
            AutomationCatalogFilter::Failed => "No failed automation proof rows.",
        }
    };

    screen_empty_state(
        "dx-automation-catalog-empty",
        dx_icon(DxUiIcon::Automations),
        message,
        cx,
    )
}
