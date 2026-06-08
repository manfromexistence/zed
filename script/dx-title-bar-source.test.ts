import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/title_bar/src/application_menu.rs";
const source = readFileSync(sourcePath, "utf8").replace(/\r\n/g, "\n");
const titleBarPath = "crates/title_bar/src/title_bar.rs";
const titleBarSource = readFileSync(titleBarPath, "utf8").replace(/\r\n/g, "\n");
const popoverMenuPath = "crates/ui/src/components/popover_menu.rs";
const popoverMenuSource = readFileSync(popoverMenuPath, "utf8").replace(/\r\n/g, "\n");

test("application menu activation checks stale entry indexes before handle use", () => {
  const activation = functionBody("navigate_menus_in_direction");

  assert.doesNotMatch(
    activation,
    /self\s*\.\s*entries\s*\[\s*current_index\s*\]/,
    "application menu activation must not direct-index the current entry",
  );
  assert.doesNotMatch(
    activation,
    /self\s*\.\s*entries\s*\[\s*next_index\s*\]/,
    "application menu activation must not direct-index the next entry",
  );
  assert.match(
    activation,
    /let\s+Some\(current_entry\)\s*=\s*self\s*\.\s*entries\s*\.\s*get\(\s*current_index\s*\)\s*else\s*\{\s*return;\s*\};/s,
    "application menu activation must check the current entry before hiding it",
  );
  assert.match(
    activation,
    /let\s+Some\(next_entry\)\s*=\s*self\s*\.\s*entries\s*\.\s*get\(\s*next_index\s*\)\s*else\s*\{\s*return;\s*\};/s,
    "application menu activation must check the next entry before cloning its handle",
  );
  assert.match(activation, /current_entry\.handle\.hide\(cx\);/);
  assert.match(
    activation,
    /let\s+next_handle\s*=\s*next_entry\.handle\.clone\(\);/,
  );
});

test("application menu hover closes deployed popovers after leaving the menu area", () => {
  const standardMenu = functionBody("render_standard_menu");

  assert.match(standardMenu, /\.on_hover\(move \|hover_enter, window, cx\| \{/);
  assert.match(standardMenu, /if \*hover_enter && !current_handle\.is_deployed\(\)/);
  assert.match(standardMenu, /else if !\*hover_enter/);
  assert.match(
    standardMenu,
    /Self::schedule_hover_away_close\(all_handles\.clone\(\), window, cx\)/,
  );
  const hoverAwayClose = functionBody("schedule_hover_away_close");
  assert.match(hoverAwayClose, /window\.on_next_frame\(move \|window, cx\| \{/);
  assert.match(hoverAwayClose, /handle\.is_deployed\(\) && handle\.is_pointer_near\(window, px\(18\.0\)\)/);
  assert.match(hoverAwayClose, /Self::schedule_hover_away_close\(handles, window, cx\)/);
  assert.match(hoverAwayClose, /handle\.hide\(cx\);/);
  assert.match(popoverMenuSource, /pub fn is_pointer_near\(&self, window: &Window, padding: Pixels\) -> bool/);
  assert.match(popoverMenuSource, /trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>/);
  assert.match(popoverMenuSource, /menu_bounds: Rc<Cell<Option<Bounds<Pixels>>>>/);
  assert.match(popoverMenuSource, /bounds\.dilate\(padding\)\.contains\(&position\)/);
  assert.match(popoverMenuSource, /element_state\.trigger_bounds\.set\(Some\(bounds\)\)/);
  assert.match(popoverMenuSource, /element_state\.menu_bounds\.set\(/);
});

test("title bar screen and right-tool buttons use domain-specific icons", () => {
  const screenDock = functionBodyFrom(titleBarSource, "render_screen_dock");
  const agentScreenButton = functionBodyFrom(titleBarSource, "render_agent_screen_button");
  const agentScreenActive = functionBodyFrom(titleBarSource, "agent_screen_is_active");
  const agentButtonIndex = screenDock.indexOf("render_agent_screen_button(agent_screen_is_active, cx)");
  const automationButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Automations", agentButtonIndex);
  const connectionsButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Connections", automationButtonIndex);
  const toolsButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Tools", connectionsButtonIndex);
  const editorButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Editor", toolsButtonIndex);
  const browserButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Browser", editorButtonIndex);
  const terminalButtonIndex = screenDock.indexOf("WorkspaceScreenKind::Terminal", browserButtonIndex);
  assert.ok(agentButtonIndex >= 0, "screen dock should render the AI button");
  assert.ok(automationButtonIndex > agentButtonIndex, "Automations should follow AI in the screen dock");
  assert.ok(connectionsButtonIndex > automationButtonIndex, "Connections should follow Automations in the screen dock");
  assert.ok(toolsButtonIndex > connectionsButtonIndex, "Tools should follow Connections in the screen dock");
  assert.ok(editorButtonIndex > toolsButtonIndex, "Editor should follow Tools in the screen dock");
  assert.ok(browserButtonIndex > editorButtonIndex, "Browser should follow Editor in the screen dock");
  assert.ok(terminalButtonIndex > browserButtonIndex, "Terminal should follow Browser in the screen dock");
  assert.doesNotMatch(
    screenDock,
    /render_screen_kind_button\(\s*WorkspaceScreenKind::Onboarding/,
    "fullscreen onboarding WebPreview is disabled for now",
  );
  assert.match(
    screenDock,
    /\.child\(self\.render_agent_screen_button\(agent_screen_is_active, cx\)\)[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Automations,[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Connections,[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Tools,[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Editor,[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Browser,[\s\S]*?\.child\(self\.render_screen_kind_button\(\s*WorkspaceScreenKind::Terminal,[\s\S]*?\.children\(\s*extra_entries/s,
    "primary screen dock buttons must stay AI, Automations, Connections, Tools, Editor, Browser, Terminal before overflow entries",
  );
  assert.match(screenDock, /let agent_screen_is_active = self\.agent_screen_is_active\(cx\);/);
  assert.match(screenDock, /&& !agent_screen_is_active/);
  assert.match(
    screenDock,
    /WorkspaceScreenKind::Automations,\s*!agent_screen_is_active\s*&& active_screen_kind == WorkspaceScreenKind::Automations/s,
    "automation dock button must not stay active while fullscreen AI is active",
  );
  assert.match(
    screenDock,
    /WorkspaceScreenKind::Editor,\s*!agent_screen_is_active\s*&& active_screen_kind == WorkspaceScreenKind::Editor/s,
    "editor dock button must not stay active while fullscreen AI is active",
  );
  assert.match(
    screenDock,
    /WorkspaceScreenKind::Browser,\s*!agent_screen_is_active\s*&& active_screen_kind == WorkspaceScreenKind::Browser/s,
    "browser dock button must not stay active while fullscreen AI is active",
  );
  assert.match(
    screenDock,
    /WorkspaceScreenKind::Terminal,\s*!agent_screen_is_active\s*&& active_screen_kind == WorkspaceScreenKind::Terminal/s,
    "terminal dock button must not stay active while fullscreen AI is active",
  );
  assert.doesNotMatch(
    titleBarSource,
    /WorkspaceScreenKind::Onboarding => \{\s*window\.dispatch_action\(zed_actions::OpenOnboarding\.boxed_clone\(\), cx\);\s*\}/s,
    "disabled onboarding should not dispatch from titlebar screen controls",
  );
  assert.doesNotMatch(
    titleBarSource,
    /\.action\("Onboarding", zed_actions::OpenOnboarding\.boxed_clone\(\)\)/,
    "disabled onboarding should not appear in the screen dock add menu",
  );
  assert.match(
    titleBarSource,
    /WorkspaceScreenKind::Browser => dx_icon\(DxUiIcon::Browser\)/,
    "Browser screen dock button should use the preview/browser tool icon",
  );
  assert.match(
    agentScreenButton,
    /"screen-dock-agent",\s*dx_icon\(DxUiIcon::Agent\)/s,
    "screen dock should expose a real AI button",
  );
  assert.match(
    agentScreenButton,
    /zed_actions::assistant::FocusAgentFullscreen\.boxed_clone\(\)/,
    "AI screen dock button should open the real Agent panel fullscreen action",
  );
  assert.match(
    titleBarSource,
    /fn render_agent_screen_button\(&self, selected: bool, _cx: &mut Context<Self>\)/,
    "screen-dock Agent button should not leave warning-grade unused parameters",
  );
  assert.match(agentScreenButton, /toggle_state\(selected\)/);
  assert.match(titleBarSource, /fn agent_screen_is_active\(&self, cx: &App\) -> bool/);
  assert.match(agentScreenActive, /self\.active_screen_kind\(cx\) == WorkspaceScreenKind::Agent/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Agent => "AI"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Automations => "Automations"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Connections => "Connections"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Tools => "Tools"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Agent => dx_icon\(DxUiIcon::Agent\)/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Automations => dx_icon\(DxUiIcon::Automations\)/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Connections => dx_icon\(DxUiIcon::Connections\)/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Tools => dx_icon\(DxUiIcon::Plugins\)/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Agent => "Open AI Screen"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Automations => "Open Automations"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Connections => "Open Connections"/);
  assert.match(titleBarSource, /WorkspaceScreenKind::Tools => "Open Tools"/);
  assert.match(titleBarSource, /zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.match(titleBarSource, /zed_actions::assistant::OpenConnections\.boxed_clone\(\)/);
  assert.match(titleBarSource, /zed_actions::assistant::OpenTools\.boxed_clone\(\)/);
  assert.doesNotMatch(agentScreenActive, /dock_at_position|visible_panel|agent_panel_is_active/);
  assert.doesNotMatch(agentScreenActive, /zoomed_is_agent_panel/);
  assert.doesNotMatch(titleBarSource, /fn agent_panel_is_active/);
  assert.match(
    titleBarSource,
    /"titlebar-shadcn-ui-panel",\s*dx_icon\(DxUiIcon::Ui\),\s*"UI"/s,
    "UI panel titlebar button should use the component blocks icon",
  );
  assert.doesNotMatch(
    titleBarSource,
    /"titlebar-dx-style-panel",\s*IconName::Sliders,\s*"Style"/s,
    "Style panel titlebar button is parked until the Web Preview workflow is production-ready",
  );
  assert.doesNotMatch(titleBarSource, /WorkspaceScreenKind::Browser => IconName::Public/);
});

test("title-bar source guard stays scoped to worker-owned files", () => {
  assert.equal(sourcePath, "crates/title_bar/src/application_menu.rs");
  assert.equal(titleBarPath, "crates/title_bar/src/title_bar.rs");
  assert.equal(popoverMenuPath, "crates/ui/src/components/popover_menu.rs");
  assert.doesNotMatch(sourcePath, /test/i);
  assert.doesNotMatch(titleBarPath, /test/i);
  assert.doesNotMatch(popoverMenuPath, /test/i);
});

function functionBody(name: string): string {
  return functionBodyFrom(source, name);
}

function functionBodyFrom(sourceText: string, name: string): string {
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

  assert.fail(`expected ${name} body to close`);
}
