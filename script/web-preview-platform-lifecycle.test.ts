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

test("onboarding Web Preview is disabled behind an explicit TODO", () => {
  const source = read("crates/onboarding/src/onboarding.rs");

  assert.match(source, /TODO\(dx-onboarding\): Re-enable the fullscreen WebPreview onboarding/);
  assert.match(source, /const DX_WEB_PREVIEW_ONBOARDING_DISABLED: bool = true;/);
  assert.doesNotMatch(
    source,
    /^\s*register_serializable_item::<Onboarding>\(cx\);/m,
    "disabled onboarding should not register a restorable item kind",
  );
  assert.match(
    source,
    /\/\/ register_serializable_item::<Onboarding>\(cx\);/,
    "the parked restore path should stay as an explicit TODO, not disappear silently",
  );
  assert.match(
    source,
    /cx\.on_action\(\|_: &OpenOnboarding, cx\| \{\s*if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return;\s*\}/s,
    "manual onboarding action should not open the fullscreen WebPreview while disabled",
  );
  const showOnboardingView = functionBody(source, "show_onboarding_view");
  assert.match(showOnboardingView, /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return open_new\(/s);
  assert.match(
    showOnboardingView,
    /Editor::new_file\(workspace, &Default::default\(\), window, cx\);/,
    "first-open startup should open the normal editor workspace while onboarding is parked",
  );
  assert.match(
    showOnboardingView,
    /kvp\.write_kvp\(FIRST_OPEN\.to_string\(\), "false"\.to_string\(\)\)/,
    "first-open startup should mark onboarding complete so it does not retry every launch",
  );
  assert.match(
    source,
    /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+use web_preview::web_preview_view::WebPreviewView;/,
  );
  assert.match(
    source,
    /#\[cfg\(any\(target_os = "windows", target_os = "macos", target_os = "linux"\)\)\]\s+dx_web_preview: Option<Entity<WebPreviewView>>,/,
  );
  const workspaceOverlay = functionBody(source, "workspace_overlay");
  assert.match(
    workspaceOverlay,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return None;\s*\}/s,
    "stale restored onboarding items must not expose the fullscreen WebPreview overlay",
  );
  const deserialize = functionBody(source, "deserialize");
  assert.match(
    deserialize,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{[\s\S]*delete_onboarding_page\(item_id, workspace_id\)\.await\?/,
    "disabled onboarding should delete stale serialized rows instead of restoring the WebPreview item",
  );
  const serialize = functionBody(source, "serialize");
  assert.match(serialize, /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return None;\s*\}/);
  const shouldSerialize = functionBody(source, "should_serialize");
  assert.match(shouldSerialize, /!DX_WEB_PREVIEW_ONBOARDING_DISABLED/);
});

test("fullscreen onboarding Web Preview is parked until the completion handoff is rebuilt", () => {
  const source = read("crates/onboarding/src/onboarding.rs");
  const dxLaunchSource = read("crates/onboarding/src/dx_launch_onboarding.rs");
  const workspaceSource = read("crates/workspace/src/workspace.rs");
  const onboardingCargo = read("crates/onboarding/Cargo.toml");
  const multiWorkspaceSource = read("crates/workspace/src/multi_workspace.rs");
  const paneSource = read("crates/workspace/src/pane.rs");
  const titleBarSource = read("crates/title_bar/src/title_bar.rs");
  const agentPanelSource = read("crates/agent_ui/src/agent_panel.rs");
  const sidebarSource = read("crates/sidebar/src/sidebar.rs");

  assert.match(onboardingCargo, /^editor\.workspace = true$/m);
  assert.match(source, /use editor::Editor;/);
  assert.match(source, /completion_requested: bool,/);
  assert.match(source, /use zed_actions::OpenOnboarding;/);
  assert.match(source, /const DX_WEB_PREVIEW_ONBOARDING_DISABLED: bool = true;/);
  assert.doesNotMatch(source, /^\s*register_serializable_item::<Onboarding>\(cx\);/m);
  assert.doesNotMatch(source, /CompleteOnboarding/);
  assert.doesNotMatch(source, /fn render_completion_control\(/);
  assert.doesNotMatch(source, /complete-dx-onboarding/);
  assert.match(source, /DxLaunchPreviewTargets::local_web_preview_onboarding\(web_preview_onboarding_url\(\)\)/);
  assert.match(source, /fn handle_finish\(&mut self, _: &Finish, window: &mut Window, cx: &mut Context<Self>\)/);
  assert.match(source, /finish_setup\(self\.workspace\.clone\(\), window, cx\);/);
  assert.match(source, /fn finish_setup<C: AppContext>\(/);
  const finishSetup = functionBody(source, "finish_setup");
  assert.match(
    finishSetup,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return;\s*\}/s,
    "Finish action must not enter the old WebPreview handoff while onboarding is parked",
  );
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
  assert.doesNotMatch(ensurePreview, /completion_workspace|complete_onboarding|finish_setup/);
  assert.match(
    ensurePreview,
    /WebPreviewView::new_for_onboarding\([\s\S]*None,[\s\S]*window,[\s\S]*cx,[\s\S]*\)/,
    "onboarding Web Preview must remain visual-only; native GPUI owns completion",
  );
  assert.match(source, /fn close_onboarding_page<C: AppContext>\(/);
  assert.match(source, /fn find_post_onboarding_item\(workspace: &Workspace, cx: &App\) -> Option<Box<dyn ItemHandle>>/);
  const closeOnboardingPage = functionBody(source, "close_onboarding_page");
  assert.match(
    closeOnboardingPage,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return;\s*\}/s,
    "close handoff must be unreachable while the WebPreview onboarding is parked",
  );
  assert.match(closeOnboardingPage, /workspace\.update\(cx, \|workspace, cx\|/);
  assert.doesNotMatch(closeOnboardingPage, /with_active_or_new_workspace/);
  assert.match(
    closeOnboardingPage,
    /if !mark_onboarding_completion_requested\(workspace, cx\) \{\s*return;\s*\}/s,
    "duplicate completion signals should not create replacement editors while completion is already in progress",
  );
  assert.match(
    closeOnboardingPage,
    /complete_onboarding_handoff\(workspace, window, cx\);/,
  );
  const completeOnboardingHandoff = functionBody(source, "complete_onboarding_handoff");
  assert.match(
    completeOnboardingHandoff,
    /if find_onboarding_page\(workspace, cx\)\.is_none\(\) \{\s*return;\s*\}/s,
    "completion should no-op after onboarding has already been removed",
  );
  assert.match(
    completeOnboardingHandoff,
    /reveal_onboarding_completion\(workspace, window, cx\);[\s\S]*?let post_onboarding_item = find_post_onboarding_item\(workspace, cx\);/s,
    "Complete must park and hide the native onboarding surface before editor creation can stall",
  );
  assert.match(
    completeOnboardingHandoff,
    /if let Some\(item\) = post_onboarding_item\.as_ref\(\) \{\s*workspace\.activate_item\(item\.as_ref\(\), true, true, window, cx\);[\s\S]*?\} else \{\s*let create_editor = Editor::new_in_workspace\(workspace, window, cx\);[\s\S]*?create_editor\.await/s,
    "Complete must still activate an existing item or create a real untitled editor after the handoff",
  );
  assert.doesNotMatch(
    completeOnboardingHandoff,
    /remove_onboarding_pages\(|pane\.remove_item\(/,
    "Complete must not destroy the native Web Preview item in the immediate completion path",
  );
  assert.doesNotMatch(
    completeOnboardingHandoff,
    /activate_screen_kind\(WorkspaceScreenKind::Editor/,
    "Complete must not rely on deferred NewFile dispatch before removing onboarding",
  );
  const markCompletionRequested = functionBody(source, "mark_onboarding_completion_requested");
  assert.match(
    markCompletionRequested,
    /if !onboarding\.completion_requested \{\s*onboarding\.completion_requested = true;\s*newly_requested = true;\s*\}/s,
    "host completion must be idempotent while editor creation is in flight",
  );
  const revealCompletion = functionBody(source, "reveal_onboarding_completion");
  assert.match(
    revealCompletion,
    /prepare_dx_web_preview_for_completion\(window, cx\);[\s\S]*?onboarding\.completion_revealed = true;/s,
    "completion should soft-handoff the native Web Preview before hiding the overlay",
  );
  assert.doesNotMatch(
    revealCompletion,
    /remove_onboarding_pages\(|pane\.remove_item\(/,
    "completion reveal must not physically remove the native Web Preview item",
  );
  assert.match(
    revealCompletion,
    /forget_completed_onboarding_page\(item_id\.as_u64\(\), workspace_id, cx\);/,
    "completion should remove any saved onboarding restore row after parking the native Web Preview",
  );
  const forgetCompletedOnboardingPage = functionBody(source, "forget_completed_onboarding_page");
  assert.match(
    forgetCompletedOnboardingPage,
    /delete_onboarding_page\(item_id, workspace_id\)\.await/,
    "completed onboarding should not be restored from the old persisted row",
  );
  assert.match(source, /pub async fn delete_onboarding_page\(/);
  assert.match(source, /DELETE FROM onboarding_pages\s+WHERE item_id = \? AND workspace_id = \?/);
  assert.match(
    source,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return None;\s*\}/,
    "disabled onboarding should never return the full-window overlay",
  );
  assert.doesNotMatch(
    source,
    /fn remove_onboarding_pages\(/,
    "completion should no longer physically remove the parked native Web Preview item",
  );
  assert.doesNotMatch(source, /window\.close|close_window|remove_window|cx\.quit|quit\(/);
  assert.match(source, /fn find_onboarding_page\(workspace: &Workspace, cx: &App\) -> Option<Entity<Onboarding>>/);
  assert.match(source, /workspace\.panes\(\)\.iter\(\)\.find_map/);
  const findPostOnboardingItem = functionBody(source, "find_post_onboarding_item");
  assert.match(findPostOnboardingItem, /item\.downcast::<Onboarding>\(\)\.is_some\(\)[\s\S]*?continue;/);
  assert.match(findPostOnboardingItem, /WorkspaceScreenKind::Editor => return Some\(item\.boxed_clone\(\)\)/);
  assert.match(findPostOnboardingItem, /fallback\.get_or_insert_with\(\|\| item\.boxed_clone\(\)\);/);
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
  const serializeOnboarding = functionBody(source, "serialize");
  assert.match(
    serializeOnboarding,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return None;\s*\}/,
    "disabled onboarding should not persist a parked Web Preview item",
  );
  const deserializeOnboarding = functionBody(source, "deserialize");
  assert.match(
    deserializeOnboarding,
    /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{[\s\S]*delete_onboarding_page\(item_id, workspace_id\)\.await\?/,
    "disabled onboarding should delete stale restore rows instead of restoring a WebPreview item",
  );
  const shouldSerializeOnboarding = functionBody(source, "should_serialize");
  assert.match(
    shouldSerializeOnboarding,
    /!DX_WEB_PREVIEW_ONBOARDING_DISABLED[\s\S]*&& !self\.completion_requested[\s\S]*&& event == &ItemEvent::UpdateTab/,
    "completed onboarding handoff should not advertise the parked item as serializable",
  );
  assert.match(source, /fn can_split\(&self\) -> bool \{\s+false\s+\}/);
  assert.match(
    paneSource,
    /mode == SplitMode::MovePane[\s\S]*?WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Onboarding[\s\S]*?return;/,
    "Onboarding, Automations, Connections, and Tools must not be movable into a split pane through MovePane commands",
  );
  assert.match(
    paneSource,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Onboarding/s,
    "Onboarding, Automations, Connections, and Tools must not expose clone or move split actions in pane chrome",
  );
  assert.match(
    source,
    /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s+WorkspaceScreenKind::Onboarding\s+\}/,
  );
  const workspaceOverlay = functionBody(source, "workspace_overlay");
  assert.match(workspaceOverlay, /if DX_WEB_PREVIEW_ONBOARDING_DISABLED \{\s*return None;\s*\}/s);
  assert.match(workspaceOverlay, /id\("onboarding-window-overlay"\)/);
  assert.match(workspaceOverlay, /\.child\(self\.render_web_preview_canvas\(window, cx\)\)/);
  assert.doesNotMatch(source, /fn workspace_overlay\([\s\S]*?\.child\(self\.render_completion_control\(cx\)\)/);
  assert.match(
    source,
    /fn requires_transparent_workspace_background\(\) -> bool \{\s*true\s*\}/,
    "Onboarding must keep the GPUI workspace transparent so the native Web Preview underlay is visible",
  );
  assert.match(
    workspaceSource,
    /pub fn active_full_window_overlay\([\s\S]*?WorkspaceScreenKind::Onboarding[\s\S]*?item\.workspace_overlay\(window, cx\)/,
    "Workspace should expose only Onboarding as a full-window overlay",
  );
  assert.match(
    multiWorkspaceSource,
    /let active_full_window_overlay = workspace\.update\(cx, \|workspace, cx\| \{\s*workspace\.active_full_window_overlay\(window, cx\)\s*\}\);/s,
    "MultiWorkspace should query the active workspace for an app-root overlay",
  );
  assert.match(
    multiWorkspaceSource,
    /if let Some\(active_full_window_overlay\) = active_full_window_overlay \{[\s\S]*return client_side_decorations_with_content_flush\([\s\S]*\.child\(active_full_window_overlay\)[\s\S]*\.child\(render_full_window_overlay_system_controls\(window, cx\)\)[\s\S]*Tiling \{\s*top: true,\s*left: true,\s*right: true,\s*bottom: true,\s*\}[\s\S]*Tiling \{\s*top: true,\s*left: true,\s*right: true,\s*bottom: true,\s*\}/s,
    "Onboarding should short-circuit normal workspace chrome, fill the full client window, and keep only system window controls above it",
  );
  const fullWindowControls = functionBody(
    multiWorkspaceSource,
    "render_full_window_overlay_system_controls",
  );
  assert.doesNotMatch(fullWindowControls, /full_window_overlay_complete_button\(\)/);
  assert.match(fullWindowControls, /WindowControlArea::Drag/);
  assert.match(fullWindowControls, /WindowControlArea::Min/);
  assert.match(fullWindowControls, /WindowControlArea::Max/);
  assert.match(fullWindowControls, /WindowControlArea::Close/);
  assert.doesNotMatch(fullWindowControls, /TitleBar|render_screen_dock|application_menu|titlebar_item/);
  assert.doesNotMatch(multiWorkspaceSource, /CompleteOnboarding|full_window_overlay_complete_button|full-window-overlay-complete-onboarding/);
  assert.match(
    workspaceSource,
    /if kind == WorkspaceScreenKind::Onboarding \{[\s\S]*TODO\(dx-onboarding\)[\s\S]*return false;\s*\}/s,
    "screen dock activation must not route onboarding while the fullscreen WebPreview is parked",
  );
  assert.doesNotMatch(workspaceSource, /if kind == WorkspaceScreenKind::Onboarding \{[\s\S]*OpenOnboarding/s);
  assert.doesNotMatch(paneSource, /Open Onboarding|OpenOnboarding\.boxed_clone\(\)/);
  assert.doesNotMatch(titleBarSource, /WorkspaceScreenKind::Onboarding => "Onboarding",/);
  assert.doesNotMatch(
    read("crates/workspace/src/screen_carousel.rs"),
    /WorkspaceScreenKind::Onboarding => "Onboarding",/,
  );
  assert.match(
    agentPanelSource,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Terminal\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::LiquidGlass/s,
    "Agent workspace snapshots should treat Onboarding and Automations like other non-editor screens",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Onboarding => Self::Other/,
    "sidebar grid persistence should deliberately fold Onboarding into the existing non-primary screen context",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Editor\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::LiquidGlass\s*\|\s*WorkspaceScreenKind::Other => self\.project_root_path\(cx\)/s,
    "sidebar grid context should use project-root shortcuts while Onboarding or Automations are active",
  );
  assert.match(
    sidebarSource,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Editor\s*\|\s*WorkspaceScreenKind::Onboarding\s*\|\s*WorkspaceScreenKind::Other => (?:self\.editor_grid_entries\(cx\)|\{\s*self\.editor_grid_entries\(cx\)\s*\})/s,
    "sidebar grid generation should show editor/project entries while Onboarding or Automations are active",
  );

  const renderStart = source.indexOf("impl Render for Onboarding");
  assert.ok(renderStart >= 0, "expected Onboarding render impl");
  const renderBody = functionBody(source.slice(renderStart), "render");
  assert.doesNotMatch(
    renderBody,
    /render_web_preview_canvas\(window, cx\)/,
    "Onboarding pane render must not mount the native Web Preview below GPUI chrome",
  );
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
    assert.match(source, /onboarding_completion_handoff: bool,/);
    assert.match(source, /pub fn prepare_for_onboarding_completion\(/);
    const prepareCompletion = functionBody(source, "prepare_for_onboarding_completion");
    assert.match(prepareCompletion, /self\.onboarding_completion_handoff = true;/);
    assert.match(prepareCompletion, /self\.is_active_item = false;/);
    assert.doesNotMatch(
      prepareCompletion,
      /release_native_preview_focus\(|set_visible\(false\)|reset_native_preview|Close\(/,
      `${name} onboarding completion should not release focus, hide, reset, or close the native Web Preview`,
    );
    if (name === "Windows") {
      assert.match(source, /fn park_for_onboarding_completion\(&self\) -> Result<\(\)>/);
      assert.match(read("crates/web_preview/src/windows_visual_webview.rs"), /pub\(crate\) fn park_composition_visual_for_handoff\(&self\) -> Result<\(\)>/);
      assert.match(
        prepareCompletion,
        /preview\.park_for_onboarding_completion\(\)/,
        "Windows onboarding completion should move the composition visual offscreen instead of closing WebView2",
      );
    }
    assert.match(source, /"onboarding-complete" => \{/);
    const ipcHandler = functionBody(source, "handle_ipc_message");
    assert.match(ipcHandler, /if let Some\(complete\) = self\.onboarding_complete\.clone\(\)/);
    assert.match(
      ipcHandler,
      /cx\.defer_in\(window, move \|_, window, cx\| \{\s*complete\(window, cx\);\s*\}\);/s,
      `${name} should defer onboarding completion until after native Web Preview IPC returns`,
    );
    assert.doesNotMatch(
      ipcHandler,
      /if let Some\(complete\) = self\.onboarding_complete\.clone\(\) \{\s*complete\(window, cx\);/s,
      `${name} should not remove onboarding synchronously inside the Web Preview IPC callback`,
    );
    assert.doesNotMatch(
      ipcHandler,
      /remove_onboarding_pages|pane\.remove_item|window\.close\(|close_window|remove_window|cx\.quit|quit\(|(?:std::)?process::exit/,
      `${name} IPC completion should only dispatch the callback, not destroy the WebView or close the app`,
    );
    const newForOnboarding = functionBody(source, "new_for_onboarding");
    assert.match(newForOnboarding, /let workspace_context = Self::fallback_workspace_context\(\);/);
    assert.doesNotMatch(newForOnboarding, /workspace\.read\(cx\)|Self::workspace_context/);
    assert.match(newForOnboarding, /Self::new_for_url\([\s\S]*onboarding_complete/s);
    const newForUrl = functionBody(source, "new_for_url");
    assert.match(newForUrl, /onboarding_complete,/);
    assert.match(newForUrl, /onboarding_completion_handoff: false,/);
    const cloneOnSplit = functionBody(source, "clone_on_split");
    assert.match(
      cloneOnSplit,
      /onboarding_completion_handoff: false,/,
      `${name} split clones must start with a fresh inactive onboarding handoff flag`,
    );
    const deactivated = functionBody(source, "deactivated");
    const workspaceDeactivated = functionBody(source, "workspace_deactivated");
    assert.match(deactivated, /if self\.onboarding_completion_handoff \{\s*return;\s*\}/);
    assert.match(workspaceDeactivated, /if self\.onboarding_completion_handoff \{\s*return;\s*\}/);
    assert.match(source, /fn is_onboarding_complete_fallback_url\(url: &str\) -> bool/);
    assert.match(source, /url == "about:blank#zed-onboarding-complete"/);
    assert.match(source, /url\.ends_with\("#zed-onboarding-complete"\)/);
    assert.match(source, /fn is_onboarding_complete_fallback_title\(title: &str\) -> bool/);
    assert.match(source, /title == "zed-onboarding-complete"/);
    const applyBrowserEvents = functionBody(source, "apply_browser_events");
    assert.match(applyBrowserEvents, /BrowserEvent::UrlChanged\(url\) => \{/);
    assert.match(
      applyBrowserEvents,
      /self\.onboarding_complete\.is_some\(\)[\s\S]*?is_onboarding_complete_fallback_url\(url\.as_str\(\)\)[\s\S]*?cx\.defer_in\(window, move \|_, window, cx\| \{\s*complete\(window, cx\);\s*\}\);[\s\S]*?continue;/s,
      `${name} should close onboarding when the page navigates to the completion fallback URL`,
    );
    assert.match(applyBrowserEvents, /BrowserEvent::TitleChanged\(title\) => \{/);
    assert.match(
      applyBrowserEvents,
      /self\.onboarding_complete\.is_some\(\)[\s\S]*?is_onboarding_complete_fallback_title\(title\.as_str\(\)\)[\s\S]*?cx\.defer_in\(window, move \|_, window, cx\| \{\s*complete\(window, cx\);\s*\}\);[\s\S]*?continue;/s,
      `${name} should close onboarding when the page emits the completion fallback title`,
    );
    const syncActivation = functionBody(source, "sync_native_preview_window_activation");
    assert.match(syncActivation, /try_borrow_mut\(\)/);
    assert.doesNotMatch(syncActivation, /native_preview\.borrow\(\)\.is_none\(\)/);
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

for (const [name, path] of desktopOnboardingPreviewViews) {
  test(`${name} web preview exposes a startup hook without opening anything eagerly`, () => {
    const source = read(path);
    const startup = functionBody(source, "ensure_startup_preview");

    assert.match(source, /pub fn ensure_startup_preview\(\s*workspace: &mut Workspace,/);
    assert.match(startup, /let _ = \(workspace, window, cx\);/);
    assert.doesNotMatch(
      startup,
      /NewWebPreview|OpenDxWwwPreview|OpenBundledDxPreview|new_for_url|open_url|navigate|load_url|dispatch_action|cx\.spawn|cx\.background_spawn|workspace\.(add|activate|open)/,
    );
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

test("Windows Web Preview favicon updates are bounded and page-scoped", () => {
  const source = read("crates/web_preview/src/web_preview_view.rs");
  const windowsHost = read("crates/web_preview/src/windows_visual_webview.rs");
  const eventEnum = source.slice(
    source.indexOf("pub(crate) enum BrowserEvent"),
    source.indexOf("pub struct WebPreviewView"),
  );
  const updateForPage = functionBody(source, "update_favicon_uri_for_page");
  const pageMatch = functionBody(source, "favicon_page_url_matches_active_url");
  const updateUri = functionBody(source, "update_favicon_uri");
  const cacheUri = functionBody(source, "cache_favicon_uri");
  const validateUri = functionBody(source, "validated_favicon_uri");
  const cacheTask = functionBody(source, "cache_web_preview_favicon_uri");
  const downloadBytes = functionBody(source, "download_favicon_bytes");
  const readFileBytes = functionBody(source, "read_favicon_file_bytes");
  const imageCheck = functionBody(source, "favicon_bytes_look_like_image");
  const writeCache = functionBody(source, "write_cached_favicon");
  const applyBrowserEvents = functionBody(source, "apply_browser_events");
  const handleIpc = functionBody(source, "handle_ipc_message");
  const tabIcon = functionBody(source, "tab_icon");
  const cloneOnSplit = functionBody(source, "clone_on_split");
  const requestFavicon = functionBody(windowsHost, "request_favicon_uri");

  assert.match(source, /const FAVICONS_DIR_NAME: &str = "favicons";/);
  assert.match(source, /const MAX_WEB_PREVIEW_FAVICON_URI_BYTES: usize = 4096;/);
  assert.match(source, /const MAX_WEB_PREVIEW_FAVICON_IMAGE_BYTES: usize = 512 \* 1024;/);
  assert.match(eventEnum, /FaviconUriChanged\s*\{\s*uri: String,\s*page_url: Option<String>,\s*\}/);
  assert.match(source, /favicon_uri: Option<SharedString>/);
  assert.match(source, /favicon_image_path: Option<SharedString>/);

  assert.match(updateForPage, /favicon_page_url_matches_active_url\(page_url\)/);
  assert.match(updateForPage, /favicon_page_allows_file_uri\(page_url\)/);
  assert.match(updateForPage, /return false;/);
  assert.match(pageMatch, /page_url\.is_empty\(\) \|\| page_url\.len\(\) > MAX_WEB_PREVIEW_FAVICON_URI_BYTES/);
  assert.match(
    pageMatch,
    /display_url_for_loaded_url\(page_url, source_apply_session_active\)\s*== self\.active_url\.as_ref\(\)/,
  );
  assert.match(updateUri, /validated_favicon_uri\(uri\.as_str\(\), allow_file_uri\)/);
  assert.match(updateUri, /favicon_cache_file_path\(&cache_dir, uri\.as_str\(\)\)/);
  assert.match(updateUri, /cached_path\.exists\(\)/);
  assert.match(updateUri, /self\.cache_favicon_uri\(uri, allow_file_uri, cx\);/);
  assert.match(cacheUri, /cx\.background_spawn\(cache_web_preview_favicon_uri/);
  assert.match(cacheUri, /allow_file_uri/);
  assert.match(cacheUri, /cx\.emit\(ItemEvent::UpdateTab\);/);

  assert.match(validateUri, /uri\.trim\(\)/);
  assert.match(validateUri, /uri\.is_empty\(\) \|\| uri\.len\(\) > MAX_WEB_PREVIEW_FAVICON_URI_BYTES/);
  assert.match(validateUri, /"http" \| "https" => Some\(parsed\.to_string\(\)\)/);
  assert.match(validateUri, /"file" if allow_file_uri => Some\(parsed\.to_string\(\)\)/);
  assert.doesNotMatch(validateUri, /data/);
  assert.match(source, /fn favicon_page_allows_file_uri\(page_url: Option<&str>\) -> bool/);
  assert.match(source, /page_url\.scheme\(\) == "file"/);
  assert.match(cacheTask, /validated_favicon_uri\(uri\.as_str\(\), allow_file_uri\)/);
  assert.match(cacheTask, /"http" \| "https" => download_favicon_bytes/);
  assert.match(cacheTask, /"file" =>/);
  assert.match(cacheTask, /favicon_bytes_look_like_image\(&bytes\)/);
  assert.match(downloadBytes, /\.take\(\(MAX_WEB_PREVIEW_FAVICON_IMAGE_BYTES \+ 1\) as u64\)/);
  assert.match(readFileBytes, /metadata\.len\(\) > MAX_WEB_PREVIEW_FAVICON_IMAGE_BYTES as u64/);
  assert.match(readFileBytes, /\.take\(\(MAX_WEB_PREVIEW_FAVICON_IMAGE_BYTES \+ 1\) as u64\)/);
  assert.match(imageCheck, /image::guess_format\(bytes\)\.is_ok\(\)/);
  assert.match(imageCheck, /prefix\.contains\("<svg"\)/);
  assert.match(writeCache, /fs::write\(&temp_path, bytes\)/);
  assert.match(writeCache, /fs::rename\(&temp_path, cache_path\)/);
  assert.match(source, /fn favicon_cache_file_path\(cache_dir: &Path, uri: &str\) -> PathBuf/);
  assert.match(source, /fn fnv1a64\(bytes: &\[u8\]\) -> u64/);

  assert.match(applyBrowserEvents, /BrowserEvent::FaviconUriChanged \{ uri, page_url \} => \{/);
  assert.match(applyBrowserEvents, /self\.update_favicon_uri_for_page\(uri, page_url\.as_deref\(\), cx\)/);
  assert.match(applyBrowserEvents, /BrowserEvent::NavigationStarted[\s\S]*self\.clear_favicon\(\)/);
  assert.match(handleIpc, /"favicon-uri" => \{/);
  assert.match(handleIpc, /payload\.get\("page_url"\)\.and_then\(Value::as_str\)/);
  assert.match(tabIcon, /self\.project_item\.is_none\(\)/);
  assert.match(tabIcon, /ui::Icon::from_path\(path\.clone\(\)\)/);
  assert.match(cloneOnSplit, /favicon_uri,/);
  assert.match(cloneOnSplit, /favicon_image_path,/);
  assert.match(source, /post\(\{ kind: "favicon-uri", uri, page_url: window\.location\.href, reason \}\);/);
  assert.match(source, /new MutationObserver\(\(\) => \{/);
  assert.match(source, /\(!\/\^file:\/i\.test\(window\.location\.href\) && \/\^file:\/i\.test\(uri\)\)/);

  assert.match(windowsHost, /const FAVICON_URI_SCRIPT: &str = r#"/);
  assert.match(windowsHost, /\(!\/\^file:\/i\.test\(window\.location\.href\) && \/\^file:\/i\.test\(uri\)\)/);
  assert.match(windowsHost, /request_favicon_uri\(webview, event_queue\.clone\(\), current_url\);/);
  assert.match(requestFavicon, /webview\.ExecuteScript\(&script, &handler\)/);
  assert.match(requestFavicon, /serde_json::from_str::<Option<String>>\(result\.as_str\(\)\)/);
  assert.match(requestFavicon, /BrowserEvent::FaviconUriChanged/);
});

for (const [name, path] of desktopOnboardingPreviewViews) {
  test(`${name} web preview shows a real loading spinner placeholder`, () => {
    const source = read(path);

    assert.match(source, /PreviewLoadState::Loading\s*=>\s*None/);
    assert.match(source, /let show_loading_placeholder =/);
    assert.match(source, /let loading_placeholder = show_loading_placeholder\.then/);
    assert.match(source, /dx_loading_icon\(IconSize::Small,\s*Color::Muted,\s*2\)/);
    assert.doesNotMatch(source, /IconName::LoadCircle/);
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
