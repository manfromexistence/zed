#![allow(dead_code)]

use crate::multibuffer_hint::MultibufferHint;
use client::{Client, UserStore, zed_urls};
use cloud_api_types::Plan;
use db::kvp::KeyValueStore;
use editor::Editor;
use fs::Fs;
use gpui::{
    Action, AnyElement, App, AppContext, AsyncWindowContext, Context, Entity, EventEmitter,
    FocusHandle, Focusable, Global, IntoElement, KeyContext, Render, ScrollHandle, SharedString,
    Subscription, Task, TaskExt as _, WeakEntity, Window, actions,
};
use notifications::status_toast::StatusToast;
use project::agent_server_store::AllAgentServersSettings;
use schemars::JsonSchema;
use serde::Deserialize;
use settings::{SettingsStore, VsCodeSettingsSource};
use std::sync::Arc;
use ui::{ParentElement as _, prelude::*};

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use web_preview::web_preview_view::WebPreviewView;
pub use workspace::welcome::ShowWelcome;
use workspace::welcome::WelcomePage;
use workspace::{
    AppState, Workspace, WorkspaceId, ZoomIn, ZoomOut,
    dock::DockPosition,
    item::{Item, ItemEvent, ItemHandle, WorkspaceScreenKind},
    notifications::NotifyResultExt as _,
    open_new, register_serializable_item, with_active_or_new_workspace,
};
use zed_actions::OpenOnboarding;

mod base_keymap_picker;
mod basics_page;
mod dx_launch_onboarding;
pub mod multibuffer_hint;
mod theme_preview;

use dx_launch_onboarding::DxLaunchPreviewTargets;

/// Imports settings from Visual Studio Code.
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct ImportVsCodeSettings {
    #[serde(default)]
    pub skip_prompt: bool,
}

/// Imports settings from Cursor editor.
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct ImportCursorSettings {
    #[serde(default)]
    pub skip_prompt: bool,
}

pub const FIRST_OPEN: &str = "first_open";

// TODO(dx-onboarding): Re-enable the fullscreen WebPreview onboarding after the
// completion handoff is rebuilt so closing the page cannot close or hang Zed.
const DX_WEB_PREVIEW_ONBOARDING_DISABLED: bool = true;

const WEB_PREVIEW_ONBOARDING_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Onboarding</title>
<style>
  :root {
    color-scheme: dark light;
    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: #101214;
    color: #f4f0e8;
  }
  * { box-sizing: border-box; }
  body {
    min-height: 100vh;
    margin: 0;
    display: grid;
    place-items: center;
    background: linear-gradient(135deg, #101214 0%, #17201d 52%, #241a17 100%);
  }
  main {
    width: min(100%, 720px);
    padding: clamp(32px, 7vw, 72px);
    display: grid;
    gap: 28px;
    justify-items: center;
    text-align: center;
  }
  .mark {
    width: 64px;
    height: 64px;
    display: grid;
    place-items: center;
    border: 1px solid rgba(244, 240, 232, 0.18);
    border-radius: 8px;
    background: rgba(244, 240, 232, 0.08);
    font-size: 18px;
    font-weight: 700;
    letter-spacing: 0;
  }
  h1 {
    margin: 0;
    max-width: 560px;
    font-size: clamp(36px, 7vw, 72px);
    line-height: 0.95;
    font-weight: 760;
    letter-spacing: 0;
  }
  button {
    appearance: none;
    border: 0;
    border-radius: 8px;
    padding: 14px 22px;
    min-width: 136px;
    background: #e7c06a;
    color: #171411;
    font: inherit;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: 0;
    cursor: pointer;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.32);
  }
  button:hover { background: #f1cd7d; }
  button:focus-visible {
    outline: 3px solid rgba(231, 192, 106, 0.38);
    outline-offset: 4px;
  }
  .status {
    min-height: 20px;
    color: rgba(244, 240, 232, 0.72);
    font-size: 13px;
  }
</style>
</head>
<body>
  <main>
    <div class="mark" aria-hidden="true">DX</div>
    <h1>Ready when you are.</h1>
    <button id="complete" type="button">Complete</button>
    <div id="status" class="status" aria-live="polite"></div>
  </main>
  <script>
    (() => {
      const button = document.getElementById("complete");
      const status = document.getElementById("status");
      const preventNativeBridgeFallback = (event) => {
        event.preventDefault();
        event.stopPropagation();
        status.textContent = "Use the Complete control above this preview.";
      };
      button.addEventListener("click", preventNativeBridgeFallback);
    })();
  </script>
</body>
</html>"##;

fn web_preview_onboarding_url() -> String {
    format!(
        "data:text/html;charset=utf-8,{}",
        percent_encode_data_url(WEB_PREVIEW_ONBOARDING_HTML)
    )
}

fn percent_encode_data_url(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(*byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    encoded
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + value - 10) as char,
        _ => unreachable!("hex digit nibble must be in range"),
    }
}

actions!(
    onboarding,
    [
        /// Finish the onboarding process.
        Finish,
        /// Sign in while in the onboarding flow.
        SignIn,
        /// Open the user account in zed.dev while in the onboarding flow.
        OpenAccount,
        /// Resets the welcome screen hints to their initial state.
        ResetHints
    ]
);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _cx| {
        workspace
            .register_action(|_workspace, _: &ResetHints, _, cx| MultibufferHint::set_count(0, cx));
    })
    .detach();

    cx.on_action(|_: &OpenOnboarding, cx| {
        if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
            return;
        }

        with_active_or_new_workspace(cx, |workspace, window, cx| {
            workspace
                .with_local_workspace(window, cx, |workspace, window, cx| {
                    open_onboarding_page(workspace, window, cx);
                })
                .detach();
        });
    });

    cx.on_action(|_: &ShowWelcome, cx| {
        with_active_or_new_workspace(cx, |workspace, window, cx| {
            workspace
                .with_local_workspace(window, cx, |workspace, window, cx| {
                    let existing = workspace
                        .active_pane()
                        .read(cx)
                        .items()
                        .find_map(|item| item.downcast::<WelcomePage>());

                    if let Some(existing) = existing {
                        workspace.activate_item(&existing, true, true, window, cx);
                    } else {
                        let settings_page = cx
                            .new(|cx| WelcomePage::new(workspace.weak_handle(), false, window, cx));
                        workspace.add_item_to_active_pane(
                            Box::new(settings_page),
                            None,
                            true,
                            window,
                            cx,
                        )
                    }
                })
                .detach();
        });
    });

    cx.observe_new(|workspace: &mut Workspace, _window, _cx| {
        workspace.register_action(|_workspace, action: &ImportVsCodeSettings, window, cx| {
            let fs = <dyn Fs>::global(cx);
            let action = *action;

            let workspace = cx.weak_entity();

            window
                .spawn(cx, async move |cx: &mut AsyncWindowContext| {
                    handle_import_vscode_settings(
                        workspace,
                        VsCodeSettingsSource::VsCode,
                        action.skip_prompt,
                        fs,
                        cx,
                    )
                    .await
                })
                .detach();
        });

        workspace.register_action(|_workspace, action: &ImportCursorSettings, window, cx| {
            let fs = <dyn Fs>::global(cx);
            let action = *action;

            let workspace = cx.weak_entity();

            window
                .spawn(cx, async move |cx: &mut AsyncWindowContext| {
                    handle_import_vscode_settings(
                        workspace,
                        VsCodeSettingsSource::Cursor,
                        action.skip_prompt,
                        fs,
                        cx,
                    )
                    .await
                })
                .detach();
        });
    })
    .detach();

    base_keymap_picker::init(cx);

    // TODO(dx-onboarding): Re-enable serialization only after the fullscreen
    // WebPreview onboarding flow is restored and completion is runtime-proven.
    // register_serializable_item::<Onboarding>(cx);
    register_serializable_item::<WelcomePage>(cx);
}

pub fn show_onboarding_view(app_state: Arc<AppState>, cx: &mut App) -> Task<anyhow::Result<()>> {
    if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
        return open_new(
            Default::default(),
            app_state,
            cx,
            |workspace, window, cx| {
                Editor::new_file(workspace, &Default::default(), window, cx);
                let kvp = KeyValueStore::global(cx);
                db::write_and_log(cx, move || async move {
                    kvp.write_kvp(FIRST_OPEN.to_string(), "false".to_string())
                        .await
                });
            },
        );
    }

    telemetry::event!("Onboarding Page Opened");
    open_new(
        Default::default(),
        app_state,
        cx,
        |workspace, window, cx| {
            {
                open_onboarding_page(workspace, window, cx);
            };
            let kvp = KeyValueStore::global(cx);
            db::write_and_log(cx, move || async move {
                kvp.write_kvp(FIRST_OPEN.to_string(), "false".to_string())
                    .await
            });
        },
    )
}

fn open_onboarding_page(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let closed_docks_for_fullscreen = close_open_docks_for_onboarding(workspace, window, cx);

    let existing = find_onboarding_page(workspace, cx);

    if let Some(existing) = existing {
        existing.update(cx, |onboarding, cx| {
            onboarding.track_closed_docks_for_fullscreen(closed_docks_for_fullscreen);
            cx.emit(ItemEvent::UpdateTab);
        });
        workspace.activate_item(&existing, true, true, window, cx);
        window.focus(&existing.focus_handle(cx), cx);
        zoom_active_onboarding_pane(workspace, window, cx);
        cx.notify();
        return;
    }

    let onboarding_page = Onboarding::new(workspace, cx);
    onboarding_page.update(cx, |onboarding, _| {
        onboarding.track_closed_docks_for_fullscreen(closed_docks_for_fullscreen);
    });
    workspace.add_item_to_center(Box::new(onboarding_page.clone()), window, cx);
    onboarding_page.update(cx, |_, cx| cx.emit(ItemEvent::UpdateTab));
    workspace.activate_item(&onboarding_page, true, true, window, cx);
    window.focus(&onboarding_page.focus_handle(cx), cx);
    zoom_active_onboarding_pane(workspace, window, cx);
    cx.notify();
}

fn find_onboarding_page(workspace: &Workspace, cx: &App) -> Option<Entity<Onboarding>> {
    workspace.panes().iter().find_map(|pane| {
        pane.read(cx)
            .items()
            .find_map(|item| item.downcast::<Onboarding>())
    })
}

fn close_open_docks_for_onboarding(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Vec<DockPosition> {
    let mut closed_docks = Vec::new();

    for dock_position in [
        DockPosition::Left,
        DockPosition::Right,
        DockPosition::Bottom,
    ] {
        if workspace.is_dock_at_position_open(dock_position, cx) {
            workspace.toggle_dock(dock_position, window, cx);
            closed_docks.push(dock_position);
        }
    }

    closed_docks
}

fn serialize_closed_docks_for_fullscreen(positions: &[DockPosition]) -> String {
    positions
        .iter()
        .map(|position| dock_position_token(*position))
        .collect::<Vec<_>>()
        .join(",")
}

fn deserialize_closed_docks_for_fullscreen(value: &str) -> Vec<DockPosition> {
    value.split(',').filter_map(dock_position_from_token).fold(
        Vec::new(),
        |mut positions, position| {
            if !positions.contains(&position) {
                positions.push(position);
            }
            positions
        },
    )
}

fn dock_position_token(position: DockPosition) -> &'static str {
    match position {
        DockPosition::Left => "left",
        DockPosition::Bottom => "bottom",
        DockPosition::Right => "right",
    }
}

fn dock_position_from_token(token: &str) -> Option<DockPosition> {
    match token {
        "left" => Some(DockPosition::Left),
        "bottom" => Some(DockPosition::Bottom),
        "right" => Some(DockPosition::Right),
        _ => None,
    }
}

fn zoom_active_onboarding_pane(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    workspace
        .active_pane()
        .update(cx, |pane, cx| pane.zoom_in(&ZoomIn, window, cx));
}

struct Onboarding {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    user_store: Entity<UserStore>,
    scroll_handle: ScrollHandle,
    dx_preview_targets: DxLaunchPreviewTargets,
    closed_docks_for_fullscreen: Vec<DockPosition>,
    completion_requested: bool,
    completion_revealed: bool,
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    dx_web_preview: Option<Entity<WebPreviewView>>,
    _settings_subscription: Subscription,
}

impl Onboarding {
    fn new(workspace: &Workspace, cx: &mut App) -> Entity<Self> {
        let font_family_cache = theme::FontFamilyCache::global(cx);

        let installed_agents = cx
            .global::<SettingsStore>()
            .get::<AllAgentServersSettings>(None)
            .clone();
        let client = Client::global(cx);
        let status = *client.status().borrow();
        let plan = workspace.user_store().read(cx).plan();
        let zed_agent_state = if status.is_signed_out()
            || matches!(
                status,
                client::Status::AuthenticationError | client::Status::ConnectionError
            ) {
            "signed_out"
        } else if status.is_signing_in() {
            "signing_in"
        } else {
            match plan {
                Some(Plan::ZedPro) => "pro",
                Some(Plan::ZedProTrial) => "trial",
                Some(Plan::ZedBusiness) => "business",
                Some(Plan::ZedStudent) => "student",
                Some(Plan::ZedFree) | None => "free",
            }
        };
        let agents_installed = basics_page::FEATURED_AGENT_IDS
            .iter()
            .filter(|id| installed_agents.contains_key(**id))
            .copied()
            .collect::<Vec<_>>();
        telemetry::event!(
            "Welcome Agent Setup Viewed",
            zed_agent = zed_agent_state,
            agents_installed = agents_installed,
        );
        let dx_preview_targets =
            DxLaunchPreviewTargets::local_web_preview_onboarding(web_preview_onboarding_url());

        cx.new(|cx| {
            cx.spawn(async move |this, cx| {
                font_family_cache.prefetch(cx).await;
                this.update(cx, |_, cx| {
                    cx.notify();
                })
            })
            .detach();

            Self {
                workspace: workspace.weak_handle(),
                focus_handle: cx.focus_handle(),
                scroll_handle: ScrollHandle::new(),
                user_store: workspace.user_store().clone(),
                dx_preview_targets,
                closed_docks_for_fullscreen: Vec::new(),
                completion_requested: false,
                completion_revealed: false,
                #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                dx_web_preview: None,
                _settings_subscription: cx
                    .observe_global::<SettingsStore>(move |_, cx| cx.notify()),
            }
        })
    }

    fn handle_finish(&mut self, _: &Finish, window: &mut Window, cx: &mut Context<Self>) {
        if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
            return;
        }

        finish_setup(self.workspace.clone(), window, cx);
    }

    fn handle_sign_in(&mut self, _: &SignIn, window: &mut Window, cx: &mut Context<Self>) {
        let client = Client::global(cx);
        let workspace = self.workspace.clone();

        window
            .spawn(cx, async move |mut cx| {
                client
                    .sign_in_with_optional_connect(true, &cx)
                    .await
                    .notify_workspace_async_err(workspace, &mut cx);
            })
            .detach();
    }
}

fn finish_setup<C: AppContext>(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut C) {
    if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
        return;
    }

    telemetry::event!("Finish Setup");
    close_onboarding_page(workspace, window, cx);
}

impl Onboarding {
    fn track_closed_docks_for_fullscreen(&mut self, positions: Vec<DockPosition>) {
        for position in positions {
            if !self.closed_docks_for_fullscreen.contains(&position) {
                self.closed_docks_for_fullscreen.push(position);
            }
        }
    }

    fn take_closed_docks_for_fullscreen(&mut self) -> Vec<DockPosition> {
        std::mem::take(&mut self.closed_docks_for_fullscreen)
    }

    fn handle_open_account(_: &OpenAccount, _: &mut Window, cx: &mut App) {
        cx.open_url(&zed_urls::account_url(cx))
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn ensure_dx_web_preview(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<WebPreviewView> {
        if let Some(preview) = self.dx_web_preview.clone() {
            return preview;
        }

        let workspace = self.workspace.clone();
        let url = self.dx_preview_targets.primary.url.clone();
        let preview = cx.new(|cx| {
            WebPreviewView::new_for_onboarding(
                workspace,
                url,
                Some("Onboarding".into()),
                None,
                window,
                cx,
            )
        });
        self.dx_web_preview = Some(preview.clone());
        preview
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn deactivate_dx_web_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(preview) = self.dx_web_preview.as_ref() {
            preview.update(cx, |preview, cx| preview.deactivated(window, cx));
        }
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn prepare_dx_web_preview_for_completion(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(preview) = self.dx_web_preview.as_ref() {
            preview.update(cx, |preview, cx| {
                preview.prepare_for_onboarding_completion(window, cx)
            });
        }
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn render_web_preview_canvas(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preview = self.ensure_dx_web_preview(window, cx);
        div().size_full().child(preview).into_any_element()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn render_web_preview_canvas(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().colors().surface_background)
            .child(
                Label::new("DX onboarding Web Preview is available on supported desktop runtimes.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .into_any_element()
    }
}

impl Render for Onboarding {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .image_cache(gpui::retain_all("onboarding-page"))
            .key_context({
                let mut ctx = KeyContext::new_with_defaults();
                ctx.add("Onboarding");
                ctx.add("menu");
                ctx
            })
            .track_focus(&self.focus_handle)
            .size_full()
            .on_action(cx.listener(Self::handle_finish))
            .on_action(cx.listener(Self::handle_sign_in))
            .on_action(Self::handle_open_account)
            .on_action(cx.listener(|_, _: &menu::SelectNext, window, cx| {
                window.focus_next(cx);
                cx.notify();
            }))
            .on_action(cx.listener(|_, _: &menu::SelectPrevious, window, cx| {
                window.focus_prev(cx);
                cx.notify();
            }))
    }
}

impl EventEmitter<ItemEvent> for Onboarding {}

impl Focusable for Onboarding {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for Onboarding {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Onboarding".into()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Onboarding Page Opened")
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn can_split(&self) -> bool {
        false
    }

    fn screen_kind(&self) -> WorkspaceScreenKind {
        WorkspaceScreenKind::Onboarding
    }

    fn workspace_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
            return None;
        }

        if self.completion_revealed {
            return None;
        }

        Some(
            div()
                .id("onboarding-window-overlay")
                .absolute()
                .inset_0()
                .size_full()
                .occlude()
                .track_focus(&self.focus_handle)
                .on_action(cx.listener(Self::handle_finish))
                .on_action(cx.listener(Self::handle_sign_in))
                .on_action(Self::handle_open_account)
                .child(self.render_web_preview_canvas(window, cx))
                .into_any_element(),
        )
    }

    fn requires_transparent_workspace_background() -> bool {
        true
    }

    fn deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            if self.completion_requested {
                window.set_background_appearance(gpui::WindowBackgroundAppearance::Opaque);
            } else {
                self.deactivate_dx_web_preview(window, cx);
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let _ = (window, cx);
    }

    fn workspace_deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            if self.completion_requested {
                window.set_background_appearance(gpui::WindowBackgroundAppearance::Opaque);
            } else {
                self.deactivate_dx_web_preview(window, cx);
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let _ = (window, cx);
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>> {
        Task::ready(Some(cx.new(|cx| Onboarding {
            workspace: self.workspace.clone(),
            user_store: self.user_store.clone(),
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            dx_preview_targets: self.dx_preview_targets.clone(),
            closed_docks_for_fullscreen: Vec::new(),
            completion_requested: false,
            completion_revealed: false,
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            dx_web_preview: None,
            _settings_subscription: cx.observe_global::<SettingsStore>(move |_, cx| cx.notify()),
        })))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(workspace::item::ItemEvent)) {
        f(*event)
    }
}

fn close_onboarding_page<C: AppContext>(
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut C,
) {
    if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
        return;
    }

    let _ = workspace.update(cx, |workspace, cx| {
        if !mark_onboarding_completion_requested(workspace, cx) {
            return;
        }

        complete_onboarding_handoff(workspace, window, cx);
    });
}

fn complete_onboarding_handoff(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if find_onboarding_page(workspace, cx).is_none() {
        return;
    }

    reveal_onboarding_completion(workspace, window, cx);

    let post_onboarding_item = find_post_onboarding_item(workspace, cx);

    if let Some(item) = post_onboarding_item.as_ref() {
        workspace.activate_item(item.as_ref(), true, true, window, cx);
    } else {
        let create_editor = Editor::new_in_workspace(workspace, window, cx);
        cx.spawn_in(window, async move |workspace, cx| {
            match create_editor.await {
                Ok(_) => Ok::<(), anyhow::Error>(()),
                Err(error) => {
                    workspace.update_in(cx, |workspace, _window, cx| {
                        reset_onboarding_completion_request(workspace, cx);
                    })?;
                    Err(error)
                }
            }
        })
        .detach_and_log_err(cx);
    }
}

fn mark_onboarding_completion_requested(workspace: &mut Workspace, cx: &mut App) -> bool {
    let onboarding_pages = workspace
        .panes()
        .iter()
        .flat_map(|pane| {
            pane.read(cx)
                .items()
                .filter_map(|item| item.downcast::<Onboarding>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut newly_requested = false;
    for onboarding in onboarding_pages {
        onboarding.update(cx, |onboarding, _| {
            if !onboarding.completion_requested {
                onboarding.completion_requested = true;
                newly_requested = true;
            }
        });
    }

    newly_requested
}

fn reset_onboarding_completion_request(workspace: &mut Workspace, cx: &mut App) {
    let onboarding_pages = workspace
        .panes()
        .iter()
        .flat_map(|pane| {
            pane.read(cx)
                .items()
                .filter_map(|item| item.downcast::<Onboarding>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for onboarding in onboarding_pages {
        onboarding.update(cx, |onboarding, cx| {
            onboarding.completion_requested = false;
            onboarding.completion_revealed = false;
            cx.notify();
        });
    }
}

fn reveal_onboarding_completion(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    for pane in workspace.panes().to_vec() {
        let contains_onboarding = pane
            .read(cx)
            .items()
            .any(|item| item.downcast::<Onboarding>().is_some());
        if contains_onboarding {
            pane.update(cx, |pane, cx| {
                pane.zoom_out(&ZoomOut, window, cx);
            });
        }
    }

    let onboarding_pages = workspace
        .panes()
        .iter()
        .flat_map(|pane| {
            pane.read(cx)
                .items()
                .filter_map(|item| Some((item.downcast::<Onboarding>()?, item.item_id())))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut dock_positions_to_restore = Vec::new();
    let mut completed_item_ids = Vec::new();
    let workspace_id = workspace.database_id();

    for (onboarding, item_id) in onboarding_pages {
        let (positions, completed) = onboarding.update(cx, |onboarding, cx| {
            let mut completed = false;
            if onboarding.completion_requested && !onboarding.completion_revealed {
                #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                onboarding.prepare_dx_web_preview_for_completion(window, cx);
                onboarding.completion_revealed = true;
                completed = true;
                cx.notify();
            }

            (onboarding.take_closed_docks_for_fullscreen(), completed)
        });
        if completed {
            completed_item_ids.push(item_id);
        }

        for position in positions {
            if !dock_positions_to_restore.contains(&position) {
                dock_positions_to_restore.push(position);
            }
        }
    }

    if let Some(workspace_id) = workspace_id {
        for item_id in completed_item_ids {
            forget_completed_onboarding_page(item_id.as_u64(), workspace_id, cx);
        }
    }

    for position in dock_positions_to_restore {
        if !workspace.is_dock_at_position_open(position, cx) {
            workspace.toggle_dock(position, window, cx);
        }
    }

    cx.notify();
}

fn forget_completed_onboarding_page(
    item_id: workspace::ItemId,
    workspace_id: WorkspaceId,
    cx: &mut Context<Workspace>,
) {
    let db = persistence::OnboardingPagesDb::global(cx);
    cx.background_spawn(async move { db.delete_onboarding_page(item_id, workspace_id).await })
        .detach_and_log_err(cx);
}

fn find_post_onboarding_item(workspace: &Workspace, cx: &App) -> Option<Box<dyn ItemHandle>> {
    let mut fallback = None;

    for pane in workspace.panes() {
        for item in pane.read(cx).items() {
            if item.downcast::<Onboarding>().is_some() {
                continue;
            }

            match item.screen_kind(cx) {
                WorkspaceScreenKind::Editor => return Some(item.boxed_clone()),
                WorkspaceScreenKind::Agent
                | WorkspaceScreenKind::Automations
                | WorkspaceScreenKind::Browser
                | WorkspaceScreenKind::Terminal
                | WorkspaceScreenKind::LiquidGlass
                | WorkspaceScreenKind::Other => {
                    fallback.get_or_insert_with(|| item.boxed_clone());
                }
                WorkspaceScreenKind::Onboarding => {}
            }
        }
    }

    fallback
}

pub async fn handle_import_vscode_settings(
    workspace: WeakEntity<Workspace>,
    source: VsCodeSettingsSource,
    skip_prompt: bool,
    fs: Arc<dyn Fs>,
    cx: &mut AsyncWindowContext,
) {
    use util::truncate_and_remove_front;

    let vscode_settings =
        match settings::VsCodeSettings::load_user_settings(source, fs.clone()).await {
            Ok(vscode_settings) => vscode_settings,
            Err(err) => {
                zlog::error!("{err:?}");
                let _ = cx.prompt(
                    gpui::PromptLevel::Info,
                    &format!("Could not find or load a {source} settings file"),
                    None,
                    &["Ok"],
                );
                return;
            }
        };

    if !skip_prompt {
        let prompt = cx.prompt(
            gpui::PromptLevel::Warning,
            &format!(
                "Importing {} settings may overwrite your existing settings. \
                Will import settings from {}",
                vscode_settings.source,
                truncate_and_remove_front(&vscode_settings.path.to_string_lossy(), 128),
            ),
            None,
            &["Ok", "Cancel"],
        );
        let result = cx.spawn(async move |_| prompt.await.ok()).await;
        if result != Some(0) {
            return;
        }
    };

    let Ok(result_channel) = cx.update(|_, cx| {
        let source = vscode_settings.source;
        let path = vscode_settings.path.clone();
        let result_channel = cx
            .global::<SettingsStore>()
            .import_vscode_settings(fs, vscode_settings);
        zlog::info!("Imported {source} settings from {}", path.display());
        result_channel
    }) else {
        return;
    };

    let result = result_channel.await;
    workspace
        .update_in(cx, |workspace, _, cx| match result {
            Ok(_) => {
                let confirmation_toast = StatusToast::new(
                    format!("Your {} settings were successfully imported.", source),
                    cx,
                    |this, _| {
                        this.icon(
                            Icon::new(IconName::Check)
                                .size(IconSize::Small)
                                .color(Color::Success),
                        )
                        .dismiss_button(true)
                    },
                );
                SettingsImportState::update(cx, |state, _| match source {
                    VsCodeSettingsSource::VsCode => {
                        state.vscode = true;
                    }
                    VsCodeSettingsSource::Cursor => {
                        state.cursor = true;
                    }
                });
                workspace.toggle_status_toast(confirmation_toast, cx);
            }
            Err(_) => {
                let error_toast = StatusToast::new(
                    "Failed to import settings. See log for details",
                    cx,
                    |this, _| {
                        this.icon(
                            Icon::new(IconName::Close)
                                .size(IconSize::Small)
                                .color(Color::Error),
                        )
                        .action("Open Log", |window, cx| {
                            window.dispatch_action(workspace::OpenLog.boxed_clone(), cx)
                        })
                        .dismiss_button(true)
                    },
                );
                workspace.toggle_status_toast(error_toast, cx);
            }
        })
        .ok();
}

#[derive(Default, Copy, Clone)]
pub struct SettingsImportState {
    pub cursor: bool,
    pub vscode: bool,
}

impl Global for SettingsImportState {}

impl SettingsImportState {
    pub fn global(cx: &App) -> Self {
        cx.try_global().cloned().unwrap_or_default()
    }
    pub fn update<R>(cx: &mut App, f: impl FnOnce(&mut Self, &mut App) -> R) -> R {
        cx.update_default_global(f)
    }
}

impl workspace::SerializableItem for Onboarding {
    fn serialized_item_kind() -> &'static str {
        "OnboardingPage"
    }

    fn cleanup(
        workspace_id: workspace::WorkspaceId,
        alive_items: Vec<workspace::ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> gpui::Task<gpui::Result<()>> {
        workspace::delete_unloaded_items(
            alive_items,
            workspace_id,
            "onboarding_pages",
            &persistence::OnboardingPagesDb::global(cx),
            cx,
        )
    }

    fn deserialize(
        _project: Entity<project::Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: workspace::WorkspaceId,
        item_id: workspace::ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Task<gpui::Result<Entity<Self>>> {
        let db = persistence::OnboardingPagesDb::global(cx);
        if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
            return window.spawn(cx, async move |_| {
                db.delete_onboarding_page(item_id, workspace_id).await?;
                Err(anyhow::anyhow!("Onboarding Web Preview is disabled"))
            });
        }

        window.spawn(cx, async move |cx| {
            if let Some(closed_docks_for_fullscreen) =
                db.get_onboarding_page(item_id, workspace_id)?
            {
                let onboarding =
                    workspace.update(cx, |workspace, cx| Onboarding::new(workspace, cx))?;
                onboarding.update(cx, |onboarding, _| {
                    onboarding.track_closed_docks_for_fullscreen(
                        deserialize_closed_docks_for_fullscreen(&closed_docks_for_fullscreen),
                    );
                });
                Ok(onboarding)
            } else {
                Err(anyhow::anyhow!("No onboarding page to deserialize"))
            }
        })
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: workspace::ItemId,
        _closing: bool,
        _window: &mut Window,
        cx: &mut ui::Context<Self>,
    ) -> Option<gpui::Task<gpui::Result<()>>> {
        if DX_WEB_PREVIEW_ONBOARDING_DISABLED {
            return None;
        }

        let workspace_id = workspace.database_id()?;
        if self.completion_requested {
            return None;
        }

        let db = persistence::OnboardingPagesDb::global(cx);
        let closed_docks_for_fullscreen =
            serialize_closed_docks_for_fullscreen(&self.closed_docks_for_fullscreen);
        Some(cx.background_spawn(async move {
            db.save_onboarding_page(item_id, workspace_id, closed_docks_for_fullscreen)
                .await
        }))
    }

    fn should_serialize(&self, event: &Self::Event) -> bool {
        !DX_WEB_PREVIEW_ONBOARDING_DISABLED
            && !self.completion_requested
            && event == &ItemEvent::UpdateTab
    }
}

mod persistence {
    use db::{
        query,
        sqlez::{domain::Domain, thread_safe_connection::ThreadSafeConnection},
        sqlez_macros::sql,
    };
    use workspace::WorkspaceDb;

    pub struct OnboardingPagesDb(ThreadSafeConnection);

    impl Domain for OnboardingPagesDb {
        const NAME: &str = stringify!(OnboardingPagesDb);

        const MIGRATIONS: &[&str] = &[
            sql!(
                        CREATE TABLE onboarding_pages (
                            workspace_id INTEGER,
                            item_id INTEGER UNIQUE,
                            page_number INTEGER,

                            PRIMARY KEY(workspace_id, item_id),
                            FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                            ON DELETE CASCADE
                        ) STRICT;
            ),
            sql!(
                        CREATE TABLE onboarding_pages_2 (
                            workspace_id INTEGER,
                            item_id INTEGER UNIQUE,

                            PRIMARY KEY(workspace_id, item_id),
                            FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                            ON DELETE CASCADE
                        ) STRICT;
                        INSERT INTO onboarding_pages_2 SELECT workspace_id, item_id FROM onboarding_pages;
                        DROP TABLE onboarding_pages;
                        ALTER TABLE onboarding_pages_2 RENAME TO onboarding_pages;
            ),
            sql!(
                        ALTER TABLE onboarding_pages
                        ADD COLUMN closed_docks_for_fullscreen TEXT NOT NULL DEFAULT "";
            ),
        ];
    }

    db::static_connection!(OnboardingPagesDb, [WorkspaceDb]);

    impl OnboardingPagesDb {
        query! {
            pub async fn save_onboarding_page(
                item_id: workspace::ItemId,
                workspace_id: workspace::WorkspaceId,
                closed_docks_for_fullscreen: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO onboarding_pages(
                    item_id,
                    workspace_id,
                    closed_docks_for_fullscreen
                )
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub async fn delete_onboarding_page(
                item_id: workspace::ItemId,
                workspace_id: workspace::WorkspaceId
            ) -> Result<()> {
                DELETE FROM onboarding_pages
                WHERE item_id = ? AND workspace_id = ?
            }
        }

        query! {
            pub fn get_onboarding_page(
                item_id: workspace::ItemId,
                workspace_id: workspace::WorkspaceId
            ) -> Result<Option<String>> {
                SELECT closed_docks_for_fullscreen
                FROM onboarding_pages
                WHERE item_id = ? AND workspace_id = ?
            }
        }
    }
}
