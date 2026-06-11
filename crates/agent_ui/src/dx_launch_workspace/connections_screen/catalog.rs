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
mod status;

use entry::{ConnectionCatalogEntry, all_connection_entries};

const MAX_CONNECTION_SEARCH_QUERY_CHARS: usize = 256;
const MAX_CONNECTION_CATALOG_RESULTS: usize = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConnectionCatalogFilter {
    All,
    Connected,
    NeedsAuth,
    Receipts,
}

pub(crate) struct DxConnectionsCatalogState {
    pub(crate) list: UniformListScrollHandle,
    query_editor: Entity<Editor>,
    filter: ConnectionCatalogFilter,
    selected_entry_id: Option<String>,
}

impl DxConnectionsCatalogState {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<AgentPanel>) -> Self {
        let query_editor = cx.new(|cx| {
            let mut input = Editor::single_line(window, cx);
            input.set_placeholder_text("Search connections...", window, cx);
            input
        });

        Self {
            list: UniformListScrollHandle::new(),
            query_editor,
            filter: ConnectionCatalogFilter::All,
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
        filter: ConnectionCatalogFilter,
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
    ) -> Option<ConnectionCatalogEntry> {
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
    ) -> Vec<ConnectionCatalogEntry> {
        let search = self.search_query(cx);
        all_connection_entries(snapshot)
            .into_iter()
            .filter(|entry| entry.matches_filter(snapshot, self.filter))
            .filter(|entry| {
                search
                    .as_deref()
                    .is_none_or(|query| entry.matches_search(snapshot, query))
            })
            .take(MAX_CONNECTION_CATALOG_RESULTS)
            .collect()
    }

    fn search_query(&self, cx: &mut App) -> Option<String> {
        let buffer = self.query_editor.read(cx).buffer().clone();
        let snapshot = buffer.read(cx).snapshot(cx);
        let query = snapshot
            .text_for_range(MultiBufferOffset(0)..snapshot.len())
            .flat_map(str::chars)
            .take(MAX_CONNECTION_SEARCH_QUERY_CHARS)
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

pub(crate) fn render_connections_catalog(
    snapshot: &DxAgentBridgeSnapshot,
    state: &mut DxConnectionsCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let count = state.filtered_entries(snapshot, cx).len();

    v_flex()
        .gap_3()
        .child(render_connection_controls(state, cx))
        .child(
            h_flex()
                .w_full()
                .items_start()
                .gap_3()
                .child(render_connection_list(count, state, window, cx))
                .child(render_selected_connection_detail(snapshot, state, cx)),
        )
        .into_any_element()
}

pub(crate) fn render_connections_catalog_rows(
    state: &DxConnectionsCatalogState,
    snapshot: Option<&DxAgentBridgeSnapshot>,
    range: std::ops::Range<usize>,
    cx: &mut Context<AgentPanel>,
) -> Vec<AnyElement> {
    let Some(snapshot) = snapshot else {
        return range
            .map(|_| {
                screen_empty_state(
                    "dx-connections-catalog-missing",
                    dx_icon(DxUiIcon::Receipts),
                    "Missing DX Agents connection snapshot.",
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
            render_connection_catalog_row(
                snapshot,
                entry,
                selected_entry_id.as_deref() == Some(entry_id.as_str()),
                entry_id,
                cx,
            )
        })
        .collect()
}

fn render_connection_list(
    count: usize,
    state: &DxConnectionsCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let body = v_flex()
        .min_h(rems_from_px(360.))
        .overflow_y_hidden()
        .map(|this| {
            if count == 0 {
                return this
                    .child(render_empty_connection_catalog(state, cx))
                    .into_any_element();
            }

            this.child(
                uniform_list(
                    "dx-connections-catalog",
                    count,
                    cx.processor(AgentPanel::render_connections_catalog_rows),
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
            "dx-connections-catalog-list",
            "Connection Catalog",
            dx_icon(DxUiIcon::Connections),
            format!("{count} entries"),
            body.into_any_element(),
            cx,
        ))
        .into_any_element()
}

fn render_connection_controls(
    state: &DxConnectionsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    h_flex()
        .w_full()
        .flex_wrap()
        .gap_2()
        .child(render_catalog_search(
            "dx-connections-catalog-search",
            &state.query_editor,
            cx,
        ))
        .child(
            ToggleButtonGroup::single_row(
                "dx-connections-filter-buttons",
                [
                    ToggleButtonSimple::new(
                        "All",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_connections_catalog_filter(ConnectionCatalogFilter::All, cx);
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Connected",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_connections_catalog_filter(
                                ConnectionCatalogFilter::Connected,
                                cx,
                            );
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Needs Auth",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_connections_catalog_filter(
                                ConnectionCatalogFilter::NeedsAuth,
                                cx,
                            );
                        }),
                    ),
                    ToggleButtonSimple::new(
                        "Receipts",
                        cx.listener(|this, _event, _window, cx| {
                            this.set_connections_catalog_filter(
                                ConnectionCatalogFilter::Receipts,
                                cx,
                            );
                        }),
                    ),
                ],
            )
            .style(ToggleButtonGroupStyle::Outlined)
            .size(ToggleButtonGroupSize::Custom(rems_from_px(30.)))
            .label_size(LabelSize::Default)
            .auto_width()
            .selected_index(match state.filter {
                ConnectionCatalogFilter::All => 0,
                ConnectionCatalogFilter::Connected => 1,
                ConnectionCatalogFilter::NeedsAuth => 2,
                ConnectionCatalogFilter::Receipts => 3,
            }),
        )
        .into_any_element()
}

fn render_connection_catalog_row(
    snapshot: &DxAgentBridgeSnapshot,
    entry: ConnectionCatalogEntry,
    selected: bool,
    entry_id: String,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let panel = cx.weak_entity();
    let click_entry_id = entry_id.clone();
    let title = entry.title(snapshot);
    let detail_label = entry.detail_label(snapshot);
    let tooltip = format!("{title}: {detail_label}");

    ListItem::new(format!("dx-connections-catalog-row-{entry_id}"))
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
                    panel.set_connections_catalog_selected_entry(click_entry_id.clone(), cx);
                })
                .ok();
        })
        .child(render_catalog_row_labels(title, detail_label))
        .into_any_element()
}

fn render_selected_connection_detail(
    snapshot: &DxAgentBridgeSnapshot,
    state: &DxConnectionsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let body = if let Some(entry) = state.selected_entry(snapshot, cx) {
        entry.detail_body(snapshot)
    } else {
        screen_empty_state(
            "dx-connections-detail-empty",
            dx_icon(DxUiIcon::Connections),
            "Select a connection entry to inspect receipt-backed details.",
            cx,
        )
    };

    v_flex()
        .w(rems_from_px(344.))
        .min_w(rems_from_px(304.))
        .flex_none()
        .child(screen_section(
            "dx-connections-selected-detail",
            "Connection Detail",
            dx_icon(DxUiIcon::Connections),
            state
                .selected_entry(snapshot, cx)
                .map(|entry| SharedString::from(entry.detail_label(snapshot)))
                .unwrap_or_else(|| "No selection".into()),
            body,
            cx,
        ))
        .into_any_element()
}

fn render_empty_connection_catalog(
    state: &DxConnectionsCatalogState,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let message = if state.search_query(cx).is_some() {
        "No connection entries match this search."
    } else {
        match state.filter {
            ConnectionCatalogFilter::All => "No connection entries are available.",
            ConnectionCatalogFilter::Connected => "No connected provider or social account rows.",
            ConnectionCatalogFilter::NeedsAuth => "No connection rows need authentication.",
            ConnectionCatalogFilter::Receipts => "No receipt-backed connection rows are available.",
        }
    };

    screen_empty_state(
        "dx-connections-catalog-empty",
        dx_icon(DxUiIcon::Connections),
        message,
        cx,
    )
}
