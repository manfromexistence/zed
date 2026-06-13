use std::collections::{HashMap, HashSet};

use crate::dx_forge_panel::package_status::invalidate_package_status_snapshot_cache;
use crate::dx_receipt_history::invalidate_tool_history_snapshot_cache;
use crate::dx_source_sets::invalidate_source_set_snapshot_cache;
use gpui::{
    Action, App, AppContext, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, Render, ScrollAnchor, ScrollHandle, WeakEntity,
    Window, point, px,
};
use ui::prelude::*;
use ui::{DxUiIcon, IconName, dx_icon};
use workspace::{
    Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};
use zed_actions::dx_forge::TogglePanel;

use super::{
    machine_cache::invalidate_machine_cache_snapshot_cache,
    panel_view,
    remote_registry::invalidate_remote_registry_snapshot_cache,
    snapshot,
    visible_rows::{DxForgeRowKey, DxForgeVisibleRow, visible_rows_for_tab},
};

const DX_FORGE_PANEL_KEY: &str = "dx_forge_panel";
const DEFAULT_PANEL_WIDTH: gpui::Pixels = px(360.0);
const MIN_PANEL_WIDTH: gpui::Pixels = px(280.0);

pub(crate) fn init(cx: &mut App) {
    cx.observe_new(
        |workspace: &mut Workspace, window, cx: &mut Context<Workspace>| {
            workspace.register_action(|workspace, _: &TogglePanel, window, cx| {
                ensure_panel(workspace, window, cx);
                workspace.toggle_panel_focus::<DxForgePanel>(window, cx);
            });

            let Some(window) = window else {
                return;
            };

            ensure_panel(workspace, window, cx);
        },
    )
    .detach();
}

fn ensure_panel(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if workspace.panel::<DxForgePanel>(cx).is_some() {
        return;
    }

    let weak_workspace = workspace.weak_handle();
    let panel = cx.new(|cx| DxForgePanel::new(weak_workspace, cx));
    workspace.add_panel(panel, window, cx);
}

pub(crate) struct DxForgePanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    active_tab: DxForgePanelTab,
    pub(super) active_item: Option<DxForgeRowKey>,
    checked_items: HashSet<DxForgeRowKey>,
    pub(super) visible_rows: Vec<DxForgeVisibleRow>,
    row_scroll_anchors: HashMap<DxForgeRowKey, ScrollAnchor>,
}

impl DxForgePanel {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            active_tab: DxForgePanelTab::Repository,
            active_item: None,
            checked_items: HashSet::default(),
            visible_rows: Vec::new(),
            row_scroll_anchors: HashMap::default(),
        }
    }

    fn workspace_roots(&self, cx: &App) -> Vec<String> {
        let Some(workspace) = self.workspace.upgrade() else {
            return Vec::new();
        };

        workspace
            .read(cx)
            .root_paths(cx)
            .into_iter()
            .map(|path| path.display().to_string())
            .collect()
    }

    fn clear_active_item(&mut self) {
        self.active_item = None;
    }

    fn sync_visible_rows(&mut self, visible_rows: Vec<DxForgeVisibleRow>) {
        let visible_keys = visible_rows
            .iter()
            .map(|row| row.item_key().clone())
            .collect::<HashSet<_>>();
        let active_item_is_visible = self
            .active_item
            .as_ref()
            .is_none_or(|item_key| visible_keys.contains(item_key));
        if !active_item_is_visible {
            self.clear_active_item();
        }

        self.row_scroll_anchors
            .retain(|item_key, _| visible_keys.contains(item_key));
        for item_key in visible_keys {
            self.row_scroll_anchors
                .entry(item_key)
                .or_insert_with(|| ScrollAnchor::for_handle(self.scroll_handle.clone()));
        }
        self.visible_rows = visible_rows;
    }

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        invalidate_machine_cache_snapshot_cache();
        invalidate_package_status_snapshot_cache();
        invalidate_remote_registry_snapshot_cache();
        invalidate_tool_history_snapshot_cache();
        invalidate_source_set_snapshot_cache();
        self.clear_active_item();
        cx.notify();
    }

    pub(super) fn set_active_tab(&mut self, tab: DxForgePanelTab, cx: &mut Context<Self>) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.scroll_handle.set_offset(point(px(0.), px(0.)));
            self.clear_active_item();
            cx.notify();
        }
    }

    pub(super) fn toggle_item_checked(&mut self, item_key: DxForgeRowKey, cx: &mut Context<Self>) {
        if !self.checked_items.insert(item_key.clone()) {
            self.checked_items.remove(&item_key);
        }
        cx.notify();
    }

    pub(super) fn activate_item(&mut self, item_key: DxForgeRowKey, cx: &mut Context<Self>) {
        if self.active_item.as_ref() != Some(&item_key) {
            self.active_item = Some(item_key);
            cx.notify();
        }
    }

    pub(super) fn row_scroll_anchor(&self, item_key: &DxForgeRowKey) -> Option<ScrollAnchor> {
        self.row_scroll_anchors.get(item_key).cloned()
    }

    pub(super) fn scroll_item_into_view(
        &self,
        item_key: &DxForgeRowKey,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(scroll_anchor) = self.row_scroll_anchor(item_key) {
            scroll_anchor.scroll_to(window, cx);
        }
    }

    pub(super) fn item_checked(&self, item_key: &DxForgeRowKey) -> bool {
        self.checked_items.contains(item_key)
    }

    pub(super) fn item_active(&self, item_key: &DxForgeRowKey) -> bool {
        self.active_item.as_ref() == Some(item_key)
    }

    pub(super) fn focus_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
    }

    fn focus_panel_on_mouse_down(
        &mut self,
        _: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DxForgePanelTab {
    Repository,
    Packages,
    Media,
    Remotes,
}

impl Focusable for DxForgePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for DxForgePanel {}

impl Panel for DxForgePanel {
    fn persistent_name() -> &'static str {
        "Forge"
    }

    fn legacy_persistent_names() -> &'static [&'static str] {
        &["DxForgePanel", "DX Forge"]
    }

    fn panel_key() -> &'static str {
        DX_FORGE_PANEL_KEY
    }

    fn position(&self, _: &Window, _: &App) -> DockPosition {
        DockPosition::Left
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        position == DockPosition::Left
    }

    fn set_position(&mut self, _: DockPosition, _: &mut Window, _: &mut Context<Self>) {}

    fn default_size(&self, _: &Window, _: &App) -> gpui::Pixels {
        DEFAULT_PANEL_WIDTH
    }

    fn min_size(&self, _: &Window, _: &App) -> Option<gpui::Pixels> {
        Some(MIN_PANEL_WIDTH)
    }

    fn icon(&self, _: &Window, _: &App) -> Option<IconName> {
        Some(dx_icon(DxUiIcon::Forge))
    }

    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some("Forge This is it??")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        TogglePanel.boxed_clone()
    }

    fn starts_open(&self, _: &Window, _: &App) -> bool {
        false
    }

    fn activation_priority(&self) -> u32 {
        4
    }
}

impl Render for DxForgePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace_roots = self.workspace_roots(cx);
        let snapshot = snapshot::forge_panel_snapshot(&workspace_roots);
        let visible_rows = visible_rows_for_tab(&snapshot, self.active_tab);
        self.sync_visible_rows(visible_rows);

        v_flex()
            .id("dx-forge-panel-action-root")
            .size_full()
            .key_context(self.dispatch_context())
            .track_focus(&self.focus_handle)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(Self::focus_panel_on_mouse_down),
            )
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::toggle_active_item_checked))
            .child(panel_view::render_panel(
                &snapshot,
                &self.workspace,
                &cx.entity().downgrade(),
                cx.entity().entity_id(),
                self.active_tab,
                &self.scroll_handle,
                window,
                cx,
            ))
    }
}
