use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString, Window,
};
use ui::{Icon, prelude::*};
use workspace::{
    Item, Workspace,
    item::{ItemEvent, WorkspaceScreenKind},
};
use zed_actions::assistant::OpenAutomations;

use crate::AgentPanel;

pub struct AutomationScreen {
    panel: Entity<AgentPanel>,
}

impl AutomationScreen {
    pub(crate) fn new(workspace: &Workspace, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel = cx.new(|cx| AgentPanel::new_automation_workspace(workspace, window, cx));
        Self { panel }
    }

    pub(crate) fn open_or_focus(
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        workspace.dismiss_zoomed_agent_panel(window, cx);

        let existing_item = workspace
            .pane_for_screen_kind(WorkspaceScreenKind::Automations, cx)
            .and_then(|pane| {
                pane.read(cx).items().find_map(|item| {
                    (item.screen_kind(cx) == WorkspaceScreenKind::Automations)
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
        _: &OpenAutomations,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        Self::open_or_focus(workspace, window, cx);
    }
}

impl EventEmitter<ItemEvent> for AutomationScreen {}

impl Focusable for AutomationScreen {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.panel.read(cx).focus_handle(cx)
    }
}

impl Item for AutomationScreen {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Automations".into()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(dx_icon(DxUiIcon::Automations)))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Automation Screen Opened")
    }

    fn screen_kind(&self) -> WorkspaceScreenKind {
        WorkspaceScreenKind::Automations
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn can_split(&self) -> bool {
        false
    }
}

impl Render for AutomationScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.panel.clone())
    }
}
