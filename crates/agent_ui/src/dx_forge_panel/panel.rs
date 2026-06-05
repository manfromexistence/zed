use crate::dx_receipt_history::invalidate_tool_history_snapshot_cache;
use crate::dx_source_sets::invalidate_source_set_snapshot_cache;
use gpui::{
    Action, App, AppContext, Context, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    ScrollHandle, WeakEntity, Window, px,
};
use ui::IconName;
use workspace::{
    Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};
use zed_actions::dx_forge::TogglePanel;

use super::{panel_view, snapshot};

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
}

impl DxForgePanel {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
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

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        invalidate_tool_history_snapshot_cache();
        invalidate_source_set_snapshot_cache();
        cx.notify();
    }
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
        Some(IconName::Forgejo)
    }

    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some("Forge")
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

        panel_view::render_panel(
            &snapshot,
            &self.workspace,
            &cx.entity().downgrade(),
            cx.entity().entity_id(),
            &self.scroll_handle,
            window,
            cx,
        )
    }
}
