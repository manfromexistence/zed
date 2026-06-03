import { readFileSync } from "node:fs";
import test from "node:test";
import assert from "node:assert/strict";

const read = (path) => readFileSync(path, "utf8");

const platformLibs = [
  ["macOS", "crates/web_preview_macos/src/lib.rs", "target_os = \"macos\""],
  ["Linux", "crates/web_preview_linux/src/lib.rs", "target_os = \"linux\""],
];

const platformViews = [
  ["macOS", "crates/web_preview_macos/src/web_preview_view.rs"],
  ["Linux", "crates/web_preview_linux/src/web_preview_view.rs"],
];

const desktopOnboardingPreviewViews = [
  ["Windows", "crates/web_preview/src/web_preview_view.rs"],
  ...platformViews,
];

const platformHostFiles = [
  "crates/web_preview_macos/src/macos_host.rs",
  "crates/web_preview_linux/src/x11_host.rs",
  "crates/web_preview_linux/src/wayland_host.rs",
];

const desktopPreviewCallsites = [
  ["font panel", "crates/font_panel/src/font_panel.rs"],
  ["media panel", "crates/media_panel/src/media_panel.rs"],
  ["shadcn UI panel", "crates/shadcn_ui_panel/src/shadcn_ui_panel.rs"],
  ["sidebar browser grid", "crates/sidebar/src/sidebar.rs"],
];

const desktopPreviewCargoManifests = [
  ["font panel", "crates/font_panel/Cargo.toml", true],
  ["media panel", "crates/media_panel/Cargo.toml", true],
  ["shadcn UI panel", "crates/shadcn_ui_panel/Cargo.toml", true],
  ["sidebar", "crates/sidebar/Cargo.toml", false],
  ["onboarding", "crates/onboarding/Cargo.toml", false],
];

test("main web_preview crate keeps Windows WebView2 isolated", () => {
  const source = read("crates/web_preview/src/web_preview.rs");

  assert.match(source, /#\[cfg\(target_os = "windows"\)\]\s+pub mod web_preview_view;/);
  assert.match(source, /#\[cfg\(target_os = "windows"\)\]\s+pub\(crate\) mod windows_visual_webview;/);
  assert.match(
    source,
    /#\[cfg\(target_os = "macos"\)\]\s+pub use web_preview_macos::\{OpenPreview, OpenPreviewToTheSide, init, web_preview_view\};/,
  );
  assert.match(
    source,
    /#\[cfg\(target_os = "linux"\)\]\s+pub use web_preview_linux::\{OpenPreview, OpenPreviewToTheSide, init, web_preview_view\};/,
  );
});

test("main web_preview crate re-exports platform action types", () => {
  const source = read("crates/web_preview/src/web_preview.rs");

  assert.match(
    source,
    /#\[cfg\(target_os = "macos"\)\]\s+pub use web_preview_macos::\{OpenPreview, OpenPreviewToTheSide, init, web_preview_view\};/,
  );
  assert.match(
    source,
    /#\[cfg\(target_os = "linux"\)\]\s+pub use web_preview_linux::\{OpenPreview, OpenPreviewToTheSide, init, web_preview_view\};/,
  );
  assert.match(
    source,
    /not\(target_os = "linux"\),\s+not\(target_os = "macos"\),\s+not\(target_os = "windows"\)\s+\)\)\]\s+pub use web_preview_linux::init;/,
  );
});

test("desktop preview entry points use native Web Preview on macOS and Linux", () => {
  for (const [name, path] of desktopPreviewCallsites) {
    const source = read(path);

    assert.match(
      source,
      /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+use web_preview::web_preview_view::WebPreviewView;/,
      `${name} should import WebPreviewView on every supported desktop platform`,
    );
    assert.match(
      source,
      /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+\{/,
      `${name} should use native Web Preview on supported desktop platforms`,
    );
    assert.match(
      source,
      /WebPreviewView::open_url_in_active_pane\(workspace, &?preview_url|WebPreviewView::open_url_in_active_pane\(workspace, url/,
      `${name} should route preview URLs through WebPreviewView`,
    );
    assert.match(
      source,
      /#\[cfg\(not\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\)\]/,
      `${name} should keep external browser fallback only for unsupported platforms`,
    );
  }
});

test("desktop preview crates depend on web_preview for every supported desktop OS", () => {
  for (const [name, path, unconditional] of desktopPreviewCargoManifests) {
    const source = read(path);

    if (unconditional) {
      assert.match(
        source,
        /^\s*web_preview\.workspace = true$/m,
        `${name} should have an unconditional web_preview dependency`,
      );
      continue;
    }

    assert.match(
      source,
      /\[target\.'cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)'\.dependencies\]\s+web_preview\.workspace = true/s,
      `${name} should enable web_preview on Windows, macOS, and Linux`,
    );
    assert.doesNotMatch(
      source,
      /\[target\.'cfg\(target_os = "windows"\)'\.dependencies\]\s+web_preview\.workspace = true/s,
      `${name} should not keep web_preview Windows-only`,
    );
  }
});

test("onboarding DX preview uses native Web Preview on macOS and Linux", () => {
  const source = read("crates/onboarding/src/onboarding.rs");

  assert.match(
    source,
    /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+use web_preview::web_preview_view::WebPreviewView;/,
  );
  assert.match(
    source,
    /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+dx_web_preview: Option<Entity<WebPreviewView>>,/,
  );
  assert.match(source, /fn ensure_dx_web_preview\(/);
  assert.match(source, /WebPreviewView::new_for_onboarding\(/);
  assert.match(
    source,
    /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+fn render_web_preview_canvas\(/,
  );
  assert.match(source, /let preview = self\.ensure_dx_web_preview\(window, cx\);/);
  assert.match(
    source,
    /#\[cfg\(not\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\)\]/,
  );
  assert.doesNotMatch(source, /Windows Web Preview runtime/);
});

test("onboarding uses a local Web Preview page with a real completion bridge", () => {
  const source = read("crates/onboarding/src/onboarding.rs");
  const dxLaunchSource = read("crates/onboarding/src/dx_launch_onboarding.rs");
  const workspaceSource = read("crates/workspace/src/workspace.rs");
  const paneSource = read("crates/workspace/src/pane.rs");
  const agentPanelSource = read("crates/agent_ui/src/agent_panel.rs");
  const sidebarSource = read("crates/sidebar/src/sidebar.rs");

  assert.match(source, /const WEB_PREVIEW_ONBOARDING_HTML: &str/);
  assert.match(source, /JSON\.stringify\(\{ kind: "onboarding-complete" \}\)/);
  assert.match(source, /DxLaunchPreviewTargets::local_web_preview_onboarding\(web_preview_onboarding_url\(\)\)/);
  assert.match(source, /fn handle_finish\(&mut self, _: &Finish, window: &mut Window, cx: &mut Context<Self>\)/);
  assert.match(source, /finish_setup\(self\.workspace\.clone\(\), window, cx\);/);
  assert.match(source, /fn finish_setup<C: AppContext>\(/);
  const finishSetup = functionBody(source, "finish_setup");
  assert.match(finishSetup, /close_onboarding_page\(workspace, window, cx\);/);
  assert.doesNotMatch(
    finishSetup,
    /go_to_welcome_page\(cx\)/,
    "complete should remove the fullscreen onboarding surface instead of replacing it with welcome",
  );
  assert.doesNotMatch(source, /fn go_to_welcome_page\(/);
  assert.doesNotMatch(source, /fn render_dx_launch_hero\(/);
  assert.doesNotMatch(source, /OpenDxWwwPreview|OpenBundledDxPreview/);
  assert.match(dxLaunchSource, /pub struct DxLaunchPreviewTargets \{\s+pub primary: DxLaunchPreviewTarget,\s+\}/);
  assert.match(dxLaunchSource, /DxLaunchPreviewTarget \{ url \}/);
  assert.doesNotMatch(dxLaunchSource, /DX_WWW|FALLBACK_HTML|preview_status_rows|missing_dx_www_detail|detect\(/);
  const ensurePreview = functionBody(source, "ensure_dx_web_preview");
  assert.match(
    ensurePreview,
    /let completion_workspace = workspace\.clone\(\);/,
  );
  assert.match(
    ensurePreview,
    /let complete_onboarding = Rc::new\(move \|window: &mut Window, cx: &mut App\| \{\s*finish_setup\(completion_workspace\.clone\(\), window, cx\);\s*\}\);/s,
  );
  assert.match(
    ensurePreview,
    /WebPreviewView::new_for_onboarding\([\s\S]*Some\(complete_onboarding\)[\s\S]*\)/,
  );
  assert.match(source, /fn close_onboarding_page<C: AppContext>\(/);
  const closeOnboardingPage = functionBody(source, "close_onboarding_page");
  assert.match(closeOnboardingPage, /workspace\.update\(cx, \|workspace, cx\|/);
  assert.doesNotMatch(closeOnboardingPage, /with_active_or_new_workspace/);
  assert.match(source, /fn find_onboarding_page\(workspace: &Workspace, cx: &App\) -> Option<Entity<Onboarding>>/);
  assert.match(source, /workspace\.panes\(\)\.iter\(\)\.find_map/);
  assert.match(source, /fn close_open_docks_for_onboarding\(/);
  assert.match(source, /closed_docks_for_fullscreen: Vec<DockPosition>/);
  assert.match(source, /fn track_closed_docks_for_fullscreen\(&mut self, positions: Vec<DockPosition>\)/);
  assert.match(source, /fn take_closed_docks_for_fullscreen\(&mut self\) -> Vec<DockPosition>/);
  assert.match(source, /cx\.emit\(ItemEvent::UpdateTab\);/);
  assert.match(source, /fn serialize_closed_docks_for_fullscreen\(positions: &\[DockPosition\]\) -> String/);
  assert.match(source, /fn deserialize_closed_docks_for_fullscreen\(value: &str\) -> Vec<DockPosition>/);
  assert.match(source, /ALTER TABLE onboarding_pages\s+ADD COLUMN closed_docks_for_fullscreen TEXT NOT NULL DEFAULT "";/);
  assert.match(source, /db\.save_onboarding_page\(item_id, workspace_id, closed_docks_for_fullscreen\)/);
  assert.match(source, /SELECT closed_docks_for_fullscreen\s+FROM onboarding_pages/);
  assert.match(source, /fn zoom_active_onboarding_pane\(/);
  assert.match(source, /pane\.zoom_in\(&ZoomIn, window, cx\)/);
  assert.match(source, /pane\.zoom_out\(&ZoomOut, window, cx\);/);
  assert.match(source, /if !workspace\.is_dock_at_position_open\(position, cx\) \{/);
  assert.match(source, /workspace\.toggle_dock\(position, window, cx\);/);
  assert.match(source, /pane\.remove_item\(onboarding_id, true, false, window, cx\);/);
  assert.match(source, /fn can_split\(&self\) -> bool \{\s+false\s+\}/);
  assert.match(
    paneSource,
    /mode == SplitMode::MovePane[\s\S]*?item\.screen_kind\(cx\) == WorkspaceScreenKind::Onboarding[\s\S]*?return;/,
    "Onboarding must not be movable into a split pane through MovePane commands",
  );
  assert.match(
    paneSource,
    /Some\(active_item\) if active_item\.screen_kind\(cx\) == WorkspaceScreenKind::Onboarding => \{\s*\(false, false\)\s*\}/,
    "Onboarding must not expose clone or move split actions in pane chrome",
  );
  assert.match(
    source,
    /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s+WorkspaceScreenKind::Onboarding\s+\}/,
  );
  assert.match(source, /\.child\(self\.render_web_preview_canvas\(window, cx\)\)/);
  assert.match(
    workspaceSource,
    /if kind == WorkspaceScreenKind::Onboarding \{\s*window\.dispatch_action\(OpenOnboarding\.boxed_clone\(\), cx\);\s*return true;\s*\}/s,
    "screen dock activation must route existing onboarding through the fullscreen onboarding opener",
  );
  assert.match(
    agentPanelSource,
    /WorkspaceScreenKind::Terminal\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::LiquidGlass/s,
    "Agent workspace snapshots should treat Onboarding like other non-editor screens",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Onboarding => Self::Other/,
    "sidebar grid persistence should deliberately fold Onboarding into the existing non-primary screen context",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Editor\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::LiquidGlass\s*\|\s*WorkspaceScreenKind::Other => self\.project_root_path\(cx\)/s,
    "sidebar grid context should use project-root shortcuts while Onboarding is active",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Editor\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::Other => (?:self\.editor_grid_entries\(cx\)|\{\s*self\.editor_grid_entries\(cx\)\s*\})/s,
    "sidebar grid generation should show editor/project entries while Onboarding is active",
  );

  const renderStart = source.indexOf("impl Render for Onboarding");
  assert.ok(renderStart >= 0, "expected Onboarding render impl");
  const renderBody = source.slice(renderStart);
  assert.doesNotMatch(
    renderBody,
    /render_dx_launch_hero\(window, cx\)/,
    "first-run onboarding should render the Web Preview surface directly",
  );
});

for (const [name, path] of desktopOnboardingPreviewViews) {
  test(`${name} Web Preview exposes onboarding completion IPC`, () => {
    const source = read(path);

    assert.match(source, /pub type OnboardingCompleteCallback = Rc<dyn Fn\(&mut Window, &mut App\)>;/);
    assert.match(source, /onboarding_complete: Option<OnboardingCompleteCallback>/);
    assert.match(source, /"onboarding-complete" => \{/);
    assert.match(source, /if let Some\(complete\) = self\.onboarding_complete\.clone\(\)/);
    assert.match(source, /complete\(window, cx\);/);
    const newForOnboarding = functionBody(source, "new_for_onboarding");
    assert.match(newForOnboarding, /Self::new_for_url\([\s\S]*onboarding_complete/s);
    const newForUrl = functionBody(source, "new_for_url");
    assert.match(newForUrl, /onboarding_complete,/);
  });
}

function functionBody(sourceText, name) {
  const start = sourceText.search(new RegExp(`fn\\s+${name}\\b`));
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = sourceText.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < sourceText.length; index += 1) {
    const char = sourceText[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return sourceText.slice(start, index + 1);
      }
    }
  }

  assert.fail(`could not find body for ${name}`);
}

for (const [name, path, cfg] of platformLibs) {
  test(`${name} web preview init registers actions and startup lifecycle`, () => {
    const source = read(path);

    assert.match(source, new RegExp(`#\\[cfg\\(${cfg.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\)\\]`));
    assert.match(source, /web_preview_view::WebPreviewView::register\(workspace, window, cx\);/);
    assert.match(source, /cx\.defer_in\(window, \|workspace, window, cx\| \{/);
    assert.match(
      source,
      /web_preview_view::WebPreviewView::ensure_startup_preview\(workspace, window, cx\);/,
    );
  });
}

for (const [name, path] of platformViews) {
  test(`${name} web preview exposes a startup hook without opening anything eagerly`, () => {
    const source = read(path);

    assert.match(source, /pub fn ensure_startup_preview\(\s*workspace: &mut Workspace,/);
    assert.match(source, /let _ = \(workspace, window, cx\);/);
  });
}

for (const [name, path] of platformViews) {
  test(`${name} web preview has browser tab parity`, () => {
    const source = read(path);

    assert.match(source, /TabContentParams/);
    assert.match(source, /WorkspaceScreenKind/);
    assert.match(source, /fn tab_content\(&self, params: TabContentParams, window: &Window,/);
    assert.match(source, /Label::new\(self\.current_tab_title\(\)\)/);
    assert.match(source, /fn tab_content_text\(&self, _detail: usize, _cx: &App\) -> SharedString \{\s+self\.current_tab_title\(\)/);
    assert.match(source, /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s+WorkspaceScreenKind::Browser\s+\}/);
    assert.match(source, /fn on_tab_click\(/);
    assert.match(source, /self\.activate_url_editor\(window, cx\);/);
    assert.match(source, /fn on_tab_confirm\(&mut self, window: &mut Window, cx: &mut Context<Self>\) -> bool/);
    assert.match(source, /self\.confirm_navigation\(&Confirm, window, cx\);/);
  });
}

for (const [name, path] of desktopOnboardingPreviewViews) {
  test(`${name} web preview shows a real loading spinner placeholder`, () => {
    const source = read(path);

    assert.match(source, /PreviewLoadState::Loading\s*=>\s*None/);
    assert.match(source, /let show_loading_placeholder =/);
    assert.match(source, /let loading_placeholder = show_loading_placeholder\.then/);
    assert.match(source, /IconName::LoadCircle/);
    assert.match(source, /\.with_rotate_animation\(2\)/);
    assert.match(source, /Label::new\("Loading Web Preview"\)/);
    assert.match(
      source,
      /\.child\(body\)[\s\S]*\.when_some\(loading_placeholder/,
      `${name} should overlay the loading placeholder on the preview body`,
    );
  });
}

for (const [name, path] of platformViews) {
  test(`${name} web preview uses pane tab controls instead of an in-body toolbar`, () => {
    const source = read(path);

    assert.match(source, /fn render_tab_bar_start_controls\(&self, cx: &mut Context<Self>\) -> AnyElement/);
    assert.match(source, /fn render_tab_bar_end_controls\(&self, cx: &mut Context<Self>\) -> AnyElement/);
    assert.match(source, /IconButton::new\("web-preview-tab-bar-add-trigger", IconName::Plus\)/);
    assert.match(source, /window\.dispatch_action\(NewWebPreview\.boxed_clone\(\), cx\);/);
    assert.match(source, /IconButton::new\("web-preview-tab-bar-back", IconName::ArrowLeft\)/);
    assert.match(source, /IconButton::new\("web-preview-tab-bar-forward", IconName::ArrowRight\)/);
    assert.match(source, /IconButton::new\("web-preview-tab-bar-reload", IconName::RotateCw\)/);
    assert.match(source, /IconButton::new\("web-preview-tab-bar-bookmark", bookmark_icon\)/);
    assert.match(source, /fn render_tab_bar_extensions_menu\(&self, entity: Entity<Self>\) -> impl IntoElement/);
    assert.match(source, /PopoverMenu::new\("web-preview-tab-bar-extensions-menu"\)/);
    assert.match(source, /fn render_tab_bar_more_menu\(&self, entity: Entity<Self>\) -> impl IntoElement/);
    assert.match(source, /PopoverMenu::new\("web-preview-tab-bar-more-menu"\)/);
    assert.match(source, /ContextMenuEntry::new\("Capture Screenshot"\)/);
    assert.match(source, /ContextMenuEntry::new\("Inspect Element"\)/);
    assert.match(source, /ContextMenuEntry::new\("Open DevTools"\)/);
    assert.match(source, /ContextMenuEntry::new\("Clear Cache"\)/);
    assert.match(source, /Some\(PaneTabBarControls::new\(\s+Some\(self\.render_tab_bar_start_controls\(cx\)\),\s+Some\(self\.render_tab_bar_end_controls\(cx\)\),\s+\)\)/);
    assert.doesNotMatch(source, /\.id\("web-preview-toolbar"\)/);
    assert.doesNotMatch(source, /fn render_toolbar_action_button\(/);
    assert.doesNotMatch(source, /web-preview-zoom-in|web-preview-zoom-out/);
  });
}

test("macOS web preview has a native host lifecycle contract", () => {
  const view = read("crates/web_preview_macos/src/web_preview_view.rs");
  const host = read("crates/web_preview_macos/src/macos_host.rs");

  assert.match(view, /crate::macos_host::MacPreviewHost::new\(window, \*host_bounds\.borrow\(\)\)\?/);
  assert.match(view, /\.with_accept_first_mouse\(true\)/);
  assert.match(view, /webview\.reparent\(host\.ns_window_ptr\(\)\)\?/);
  assert.match(view, /sync_macos_native_preview_target\(/);
  assert.match(view, /set_macos_native_preview_visible\(/);
  assert.match(host, /addChildWindow: initialized\s+ordered: NSWindowOrderingMode::Below/);
  assert.match(host, /orderWindow: NSWindowOrderingMode::Below/);
  assert.match(host, /pub\(crate\) fn focus_gpui_view\(&self\)/);
  assert.match(host, /pub\(crate\) fn capture_image\(&self\) -> Result<RgbaImage>/);
});

test("Linux web preview has X11 and Wayland native host contracts", () => {
  const view = read("crates/web_preview_linux/src/web_preview_view.rs");
  const x11Host = read("crates/web_preview_linux/src/x11_host.rs");
  const waylandHost = read("crates/web_preview_linux/src/wayland_host.rs");

  assert.match(view, /use gpui_linux::exported_wayland_window_handle;/);
  assert.match(view, /fn resolve_linux_native_preview_target\(window: &Window\) -> Result<LinuxNativePreviewTarget>/);
  assert.match(view, /ensure_linux_webview_runtime\(window_system\)\?/);
  assert.match(view, /create_native_preview_for_linux_x11_window\(/);
  assert.match(view, /create_native_preview_for_linux_wayland_window\(/);
  assert.match(view, /\.build_gtk\(host\.container\(\)\)/);
  assert.match(view, /fn pump_linux_webview_events\(\) -> bool/);
  assert.match(view, /sync_linux_native_preview_target\(/);
  assert.match(x11Host, /pub\(crate\) struct X11PreviewHost/);
  assert.match(x11Host, /attach_transient_parent\(&window, parent_xid\)\?/);
  assert.match(x11Host, /pub\(crate\) fn capture_image\(&self\) -> Result<RgbaImage>/);
  assert.match(waylandHost, /pub\(crate\) struct WaylandPreviewHost/);
  assert.match(waylandHost, /set_transient_for_exported\(exported_parent_handle\)/);
  assert.match(waylandHost, /pub\(crate\) fn capture_image\(&self\) -> Result<RgbaImage>/);
});

test("platform host support stays in focused files", () => {
  for (const path of platformHostFiles) {
    const source = read(path);
    assert.doesNotMatch(source, /WindowsVisualWebView|WebView2|CoreWebView2/);
  }
});
