use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString, Window,
};
use ui::{Icon, IconName, prelude::*};
use workspace::{
    Item, Workspace,
    item::{ItemEvent, WorkspaceScreenKind},
};

use crate::AgentPanel;

pub struct AgentScreen {
    panel: Entity<AgentPanel>,
}

impl AgentScreen {
    pub(crate) fn new(workspace: &Workspace, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel = cx.new(|cx| AgentPanel::new_builder_workspace(workspace, window, cx));
        Self { panel }
    }

    pub(crate) fn open_or_focus(
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        workspace.dismiss_zoomed_agent_panel(window, cx);

        if let Some(pane) = workspace.pane_for_screen_kind(WorkspaceScreenKind::Agent, cx)
            && let Some(item) = pane.read(cx).items().find_map(|item| {
                (item.screen_kind(cx) == WorkspaceScreenKind::Agent).then(|| item.boxed_clone())
            })
        {
            workspace.activate_item(&*item, true, true, window, cx);
            return;
        }

        let target_pane = workspace.screen_host_pane();
        let item = cx.new(|cx| Self::new(workspace, window, cx));
        workspace.add_item(target_pane, Box::new(item), None, true, true, window, cx);
    }
}

impl EventEmitter<ItemEvent> for AgentScreen {}

impl Focusable for AgentScreen {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.panel.read(cx).focus_handle(cx)
    }
}

impl Item for AgentScreen {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "AI".into()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ZedAgent))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Agent Screen Opened")
    }

    fn screen_kind(&self) -> WorkspaceScreenKind {
        WorkspaceScreenKind::Agent
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn can_split(&self) -> bool {
        false
    }
}

impl Render for AgentScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panel.clone())
    }
}
