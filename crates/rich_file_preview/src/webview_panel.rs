use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use agent_client_protocol as acp;
use agent_ui::{AgentPanel, NewTextThread};
use anyhow::Context as _;
use editor::Editor;
use gpui::{
    Action, App, Context, EventEmitter, FocusHandle, Focusable, Pixels, Render, Task, WeakEntity,
    Window, actions, canvas, px,
};
use ui::{Button, Color, IconButton, IconName, Label, prelude::*};
use workspace::dock::{DockPosition, PanelEvent};
use workspace::{Panel, Workspace};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::http::Request;
use wry::{Rect, WebContext, WebView, WebViewBuilder};

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use wry::WebViewBuilderExtUnix;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use crate::browser_extensions::{BrowserProfile, detect_browser_profiles, preferred_profile_index};
use crate::web_inspector::{
    CapturedElement, INIT_SCRIPT, WebIpcMessage, apply_css_script, arm_inspector_script,
    cancel_inspector_script, clear_css_script,
};
use crate::webview_dev_session::{DevSessionMode, DevSessionPolicy};

const DEFAULT_URL: &str = "https://zed.dev";

actions!(
    rich_file_preview,
    [
        ToggleEmbeddedWebPreview,
        OpenEmbeddedWebPreviewDevTools,
        PickEmbeddedWebElement,
        InspectEmbeddedWebElementToAi,
        CopyEmbeddedWebElementToAi,
        RefreshEmbeddedWebBrowserProfiles,
        ClearEmbeddedWebSession,
        CycleEmbeddedWebSessionMode,
        ReloadEmbeddedWebPreview,
        ApplyEmbeddedWebCss,
        ClearEmbeddedWebCss
    ]
);

#[derive(Clone, Debug, PartialEq, Eq)]
struct WebViewLaunch {
    data_directory: std::path::PathBuf,
    incognito: bool,
    visible: bool,
    extensions_path: Option<std::path::PathBuf>,
}

#[derive(Default)]
struct WebViewHost {
    webview: Option<WebView>,
    web_context: Option<WebContext>,
    launch: Option<WebViewLaunch>,
}

impl WebViewHost {
    fn ensure(
        &mut self,
        window: &Window,
        launch: &WebViewLaunch,
        initial_url: &str,
        ipc_tx: Sender<String>,
    ) -> anyhow::Result<bool> {
        if self.webview.is_some() && self.launch.as_ref() == Some(launch) {
            return Ok(false);
        }

        std::fs::create_dir_all(&launch.data_directory).with_context(|| {
            format!(
                "creating embedded preview data directory {}",
                launch.data_directory.display()
            )
        })?;

        self.webview = None;
        self.web_context = None;

        let mut web_context = WebContext::new(Some(launch.data_directory.clone()));
        let mut builder = WebViewBuilder::new_with_web_context(&mut web_context)
            .with_visible(launch.visible)
            .with_incognito(launch.incognito)
            .with_devtools(true)
            .with_autoplay(true)
            .with_clipboard(true)
            .with_url(initial_url)
            .with_initialization_script(INIT_SCRIPT)
            .with_ipc_handler(move |request: Request<String>| {
                let _ = ipc_tx.send(request.body().clone());
            });

        #[cfg(target_os = "windows")]
        {
            let browser_args = "--enable-features=Vulkan,UseSkiaRenderer,WebGPUServiceInProcess --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";
            builder = builder.with_additional_browser_args(browser_args);
            if let Some(extensions_path) = launch.extensions_path.clone() {
                builder = builder
                    .with_browser_extensions_enabled(true)
                    .with_extensions_path(extensions_path);
            } else {
                builder = builder.with_browser_extensions_enabled(false);
            }
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            if let Some(extensions_path) = launch.extensions_path.clone() {
                builder = builder.with_extensions_path(extensions_path);
            }
        }

        let webview = builder.build_as_child(window)?;
        self.webview = Some(webview);
        self.web_context = Some(web_context);
        self.launch = Some(launch.clone());
        Ok(true)
    }

    fn set_bounds(&self, bounds: gpui::Bounds<Pixels>) {
        if let Some(webview) = &self.webview {
            let rect = Rect {
                position: LogicalPosition::new(
                    f32::from(bounds.origin.x),
                    f32::from(bounds.origin.y),
                )
                .into(),
                size: LogicalSize::new(f32::from(bounds.size.width), f32::from(bounds.size.height))
                    .into(),
            };
            let _ = webview.set_bounds(rect);
        }
    }

    fn set_visible(&self, visible: bool) {
        if let Some(webview) = &self.webview {
            let _ = webview.set_visible(visible);
        }
    }

    fn open_devtools(&self) {
        if let Some(webview) = &self.webview {
            webview.open_devtools();
        }
    }

    fn evaluate_script(&self, script: &str) -> anyhow::Result<()> {
        if let Some(webview) = &self.webview {
            webview.evaluate_script(script)?;
        }
        Ok(())
    }

    fn load_url(&self, url: &str) -> anyhow::Result<()> {
        if let Some(webview) = &self.webview {
            webview.load_url(url)?;
        }
        Ok(())
    }

    fn clear_all_browsing_data(&self) -> anyhow::Result<()> {
        if let Some(webview) = &self.webview {
            webview.clear_all_browsing_data()?;
        }
        Ok(())
    }
}

pub struct EmbeddedWebPreviewPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    position: DockPosition,
    active: bool,
    current_url: String,
    current_title: String,
    last_error: Option<String>,
    status_message: Option<String>,
    webgpu_available: Option<bool>,
    hovered_capture: Option<CapturedElement>,
    selected_capture: Option<CapturedElement>,
    pending_ai_capture: Option<CapturedElement>,
    host: Rc<RefCell<WebViewHost>>,
    ipc_tx: Sender<String>,
    ipc_rx: Receiver<String>,
    browser_profiles: Vec<BrowserProfile>,
    selected_profile_ix: Option<usize>,
    session_policy: DevSessionPolicy,
    url_editor: gpui::Entity<Editor>,
    css_editor: gpui::Entity<Editor>,
    pending_navigation: bool,
    _poll_task: Task<()>,
    _profile_scan_task: Task<()>,
}

impl EmbeddedWebPreviewPanel {
    pub fn register(workspace: &mut Workspace) {
        workspace
            .register_action(|workspace, _: &ToggleEmbeddedWebPreview, window, cx| {
                workspace.toggle_panel_focus::<EmbeddedWebPreviewPanel>(window, cx);
            })
            .register_action(
                |workspace, _: &OpenEmbeddedWebPreviewDevTools, window, cx| {
                    if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                        workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                        panel.update(cx, |panel, _| panel.open_devtools());
                    }
                },
            )
            .register_action(|workspace, _: &PickEmbeddedWebElement, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| panel.arm_inspector(false));
                }
            })
            .register_action(|workspace, _: &InspectEmbeddedWebElementToAi, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| panel.arm_inspector(true));
                }
            })
            .register_action(|workspace, _: &CopyEmbeddedWebElementToAi, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, cx| panel.copy_latest_capture_to_ai(window, cx));
                }
            })
            .register_action(
                |workspace, _: &RefreshEmbeddedWebBrowserProfiles, window, cx| {
                    if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                        workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                        panel.update(cx, |panel, cx| panel.refresh_browser_profiles(cx));
                    }
                },
            )
            .register_action(|workspace, _: &ClearEmbeddedWebSession, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| panel.clear_session());
                }
            })
            .register_action(|workspace, _: &CycleEmbeddedWebSessionMode, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| {
                        panel.session_policy.mode = panel.session_policy.mode.next();
                        panel.pending_navigation = true;
                    });
                }
            })
            .register_action(|workspace, _: &ReloadEmbeddedWebPreview, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| panel.pending_navigation = true);
                }
            })
            .register_action(|workspace, _: &ApplyEmbeddedWebCss, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, cx| panel.apply_css_from_editor(cx));
                }
            })
            .register_action(|workspace, _: &ClearEmbeddedWebCss, window, cx| {
                if let Some(panel) = workspace.panel::<EmbeddedWebPreviewPanel>(cx) {
                    workspace.focus_panel::<EmbeddedWebPreviewPanel>(window, cx);
                    panel.update(cx, |panel, _| panel.clear_css_override());
                }
            });
    }

    pub fn new(
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (ipc_tx, ipc_rx) = channel();
        let url_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Enter a URL or local dev origin", window, cx);
            editor.set_text(DEFAULT_URL, window, cx);
            editor
        });

        let css_editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("color: red; display: grid; padding: 12px;", window, cx);
            editor
        });

        let host = Rc::new(RefCell::new(WebViewHost::default()));
        let poll_task = cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(80))
                    .await;
                if this.update(cx, |this, cx| this.poll_ipc(cx)).is_err() {
                    break;
                }
            }
        });

        let mut panel = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            position: DockPosition::Right,
            active: false,
            current_url: DEFAULT_URL.to_string(),
            current_title: String::new(),
            last_error: None,
            status_message: Some(
                "Detecting browser profiles and preparing the embedded preview".to_string(),
            ),
            webgpu_available: None,
            hovered_capture: None,
            selected_capture: None,
            pending_ai_capture: None,
            host,
            ipc_tx,
            ipc_rx,
            browser_profiles: Vec::new(),
            selected_profile_ix: None,
            session_policy: DevSessionPolicy::default(),
            url_editor,
            css_editor,
            pending_navigation: true,
            _poll_task: poll_task,
            _profile_scan_task: Task::ready(()),
        };
        panel.refresh_browser_profiles(cx);
        panel
    }

    fn selected_browser_profile(&self) -> Option<&BrowserProfile> {
        self.selected_profile_ix
            .and_then(|index| self.browser_profiles.get(index))
    }

    fn importing_extensions(&self) -> bool {
        self.selected_browser_profile()
            .is_some_and(BrowserProfile::is_preview_compatible)
    }

    fn sync_webview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected_profile = self.selected_browser_profile().cloned();
        let resolved_session = self.session_policy.resolve_for_url(
            &self.current_url,
            selected_profile.as_ref(),
            self.importing_extensions(),
        );
        let launch = WebViewLaunch {
            data_directory: resolved_session.data_directory.clone(),
            incognito: resolved_session.incognito,
            visible: self.active,
            extensions_path: selected_profile
                .as_ref()
                .filter(|profile| profile.is_preview_compatible())
                .and_then(|profile| profile.extensions_path.clone()),
        };
        let initial_url = if resolved_session.should_clear_before_navigation {
            "about:blank"
        } else {
            self.current_url.as_str()
        };

        let mut host = self.host.borrow_mut();
        match host.ensure(window, &launch, initial_url, self.ipc_tx.clone()) {
            Ok(rebuilt) => {
                if resolved_session.should_clear_before_navigation {
                    if let Err(error) = host.clear_all_browsing_data() {
                        self.last_error =
                            Some(format!("failed to clear local auth state: {error:#}"));
                    }
                    if let Err(error) = host.load_url(&self.current_url) {
                        self.last_error =
                            Some(format!("failed to load {}: {error:#}", self.current_url));
                    }
                } else if !rebuilt {
                    if let Err(error) = host.load_url(&self.current_url) {
                        self.last_error = Some(format!(
                            "failed to navigate to {}: {error:#}",
                            self.current_url
                        ));
                    }
                }

                self.status_message = Some(format!(
                    "{} | {} | {}",
                    selected_profile
                        .as_ref()
                        .map(BrowserProfile::display_name)
                        .unwrap_or_else(|| "Clean embedded preview".to_string()),
                    resolved_session.description,
                    if self.importing_extensions() {
                        format!(
                            "{} imported extension(s)",
                            selected_profile
                                .as_ref()
                                .map_or(0, BrowserProfile::extension_count)
                        )
                    } else {
                        "No imported browser extensions".to_string()
                    }
                ));
            }
            Err(error) => {
                self.last_error = Some(format!("failed to create embedded webview: {error:#}"));
            }
        }

        cx.notify();
    }

    fn refresh_browser_profiles(&mut self, cx: &mut Context<Self>) {
        self.status_message =
            Some("Scanning local browser profiles and installed extensions".to_string());
        self._profile_scan_task = cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { detect_browser_profiles() })
                .await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(profiles) => {
                        let previous_key = this
                            .selected_browser_profile()
                            .map(BrowserProfile::profile_key);
                        let selected_profile_ix = previous_key
                            .as_ref()
                            .and_then(|key| {
                                profiles
                                    .iter()
                                    .position(|profile| profile.profile_key() == *key)
                            })
                            .or_else(|| preferred_profile_index(&profiles));
                        this.browser_profiles = profiles;
                        this.selected_profile_ix = selected_profile_ix;
                        this.status_message = Some(format!(
                            "Detected {} browser profile(s) for extension import and visibility",
                            this.browser_profiles.len()
                        ));
                        this.pending_navigation = true;
                    }
                    Err(error) => {
                        this.last_error = Some(format!("browser profile scan failed: {error:#}"));
                    }
                }
                cx.notify();
            });
        });
    }

    fn poll_ipc(&mut self, cx: &mut Context<Self>) {
        let mut changed = false;
        while let Ok(message) = self.ipc_rx.try_recv() {
            match serde_json::from_str::<WebIpcMessage>(&message) {
                Ok(WebIpcMessage::Capability { webgpu, url, title }) => {
                    self.webgpu_available = Some(webgpu);
                    self.current_url = url;
                    self.current_title = title.unwrap_or_default();
                    changed = true;
                }
                Ok(WebIpcMessage::Navigation { url, title }) => {
                    self.current_url = url;
                    self.current_title = title.unwrap_or_default();
                    changed = true;
                }
                Ok(WebIpcMessage::Hover(capture)) => {
                    self.hovered_capture = Some(capture);
                    changed = true;
                }
                Ok(WebIpcMessage::Selection(capture)) => {
                    if capture.auto_send_to_ai {
                        self.pending_ai_capture = Some(capture.clone());
                    }
                    self.selected_capture = Some(capture);
                    changed = true;
                }
                Ok(WebIpcMessage::Log { message }) => {
                    self.status_message = Some(message);
                    changed = true;
                }
                Err(error) => {
                    self.last_error = Some(format!("webview ipc parse failed: {error}"));
                    changed = true;
                }
            }
        }

        if changed {
            cx.notify();
        }
    }

    fn normalize_url(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return DEFAULT_URL.to_string();
        }
        if trimmed.contains("://") {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        }
    }

    fn sync_url_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let editor_text = self.url_editor.read(cx).text(cx);
        if editor_text != self.current_url && !self.url_editor.read(cx).is_focused(window) {
            self.url_editor.update(cx, |editor, cx| {
                editor.set_text(self.current_url.clone(), window, cx);
            });
        }
    }

    fn read_url_editor_text(&self, cx: &App) -> String {
        self.url_editor.read(cx).text(cx)
    }

    fn read_css_editor_text(&self, cx: &App) -> String {
        self.css_editor.read(cx).text(cx)
    }

    fn set_selected_profile(&mut self, index: Option<usize>) {
        self.selected_profile_ix = index;
        self.pending_navigation = true;
    }

    fn open_devtools(&mut self) {
        self.host.borrow().open_devtools();
    }

    fn arm_inspector(&mut self, copy_to_ai: bool) {
        if let Err(error) = self
            .host
            .borrow()
            .evaluate_script(&arm_inspector_script(copy_to_ai))
        {
            self.last_error = Some(format!("failed to arm inspector: {error:#}"));
        } else {
            self.status_message = Some(if copy_to_ai {
                "Inspector armed. Click an element to send it straight to the agent.".to_string()
            } else {
                "Inspector armed. Hover to inspect, click to select.".to_string()
            });
        }
    }

    fn cancel_inspector(&mut self) {
        if let Err(error) = self
            .host
            .borrow()
            .evaluate_script(cancel_inspector_script())
        {
            self.last_error = Some(format!("failed to cancel inspector: {error:#}"));
        }
    }

    fn clear_session(&mut self) {
        if let Err(error) = self.host.borrow().clear_all_browsing_data() {
            self.last_error = Some(format!("failed to clear preview session: {error:#}"));
        } else {
            self.status_message = Some(
                "Cleared cookies, storage, and browsing data for the active preview session."
                    .to_string(),
            );
        }
    }

    fn apply_css_from_editor(&mut self, cx: &App) {
        let css = self.read_css_editor_text(cx);
        if css.trim().is_empty() {
            self.status_message =
                Some("Enter CSS declarations before applying an override.".to_string());
            return;
        }
        if let Err(error) = self.host.borrow().evaluate_script(&apply_css_script(&css)) {
            self.last_error = Some(format!("failed to apply CSS override: {error:#}"));
        } else {
            self.status_message =
                Some("Applied CSS override to the currently selected DOM node.".to_string());
        }
    }

    fn clear_css_override(&mut self) {
        if let Err(error) = self.host.borrow().evaluate_script(clear_css_script()) {
            self.last_error = Some(format!("failed to clear CSS override: {error:#}"));
        } else {
            self.status_message =
                Some("Cleared CSS overrides for the selected DOM node.".to_string());
        }
    }

    fn copy_capture_to_ai(
        &mut self,
        capture: CapturedElement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workspace) = self.workspace.upgrade() else {
            self.last_error = Some("workspace is no longer available".to_string());
            cx.notify();
            return;
        };

        workspace.update(cx, |workspace, cx| {
            workspace.focus_panel::<AgentPanel>(window, cx);
            if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                if panel.read(cx).active_thread_view(cx).is_none() {
                    window.dispatch_action(NewTextThread.boxed_clone(), cx);
                }

                let prompt = vec![acp::ContentBlock::Text(acp::TextContent::new(format!(
                    "Use this inspected web element as implementation context.\n\nURL: {}\nSelector: {}\nDOM Path: {}\n\nText:\n{}\n\nHTML:\n```html\n{}\n```\n\nComputed CSS:\n```css\n{}\n```\n\nInline CSS:\n```css\n{}\n```",
                    capture.url,
                    capture.selector,
                    capture.dom_path,
                    capture.text,
                    capture.html,
                    capture.css,
                    capture.inline_css,
                )))];
                let panel = panel.clone();
                window.defer(cx, move |window, cx| {
                    panel.update(cx, |panel, cx| {
                        if let Some(thread_view) = panel.active_thread_view(cx) {
                            thread_view.update(cx, |thread_view, cx| {
                                thread_view.message_editor.update(cx, |editor, cx| {
                                    editor.append_message(prompt, Some("\n\n"), window, cx);
                                });
                            });
                        }
                    });
                });
            }
        });

        self.status_message =
            Some("Sent inspected DOM and CSS context to the active agent thread.".to_string());
    }

    fn copy_latest_capture_to_ai(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let capture = self
            .selected_capture
            .clone()
            .or_else(|| self.hovered_capture.clone());
        let Some(capture) = capture else {
            self.last_error =
                Some("Inspect or hover an element before copying it to AI.".to_string());
            cx.notify();
            return;
        };
        self.copy_capture_to_ai(capture, window, cx);
    }

    fn commit_pending_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_navigation {
            self.pending_navigation = false;
            let next_url = Self::normalize_url(&self.read_url_editor_text(cx));
            if self.current_url != next_url {
                self.current_url = next_url;
            }
            self.sync_webview(window, cx);
        }

        if let Some(capture) = self.pending_ai_capture.take() {
            self.copy_capture_to_ai(capture, window, cx);
        }
    }

    fn browser_button_label(&self, index: Option<usize>) -> String {
        match index.and_then(|idx| self.browser_profiles.get(idx)) {
            Some(profile) => format!(
                "{} ({}, {} ext)",
                profile.display_name(),
                profile.preview_support.label(),
                profile.extension_count()
            ),
            None => "Clean Preview (no imported extensions)".to_string(),
        }
    }
}

impl Focusable for EmbeddedWebPreviewPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for EmbeddedWebPreviewPanel {}

impl Panel for EmbeddedWebPreviewPanel {
    fn persistent_name() -> &'static str {
        "Embedded Web Preview"
    }

    fn panel_key() -> &'static str {
        "embedded-web-preview"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        self.position
    }

    fn position_is_valid(&self, _position: DockPosition) -> bool {
        true
    }

    fn set_position(
        &mut self,
        position: DockPosition,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.position = position;
        cx.notify();
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(460.0)
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::ToolWeb)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Embedded Web Preview")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(ToggleEmbeddedWebPreview)
    }

    fn set_active(&mut self, active: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.active = active;
        self.host.borrow().set_visible(active);
        self.pending_navigation = true;
        cx.notify();
    }

    fn activation_priority(&self) -> u32 {
        12
    }
}

impl Render for EmbeddedWebPreviewPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.poll_ipc(cx);
        self.sync_url_editor(window, cx);
        self.commit_pending_state(window, cx);

        let host = self.host.clone();
        let capability = match self.webgpu_available {
            Some(true) => "WebGPU ready",
            Some(false) => "WebGPU unavailable",
            None => "Detecting WebGPU",
        };
        let hovered_summary = self
            .hovered_capture
            .as_ref()
            .map(CapturedElement::summary)
            .unwrap_or_else(|| "No hovered element yet".to_string());
        let selected_summary = self
            .selected_capture
            .as_ref()
            .map(CapturedElement::summary)
            .unwrap_or_else(|| "No selected element yet".to_string());
        let session_label = match self.session_policy.mode {
            DevSessionMode::SharedWorkspace => "Shared browsing context",
            DevSessionMode::IsolatedPerOrigin => "Per-origin isolated browsing",
            DevSessionMode::Incognito => "Incognito browsing",
        };

        let mut browser_buttons = vec![
            Button::new("browser-clean-profile", self.browser_button_label(None))
                .on_click(cx.listener(|this, _, _, _| this.set_selected_profile(None)))
                .into_any_element(),
        ];
        browser_buttons.extend(self.browser_profiles.iter().enumerate().map(
            |(index, _profile)| {
                Button::new(
                    ("browser-profile", index),
                    self.browser_button_label(Some(index)),
                )
                .on_click(cx.listener(move |this, _, _, _| this.set_selected_profile(Some(index))))
                .into_any_element()
            },
        ));

        let extension_list = self
            .selected_browser_profile()
            .map(|profile| {
                if profile.extensions.is_empty() {
                    vec![
                        Label::new("No extensions detected in the selected browser profile.")
                            .color(Color::Muted)
                            .into_any_element(),
                    ]
                } else {
                    profile
                        .extensions
                        .iter()
                        .take(8)
                        .map(|extension| {
                            Label::new(format!(
                                "{} {}{}",
                                extension.name,
                                extension.version,
                                extension
                                    .description
                                    .as_ref()
                                    .map(|description| format!(" - {}", description))
                                    .unwrap_or_default()
                            ))
                            .into_any_element()
                        })
                        .collect::<Vec<_>>()
                }
            })
            .unwrap_or_else(|| {
                vec![
                    Label::new(
                        "Using a clean embedded browser profile without imported extensions.",
                    )
                    .color(Color::Muted)
                    .into_any_element(),
                ]
            });

        v_flex()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .gap_2()
            .p_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().editor_background)
                            .child(self.url_editor.clone()),
                    )
                    .child(Button::new("web-preview-go", "Go").on_click(cx.listener(
                        |this, _, _, _| {
                            this.pending_navigation = true;
                        },
                    )))
                    .child(
                        IconButton::new("web-preview-reload", IconName::RotateCw).on_click(
                            cx.listener(|this, _, _, _| {
                                this.pending_navigation = true;
                            }),
                        ),
                    )
                    .child(
                        IconButton::new("web-preview-devtools", IconName::Code).on_click(
                            cx.listener(|this, _, _, _| {
                                this.open_devtools();
                            }),
                        ),
                    ),
            )
            .child(Label::new(format!("{} | {}", capability, session_label)).color(Color::Muted))
            .when_some(self.status_message.clone(), |this, message| {
                this.child(Label::new(message).color(Color::Info))
            })
            .when_some(self.last_error.clone(), |this, error| {
                this.child(Label::new(error).color(Color::Error))
            })
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        Button::new("refresh-browsers", "Refresh Browsers").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.refresh_browser_profiles(cx);
                            },
                        )),
                    )
                    .child(
                        Button::new(
                            "cycle-session-mode",
                            format!("Session: {}", self.session_policy.mode.label()),
                        )
                        .on_click(cx.listener(|this, _, _, _| {
                            this.session_policy.mode = this.session_policy.mode.next();
                            this.pending_navigation = true;
                        })),
                    )
                    .child(
                        Button::new(
                            "toggle-local-auth-clear",
                            if self.session_policy.auto_clear_local_auth {
                                "Auto-clear localhost auth: On"
                            } else {
                                "Auto-clear localhost auth: Off"
                            },
                        )
                        .on_click(cx.listener(|this, _, _, _| {
                            this.session_policy.auto_clear_local_auth =
                                !this.session_policy.auto_clear_local_auth;
                            this.pending_navigation = true;
                        })),
                    )
                    .child(
                        Button::new("clear-session", "Clear Session").on_click(cx.listener(
                            |this, _, _, _| {
                                this.clear_session();
                            },
                        )),
                    )
                    .child(Button::new("inspect", "Inspect").on_click(cx.listener(
                        |this, _, _, _| {
                            this.arm_inspector(false);
                        },
                    )))
                    .child(
                        Button::new("inspect-to-ai", "Inspect To AI").on_click(cx.listener(
                            |this, _, _, _| {
                                this.arm_inspector(true);
                            },
                        )),
                    )
                    .child(
                        Button::new("cancel-inspect", "Cancel Inspect").on_click(cx.listener(
                            |this, _, _, _| {
                                this.cancel_inspector();
                            },
                        )),
                    )
                    .child(
                        Button::new("copy-hover", "Copy Hover To AI").on_click(cx.listener(
                            |this, _, window, cx| {
                                if let Some(capture) = this.hovered_capture.clone() {
                                    this.copy_capture_to_ai(capture, window, cx);
                                }
                            },
                        )),
                    )
                    .child(
                        Button::new("copy-selected", "Copy Selected To AI").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.copy_latest_capture_to_ai(window, cx);
                            },
                        )),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Detected Browser Profiles").color(Color::Accent))
                    .children(browser_buttons),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Imported / Detected Extensions").color(Color::Accent))
                    .children(extension_list),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().colors().border)
                            .bg(cx.theme().colors().editor_background)
                            .child(self.css_editor.clone()),
                    )
                    .child(Button::new("apply-css", "Apply CSS").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.apply_css_from_editor(cx);
                        },
                    )))
                    .child(Button::new("clear-css", "Clear CSS").on_click(cx.listener(
                        |this, _, _, _| {
                            this.clear_css_override();
                        },
                    ))),
            )
            .child(Label::new(format!("Hover: {}", hovered_summary)).color(Color::Muted))
            .child(Label::new(format!("Selected: {}", selected_summary)).color(Color::Muted))
            .child(
                div()
                    .flex_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .bg(gpui::rgb(0x0B0F16))
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                host.borrow().set_bounds(bounds);
                                window.paint_quad(gpui::fill(bounds, gpui::rgb(0x0B0F16)));
                            },
                        )
                        .size_full(),
                    ),
            )
    }
}
