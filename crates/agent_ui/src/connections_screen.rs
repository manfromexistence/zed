use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString, Window,
};
use ui::{Icon, prelude::*};
use workspace::{
    Item, Workspace,
    item::{ItemEvent, WorkspaceScreenKind},
};
use zed_actions::assistant::OpenConnections;

use crate::AgentPanel;

pub struct ConnectionsScreen {
    panel: Entity<AgentPanel>,
}

impl ConnectionsScreen {
    pub(crate) fn new(workspace: &Workspace, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel = cx.new(|cx| AgentPanel::new_connections_workspace(workspace, window, cx));
        Self { panel }
    }

    pub(crate) fn open_or_focus(
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        workspace.dismiss_zoomed_agent_panel(window, cx);

        let existing_item = workspace
            .pane_for_screen_kind(WorkspaceScreenKind::Connections, cx)
            .and_then(|pane| {
                pane.read(cx).items().find_map(|item| {
                    (item.screen_kind(cx) == WorkspaceScreenKind::Connections)
                        .then(|| item.boxed_clone())
                })
            });
        if let Some(item) = existing_item {
            workspace.activate_item(&*item, true, true, window, cx);
            return;
        }

        let target_pane = workspace.screen_host_pane();
        let item = cx.new(|cx| Self::new(workspace, window, cx));
        workspace.add_item(target_pane, Box::new(item), None, true, true, window, cx);
    }

    pub fn open(
        workspace: &mut Workspace,
        _: &OpenConnections,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        Self::open_or_focus(workspace, window, cx);
    }
}

impl EventEmitter<ItemEvent> for ConnectionsScreen {}

impl Focusable for ConnectionsScreen {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.panel.read(cx).focus_handle(cx)
    }
}

impl Item for ConnectionsScreen {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Connections".into()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(dx_icon(DxUiIcon::Connections)))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Connections Screen Opened")
    }

    fn screen_kind(&self) -> WorkspaceScreenKind {
        WorkspaceScreenKind::Connections
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn can_split(&self) -> bool {
        false
    }
}

impl Render for ConnectionsScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panel.clone())
    }
}
