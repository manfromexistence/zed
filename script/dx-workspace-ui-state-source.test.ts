import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const dock = read("crates/workspace/src/dock.rs");
const defaultSettings = read("assets/settings/default.json");
const agentSettings = read("crates/agent_settings/src/agent_settings.rs");
const agentProfileSettings = read("crates/agent_settings/src/agent_profile.rs");
const agentThread = read("crates/agent/src/thread.rs");
const historyManager = read("crates/workspace/src/history_manager.rs");
const item = read("crates/workspace/src/item.rs");
const pane = read("crates/workspace/src/pane.rs");
const workspace = read("crates/workspace/src/workspace.rs");
const multiWorkspace = read("crates/workspace/src/multi_workspace.rs");
const titleBar = read("crates/title_bar/src/title_bar.rs");
const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
const agentScreen = read("crates/agent_ui/src/agent_screen.rs");
const automationScreen = read("crates/agent_ui/src/automation_screen.rs");
const conversationView = read("crates/agent_ui/src/conversation_view.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const composerProfileOptions = read(
  "crates/agent_ui/src/conversation_view/composer_profile_options.rs",
);
const profileSelector = read("crates/agent_ui/src/profile_selector.rs");
const manageProfilesModal = read("crates/agent_ui/src/agent_configuration/manage_profiles_modal.rs");
const dxLaunchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
const dxAgentWorkspace = read("crates/agent_ui/src/dx_launch_workspace/agent_workspace.rs");
const dxLaunchAuditSummary = read("crates/agent_ui/src/dx_launch_workspace/audit/summary.rs");
const dxLaunchAuditStatus = read("crates/agent_ui/src/dx_launch_workspace/audit/status.rs");
const dxLaunchAuditWarnings = read("crates/agent_ui/src/dx_launch_workspace/audit/warnings.rs");
const dxLaunchContracts = read("crates/agent_ui/src/dx_launch_workspace/contracts.rs");
const dxLaunchContractStatus = read("crates/agent_ui/src/dx_launch_workspace/contracts/status.rs");
const dxLaunchReadinessExamples = read("crates/agent_ui/src/dx_launch_workspace/readiness/examples.rs");
const dxLaunchReadinessStatus = read("crates/agent_ui/src/dx_launch_workspace/readiness/status.rs");
const dxLaunchReadinessWarnings = read("crates/agent_ui/src/dx_launch_workspace/readiness/warnings.rs");
const dxLaunchWwwWarnings = read("crates/agent_ui/src/dx_launch_workspace/www_evidence/warnings.rs");
const dxAgentBridgeWarnings = read(
  "crates/agent_ui/src/dx_launch_workspace/agents/bridge/review/warnings.rs",
);
const dxAgentSocialActions = read(
  "crates/agent_ui/src/dx_launch_workspace/agents/social_actions.rs",
);
const dxLaunchSources = read("crates/agent_ui/src/dx_launch_workspace/sources.rs");
const dxLaunchSourceController = read(
  "crates/agent_ui/src/dx_launch_workspace/sources/controller.rs",
);
const dxLaunchSourceRows = read("crates/agent_ui/src/dx_launch_workspace/sources/rows.rs");
const dxLaunchSourceAttachments = read(
  "crates/agent_ui/src/dx_launch_workspace/sources/attachments.rs",
);
const dxLaunchSourceReceipts = read(
  "crates/agent_ui/src/dx_launch_workspace/sources/receipts.rs",
);
const dxLaunchStylePanel = read("crates/agent_ui/src/dx_launch_workspace/style_panel.rs");
const dxLaunchCheckPanel = read("crates/agent_ui/src/dx_launch_workspace/check.rs");
const dxLaunchStatusSummary = read(
  "crates/agent_ui/src/dx_launch_workspace/launch_status/summary.rs",
);
const dxSourceSets = read("crates/agent_ui/src/dx_source_sets.rs");
const dxSourceSetFormatting = read(
  "crates/agent_ui/src/dx_source_sets/formatting.rs",
);
const dxSourceSetDxEditorToolchain = read(
  "crates/agent_ui/src/dx_source_sets/dx_editor_toolchain.rs",
);
const dxCheckScore = read("crates/agent_ui/src/dx_check_score.rs");
const dxStylePanelCards = read("crates/agent_ui/src/dx_style_panel/panel_cards.rs");
const agentConfiguration = read("crates/agent_ui/src/agent_configuration.rs");
const sidebar = read("crates/sidebar/src/sidebar.rs");
const threadItem = read("crates/ui/src/components/ai/thread_item.rs");
const projectPanel = read("crates/project_panel/src/project_panel.rs");
const gitPanel = read("crates/git_ui/src/git_panel.rs");
const outlinePanel = read("crates/outline_panel/src/outline_panel.rs");
const collabPanel = read("crates/collab_ui/src/collab_panel.rs");
const iconPicker = read("crates/icon_picker/src/icon_picker.rs");
const fontPanel = read("crates/font_panel/src/font_panel.rs");
const mediaPanel = read("crates/media_panel/src/media_panel.rs");
const uiPanel = read("crates/shadcn_ui_panel/src/shadcn_ui_panel.rs");
const stylePanel = read("crates/agent_ui/src/dx_style_panel/panel_view.rs");

const functionBody = (source: string, name: string) => {
  const start = source.indexOf(`fn ${name}(`);
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = source.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
};

const assertBefore = ({
  body,
  before,
  after,
  message,
}: {
  body: string;
  before: string | RegExp;
  after: string | RegExp;
  message: string;
}) => {
  const indexOfPattern = (pattern: string | RegExp) => {
    if (typeof pattern === "string") {
      return body.indexOf(pattern);
    }
    return body.match(pattern)?.index ?? -1;
  };

  const beforeIndex = indexOfPattern(before);
  const afterIndex = indexOfPattern(after);
  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
};

const sourceWindow = (
  source: string,
  needle: string,
  before = 600,
  after = 600,
) => {
  const index = source.indexOf(needle);
  assert.ok(index >= 0, `missing ${needle}`);
  return source.slice(Math.max(0, index - before), index + needle.length + after);
};

const sourceBetween = (source: string, startNeedle: string, endNeedle: string) => {
  const start = source.indexOf(startNeedle);
  assert.ok(start >= 0, `missing ${startNeedle}`);
  const end = source.indexOf(endNeedle, start + startNeedle.length);
  assert.ok(end > start, `missing ${endNeedle}`);
  return source.slice(start, end);
};

const objectBlock = (source: string, key: string, fromIndex = 0) => {
  const keyIndex = source.indexOf(`"${key}": {`, fromIndex);
  assert.ok(keyIndex >= 0, `expected object key ${key}`);

  const bodyStart = source.indexOf("{", keyIndex);
  assert.ok(bodyStart > keyIndex, `expected object body for ${key}`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(keyIndex, index + 1);
      }
    }
  }

  assert.fail(`expected object ${key} to close`);
};

const repeatedStringIds = (ids: string[]) => {
  const seen = new Set<string>();
  const repeated = new Set<string>();

  for (const id of ids) {
    if (seen.has(id)) {
      repeated.add(id);
    }
    seen.add(id);
  }

  return [...repeated].sort();
};

test("history entries cap workspace path materialization before collection", () => {
  assert.match(historyManager, /const MAX_HISTORY_ENTRY_PATHS: usize = 32;/);

  const entryImpl = historyManager.slice(
    historyManager.indexOf("impl HistoryManagerEntry"),
  );
  const newEntry = functionBody(entryImpl, "new");
  assert.doesNotMatch(
    newEntry,
    /ordered_paths\(\)[\s\S]*collect::<SmallVec/,
    "history entries must not use PathList::ordered_paths because it sorts before caller caps",
  );
  assertBefore({
    body: newEntry,
    before: /paths\.paths\(\)\.len\(\)\.min\(MAX_HISTORY_ENTRY_PATHS\)/,
    after: "path.push(source_path.compact());",
    message: "history path list must be capped before SmallVec push materialization",
  });
  assert.match(newEntry, /workspace history entry path list is too large/);
  assert.match(newEntry, /\.log_err\(\)/);
});

test("history deletion lists fail closed before delete-id collection", () => {
  assert.match(
    historyManager,
    /const MAX_JUMP_LIST_REMOVED_ENTRIES: usize = MAX_JUMP_LIST_ENTRIES;/,
  );

  const jumpList = functionBody(historyManager, "update_jump_list");
  assertBefore({
    body: jumpList,
    before: "if user_removed.len() > MAX_JUMP_LIST_REMOVED_ENTRIES",
    after: /let mut deleted_ids\s*=\s*Vec::with_capacity\(/,
    message: "jump-list removal payloads must be capped before delete-id materialization",
  });
  assert.match(jumpList, /refusing to process oversized jump-list removal payload/);
  assertBefore({
    body: jumpList,
    before: "deleted_ids.len() >= MAX_HISTORY_DELETION_IDS",
    after: "deleted_ids.push(entry.id);",
    message: "history delete-id collection must be capped before pushing ids",
  });
});

test("dock panel-size persist batches are bounded before deferred persistence", () => {
  assert.match(
    dock,
    /const MAX_PANEL_SIZE_STATE_PERSIST_BATCH: usize = 128;/,
  );

  const resizeAll = functionBody(dock, "resize_all_panels");
  assertBefore({
    body: resizeAll,
    before:
      /Vec::with_capacity\(\s*self\.panel_entries\s*\.len\(\)\s*\.min\(MAX_PANEL_SIZE_STATE_PERSIST_BATCH\),\s*\)/,
    after: "size_states_to_persist.push(",
    message: "panel-size persist batches must reserve only the capped batch size",
  });
  assertBefore({
    body: resizeAll,
    before: "size_states_to_persist.len() < MAX_PANEL_SIZE_STATE_PERSIST_BATCH",
    after: "size_states_to_persist.push(",
    message: "panel-size persist batches must cap entries before vector push",
  });
  assertBefore({
    body: resizeAll,
    before: "skipped_panel_size_state_persist_count",
    after: "cx.defer(move |cx|",
    message: "oversized panel-size persist batches must warn before deferred persistence",
  });
});

test("dock panel activation checks target index before active-panel side effects", () => {
  const activatePanel = functionBody(dock, "activate_panel");
  assertBefore({
    body: activatePanel,
    before: /self\s*\.panel_entries\s*\.get\(\s*panel_ix\s*\)/,
    after: /active_panel\.panel\.set_active\(false, window, cx\)/,
    message:
      "activate_panel must verify the target panel exists before deactivating the current panel",
  });
  assertBefore({
    body: activatePanel,
    before: /self\s*\.panel_entries\s*\.get\(\s*panel_ix\s*\)/,
    after: "self.active_panel_index = Some(panel_ix);",
    message:
      "activate_panel must verify the target panel exists before storing the active index",
  });
});

test("side dock stack controls use real panel entries and preserve single-panel activation", () => {
  assert.match(dock, /const MAX_STACKED_PANELS: usize = 3;/);
  assert.match(dock, /stacked_panel_ids: Vec<EntityId>/);
  assert.match(dock, /pub fn stack_panel\(/);
  assert.match(dock, /pub fn unstack_panel\(/);
  assert.match(dock, /pub fn show_single_panel\(/);

  const activatePanel = functionBody(dock, "activate_panel");
  const canSplitPanel = functionBody(dock, "can_split_panel");
  const restoreStackedPanels = functionBody(dock, "restore_stacked_panels");
  assertBefore({
    body: activatePanel,
    before: /self\s*\.panel_entries\s*\.get\(\s*panel_ix\s*\)/,
    after: /self\.stacked_panel_ids\.clear\(\)/,
    message:
      "ordinary activation must verify the target panel before clearing stack state",
  });

  const dockRender = functionBody(
    dock.slice(dock.indexOf("impl Render for Dock")),
    "render",
  );
  assert.match(dockRender, /let visible_panels = self\s*\.visible_entries\(cx\)/);
  assert.match(dockRender, /\.when\(is_stacked, \|this\| this\.flex\(\)\.flex_col\(\)\)/);
  assert.match(dockRender, /\.flex_1\(\)/);
  assert.match(dockRender, /\.border_t_1\(\)/);
  assert.match(dockRender, /dock\.activate_panel\(panel_ix, window, cx\);/);
  assert.match(canSplitPanel, /self\.supports_panel_stack\(\)/);
  assert.match(canSplitPanel, /self\.panel_index_for_id\(panel_id\)\.is_some\(\)/);
  assert.match(canSplitPanel, /self\.first_stack_candidate_for\(panel_id, cx\)\.is_some\(\)/);
  assert.doesNotMatch(canSplitPanel, /stacked_count|MAX_STACKED_PANELS/);

  const panelButtonsRender = functionBody(
    dock.slice(dock.indexOf("impl Render for PanelButtons")),
    "render",
  );
  assert.match(
    panelButtonsRender,
    /format!\(\s*"Add to \{\} Dock Stack",\s*dock_position\.label\(\)\s*\)/s,
  );
  assert.match(
    panelButtonsRender,
    /format!\(\s*"Remove from \{\} Dock Stack",\s*dock_position\.label\(\)\s*\)/s,
  );
  assert.match(panelButtonsRender, /"Show Only This Panel"/);
  assert.doesNotMatch(dockRender, /"dock-panel-stack-actions"/);
  assert.doesNotMatch(dockRender, /"dock-panel-stack-split"/);
  assert.doesNotMatch(dockRender, /"dock-panel-stack-close"/);
  assert.match(workspace, /SplitActiveSidePanel/);
  assert.match(workspace, /CloseActiveSidePanel/);
  assert.match(workspace, /fn split_active_side_panel\(/);
  assert.match(workspace, /fn close_active_side_panel\(/);
  assert.doesNotMatch(agentPanel, /"agent-panel-split-side-panel"/);
  assert.match(agentPanel, /"agent-panel-close-side-panel"/);
  for (const [source, prefix, name] of [
    [iconPicker, "icon-picker", "Icon picker"],
    [fontPanel, "font-panel", "Font panel"],
    [mediaPanel, "media-panel", "Media panel"],
    [uiPanel, "shadcn-ui", "UI panel"],
    [stylePanel, "dx-style-panel", "Style panel"],
  ] as const) {
    assert.match(
      source,
      new RegExp(
        `side_panel_header_controls\\(\\s*"${prefix}",[\\s\\S]*?(?:self\\.)?workspace\\.clone\\(\\)[\\s\\S]*?(?:cx\\.entity\\(\\)\\.entity_id\\(\\)|panel_id)`,
      ),
      `${name} must target its own panel entity for close controls`,
    );
    assert.doesNotMatch(
      source,
      /workspace::SplitActiveSidePanel|workspace::CloseActiveSidePanel/,
      `${name} header controls must not depend on active side-panel focus`,
    );
  }
  assert.doesNotMatch(dockRender, /"dock-panel-inline-split"/);
  assert.doesNotMatch(dockRender, /"dock-panel-inline-close"/);
  assert.doesNotMatch(dockRender, /"dock-panel-inline-control-mask"/);
  assert.doesNotMatch(panelButtonsRender, /"dock-panel-stack"/);
  assert.match(dockRender, /cursor_row_resize/);
  assert.match(restoreStackedPanels, /self\.stacked_panel_ids = stacked_panel_ids;/);
  assert.match(restoreStackedPanels, /self\.trim_stack_for_new_panel\(active_panel_id, Some\(active_panel_id\), cx\);/);
  assert.match(restoreStackedPanels, /self\.stacked_panel_ids\.insert\(0, active_panel_id\);/);
  assert.match(restoreStackedPanels, /self\.pin_agent_panel_to_left_stack_bottom\(cx\);/);
  assert.match(panelButtonsRender, /dock\.stack_panel\(panel_id, window, cx\)/);
  assert.match(panelButtonsRender, /dock\.unstack_panel\(panel_id, window, cx\)/);
  assert.match(panelButtonsRender, /dock\.show_single_panel\(panel_id, window, cx\)/);

  assert.match(workspace, /pub fn persist_dock_stack_state\(/);
  assert.match(workspace, /dock::PANEL_STACK_STATE_KEY/);
  assert.match(workspace, /MAX_PANEL_STACK_STATE_JSON_BYTES/);
  assert.match(workspace, /stacked_panels: left_stacked_panels/);
  assert.match(workspace, /stacked_panels: right_stacked_panels/);
});

test("core side panels expose dock close controls in visible headers", () => {
  const sidePanelHeaderControls = functionBody(dock, "side_panel_header_controls");
  const projectSelectionToolbar = functionBody(
    projectPanel,
    "render_selected_entries_toolbar",
  );
  const outlineFilterFooter = functionBody(outlinePanel, "render_filter_footer");
  const collabHeader = functionBody(collabPanel, "render_panel_header");
  const collabSignedIn = functionBody(collabPanel, "render_signed_in");
  const gitTabBar = functionBody(gitPanel, "render_tab_bar");
  const gitExpandedCommitHeader = functionBody(
    gitPanel,
    "render_expanded_commit_header",
  );
  const gitRender = functionBody(
    gitPanel.slice(gitPanel.indexOf("impl Render for GitPanel")),
    "render",
  );

  for (const source of [projectPanel, outlinePanel, collabPanel, gitPanel]) {
    assert.match(source, /side_panel_header_controls/);
  }

  assert.doesNotMatch(projectPanel, /fn render_panel_header/);
  assert.match(projectPanel, /render_side_panel_header_controls\("dx-explorer", cx\)/);
  assert.match(
    projectSelectionToolbar,
    /side_panel_header_controls\(\s*"project-panel-selection"/,
  );
  assert.match(outlineFilterFooter, /side_panel_header_controls\(\s*"outline-panel"/);
  assert.match(collabHeader, /side_panel_header_controls\(\s*"collab-panel"/);
  assert.match(collabSignedIn, /self\.render_panel_header\(cx\)/);
  assert.match(gitTabBar, /render_side_panel_header_controls\(cx\)/);
  assert.match(
    gitExpandedCommitHeader,
    /render_side_panel_header_controls\(cx\)/,
  );
  assert.match(
    gitRender,
    /if self\.commit_editor_expanded[\s\S]*render_expanded_commit_header\(cx\)/,
  );

  assert.doesNotMatch(
    sidePanelHeaderControls,
    /\.disabled\(!can_split\)|\.disabled\(!panel_is_registered\)/,
    "core side-panel close buttons must stay visible in narrow headers",
  );
  assert.doesNotMatch(
    sidePanelHeaderControls,
    /ButtonStyle::Subtle/,
    "core side-panel close buttons should use the same visible square treatment as tool panels",
  );
  assert.match(
    sidePanelHeaderControls,
    /IconButtonShape::Square/,
    "core side-panel close buttons should remain compact square header actions",
  );
  assert.match(
    sidePanelHeaderControls,
    /format!\("\{id_prefix\}-close-side-panel"\)[\s\S]*\.tab_index\(0(?:_isize)?\)/,
    "core side-panel close buttons must stay keyboard reachable in visible headers",
  );
  assert.match(
    sidePanelHeaderControls,
    /contains_side_panel_by_id\(panel_id, cx\)/,
    "close tooltip should still use real side-panel registration state",
  );
});

test("agent fullscreen keeps editor docks while sidebar button remains dock-scoped", () => {
  const fullscreenCenter = functionBody(agentPanel, "render_fullscreen_agent_center");
  const messageEditor = functionBody(threadView, "render_message_editor");
  const renderEntry = functionBody(threadView, "render_entry");
  const threadRender = functionBody(
    threadView.slice(threadView.indexOf("impl Render for ThreadView")),
    "render",
  );
  const toolbar = functionBody(agentPanel, "render_toolbar");
  const toolbarBackButton = functionBody(agentPanel, "render_toolbar_back_button");
  const panelOptionsMenu = functionBody(agentPanel, "render_panel_options_menu");
  const responseIndicator = functionBody(agentPanel, "render_toolbar_response_indicator");
  const responseIndicatorAnchors = functionBody(agentPanel, "toolbar_response_indicator_anchors");
  const responseSegment = functionBody(agentPanel, "toolbar_response_indicator_segment");
  const responsePageButton = functionBody(agentPanel, "toolbar_response_indicator_page_button");
  const activeVisibleThread = functionBody(agentPanel, "active_visible_thread_view");
  const visibleThreadForConversation = functionBody(
    agentPanel,
    "active_visible_thread_view_for_conversation",
  );
  const subscribeActiveThread = functionBody(agentPanel, "subscribe_to_active_thread_view");
  const focusAgentFullscreen = functionBody(agentPanel, "focus_fullscreen");
  const scrollAnchor = functionBody(threadView, "scroll_to_response_anchor");
  const scrollRequest = functionBody(threadView, "apply_response_anchor_scroll_request");
  const applyScrollAnchor = functionBody(threadView, "apply_response_anchor_scroll");
  const isResponseAnchorEntry = functionBody(threadView, "is_response_anchor_entry");
  const responseAnchorStruct = sourceWindow(threadView, "pub(crate) struct AgentResponseAnchor", 0, 260);
  const responseAnchorRequest = sourceWindow(threadView, "struct ResponseAnchorScrollRequest", 0, 180);
  const visibleRangeAnchor = functionBody(threadView, "response_anchor_for_visible_range");
  const currentScrollAnchor = functionBody(threadView, "response_anchor_for_current_scroll_position");
  const responseAnchors = functionBody(threadView, "response_anchors");
  const viewportReferenceAnchor = functionBody(threadView, "response_anchor_viewport_reference_ix");
  const syncResponseAnchor = functionBody(threadView, "sync_response_anchor_from_scroll_position");
  const scrollToMostRecentUserPrompt = functionBody(threadView, "scroll_to_most_recent_user_prompt");
  const scrollToEnd = functionBody(threadView, "scroll_to_end");
  const scrollToTop = functionBody(threadView, "scroll_to_top");
  const scrollOutputPageUp = functionBody(threadView, "scroll_output_page_up");
  const scrollOutputPageDown = functionBody(threadView, "scroll_output_page_down");
  const scrollOutputLineUp = functionBody(threadView, "scroll_output_line_up");
  const scrollOutputLineDown = functionBody(threadView, "scroll_output_line_down");
  const scrollOutputToPreviousMessage = functionBody(threadView, "scroll_output_to_previous_message");
  const scrollOutputToNextMessage = functionBody(threadView, "scroll_output_to_next_message");
  const dockVisibleEntries = functionBody(dock, "visible_entries");
  const dockVisibleEntriesForZoomedAgent = functionBody(dock, "visible_entries_for_zoomed_agent");
  const dockVisiblePanelForLayout = functionBody(dock, "visible_panel_for_layout");
  const dockZoomedAgentPanelId = functionBody(dock, "zoomed_agent_panel_id");
  const workspaceRenderDock = functionBody(workspace, "render_dock");
  const workspaceRender = functionBody(
    workspace.slice(workspace.indexOf("impl Render for Workspace")),
    "render",
  );
  const workspaceRenderCenterScreen = functionBody(workspace, "render_center_screen");
  const workspaceActivateScreenKind = functionBody(workspace, "activate_screen_kind");
  const workspaceFocusOrUnfocusPanel = sourceWindow(
    workspace,
    "fn focus_or_unfocus_panel",
    0,
    1800,
  );
  const workspaceFocusZoomedAgentPanel = functionBody(workspace, "focus_zoomed_agent_panel");
  const workspaceDismissAgentFullscreen = functionBody(workspace, "dismiss_zoomed_agent_panel");
  const workspaceToggleDock = functionBody(workspace, "toggle_dock");
  const workspaceDismissZoomed = functionBody(workspace, "dismiss_zoomed_items_to_reveal");
  const dockFocusIn = sourceWindow(dock, "cx.on_focus_in(&focus_handle", 0, 900);
  const panelButtonsAgentClick = sourceWindow(
    dock,
    "if is_agent_sidechat_button && agent_screen_is_active",
    0,
    360,
  );
  const toggleAgentPanelFocus = functionBody(agentPanel, "toggle_focus");
  const focusAgentPanelFullscreen = functionBody(agentPanel, "focus_fullscreen");
  const focusAgentPanel = functionBody(agentPanel, "focus");
  const toggleAgentPanel = functionBody(agentPanel, "toggle");
  const profilesSupported = functionBody(conversationView, "profiles_supported");
  const sourcesRailButton = sourceBetween(
    toolbar,
    "let agent_sources_rail_button",
    "let agent_progress_rail_button",
  );
  const progressRailButton = sourceBetween(
    toolbar,
    "let agent_progress_rail_button",
    "let close_panel_button",
  );
  const closePanelButton = sourceBetween(
    toolbar,
    "let panel_id = cx.entity().entity_id();",
    "let max_content_width",
  );
  const panelOptionsTrigger = sourceWindow(
    panelOptionsMenu,
    'IconButton::new("agent-options-menu", IconName::Ellipsis)',
    0,
    260,
  );
  assert.match(agentPanel, /"agent-toolbar-toggle-sources-rail"/);
  assert.match(agentPanel, /"agent-toolbar-toggle-progress-rail"/);
  assert.match(agentPanel, /let rails_available = is_full_screen;/);
  assert.doesNotMatch(toolbar, /has_sources_rail_content|has_progress_rail_content/);
  for (const [button, label] of [
    [toolbarBackButton, "Agent toolbar overlay back button"],
    [sourcesRailButton, "Agent sources rail toggle"],
    [progressRailButton, "Agent progress rail toggle"],
    [closePanelButton, "Agent side-panel close button"],
    [panelOptionsTrigger, "Agent options menu trigger"],
  ] as const) {
    assert.match(button, /\.tab_index\(0(?:_isize)?\)/, `${label} must be tabbable`);
  }
  for (const [button, label] of [
    [toolbarBackButton, "Agent toolbar overlay back button"],
    [sourcesRailButton, "Agent sources rail toggle"],
    [progressRailButton, "Agent progress rail toggle"],
    [closePanelButton, "Agent side-panel close button"],
    [panelOptionsTrigger, "Agent options menu trigger"],
  ] as const) {
    assert.doesNotMatch(
      button,
      /\.track_focus\(&focus_handle\)/,
      `${label} must not reuse the panel focus handle as its button focus id`,
    );
  }
  assert.match(closePanelButton, /let panel_id = cx\.entity\(\)\.entity_id\(\);/);
  assert.match(closePanelButton, /let workspace = self\.workspace\.clone\(\);/);
  assert.match(closePanelButton, /if let Some\(workspace\) = workspace\.upgrade\(\)/);
  assert.match(closePanelButton, /workspace\.close_side_panel_by_id\(panel_id, window, cx\)/);
  assert.doesNotMatch(closePanelButton, /CloseActiveSidePanel/);
  assert.match(sourcesRailButton, /\.toggle_state\(sources_rail_open\)/);
  assert.match(sourcesRailButton, /fullscreen_sources_rail_open = !this\.fullscreen_sources_rail_open/);
  assert.doesNotMatch(sourcesRailButton, /sources_rail_available|Sources rail has no sources yet/);
  assert.match(progressRailButton, /\.toggle_state\(progress_rail_open\)/);
  assert.match(progressRailButton, /fullscreen_progress_rail_open = !this\.fullscreen_progress_rail_open/);
  assert.doesNotMatch(progressRailButton, /progress_rail_available|Progress rail has no agent activity yet/);
  assert.match(agentPanel, /enum AgentPanelHostKind \{/);
  assert.match(agentPanel, /Sidechat,/);
  assert.match(agentPanel, /BuilderWorkspace,/);
  assert.match(agentPanel, /AutomationWorkspace,/);
  assert.match(agentPanel, /host_kind: AgentPanelHostKind/);
  assert.match(agentPanel, /host_kind: AgentPanelHostKind::Sidechat,[\s\S]*?manual_zoom_override: Some\(false\)/);
  assert.match(agentPanel, /fullscreen_sources_rail_open/);
  assert.match(agentPanel, /fullscreen_progress_rail_open/);
  assert.match(agentPanel, /fullscreen_sources_rail_open: true/);
  assert.match(agentPanel, /fullscreen_progress_rail_open: true/);
  assert.match(agentPanel, /pub\(crate\) fn new_builder_workspace\(/);
  assert.match(agentPanel, /panel\.host_kind = AgentPanelHostKind::BuilderWorkspace/);
  assert.match(agentPanel, /panel\.fullscreen_sources_rail_pinned = false/);
  assert.match(agentPanel, /panel\.fullscreen_progress_rail_pinned = false/);
  assert.match(agentPanel, /pub\(crate\) fn new_automation_workspace\(/);
  assert.match(agentPanel, /panel\.host_kind = AgentPanelHostKind::AutomationWorkspace/);
  assert.match(agentPanel, /fn render_automation_workspace_screen\(/);
  assert.match(
    agentPanel,
    /render_automation_screen\(\s*status\.as_ref\(\),\s*&mut self\.automation_catalog_state,\s*window,\s*cx,/,
  );
  assert.match(focusAgentPanelFullscreen, /crate::AgentScreen::open_or_focus\(workspace, window, cx\);/);
  assert.match(agentScreen, /pub struct AgentScreen \{\s*panel: Entity<AgentPanel>,\s*\}/);
  assert.match(agentScreen, /AgentPanel::new_builder_workspace\(workspace, window, cx\)/);
  assert.match(agentScreen, /pub\(crate\) fn open_or_focus\(/);
  assert.match(agentScreen, /workspace\.dismiss_zoomed_agent_panel\(window, cx\);/);
  assert.match(agentScreen, /workspace[\s\S]*?\.pane_for_screen_kind\(WorkspaceScreenKind::Agent, cx\)/);
  assert.match(agentScreen, /item\.screen_kind\(cx\) == WorkspaceScreenKind::Agent/);
  assert.match(agentScreen, /workspace\.activate_item\(&\*item, true, true, window, cx\);/);
  assert.match(agentScreen, /workspace\.screen_host_pane\(\)/);
  assert.match(agentScreen, /workspace\.add_item\(target_pane, Box::new\(item\), None, true, true, window, cx\);/);
  assert.match(agentScreen, /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s*WorkspaceScreenKind::Agent\s*\}/);
  assert.match(agentScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(agentScreen, /div\(\)\.size_full\(\)\.child\(self\.panel\.clone\(\)\)/);
  assert.match(automationScreen, /pub struct AutomationScreen \{\s*panel: Entity<AgentPanel>,\s*\}/);
  assert.match(automationScreen, /AgentPanel::new_automation_workspace\(workspace, window, cx\)/);
  assert.match(automationScreen, /pub\(crate\) fn open_or_focus\(/);
  assert.match(
    automationScreen,
    /workspace[\s\S]*?\.pane_for_screen_kind\(WorkspaceScreenKind::Automations, cx\)/,
  );
  assert.match(automationScreen, /item\.screen_kind\(cx\) == WorkspaceScreenKind::Automations/);
  assert.match(automationScreen, /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s*WorkspaceScreenKind::Automations\s*\}/);
  assert.match(automationScreen, /fn tab_content_text\(&self,[\s\S]*"Automations"\.into\(\)/);
  assert.match(automationScreen, /dx_icon\(DxUiIcon::Automations\)/);
  assert.match(automationScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(automationScreen, /zed_actions::assistant::OpenAutomations/);
  assert.match(agentPanel, /fn render_fullscreen_agent_center\(/);
  assert.match(agentPanel, /fn render_toolbar_response_indicator\(/);
  assert.match(agentPanel, /fn toolbar_response_indicator_segment\(/);
  assert.match(toolbar, /\.id\("agent-panel-toolbar"\)[\s\S]*\.relative\(\)/);
  assert.match(toolbar, /render_toolbar_response_indicator\(cx\)/);
  assert.match(workspace, /zoomed_is_agent_panel: bool/);
  assert.match(workspace, /zoomed_is_agent_panel: false/);
  assert.match(workspace, /pub fn zoomed_is_agent_panel\(&self\) -> bool/);
  assert.match(workspace, /pub fn client_side_decorations_with_content_flush\(/);
  assert.match(workspace, /content_flush_tiling: Tiling/);
  assert.match(workspace, /let content_tiling = Tiling\s*\{/);
  assert.match(workspace, /\.when\(!content_tiling\.right/);
  assert.match(workspaceRenderDock, /self\.zoomed_position == Some\(position\) && !self\.zoomed_is_agent_panel/);
  assert.match(workspaceRender, /self\.zoomed\.is_none\(\) \|\| self\.zoomed_is_agent_panel/);
  assert.match(dock, /dock\.zoom_layer_open =\s*workspace\.zoomed\.is_some\(\) && !workspace\.zoomed_is_agent_panel\(\);/);
  assert.match(dock, /fn zoomed_agent_panel_id\(&self, cx: &App\) -> Option<EntityId>/);
  assert.match(dockZoomedAgentPanelId, /workspace\.zoomed_is_agent_panel\(\)/);
  assert.match(dockZoomedAgentPanelId, /workspace\.zoomed_position != Some\(self\.position\)/);
  assert.match(dockZoomedAgentPanelId, /workspace[\s\S]*?\.zoomed_item\(\)[\s\S]*?\.and_then\(\|view\| view\.upgrade\(\)\)[\s\S]*?\.map\(\|view\| view\.entity_id\(\)\)/);
  assert.match(dockVisibleEntries, /self\.visible_entries_for_zoomed_agent\(self\.zoomed_agent_panel_id\(cx\), cx\)/);
  assert.match(dockVisibleEntriesForZoomedAgent, /zoomed_agent_panel_id: Option<EntityId>/);
  assert.match(dockVisibleEntriesForZoomedAgent, /entries\.retain\(\|\(_, entry\)\| Some\(entry\.panel\.panel_id\(\)\) != zoomed_agent_panel_id\)/);
  assert.doesNotMatch(
    dockVisibleEntriesForZoomedAgent,
    /if entries\.is_empty\(\)[\s\S]*?entry\.panel\.enabled\(cx\)/s,
    "Agent fullscreen should not promote Project or another dock panel as a fallback",
  );
  assert.match(dock, /\.visible_entries\(cx\)/);
  assert.match(dockVisiblePanelForLayout, /self\.visible_entries_for_zoomed_agent\(zoomed_agent_panel_id, cx\)/);
  assert.match(dockVisiblePanelForLayout, /entry\.panel\.clone\(\)/);
  assert.doesNotMatch(dockVisiblePanelForLayout, /workspace\.read\(cx\)|zoomed_agent_panel_id\(cx\)/);
  assert.match(
    workspaceRenderDock,
    /let zoomed_agent_panel_id =[\s\S]*self\.zoomed_is_agent_panel[\s\S]*self\.zoomed_position == Some\(position\)[\s\S]*self\.zoomed_item\(\)\.and_then\(\|view\| view\.upgrade\(\)\)[\s\S]*\.map\(\|view\| view\.entity_id\(\)\);/s,
    "render_dock must pass its existing zoomed Agent state into Dock sizing without re-reading Workspace through Dock",
  );
  assert.match(
    workspaceRenderDock,
    /dock_state\.visible_panel_for_layout\(zoomed_agent_panel_id, cx\)/,
  );
  assert.match(
    workspaceRenderDock,
    /visible_panel\.is_none\(\) && zoomed_agent_panel_id\.is_some\(\)[\s\S]*?return None;/s,
    "the Agent fullscreen source dock should not reserve layout width when the zoomed Agent is the only visible panel",
  );
  assert.doesNotMatch(
    workspaceRenderDock,
    /dock\.visible_panel\(\)/,
    "render_dock must size from the same filtered panel model that Dock::render uses",
  );
  assert.match(workspaceRenderCenterScreen, /if self\.zoomed_is_agent_panel/);
  assert.match(workspaceRenderCenterScreen, /"workspace-agent-screen-center"/);
  assert.match(workspaceRenderCenterScreen, /\.child\(zoomed_view\)/);
  assert.match(workspaceRenderCenterScreen, /self\.render_screen_carousel_center\(center, cx\)/);
  assert.match(
    workspaceActivateScreenKind,
    /self\.dismiss_zoomed_agent_panel\(window, cx\);[\s\S]*?let target_pane = self\.screen_host_pane\(\);/s,
    "screen-dock navigation must leave Agent fullscreen before activating Editor, Browser, or Terminal",
  );
  assert.match(
    workspaceActivateScreenKind,
    /WorkspaceScreenKind::Agent => \{\s*window\.dispatch_action\(\s*zed_actions::assistant::FocusAgentFullscreen\.boxed_clone\(\),\s*cx,\s*\);\s*\}/s,
    "screen-dock Agent activation must route through the real Agent screen action",
  );
  assert.match(
    workspaceActivateScreenKind,
    /WorkspaceScreenKind::Automations => \{\s*window\.dispatch_action\(\s*zed_actions::assistant::OpenAutomations\.boxed_clone\(\),\s*cx,\s*\);\s*\}/s,
    "screen-dock Automation activation must route through the real automation screen action",
  );
  assert.match(workspaceDismissAgentFullscreen, /if !self\.zoomed_is_agent_panel/);
  assert.match(workspaceDismissAgentFullscreen, /dock\.zoom_out\(window, cx\)/);
  assert.match(workspaceDismissAgentFullscreen, /self\.zoomed_is_agent_panel = false/);
  assert.match(workspaceDismissAgentFullscreen, /cx\.emit\(Event::ZoomChanged\)/);
  assert.match(workspaceFocusZoomedAgentPanel, /panel_for_id\(zoomed_panel_id\)/);
  assert.match(workspaceFocusZoomedAgentPanel, /panel\.panel_focus_handle\(cx\)\.focus\(window, cx\)/);
  assert.match(
    workspaceFocusOrUnfocusPanel,
    /if !self\.focus_zoomed_agent_panel\(window, cx\) \{[\s\S]*?window\.focus\(&pane\.focus_handle\(cx\), cx\)/s,
    "side-panel toggle-off should return focus to the Agent fullscreen panel instead of clearing it through center focus",
  );
  assert.match(
    dockFocusIn,
    /workspace\.zoomed_is_agent_panel\(\)[\s\S]*?workspace[\s\S]*?\.zoomed_item\(\)[\s\S]*?\.and_then\(\|view\| view\.upgrade\(\)\)[\s\S]*?!panel\.is_agent_panel\(cx\)/s,
    "focusing Project/Git/Outline side panels should preserve a live Agent fullscreen zoom owner",
  );
  assert.match(
    workspace,
    /let centered_layout = self\.centered_layout\s*&& !self\.zoomed_is_agent_panel\s*&& self\.center\.panes\(\)\.len\(\) == 1/s,
    "Agent fullscreen should not inherit centered-editor padding gutters",
  );
  assert.match(workspace, /\.children\(\(!self\.zoomed_is_agent_panel\)\.then\(\|\| \{/);
  assert.doesNotMatch(workspace, /WorkspaceSettings::get_global\(cx\)\.zoomed_padding\s*\|\|\s*self\.zoomed_is_agent_panel/);
  assert.match(workspaceToggleDock, /&& !self\.zoomed_is_agent_panel\s*&& self\.zoomed_position != Some\(dock_side\)/);
  assert.match(workspaceToggleDock, /if reveal_dock && !self\.zoomed_is_agent_panel/);
  assert.match(
    workspace,
    /pub fn reveal_panel<T: Panel>\(&mut self, window: &mut Window, cx: &mut Context<Self>\) \{[\s\S]*?if !self\.zoomed_is_agent_panel \{\s*self\.dismiss_zoomed_items_to_reveal\(dock_position, window, cx\);\s*\}/s,
  );
  assert.match(workspaceDismissZoomed, /let preserve_agent_fullscreen = self\.zoomed_is_agent_panel && dock_to_reveal\.is_some\(\);/);
  assert.match(workspaceDismissZoomed, /preserve_agent_fullscreen && panel\.is_agent_panel\(cx\)/);
  assert.match(workspaceDismissZoomed, /self\.zoomed_position != dock_to_reveal && !preserve_agent_fullscreen/);
  assert.match(multiWorkspace, /client_side_decorations_with_content_flush/);
  assert.match(multiWorkspace, /let agent_fullscreen_flush_right = false;/);
  assert.doesNotMatch(multiWorkspace, /let agent_fullscreen_flush_right = workspace\.read\(cx\)\.zoomed_is_agent_panel\(\);/);
  assert.match(multiWorkspace, /right: agent_fullscreen_flush_right/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = panel\.is_agent_panel\(cx\)/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = panel\.read\(cx\)\.is_agent_panel\(\)/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = false/);
  assert.match(threadView, /struct AgentResponseAnchor/);
  assert.match(threadView, /const RESPONSE_ANCHOR_SCROLL_RETRY_FRAMES: usize = 6;/);
  assert.match(threadView, /const FLOATING_MESSAGE_EDITOR_SAFE_PADDING_PX: f32 = 118\.0;/);
  assert.match(threadView, /struct ResponseAnchorScrollRequest/);
  assert.match(threadView, /response_anchor_scroll_request: Option<ResponseAnchorScrollRequest>/);
  assert.match(threadView, /active_response_anchor_entry_ix: Option<usize>/);
  assert.match(responseAnchorStruct, /pub\(crate\) entry_ix: usize/);
  assert.doesNotMatch(responseAnchorStruct, /Range<|ListOffset|scroll_position|message_id/);
  assert.match(responseAnchorRequest, /entry_ix: usize/);
  assert.match(responseAnchorRequest, /frames_remaining: usize/);
  assert.doesNotMatch(responseAnchorRequest, /Range<|ListOffset|scroll_position|offset_in_item/);
  assert.match(threadView, /visible_entry_range: Option<Range<usize>>/);
  assert.match(threadView, /let visible_range = event\.visible_range\.clone\(\)/);
  assert.match(threadView, /this\.visible_entry_range = Some\(visible_range\.clone\(\)\)/);
  assert.match(
    threadView,
    /if let Some\(request\) = this\.response_anchor_scroll_request \{[\s\S]*?this\.active_response_anchor_entry_ix = Some\(request\.entry_ix\);[\s\S]*?preserve_response_anchor = true;[\s\S]*?\}/s,
    "programmatic response-anchor retries should preserve the requested anchor for the full retry window instead of unlocking as soon as the bottom-clamped target becomes visible",
  );
  assert.doesNotMatch(threadView, /request_is_visible/, "visible-but-bottom-clamped prompt clicks must not unlock the clicked anchor early");
  assert.doesNotMatch(
    threadView,
    /else if scroll_top\.item_ix != request\.entry_ix[\s\S]*?response_anchor_scroll_request = None/,
    "manual scroll drift should not unlock a pending response-anchor click before the target resolves",
  );
  assert.match(
    threadView,
    /this\.active_response_anchor_entry_ix\s*=\s*this\.response_anchor_for_visible_range\(visible_range\.clone\(\), cx\)/s,
  );
  assert.match(activeVisibleThread, /Self::active_visible_thread_view_for_conversation\(server_view, cx\)/);
  assert.match(visibleThreadForConversation, /\.active_thread\(\)\s*\.cloned\(\)\s*\.or_else\(\|\| server_view\.root_thread_view\(\)\)/);
  assert.match(subscribeActiveThread, /Self::active_visible_thread_view_for_conversation\(server_view, cx\)/);
  assert.match(threadView, /ScrollPositionChanged/);
  assert.match(
    threadView,
    /pub\(crate\) fn response_anchors\(\s*&self,\s*max_anchors: usize,\s*cx: &App,\s*\) -> Vec<AgentResponseAnchor>/s,
  );
  assert.match(threadView, /if max_anchors == 0 \{\s*return Vec::new\(\);\s*\}/);
  assert.match(threadView, /let mut prompt_count = 0usize;[\s\S]*?for \(entry_ix, entry\) in entries\.iter\(\)\.enumerate\(\)/);
  assert.match(threadView, /prompt_count \+= 1/);
  assert.match(threadView, /if prompt_count == 0 \{\s*return Vec::new\(\);/);
  assert.match(threadView, /let mut current_prompt_position = None;/);
  assert.match(threadView, /if current_prompt_ix == Some\(entry_ix\) \{[\s\S]*?current_prompt_position = Some\(prompt_count\);/);
  assert.match(threadView, /current_prompt_position\.unwrap_or_else\(\|\| prompt_count\.saturating_sub\(1\)\)/);
  assert.match(threadView, /let mut prompt_ix = 0usize;/);
  assert.match(threadView, /let prompt_ordinal = prompt_ix \+ 1;/);
  assert.match(threadView, /let is_in_window = \(start\.\.end\)\.contains\(&prompt_ix\);/);
  assert.doesNotMatch(
    responseAnchors,
    /collect::<Vec<_>>\(\)/,
    "response anchor rendering should not allocate every prompt label just to draw a small toolbar window",
  );
  assert.match(threadView, /fn response_anchor_for_visible_range\(/);
  assert.match(threadView, /fn response_anchor_for_current_scroll_position\(/);
  assert.match(threadView, /fn response_anchor_for_scroll_position\(/);
  assert.match(threadView, /fn response_anchor_viewport_reference_ix\(/);
  assert.match(threadView, /fn is_response_anchor_entry\(/);
  assert.match(threadView, /fn last_response_anchor_entry\(&self, cx: &App\) -> Option<usize>/);
  assert.match(visibleRangeAnchor, /self\.list_state\.is_scrolled_to_end\(\) == Some\(true\)[\s\S]*?return self\.last_response_anchor_entry\(cx\);/);
  assert.match(currentScrollAnchor, /self\.list_state\.is_scrolled_to_end\(\) == Some\(true\)[\s\S]*?return self\.last_response_anchor_entry\(cx\);/);
  assert.match(visibleRangeAnchor, /logical_scroll_top\(\)\.item_ix/);
  assert.doesNotMatch(visibleRangeAnchor, /\.take\(end\.saturating_add\(1\)\)/);
  assert.match(currentScrollAnchor, /let scroll_top = self\.list_state\.logical_scroll_top\(\);/);
  assert.match(currentScrollAnchor, /visible_entry_range[\s\S]*?filter\(\|range\| range\.contains\(&scroll_top\.item_ix\)\)[\s\S]*?unwrap_or_else\(\|\| scroll_top\.item_ix\.\.scroll_top\.item_ix\.saturating_add\(1\)\)/);
  const scrollPositionAnchor = functionBody(threadView, "response_anchor_for_scroll_position");
  assert.match(scrollPositionAnchor, /scroll_item_ix: usize/);
  assert.match(scrollPositionAnchor, /Self::response_anchor_viewport_reference_ix\(start\.\.end, scroll_item_ix, entries\.len\(\)\)/);
  assert.doesNotMatch(scrollPositionAnchor, /entry_ix\.abs_diff\(reference_ix\)/);
  assert.doesNotMatch(scrollPositionAnchor, /\.min_by_key/);
  assert.match(scrollPositionAnchor, /\.take\(reference_ix\.saturating_add\(1\)\.min\(entries\.len\(\)\)\)/);
  assert.match(scrollPositionAnchor, /\.rev\(\)[\s\S]*?\.find_map/);
  assert.match(scrollPositionAnchor, /\.skip\(start\)[\s\S]*?\.take\(end\.saturating_sub\(start\)\)[\s\S]*?\.find_map/);
  assert.match(viewportReferenceAnchor, /let visible_span = end\.saturating_sub\(start\);/);
  assert.match(viewportReferenceAnchor, /start \+ visible_span \/ 2/);
  assert.match(viewportReferenceAnchor, /scroll_item_ix\.min\(entries_len\.saturating_sub\(1\)\)/);
  assert.match(isResponseAnchorEntry, /matches!\(entry, AgentThreadEntry::UserMessage\(_\)\)/);
  assert.match(threadView, /active_response_anchor_entry_ix[\s\S]*?response_anchor_for_current_scroll_position/);
  assert.match(responseAnchors, /let selected_prompt_ix = self[\s\S]*?active_response_anchor_entry_ix[\s\S]*?is_response_anchor_entry/);
  assert.match(responseAnchors, /let scroll_prompt_ix = self\.response_anchor_for_current_scroll_position\(cx\);/);
  assert.doesNotMatch(responseAnchors, /let current_ix = self\.list_state\.logical_scroll_top\(\)\.item_ix/);
  assert.match(
    responseAnchors,
    /if self\.response_anchor_scroll_request\.is_some\(\) \{[\s\S]*?selected_prompt_ix\.or\(scroll_prompt_ix\)[\s\S]*?\} else \{[\s\S]*?scroll_prompt_ix\.or\(selected_prompt_ix\)/,
    "a pending indicator click should lock the active target until the requested prompt resolves",
  );
  assert.match(
    responseAnchors,
    /Some\(AgentResponseAnchor \{[\s\S]*?(entry_ix: \*entry_ix|entry_ix,)[\s\S]*?is_current: current_prompt_ix == Some\(\*?entry_ix\),[\s\S]*?\}\)/,
    "response indicators should be stable message-entry anchors, not transient scroll offsets",
  );
  assert.match(threadView, /fn sync_response_anchor_from_scroll_position\(/);
  assert.match(syncResponseAnchor, /if self\.response_anchor_scroll_request\.is_some\(\) \{[\s\S]*?return;/);
  assert.match(syncResponseAnchor, /visible_entry_range[\s\S]*?filter\(\|range\| range\.contains\(&scroll_top\.item_ix\)\)/);
  assert.match(syncResponseAnchor, /unwrap_or_else\(\|\| scroll_top\.item_ix\.\.scroll_top\.item_ix\.saturating_add\(1\)\)/);
  assert.match(syncResponseAnchor, /self\.response_anchor_for_scroll_position\(visible_range, scroll_top\.item_ix, cx\)/);
  assert.match(syncResponseAnchor, /self\.active_response_anchor_entry_ix = next_anchor/);
  assert.match(syncResponseAnchor, /cx\.emit\(AcpThreadViewEvent::ScrollPositionChanged\)/);
  for (const [name, body] of [
    ["scroll_to_most_recent_user_prompt", scrollToMostRecentUserPrompt],
    ["scroll_to_end", scrollToEnd],
    ["scroll_to_top", scrollToTop],
    ["scroll_output_page_up", scrollOutputPageUp],
    ["scroll_output_page_down", scrollOutputPageDown],
    ["scroll_output_line_up", scrollOutputLineUp],
    ["scroll_output_line_down", scrollOutputLineDown],
    ["scroll_output_to_previous_message", scrollOutputToPreviousMessage],
    ["scroll_output_to_next_message", scrollOutputToNextMessage],
  ]) {
    assert.match(
      body,
      /sync_response_anchor_from_scroll_position\(cx\)/,
      `${name} should keep the fullscreen response marker in sync after non-wheel manual scrolling`,
    );
  }
  assert.match(
    threadView,
    /Label::new\("Scroll to Subagent"\)[\s\S]*?this\.list_state\.scroll_to\(ListOffset \{[\s\S]*?this\.sync_response_anchor_from_scroll_position\(cx\);/s,
    "subagent progress jump-list clicks should keep the fullscreen response marker in sync",
  );
  assert.match(
    threadView,
    /Button::new\("main-agent-permission-scroll-to", "Scroll"\)[\s\S]*?this\.list_state\.scroll_to\(ListOffset \{[\s\S]*?this\.sync_response_anchor_from_scroll_position\(cx\);/s,
    "awaiting-permission jump-list clicks should keep the fullscreen response marker in sync",
  );
  assert.match(threadView, /pub\(crate\) fn scroll_to_response_anchor\(/);
  assert.match(scrollAnchor, /window: &mut Window/);
  assert.match(scrollAnchor, /if !self\.is_response_anchor_entry\(entry_ix, cx\)/);
  assert.match(scrollAnchor, /self\.response_anchor_scroll_request = Some\(ResponseAnchorScrollRequest \{/);
  assert.match(scrollAnchor, /self\.active_response_anchor_entry_ix = Some\(entry_ix\)/);
  assert.match(scrollAnchor, /frames_remaining: RESPONSE_ANCHOR_SCROLL_RETRY_FRAMES/);
  assert.match(scrollAnchor, /self\.apply_response_anchor_scroll_request\(window, cx\)/);
  assert.match(scrollRequest, /let record_navigation = request\.frames_remaining == RESPONSE_ANCHOR_SCROLL_RETRY_FRAMES;/);
  assert.doesNotMatch(scrollRequest, /current_scroll_top/);
  assert.doesNotMatch(scrollRequest, /offset_in_item != px\(0\.0\)/);
  assert.match(scrollRequest, /self\.response_anchor_scroll_request = None;/);
  assert.match(scrollRequest, /self\.active_response_anchor_entry_ix = Some\(request\.entry_ix\);/);
  assert.match(scrollRequest, /window\.on_next_frame\(move \|window, cx\|/);
  assert.match(scrollRequest, /thread_view\.apply_response_anchor_scroll_request\(window, cx\)/);
  assert.match(applyScrollAnchor, /record_navigation: bool/);
  assert.match(applyScrollAnchor, /if !self\.is_response_anchor_entry\(entry_ix, cx\)/);
  assert.match(applyScrollAnchor, /self\.should_be_following = false/);
  assert.match(applyScrollAnchor, /workspace\.unfollow\(CollaboratorId::Agent, window, cx\)/);
  assert.doesNotMatch(applyScrollAnchor, /set_follow_mode\(gpui::FollowMode::Normal\)/);
  assert.match(applyScrollAnchor, /self\.list_state\.scroll_to\(scroll_position\)/);
  assert.match(applyScrollAnchor, /self\.thread\.update\(cx, \|thread, _cx\| \{/);
  assert.match(applyScrollAnchor, /thread\.set_ui_scroll_position\(Some\(scroll_position\)\)/);
  assert.match(applyScrollAnchor, /self\.schedule_save\(cx\)/);
  assert.doesNotMatch(threadView, /scroll_to_reveal_item\(entry_ix\)/);
  assert.match(applyScrollAnchor, /cx\.emit\(AcpThreadViewEvent::ScrollPositionChanged\)/);
  assert.match(responseIndicator, /self\.active_visible_thread_view\(cx\)/);
  assert.doesNotMatch(responseIndicator, /self\.active_thread_view\(cx\)/);
  assert.match(agentPanel, /const MAX_TOOLBAR_RESPONSE_INDICATORS: usize = 32;/);
  assert.match(agentPanel, /const TOOLBAR_RESPONSE_INDICATOR_EDGE_CONTEXT: usize = 2;/);
  assert.match(
    responseIndicator,
    /response_anchors\(\s*MAX_TOOLBAR_RESPONSE_INDICATORS \+ TOOLBAR_RESPONSE_INDICATOR_EDGE_CONTEXT,\s*cx,\s*\)/s,
    "the toolbar should request edge context so previous/next controls know when a bounded prompt window can page",
  );
  assert.match(agentPanel, /struct ToolbarResponseIndicatorAnchors \{/);
  assert.match(agentPanel, /previous_entry_ix: Option<usize>/);
  assert.match(agentPanel, /next_entry_ix: Option<usize>/);
  assert.match(responseIndicator, /indicator_anchors\.anchors\.is_empty\(\)/);
  assert.match(responseIndicator, /when_some\(indicator_anchors\.previous_entry_ix/);
  assert.match(responseIndicator, /when_some\(indicator_anchors\.next_entry_ix/);
  assert.match(responseIndicatorAnchors, /anchors\.len\(\) <= MAX_TOOLBAR_RESPONSE_INDICATORS/);
  assert.match(responseIndicatorAnchors, /position\(\|anchor\| anchor\.is_current\)/);
  assert.match(responseIndicatorAnchors, /let half_window = MAX_TOOLBAR_RESPONSE_INDICATORS \/ 2;/);
  assert.match(responseIndicatorAnchors, /current_index\.saturating_sub\(half_window\)/);
  assert.match(responseIndicatorAnchors, /previous_entry_ix/);
  assert.match(responseIndicatorAnchors, /next_entry_ix/);
  assert.match(responseIndicatorAnchors, /anchors\.drain\(\.\.start\)/);
  assert.match(responseIndicatorAnchors, /anchors\.truncate\(MAX_TOOLBAR_RESPONSE_INDICATORS\)/);
  assert.match(responseIndicator, /toolbar_response_indicator_page_button/);
  assert.match(agentPanel, /IconName::ChevronLeft/);
  assert.match(agentPanel, /IconName::ChevronRight/);
  assert.match(
    responsePageButton,
    /IconButton::new\(\s*format!\("agent-toolbar-response-indicator-page-\{label\}-\{entry_ix\}"\),\s*icon,\s*\)/,
  );
  assert.match(responsePageButton, /\.tab_index\(0(?:_isize)?\)/);
  assert.match(responsePageButton, /thread\.scroll_to_response_anchor\(entry_ix, window, cx\)/);
  assert.match(responsePageButton, /cx\.stop_propagation\(\)/);
  assert.match(agentPanel, /AcpThreadViewEvent::ScrollPositionChanged => \{\s*cx\.notify\(\);\s*\}/);
  assert.match(agentPanel, /Tooltip::with_meta\(label\.clone\(\), None, detail\.clone\(\), cx\)/);
  assert.match(responseSegment, /thread\.scroll_to_response_anchor\(entry_ix, window, cx\)/);
  assert.doesNotMatch(responseSegment, /window\.on_next_frame/);
  assert.match(responseSegment, /cx\.stop_propagation\(\)/);
  assert.match(responseIndicator, /\.gap_0\(\)/);
  assert.match(responseIndicator, /\.px_0p5\(\)/);
  assert.match(responseSegment, /\.w\(px\(9\.0\)\)/);
  assert.match(responseSegment, /\.tab_index\(0(?:_isize)?\)/);
  assert.match(responseSegment, /\.focus_visible\(/);
  assert.match(responseSegment, /\.cursor_pointer\(\)/);
  assert.doesNotMatch(responseSegment, /CursorStyle::PointingHand/);
  assert.match(responseSegment, /\.w\(px\(2\.0\)\)\.h\(height\)/);
  assert.match(agentPanel, /"agent-fullscreen-center"/);
  assert.match(threadRender, /\.key_context\("AcpThread"\)[\s\S]*?\.relative\(\)/);
  assert.doesNotMatch(fullscreenCenter, /\.px_4\(\)/);
  assert.doesNotMatch(fullscreenCenter, /\.pb_3\(\)/);
  assert.doesNotMatch(fullscreenCenter, /max_content_width/);
  assert.match(messageEditor, /\.pt_0p5\(\)/);
  assert.match(messageEditor, /\.pb_2\(\)/);
  assert.match(messageEditor, /this\.absolute\(\)\.left_0\(\)\.right_0\(\)\.bottom_0\(\)/);
  assert.match(messageEditor, /let \(border_focused, border, panel_background\) =/);
  assert.match(messageEditor, /\.when\(!has_messages, \|this\| this\.bg\(panel_background\)\)/);
  assert.match(renderEntry, /\.pb\(px\(FLOATING_MESSAGE_EDITOR_SAFE_PADDING_PX\)\)/);
  assert.match(threadView, /render_entry\(\s*index,\s*entries\.len\(\),\s*!this\.generating_indicator_in_list,/s);
  assert.match(threadView, /render_generating\(confirmation, cx\)[\s\S]*?\.pb\(px\(FLOATING_MESSAGE_EDITOR_SAFE_PADDING_PX\)\)/);
  assert.match(messageEditor, /\.p_1p5\(\)/);
  assert.match(threadView, /EditorMode::AutoHeight\s*\{\s*min_lines: 2,\s*max_lines: Some\(2\),/s);
  assert.doesNotMatch(messageEditor, /render_composer_status_row/);
  assert.doesNotMatch(threadView, /fn render_access_control/);
  assert.doesNotMatch(messageEditor, /render_access_control/);
  assert.match(messageEditor, /render_add_context_button\(cx\)[\s\S]*?profile_selector\.clone\(\)[\s\S]*?render_profile_option_slots\(cx\)/);
  assert.match(messageEditor, /render_profile_option_slots\(cx\)/);
  assert.doesNotMatch(profilesSupported, /supports_tools\(\)/);
  assert.match(profilesSupported, /self\.read\(cx\)\.model\(\)\.is_some\(\)/);
  assert.match(composerProfileOptions, /enum ComposerProfileKind/);
  assert.match(composerProfileOptions, /use agent_settings::\{AgentProfile, DxAiProfileKind\};/);
  assert.match(composerProfileOptions, /enum ComposerSlotControlState/);
  assert.match(composerProfileOptions, /DisplayOnly/);
  assert.match(composerProfileOptions, /BackendPending/);
  assert.match(composerProfileOptions, /struct ComposerSlotContract/);
  assert.match(composerProfileOptions, /pub\(super\) contract: ComposerSlotContract/);
  assert.match(composerProfileOptions, /static ASK_COMPOSER_SLOTS/);
  assert.match(composerProfileOptions, /static AGENTS_COMPOSER_SLOTS/);
  assert.match(composerProfileOptions, /static MEDIA_COMPOSER_SLOTS: \[ComposerOptionSlot; 5\]/);
  assert.match(composerProfileOptions, /static SEARCH_COMPOSER_SLOTS/);
  assert.match(composerProfileOptions, /static STUDY_COMPOSER_SLOTS/);
  assert.match(composerProfileOptions, /pub\(super\) id: &'static str/);
  assert.match(composerProfileOptions, /"media-output-music"/);
  assert.match(composerProfileOptions, /"media-output-3d"/);
  assert.match(composerProfileOptions, /"media-output-docs"/);
  assert.match(composerProfileOptions, /"media-provider"/);
  assert.match(composerProfileOptions, /"media-quality-production"/);
  assert.doesNotMatch(composerProfileOptions, /\bdummy\b/);
  assert.doesNotMatch(composerProfileOptions, /notebook/i);
  assert.doesNotMatch(composerProfileOptions, /"Choose /);
  assert.match(composerProfileOptions, /"Study sources"/);
  assert.deepEqual(
    repeatedStringIds([...composerProfileOptions.matchAll(/\bslot\(\s*"([^"]+)"/g)].map(([, id]) => id)),
    [],
    "composer slot ids must stay unique",
  );
  assert.deepEqual(
    repeatedStringIds([...composerProfileOptions.matchAll(/\boption\(\s*"([^"]+)"/g)].map(([, id]) => id)),
    [],
    "composer option ids must stay unique",
  );
  assert.match(composerProfileOptions, /"DX MetaSearch and Web Preview evidence"/);
  assert.equal(
    [
      ...composerProfileOptions.matchAll(
        /control_state: ComposerSlotControlState::BackendPending/g,
      ),
    ].length,
    1,
    "only Media provider controls should be backend-pending",
  );
  assert.match(composerProfileOptions, /MEDIA_PROVIDER_CONTRACT[\s\S]*?BackendPending/);
  assert.match(composerProfileOptions, /SEARCH_CONTRACT[\s\S]*?DisplayOnly/);
  assert.match(composerProfileOptions, /STUDY_CONTRACT[\s\S]*?DisplayOnly/);
  assert.match(composerProfileOptions, /MEDIA_RECEIPT_CONTRACT[\s\S]*?DisplayOnly/);
  assert.match(threadView, /fn render_composer_option_overflow/);
  assert.match(threadView, /const MAX_VISIBLE_PROFILE_OPTION_SLOTS: usize = 4;/);
  assert.match(
    functionBody(threadView, "render_profile_option_slots"),
    /let Some\(profile_kind\) = self\.composer_profile_kind\(cx\) else \{\s*return Vec::new\(\);\s*\}/s,
  );
  assert.match(
    functionBody(threadView, "render_profile_option_slots"),
    /\.take\(MAX_VISIBLE_PROFILE_OPTION_SLOTS\)/,
  );
  assert.doesNotMatch(
    functionBody(threadView, "render_profile_option_slots"),
    /\.take\(3\)/,
  );
  assert.match(threadView, /fn composer_profile_kind_for_id\(profile_id: &str\) -> Option<ComposerProfileKind>/);
  assert.match(threadView, /fn composer_profile_kind\(&self, cx: &App\) -> Option<ComposerProfileKind>/);
  assert.match(functionBody(threadView, "composer_profile_kind_for_id"), /ComposerProfileKind::for_profile_id\(profile_id\)/);
  assert.match(composerProfileOptions, /DxAiProfileKind::Ask => ComposerProfileKind::Ask/);
  assert.match(composerProfileOptions, /DxAiProfileKind::Agents => ComposerProfileKind::Agents/);
  assert.match(composerProfileOptions, /DxAiProfileKind::Search => ComposerProfileKind::Search/);
  assert.match(composerProfileOptions, /DxAiProfileKind::Study => ComposerProfileKind::Study/);
  assert.match(composerProfileOptions, /DxAiProfileKind::Media => ComposerProfileKind::Media/);
  assert.doesNotMatch(functionBody(threadView, "composer_profile_kind"), /contains\(/);
  assert.doesNotMatch(functionBody(threadView, "composer_profile_kind"), /ComposerProfileKind::Agents\s*$/);
  assert.match(functionBody(threadView, "render_composer_option_overflow"), /for option in slot\.options/);
  assert.match(functionBody(threadView, "render_composer_option_slot"), /custom_row[\s\S]*composer_option_menu_row\(slot, option\)/);
  assert.match(functionBody(threadView, "render_composer_option_overflow"), /custom_row[\s\S]*composer_option_menu_row\(slot, option\)/);
  assert.doesNotMatch(functionBody(threadView, "render_composer_option_slot"), /custom_entry|handler|thread\.send/);
  assert.doesNotMatch(functionBody(threadView, "render_composer_option_overflow"), /custom_entry|handler|thread\.send/);
  assert.match(threadView, /fn composer_slot_contract_row\(slot: ComposerOptionSlot\) -> AnyElement/);
  assert.match(threadView, /informational until profile preferences can be saved with the request/);
  assert.match(threadView, /No generation starts from this menu/);
  assert.match(threadView, /Provider setup is required before this choice can start generation\./);
  assert.doesNotMatch(functionBody(threadView, "composer_option_menu_row"), /on_click|handler|set_profile|set_mode|update_settings_file|thread\.send/);
  assert.doesNotMatch(threadView, /fn render_dx_agent_action/);
  assert.doesNotMatch(threadView, /dx-agent-action/);
  assert.doesNotMatch(functionBody(threadView, "composer_profile_kind_for_id"), /builtin_profiles::/);
  assert.match(profileSelector, /fn profile_display_name/);
  assert.match(profileSelector, /AgentProfile::display_name\(profile_id, name\)/);
  assert.match(profileSelector, /fn set_selected_profile\(/);
  assert.match(profileSelector, /fn reconcile_current_profile\(/);
  assert.match(profileSelector, /ProfilePickerDelegate::candidates_from\(profiles\)/);
  assert.match(profileSelector, /\.map_or\(0, \|current_index\| \(current_index \+ 1\) % candidates\.len\(\)\)/);
  assert.match(profileSelector, /provider\.set_profile\(fallback_profile_id\.clone\(\), cx\)/);
  assert.match(profileSelector, /AgentProfile::builtin_sort_index\(&a\.id\)/);
  assert.match(profileSelector, /AgentProfile::dx_builtin_metadata\(&candidate\.id\)\.map\(\|metadata\| metadata\.summary\)/);
  assert.match(manageProfilesModal, /AgentProfile::builtin_sort_index\(&a\.id\)/);
  assert.match(profileSelector, /custom_profiles\.sort_unstable_by/);
  assert.match(profileSelector, /set_selected_profile\([\s\S]*?source = source/);
  assert.match(agentProfileSettings, /pub fn normalize_id\(/);
  assert.match(agentProfileSettings, /pub fn normalize_id_from_profiles\(/);
  const normalizeProfileId = functionBody(agentProfileSettings, "normalize_id_from_profiles");
  assert.match(normalizeProfileId, /let ask_profile = AgentProfileId\(builtin_profiles::ASK\.into\(\)\)/);
  assertBefore({
    body: normalizeProfileId,
    before: "let ask_profile = AgentProfileId(builtin_profiles::ASK.into())",
    after: "profile_id.as_str() == builtin_profiles::LEGACY_MINIMAL",
    message: "legacy Minimal fallback must know whether Ask exists before normalizing",
  });
  assert.match(
    normalizeProfileId,
    /profile_id\.as_str\(\) == builtin_profiles::LEGACY_MINIMAL[\s\S]*?profiles\.contains_key\(&ask_profile\)/,
  );
  assert.match(agentProfileSettings, /pub fn display_name\(/);
  assert.match(agentProfileSettings, /enum DxAiProfileKind/);
  assert.match(agentProfileSettings, /enum DxAiProfileBackendState/);
  assert.match(agentProfileSettings, /struct DxAiProfileMetadata/);
  assert.match(agentProfileSettings, /pub const DX_PROFILE_ORDER: \[&str; 5\] = \[ASK, WRITE, SEARCH, STUDY, MEDIA\]/);
  assert.match(agentProfileSettings, /pub fn dx_builtin_metadata_for_id\(profile_id: &str\) -> Option<DxAiProfileMetadata>/);
  assert.match(agentProfileSettings, /DxAiProfileKind::Agents/);
  assert.match(agentProfileSettings, /DxAiProfileBackendState::ProviderPending/);
  assert.match(agentProfileSettings, /pub const MEDIA: &str = "media"/);
  assert.match(agentProfileSettings, /pub const SEARCH: &str = "search"/);
  assert.match(agentProfileSettings, /pub const STUDY: &str = "study"/);
  assert.match(agentProfileSettings, /pub const LEGACY_MINIMAL: &str = "minimal"/);
  assert.match(agentProfileSettings, /WRITE\s*\|\s*ASK\s*\|\s*MEDIA\s*\|\s*SEARCH\s*\|\s*STUDY/);
  assert.match(agentProfileSettings, /id\.as_str\(\) == builtin_profiles::LEGACY_MINIMAL/);
  assert.match(agentSettings, /let profiles: IndexMap<AgentProfileId, AgentProfileSettings>/);
  assert.match(agentSettings, /let default_profile = AgentProfile::normalize_id_from_profiles/);
  assert.match(agentThread, /let profile_id = AgentProfile::normalize_id\(settings\.default_profile\.clone\(\), cx\)/);
  assert.match(agentThread, /let profile_id = AgentProfile::normalize_id\(profile_id, cx\)/);
  assert.match(functionBody(agentThread, "set_profile"), /let profile_id = AgentProfile::normalize_id\(profile_id, cx\)/);
  const settingsProfiles = objectBlock(defaultSettings, "profiles");
  const askProfile = objectBlock(settingsProfiles, "ask");
  const mediaProfile = objectBlock(settingsProfiles, "media");
  const searchProfile = objectBlock(settingsProfiles, "search");
  const studyProfile = objectBlock(settingsProfiles, "study");
  assert.match(defaultSettings, /"default_profile": "write"/);
  assert.match(defaultSettings, /"write": \{\s*"name": "Agents"/);
  assert.match(defaultSettings, /"ask": \{\s*"name": "Ask"/);
  assert.doesNotMatch(askProfile, /"spawn_agent": true/);
  assert.doesNotMatch(askProfile, /"update_plan": true/);
  assert.match(defaultSettings, /"media": \{\s*"name": "Media"/);
  assert.match(mediaProfile, /"plan_dx_media_tool": true[\s\S]*?"gate_dx_media_tool_runner": true[\s\S]*?"prepare_dx_source_attachment": true/);
  assert.doesNotMatch(mediaProfile, /"list_dx_launch_demo_recipes": true/);
  assert.doesNotMatch(mediaProfile, /"execute_dx_media_tool": true/);
  assert.doesNotMatch(mediaProfile, /"prepare_agent_plugin_runtime": true/);
  assert.match(agentPanel, /const DX_MEDIA_PROOF_PROMPT: &str = "Prepare the DX media provider proof flow/);
  assert.doesNotMatch(agentPanel, /DX_MEDIA_PROOF_PROMPT[\s\S]*?list_dx_launch_demo_recipes/);
  assert.doesNotMatch(agentPanel, /DX_MEDIA_PROOF_PROMPT[\s\S]*?execute_dx_media_tool/);
  assert.match(
    agentPanel,
    /"Plan, gate, and attach media receipts; execution waits for an approved provider run gate\."/,
  );
  assert.match(defaultSettings, /"search": \{\s*"name": "Search"/);
  assert.doesNotMatch(searchProfile, /"prepare_agent_plugin_runtime": true/);
  assert.match(defaultSettings, /"study": \{\s*"name": "Study"/);
  assert.match(studyProfile, /"extract_dx_metasearch_source": true[\s\S]*?"prepare_dx_source_attachment": true[\s\S]*?"prepare_dx_metasearch_context": true/);
  assert.doesNotMatch(studyProfile, /"spawn_agent": true/);
  assert.doesNotMatch(studyProfile, /"prepare_agent_plugin_runtime": true/);
  assert.doesNotMatch(defaultSettings, /"minimal": \{/);
  assert.doesNotMatch(agentProfileSettings, /notebook/i);
  assert.match(agentProfileSettings, /builtin_profiles::SEARCH => Some\(DxAiProfileMetadata/);
  assert.match(agentProfileSettings, /DxAiProfileKind::Search[\s\S]*?DxAiProfileBackendState::EvidenceBacked/);
  assert.match(agentProfileSettings, /builtin_profiles::STUDY => Some\(DxAiProfileMetadata/);
  assert.match(agentProfileSettings, /DxAiProfileKind::Study[\s\S]*?DxAiProfileBackendState::ReceiptBacked/);
  assert.match(agentProfileSettings, /builtin_profiles::MEDIA => Some\(DxAiProfileMetadata/);
  assert.match(agentProfileSettings, /DxAiProfileKind::Media[\s\S]*?DxAiProfileBackendState::ProviderPending/);
  assert.match(manageProfilesModal, /fn profile_icon\(profile_id: &AgentProfileId\) -> IconName/);
  assert.match(manageProfilesModal, /builtin_profiles::WRITE => IconName::Sparkle/);
  assert.match(manageProfilesModal, /builtin_profiles::MEDIA => dx_icon\(DxUiIcon::Media\)/);
  assert.match(manageProfilesModal, /builtin_profiles::SEARCH => dx_icon\(DxUiIcon::Search\)/);
  assert.match(manageProfilesModal, /builtin_profiles::STUDY => IconName::Book/);
  assert.match(manageProfilesModal, /AgentProfile::available_profiles\(cx\)/);
  assert.match(manageProfilesModal, /AgentProfile::display_name\(&mode\.profile_id, &profile\.name\)/);
  assert.match(agentPanel, /then_some\(IconName::Chat\)/);
  assert.match(focusAgentFullscreen, /crate::AgentScreen::open_or_focus\(workspace, window, cx\);/);
  assert.doesNotMatch(focusAgentFullscreen, /workspace\.focus_panel::<Self>|PanelEvent::ZoomIn/);
  assert.match(functionBody(agentPanel, "focus"), /cx\.emit\(PanelEvent::ZoomOut\)/);
  assert.match(workspace, /panel\.set_zoomed\(false, window, cx\);\s*dock\.set_open\(false, window, cx\);/);
  assert.match(dock, /let agent_screen_is_active = workspace/);
  assert.match(dock, /zoomed_is_agent_panel\(\)/);
  assert.match(dock, /active_item\(cx\)/);
  assert.match(dock, /item\.screen_kind\(cx\) == WorkspaceScreenKind::Agent/);
  assert.match(dock, /let is_agent_sidechat_button = entry\.panel\.is_agent_panel\(cx\)/);
  assert.match(dock, /!\(is_agent_sidechat_button && agent_screen_is_active\)/);
  assert.match(
    panelButtonsAgentClick,
    /if is_agent_sidechat_button && agent_screen_is_active \{\s*window\.dispatch_action\(action\.boxed_clone\(\), cx\);\s*\} else if use_side_stack_click/s,
    "bottom Agent sidechat button must use the Agent panel action while the AI Screen is active instead of stacking the builder surface",
  );
  assert.match(
    toggleAgentPanelFocus,
    /panel\.activate_sidechat_host\(\);\s*if panel\.zoomed/s,
    "ToggleFocus should always record sidechat intent before deciding whether to zoom out",
  );
  assert.match(
    focusAgentPanel,
    /panel\.activate_sidechat_host\(\);\s*if panel\.zoomed/s,
    "FocusAgent should always record sidechat intent before deciding whether to zoom out",
  );
  assert.match(
    toggleAgentPanel,
    /panel\.activate_sidechat_host\(\);\s*if panel\.zoomed/s,
    "Toggle should always record sidechat intent before deciding whether to zoom out",
  );
  assert.doesNotMatch(messageEditor, /\.border_t_1\(\)/);
  assert.match(threadView, /\.rounded_md\(\)/);
  assert.match(threadView, /\.shadow_sm\(\)/);
  assert.match(agentPanel, /PanelEvent::ZoomOut/);
  assert.match(agentPanel, /PanelEvent::ZoomIn/);
  assert.doesNotMatch(agentPanel, /"agent-toolbar-toggle-left-dock"/);
  assert.doesNotMatch(agentPanel, /"agent-toolbar-toggle-right-dock"/);
  assert.doesNotMatch(agentPanel, /let full_screen_button =/);
});

test("sidebar chat groups expose persistent sort and icon override controls", () => {
  assert.match(sidebar, /enum SidebarThreadSortMode/);
  assert.match(sidebar, /SidebarThreadSortMode[\s\S]*Manual/);
  assert.match(sidebar, /Self::Manual => "Custom"/);
  assert.match(sidebar, /thread_sort_mode: SidebarThreadSortMode/);
  assert.match(sidebar, /manual_thread_order:\s*Vec<ThreadId>/);
  assert.match(sidebar, /const MAX_SIDEBAR_MANUAL_THREAD_ORDER: usize = 512;/);
  assert.match(sidebar, /thread_icon_overrides: HashMap<ThreadId, IconName>/);
  assert.match(sidebar, /struct DraggedSidebarThread/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*"dragged-sidebar-thread"/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*Label::new\(self\.label\.clone\(\)\)/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*\.w\(px\(236\.0\)\)/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*\.color\(Color::Default\)/);
  assert.match(
    sidebar,
    /let dragged_thread = DraggedSidebarThread \{[\s\S]*?subtitle: \(!timestamp\.is_empty\(\)\)\.then_some\(timestamp\.clone\(\)\),/,
  );
  assert.match(sidebar, /subtitle: None,\s*action: SerializedSidebarGridAction::OpenThread/s);
  assert.match(sidebar, /matches!\(action, SidebarGridAction::OpenThread\(_\)\)/);
  assert.match(sidebar, /struct ThreadIconPickerMenu/);
  const threadIconPickerStart = sidebar.indexOf("struct ThreadIconPickerMenu");
  assert.ok(threadIconPickerStart >= 0, "expected thread icon picker menu");
  const threadIconPickerEnd = sidebar.indexOf("pub struct Sidebar", threadIconPickerStart);
  assert.ok(threadIconPickerEnd > threadIconPickerStart, "expected sidebar after picker menu");
  const threadIconPicker = sidebar.slice(threadIconPickerStart, threadIconPickerEnd);
  assert.match(
    sidebar,
    /thread_icon_picker_handles: RefCell<HashMap<ThreadId, PopoverMenuHandle<ThreadIconPickerMenu>>>/,
  );
  assert.match(threadIconPicker, /"thread-icon-picker-grid"/);
  assert.match(threadIconPicker, /"thread-icon-picker-header"/);
  assert.match(threadIconPicker, /Choose Icons for your Chat/);
  assert.match(threadIconPicker, /"thread-icon-picker-grid-icons"/);
  assert.match(threadIconPicker, /"thread-icon-picker-close"/);
  assert.match(threadIconPicker, /THREAD_ICON_PICKER_COLUMNS: u16 = 9/);
  assert.match(threadIconPicker, /\.grid\(\)/);
  assert.match(threadIconPicker, /\.grid_cols\(THREAD_ICON_PICKER_COLUMNS\)/);
  assert.doesNotMatch(threadIconPicker, /\.flex_wrap\(\)/);
  assert.doesNotMatch(sidebar, /\.pr_5\(\)/);
  assert.match(sidebar, /"sidebar-chat-sort-\{label\}"/);
  assert.match(sidebar, /"thread-icon-picker"/);
  assert.match(sidebar, /\.with_handle\(icon_picker_handle\)/);
  assert.match(sidebar, /is_hovered \|\| is_icon_picker_open \|\| is_focused/);
  assert.match(sidebar, /\.hovered\(is_hovered \|\| is_icon_picker_open\)/);
  assert.match(sidebar, /\(!is_draft\)\.then\(\|\|/);
  assert.match(sidebar, /fn reorder_thread_around\(/);
  assert.match(sidebar, /dragged_thread_id == target_thread_id \|\| self\.has_filter_query\(cx\)/);
  assert.match(sidebar, /self\.thread_sort_mode = SidebarThreadSortMode::Manual/);
  assert.match(sidebar, /ordered_thread_ids\.truncate\(MAX_SIDEBAR_MANUAL_THREAD_ORDER\)/);
  assert.match(sidebar, /\.drag_over::<DraggedSidebarThread>/);
  assert.match(sidebar, /this\.reorder_thread_around\(dragged\.thread_id, target_thread_id, cx\)/);
  assert.match(sidebar, /shortcut\.icon = icon_name/);
  assert.match(sidebar, /ThreadMetadataStore::global\(cx\)[\s\S]*?\.entry\(\*thread_id\)[\s\S]*?\.is_some\(\)/);
  assert.match(sidebar, /IconButton::new\(\("thread-icon-picker", ix\), IconName::Sparkle\)/);
  assert.match(sidebar, /IconName::iter\(\)/);
  assert.match(sidebar, /SerializedThreadIconOverride/);
  const focusSidebarFilter = functionBody(sidebar, "focus_sidebar_filter");
  const sidebarToggleButton = functionBody(sidebar, "render_sidebar_toggle_button");
  assert.match(focusSidebarFilter, /if !self\.activity_bar_expanded \{/);
  assert.match(
    focusSidebarFilter,
    /self\.activity_bar_expanded = true;/,
    "focusing search from collapsed mode should expand the sidebar before moving focus",
  );
  assert.match(
    focusSidebarFilter,
    /self\.serialize\(cx\);/,
    "search-triggered expansion should persist collapsed sidebar state consistently",
  );
  assert.match(sidebarToggleButton, /match \(on_right, is_activity_bar\)/);
  assert.match(sidebarToggleButton, /ThreadsSidebarLeftClosed/);
  assert.match(sidebarToggleButton, /ThreadsSidebarRightClosed/);
  const serializedState = functionBody(sidebar, "serialized_state");
  const restoreSerializedState = functionBody(sidebar, "restore_serialized_state");
  assert.match(
    serializedState,
    /manual_thread_order:\s*self[\s\S]*?\.manual_thread_order[\s\S]*?\.take\(MAX_SIDEBAR_MANUAL_THREAD_ORDER\)[\s\S]*?\.collect\(\)/,
  );
  assert.match(
    restoreSerializedState,
    /self\.manual_thread_order = serialized[\s\S]*?\.manual_thread_order[\s\S]*?\.take\(MAX_SIDEBAR_MANUAL_THREAD_ORDER\)[\s\S]*?\.collect\(\)/,
  );
  assert.match(threadItem, /let timestamp_color = if self\.selected \|\| self\.hovered/);
  assert.match(threadItem, /let timestamp_color = if self\.selected \|\| self\.hovered \{[\s\S]*Color::Default[\s\S]*\} else \{[\s\S]*Color::Muted[\s\S]*\};/);
  assert.match(threadItem, /Label::new\(timestamp\.clone\(\)\)[\s\S]*\.color\(timestamp_color\)/);
  assert.match(threadItem, /self\.hovered \|\| self\.focused/);
  assert.doesNotMatch(sidebar, /ContextMenuEntry::new\(format!\("\{icon_name:\?\}"\)\)/);
});

test("agent rails and project badges keep compact production layout", () => {
  const launchChrome = functionBody(dxLaunchWorkspace, "render_workspace_chrome");
  const sourcesRail = functionBody(dxLaunchWorkspace, "render_sources_rail");
  const progressRail = functionBody(dxLaunchWorkspace, "render_right_rail");
  const hasSourcesRailContent = functionBody(dxLaunchWorkspace, "has_sources_rail_content");
  const hasProgressRailContent = functionBody(dxLaunchWorkspace, "has_progress_rail_content");
  const hasAgentProgress = functionBody(dxLaunchWorkspace, "has_agent_progress");
  const hasAgentEnvironment = functionBody(dxLaunchWorkspace, "has_agent_environment");
  const hasAgentReadiness = functionBody(dxLaunchWorkspace, "has_agent_readiness");
  const diagnosticsMenu = functionBody(dxLaunchWorkspace, "diagnostics_menu");
  const railSection = functionBody(dxLaunchWorkspace, "rail_section");
  const subagentSummary = functionBody(dxLaunchWorkspace, "subagent_summary");
  const agentOverview = functionBody(dxAgentWorkspace, "agent_overview_section");
  const agentEnvironment = functionBody(dxAgentWorkspace, "agent_environment_section");
  const agentSources = functionBody(dxAgentWorkspace, "agent_sources_section");
  const sourceRow = functionBody(dxLaunchSourceRows, "source_item_row");
  const sourceRowControls = functionBody(agentPanel, "render_dx_launch_source_row_controls");
  const toolbar = functionBody(agentPanel, "render_toolbar");
  assert.match(dxLaunchWorkspace, /^mod agent_workspace;$/m);
  assert.match(dxAgentWorkspace, /fn agent_overview_section\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn render_response_controller\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_indicator_segment\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_controller_pill\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_controller_tick\(/);
  assert.match(dxLaunchWorkspace, /enum DxLaunchRailSection/);
  assert.match(dxLaunchWorkspace, /struct DxLaunchRailControls/);
  assert.match(dxLaunchWorkspace, /fn rail_section\(/);
  assert.match(launchChrome, /\.when\(show_sources_rail, \|this\|/);
  assert.match(launchChrome, /\.when\(show_progress_rail, \|this\|/);
  assert.match(
    hasSourcesRailContent,
    /status\.source_sets\.total_sources > 0 \|\| has_source_actions\(status\)/,
  );
  assert.match(
    hasProgressRailContent,
    /has_agent_progress\(status\)[\s\S]*has_agent_environment\(status\)[\s\S]*has_agent_subagents\(status\)[\s\S]*has_agent_readiness\(status\)/,
  );
  assert.doesNotMatch(hasAgentProgress, /launch_status\.latest_present/);
  assert.doesNotMatch(hasAgentEnvironment, /visible_worktree_count > 0/);
  assert.doesNotMatch(hasAgentReadiness, /acceptance_count|passed_count/);
  assert.match(dxLaunchWorkspace, /ListHeader/);
  assert.match(dxLaunchWorkspace, /ListItem/);
  assert.match(dxLaunchWorkspace, /ListItemSpacing/);
  assert.match(railSection, /ListHeader::new\(label\)/);
  assert.match(railSection, /\.toggle\(Some\(is_open\)\)/);
  assert.match(railSection, /\.start_slot\(Icon::new\(icon\)/);
  assert.match(railSection, /\.on_toggle\(move \|event, window, cx\| \{/);
  assert.match(railSection, /on_toggle\(section, event, window, cx\)/);
  assert.doesNotMatch(railSection, /Disclosure::new/);
  assert.doesNotMatch(railSection, /\.cursor_pointer\(\)/);
  assert.match(dxLaunchWorkspace, /"dx-agent-overview-section"/);
  assert.match(dxLaunchWorkspace, /"dx-agent-threads-section"/);
  assert.match(dxLaunchWorkspace, /"Environment"/);
  assert.match(dxLaunchWorkspace, /"dx-agent-tasks-section"/);
  assert.match(dxLaunchWorkspace, /"Sources"/);
  assert.match(dxLaunchWorkspace, /"dx-agent-subagents-section"/);
  assert.match(dxLaunchWorkspace, /"dx-agent-approvals-section"/);
  assert.match(agentOverview, /"Quality"/);
  assert.match(agentOverview, /"Active tasks"/);
  assert.match(agentOverview, /"Release gate"/);
  assert.match(agentEnvironment, /"Background threads"/);
  assert.match(agentEnvironment, /"Trusted tools"/);
  assert.match(agentSources, /"Attachable"/);
  assert.match(agentSources, /"Managed receipts"/);
  assert.match(dxLaunchWorkspace, /fn subagent_pixel_icon/);
  assert.match(dxLaunchWorkspace, /gpui::hsla\(210\.0 \/ 360\.0/);
  assert.match(dxLaunchWorkspace, /status\.subagent_rows\.iter\(\)\.take\(6\)/);
  assert.doesNotMatch(subagentSummary, /agent_bridge\.automations/);
  assert.doesNotMatch(subagentSummary, /automation_count/);
  assert.match(dxLaunchWorkspace, /enum DxSubagentStatus/);
  assert.match(dxLaunchWorkspace, /DxSubagentStatus::Running/);
  assert.match(dxLaunchWorkspace, /DxSubagentStatus::Queued/);
  assert.match(dxLaunchWorkspace, /DxSubagentStatus::Blocked/);
  assert.match(dxLaunchWorkspace, /DxSubagentStatus::Failed/);
  assert.match(dxLaunchWorkspace, /DxSubagentStatus::Idle/);
  assert.match(dxLaunchWorkspace, /fn subagent_pixel_icon\(status: DxSubagentStatus\)/);
  assert.match(agentPanel, /fn dx_subagent_status_rows/);
  assert.match(agentPanel, /AgentThreadEntry::ToolCall/);
  assert.match(agentPanel, /subagent_session_info/);
  assert.match(agentPanel, /ThreadStatus::Generating/);
  assert.match(agentPanel, /has_in_progress_tool_calls/);
  assert.match(agentPanel, /has_queued_messages/);
  assert.ok(
    agentPanel.indexOf("thread.status() == ThreadStatus::Generating") <
      agentPanel.indexOf("thread_view.has_queued_messages()"),
    "active generating/tool-call threads should stay Running even when follow-up messages are queued",
  );
  assert.match(agentPanel, /had_error/);
  assert.match(agentPanel, /pending_tool_call_for_session/);
  assert.match(agentPanel, /dx_subagent_status_from_tool_call/);
  assert.match(agentPanel, /dx_session_id_label/);
  assert.doesNotMatch(agentPanel, /dx_session_label/);
  assert.match(agentPanel, /ToolCallStatus::WaitingForConfirmation/);
  assert.doesNotMatch(dxLaunchWorkspace, /muted_card\("No live subagent state", cx\)/);
  assert.match(dxLaunchWorkspace, /fn metric_row\(/);
  assert.match(dxLaunchWorkspace, /ListItem::new\(rail_stable_id\("dx-launch-metric"/);
  assert.match(dxLaunchWorkspace, /fn signal_row\(/);
  assert.match(dxLaunchWorkspace, /fn source_row\(/);
  assert.match(dxLaunchWorkspace, /fn muted_card\(/);
  assert.match(dxLaunchWorkspace, /ListItem::new\(rail_stable_id\("dx-launch-empty"/);
  assert.match(agentPanel, /collapsed_dx_launch_rail_sections: HashSet<DxLaunchRailSection>/);
  assert.match(agentPanel, /fullscreen_sources_rail_pinned: bool/);
  assert.match(agentPanel, /fullscreen_progress_rail_pinned: bool/);
  assert.match(agentPanel, /fullscreen_sources_rail_pinned: false/);
  assert.match(agentPanel, /fullscreen_progress_rail_pinned: false/);
  assert.match(agentPanel, /default_collapsed_dx_launch_rail_sections/);
  assert.match(agentPanel, /DxLaunchRailSection::SourceTools/);
  assert.match(agentPanel, /DxLaunchRailSection::AgentOverview/);
  assert.match(agentPanel, /DxLaunchRailSection::AgentThreads/);
  assert.match(agentPanel, /DxLaunchRailSection::AgentTasks/);
  assert.match(agentPanel, /DxLaunchRailSection::AgentSubagents/);
  assert.match(agentPanel, /DxLaunchRailSection::AgentApprovals/);
  assert.match(agentPanel, /toggle_dx_launch_rail_section/);
  assert.match(toolbar, /"Hide sources rail"/);
  assert.match(toolbar, /"Show sources rail"/);
  assert.match(toolbar, /"Hide progress rail"/);
  assert.match(toolbar, /"Show progress rail"/);
  assert.doesNotMatch(toolbar, /Sources rail has no sources yet|Progress rail has no agent activity yet/);
  assert.match(
    toolbar,
    /let rails_available = is_full_screen;/,
    "fullscreen rail toolbar availability must not clone or rebuild launch status during render",
  );
  const renderActivityBar = functionBody(threadView, "render_activity_bar");
  assert.match(renderActivityBar, /let pending_edits = thread\.has_pending_edit_tool_calls\(\);/);
  assert.doesNotMatch(renderActivityBar, /let pending_edits = false/);
  assert.doesNotMatch(toolbar, /fullscreen_launch_status/);
  assert.doesNotMatch(toolbar, /with_live_dx_launch_status\(cache\.status\.clone\(\), cx\)/);
  assert.doesNotMatch(toolbar, /Show or hide sources|Show or hide progress/);
  assert.match(agentPanel, /DxLaunchRailControls\s*\{/);
  assert.match(dxLaunchWorkspace, /enum DxLaunchRailSide/);
  assert.match(dxLaunchWorkspace, /fn rail_pin_header\(/);
  assert.match(dxLaunchWorkspace, /IconButton::new\(format!\("\{id\}-pin"\), IconName::Pin\)/);
  assert.match(agentPanel, /on_toggle_pin: Arc::new/);
  assert.match(agentPanel, /render_workspace_chrome\([\s\S]*rail_controls/);
  assert.doesNotMatch(dxLaunchWorkspace, /section_title\("Guided Actions"/);
  assert.doesNotMatch(dxLaunchWorkspace, /section_title\("Source Tools"/);
  assert.match(launchChrome, /\.relative\(\)/);
  assert.match(launchChrome, /\.overflow_hidden\(\)/);
  assert.match(launchChrome, /\.child\(div\(\)\.size_full\(\)\.min_w_0\(\)\.overflow_hidden\(\)\.child\(center\)\)/);
  assert.doesNotMatch(launchChrome, /render_response_controller/);
  assert.match(sourcesRail, /\.absolute\(\)/);
  assert.match(sourcesRail, /\.left_2\(\)/);
  assert.match(sourcesRail, /\.w\(px\(300\.0\)\)/);
  assert.match(sourcesRail, /\.max_h\(vh\(0\.86, window\)\)/);
  assert.match(sourcesRail, /\.rounded_lg\(\)/);
  assert.match(sourcesRail, /\.shadow_md\(\)/);
  assert.match(sourcesRail, /\.occlude\(\)/);
  assert.match(sourcesRail, /rail_pin_header\(\s*"dx-sources-rail-pin"/);
  assert.match(sourcesRail, /"dx-sources-controller-section"/);
  assert.match(sourcesRail, /DxLaunchRailSection::SourceController/);
  assert.match(sourcesRail, /sources::source_controller_state\(&status\.source_sets, cx\)/);
  assert.doesNotMatch(sourcesRail, /sidebar_actions/);
  assert.match(sourcesRail, /sources::source_set_stack\(&status\.source_sets, source_row_controls, cx\)/);
  assert.match(sourcesRail, /"dx-sources-tools-section"/);
  assert.match(sourcesRail, /"Next Actions"/);
  assert.match(sourcesRail, /DxLaunchRailSection::SourceTools/);
  assert.match(sourcesRail, /source_actions/);
  assert.doesNotMatch(
    sourcesRail,
    /_sidebar_actions|Vec::new\(\)|No source actions yet/,
  );
  assert.doesNotMatch(sourcesRail, /dx-workspace-state-section/);
  assert.match(progressRail, /\.absolute\(\)/);
  assert.match(progressRail, /\.right_2\(\)/);
  assert.match(progressRail, /\.top_2\(\)/);
  assert.doesNotMatch(progressRail, /\.bottom_2\(\)/);
  assert.match(progressRail, /\.w\(px\(300\.0\)\)/);
  assert.match(progressRail, /\.max_h\(vh\(0\.86, window\)\)/);
  assert.match(progressRail, /\.rounded_lg\(\)/);
  assert.match(progressRail, /\.border_1\(\)/);
  assert.match(progressRail, /\.shadow_md\(\)/);
  assert.match(progressRail, /\.occlude\(\)/);
  assert.match(progressRail, /rail_pin_header\(\s*"dx-progress-rail-pin"/);
  assert.match(
    diagnosticsMenu,
    /IconButton::new\("dx-launch-diagnostics-button", dx_icon\(DxUiIcon::Settings\)\)/,
  );
  assert.doesNotMatch(diagnosticsMenu, /Button::new\("dx-launch-diagnostics-button", "Diagnostics"\)|\.full_width\(\)/);
  assert.doesNotMatch(sourcesRail, /\.border_r_1\(\)/);
  assert.doesNotMatch(progressRail, /\.right_0\(\)/);
  assert.doesNotMatch(progressRail, /\.border_l_1\(\)/);
  assert.doesNotMatch(dxLaunchWorkspace, /sources", status\.source_sets\.total_sources/);
  assert.doesNotMatch(dxLaunchWorkspace, /tasks", status\.background_task_count/);
  assert.doesNotMatch(dxLaunchWorkspace, /Current Agent panel conversation state/);
  assert.doesNotMatch(dxLaunchWorkspace, /Background Agent work visible in the right rail/);
  assert.match(sourceRow, /Tooltip::with_meta\(/);
  assert.match(sourceRow, /IconSize::Small/);
  assert.match(sourceRow, /LabelSize::Small/);
  assert.doesNotMatch(sourceRow, /Label::new\(source\.path\.clone\(\)\)/);
  assert.match(sourceRowControls, /element: h_flex\(\)/);
  assert.doesNotMatch(sourceRowControls, /\.full_width\(\)/);
  assert.match(dxLaunchStylePanel, /^mod rows;$/m);
  assert.match(dxLaunchStylePanel, /use rows::\{style_detail_row, style_note_row, style_path_row, style_section\};/);
  assert.match(
    dxLaunchStylePanel,
    /style_section\(\s*"dx-style-overview-section",\s*"Style",\s*dx_icon\(DxUiIcon::Style\),\s*snapshot\.status\.clone\(\)/,
  );
  assert.match(dxLaunchStylePanel, /style_detail_row\(\s*"dx-style-generators",\s*"Generators",/);
  assert.match(dxLaunchStylePanel, /format!\("\{\} declared", snapshot\.visual_generator_count\)/);
  assert.match(dxLaunchStylePanel, /style_detail_row\(\s*"dx-style-web-preview",\s*"Web Preview",/);
  assert.match(dxLaunchStylePanel, /"Bridge ready"/);
  assert.match(dxLaunchStylePanel, /"Bridge available"/);
  assert.match(dxLaunchStylePanel, /"Bridge missing"/);
  assert.match(
    dxLaunchStylePanel,
    /Button::new\("dx-style-open-generator-preview", "Open Style Controls"\)/,
  );
  assert.match(dxLaunchStylePanel, /style_contract_row\(\s*"Control Catalog",/);
  assert.doesNotMatch(
    dxLaunchStylePanel,
    /Style Cockpit|Open Generator Workspace|Open Generator"|Open Style Generator|Generator Host|Generator Contract|Web Preview ready|Web Preview host present|Web Preview host missing|Host source|Readiness Contracts|Readiness Fixtures|controls ready|host connected|host unavailable|cataloged/,
  );
  assert.match(dxLaunchCheckPanel, /"Readiness score"/);
  assert.doesNotMatch(dxLaunchCheckPanel, /"Rail score"/);
  assert.match(dxCheckScore, /score >= 85 && blockers\.is_empty\(\)/);
  assert.doesNotMatch(dxCheckScore, /let state = if score >= 85 \{\s*"Demo ready"/);
  assert.match(dxAgentWorkspace, /"Quality"/);
  assert.match(dxAgentWorkspace, /"Accounts"/);
  assert.match(dxAgentWorkspace, /"Trusted bridge"/);
  assert.match(dxAgentWorkspace, /"Active tasks"/);
  assert.match(dxAgentWorkspace, /"Release gate"/);
  assert.match(dxAgentWorkspace, /"Trusted tools"/);
  assert.doesNotMatch(dxLaunchWorkspace, /No active subagents|Show \{\} more|is working/);
  assert.doesNotMatch(dxLaunchWorkspace, /source bridge wired|source bridge missing|No automation receipts|Fresh proof|worktree\(s\)|task\(s\)/);
  assert.match(dxLaunchWorkspace, /"Attach"/);
  assert.match(dxLaunchAuditSummary, /"Scenario"/);
  assert.match(dxLaunchAuditSummary, /"Scenario Agents"/);
  assert.match(dxLaunchReadinessExamples, /format!\("Scenario \{\}", ix \+ 1\)/);
  assert.match(dxLaunchAuditStatus, /Missing launch scenario root/);
  assert.match(dxLaunchReadinessStatus, /Missing source-owned launch scenarios/);
  assert.match(dxLaunchContracts, /\{\} packets, \{\} fixture families/);
  assert.match(dxLaunchContracts, /\{\} commands, \{\} actions/);
  assert.match(dxLaunchAuditWarnings, /command-safety review before final handoff/);
  assert.match(dxLaunchContractStatus, /command-safety review before agent import/);
  assert.match(dxLaunchReadinessWarnings, /command-safety review before import/);
  assert.match(dxLaunchWwwWarnings, /runtime status gated/);
  assert.match(dxAgentBridgeWarnings, /command-safety review before import/);
  assert.match(dxAgentBridgeWarnings, /command-safety review before recovery actions/);
  assert.match(agentPanel, /Review schemas, fixtures, smoke, and launch scenarios/);
  assert.doesNotMatch(
    [
      dxLaunchAuditSummary,
      dxLaunchAuditStatus,
      dxLaunchReadinessExamples,
      dxLaunchReadinessStatus,
      dxLaunchContracts,
      dxLaunchAuditWarnings,
      dxLaunchContractStatus,
      dxLaunchReadinessWarnings,
      dxLaunchWwwWarnings,
      dxAgentBridgeWarnings,
      agentPanel,
    ].join("\n"),
    /"Example"|"Example Agents"|"Example Tokens"|"Example Discovery"|Missing launch example root|Missing source-owned launch examples|fixture\(s\)|fixture familie\(s\)|packet\(s\)|command fanout|GPUI import|runtime-green|bridge import|recovery controls|status examples/,
  );
  assert.doesNotMatch(dxLaunchWorkspace, /section_title\("Token And Tool Slots"/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn token_meter_slots\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn background_task_state\(/);

  const badgeSlot = projectPanel.slice(
    projectPanel.indexOf(".end_slot::<AnyElement>("),
    projectPanel.indexOf(".child(if let Some(icon)", projectPanel.indexOf(".end_slot::<AnyElement>(")),
  );
  assert.match(badgeSlot, /\.ml_auto\(\)/);
  assert.doesNotMatch(badgeSlot, /\.pr_1\(\)/);
  assert.match(badgeSlot, /\.justify_end\(\)/);
});

test("agent queued message rows stay locked to queued data", () => {
  const clearQueue = functionBody(threadView, "clear_queue");
  const renderQueueEntries = functionBody(threadView, "render_message_queue_entries");
  const threadRender = threadView.slice(threadView.indexOf("impl Render for ThreadView"));

  assert.match(clearQueue, /self\.local_queued_messages\.clear\(\);/);
  assert.match(clearQueue, /self\.queued_message_editors\.clear\(\);/);
  assert.match(clearQueue, /self\.queued_message_editor_subscriptions\.clear\(\);/);
  assert.match(clearQueue, /self\.last_synced_queue_length = 0;/);
  assert.match(clearQueue, /self\.can_fast_track_queue = false;/);
  assert.match(clearQueue, /self\.sync_queue_flag_to_native_thread\(cx\);/);
  assert.match(
    threadRender,
    /on_action\(cx\.listener\(\|this, _: &ClearMessageQueue[\s\S]*?this\.clear_queue\(cx\);/,
  );
  assert.match(
    renderQueueEntries,
    /let queued_message_count = self\.local_queued_messages\.len\(\);/,
  );
  assert.match(
    renderQueueEntries,
    /let visible_queue_len = queued_message_count\.min\(queued_message_editors\.len\(\)\);/,
  );
  assert.match(renderQueueEntries, /\.take\(visible_queue_len\)/);
  assert.match(renderQueueEntries, /index \+ 1 < visible_queue_len/);
  assert.match(renderQueueEntries, /self\.can_fast_track_queue && visible_queue_len > 0/);
});

test("agent launch rails use professional operator-facing copy", () => {
  const sourceActions = functionBody(agentPanel, "render_dx_launch_source_actions");
  const sourcesRail = functionBody(dxLaunchWorkspace, "render_sources_rail");
  const sourceController = functionBody(dxLaunchSourceController, "source_controller_state");
  const guidedCards = functionBody(agentPanel, "render_dx_launch_guided_cards");
  const agentOverview = functionBody(dxAgentWorkspace, "agent_overview_section");
  const agentEnvironment = functionBody(dxAgentWorkspace, "agent_environment_section");
  const agentSources = functionBody(dxAgentWorkspace, "agent_sources_section");
  const agentApprovals = functionBody(dxAgentWorkspace, "agent_approvals_section");
  const subagentSummary = functionBody(dxLaunchWorkspace, "subagent_summary");
  const styleState = functionBody(dxLaunchStylePanel, "dx_style_panel_state");
  const webPreviewState = functionBody(dxStylePanelCards, "web_preview_state");
  const sourceSetStatus = functionBody(dxSourceSetFormatting, "source_set_status");

  assert.doesNotMatch(agentPanel, /render_dx_launch_sidebar_actions/);
  assert.match(sourcesRail, /"Source Controller"/);
  assert.match(sourcesRail, /"Next Actions"/);
  assert.match(sourcesRail, /status\.source_sets\.total_sources > 0/);
  assert.match(sourcesRail, /has_source_actions\(status\)/);
  assert.match(sourceController, /"Workspace roots"/);
  assert.match(sourceController, /"Managed receipts"/);
  assert.match(sourceActions, /"Review Source"/);
  assert.match(sourceActions, /"Review Deploy Readiness"/);
  assert.doesNotMatch(sourceActions, /"No source actions yet"/);
  assert.match(`${dxAgentSocialActions}\n${agentConfiguration}`, /QR supported/);
  assert.match(`${dxAgentSocialActions}\n${agentConfiguration}`, /link supported/);
  assert.doesNotMatch(`${dxAgentSocialActions}\n${agentConfiguration}`, /QR ready|link ready/);
  assert.match(guidedCards, /"Prepare Handoff"/);
  assert.match(guidedCards, /"Review Gate"/);
  assert.match(guidedCards, /"Review Audit"/);
  assert.match(guidedCards, /"Review Sources"/);
  assert.match(guidedCards, /"Review DX-WWW"/);
  assert.match(guidedCards, /"Prepare Runtime Proof"/);
  assert.match(guidedCards, /"Prepare Import"/);
  assert.match(guidedCards, /"Prepare Evidence Form"/);
  assert.match(guidedCards, /"Prepare Approval"/);
  assert.match(guidedCards, /"Review Guard"/);
  assert.doesNotMatch(
    agentPanel,
    /"Draft Action"|"Draft Check"|"Draft Handoff"|"Draft Gate"|"Draft Audit"|"Draft Source"|"Draft WWW"|"Draft Proof"|"Draft Import"|"Draft Form"|"Draft Approval"|"Draft Guard"|"No source actions available"/,
  );

  assert.match(agentOverview, /"Bridge"/);
  assert.match(agentOverview, /"Active tasks"/);
  assert.match(agentOverview, /"Release gate"/);
  assert.match(agentOverview, /"Quality"/);
  assert.match(agentEnvironment, /"Background threads"/);
  assert.match(agentEnvironment, /"Accounts"/);
  assert.match(agentEnvironment, /"Trusted tools"/);
  assert.match(agentSources, /"Attachable"/);
  assert.match(agentSources, /"Managed receipts"/);
  assert.match(agentApprovals, /"Trusted bridge"/);
  assert.match(agentApprovals, /"Blocked tools"/);
  assert.match(subagentSummary, /"Active Tasks"/);
  assert.doesNotMatch(subagentSummary, /"No live subagent state"/);
  assert.doesNotMatch(
    dxLaunchWorkspace,
    /source bridge wired|source bridge missing|No automation receipts|Fresh proof|worktree\(s\)|task\(s\)/,
  );

  assert.match(dxSourceSets, /"No workspace root found"/);
  assert.match(dxSourceSets, /"No source pack receipts found"/);
  assert.match(dxSourceSets, /"No media outputs found"/);
  assert.match(dxSourceSets, /"No restore previews found"/);
  assert.match(dxSourceSets, /"No reduced context receipts found"/);
  assert.match(sourceSetStatus, /"No workspace open"/);
  assert.match(dxSourceSetDxEditorToolchain, /"No extensionless dx config found"/);
  assert.match(dxLaunchSourceAttachments, /"No attachable sources found"/);
  assert.match(dxLaunchSourceAttachments, /"Attachable"/);
  assert.match(dxLaunchSourceAttachments, /"Source pack or media receipt required for attachments"/);
  assert.match(dxLaunchSourceReceipts, /"Receipt directory not found: \{\}"/);
  assert.match(dxLaunchSourceReceipts, /"No DX receipts found"/);
  assert.match(dxCheckScore, /"No managed attachable source receipts"/);
  assert.match(dxCheckScore, /"\{\} worktrees, \{\} roots"/);
  assert.match(dxCheckScore, /"\{\} attachable, \{\} total"/);

  assert.match(styleState, /"Bridge ready"/);
  assert.match(styleState, /"Bridge available"/);
  assert.match(styleState, /"Bridge missing"/);
  assert.match(webPreviewState, /"Ready"/);
  assert.match(agentConfiguration, /"\{\} active tasks, \{\} automations,/);
  assert.match(dxLaunchStatusSummary, /"\{\} automations, \{\} active, \{\} QR-ready"/);
  assert.doesNotMatch(
    [
      dxSourceSets,
      dxSourceSetFormatting,
      dxSourceSetDxEditorToolchain,
      dxLaunchSourceAttachments,
      dxLaunchSourceReceipts,
      dxCheckScore,
      dxLaunchStylePanel,
      dxStylePanelCards,
      agentConfiguration,
      dxLaunchStatusSummary,
    ].join("\n"),
    /No source-pack receipts|No produced media outputs|No reduced-context receipts|Attach-ready|attach-ready|source bridge wired|source bridge missing|task\(s\)|worktree\(s\)|automation\(s\)|Receipts not found|Waiting for first DX receipt|No extensionless dx config"/,
  );
});

test("agent layout preset keeps project, git, outline, and collab on the left", () => {
  const agentLayoutStart = agentSettings.indexOf("const AGENT: Self = Self");
  assert.ok(agentLayoutStart >= 0, "expected Agent preset layout");
  const agentLayout = agentSettings.slice(
    agentLayoutStart,
    agentSettings.indexOf("const EDITOR: Self = Self", agentLayoutStart),
  );

  assert.match(agentLayout, /agent_dock:\s*Some\(DockPosition::Left\)/);
  assert.match(agentLayout, /project_panel_dock:\s*Some\(DockSide::Left\)/);
  assert.match(agentLayout, /outline_panel_dock:\s*Some\(DockSide::Left\)/);
  assert.match(agentLayout, /collaboration_panel_dock:\s*Some\(DockPosition::Left\)/);
  assert.match(agentLayout, /git_panel_dock:\s*Some\(DockPosition::Left\)/);
});

test("core left panels expose close controls in native headers", () => {
  const projectSelectionToolbar = functionBody(
    projectPanel,
    "render_selected_entries_toolbar",
  );
  const emptyProjectWrapperStart = projectPanel.indexOf(
    '.id("empty-project_panel-wrapper")',
  );
  assert.ok(emptyProjectWrapperStart >= 0, "expected Project empty wrapper");
  const emptyProjectWrapper = projectPanel.slice(
    emptyProjectWrapperStart,
    projectPanel.indexOf("ProjectEmptyState::new", emptyProjectWrapperStart) + 240,
  );
  const gitTabBar = functionBody(gitPanel, "render_tab_bar");
  const gitExpandedCommitHeader = functionBody(
    gitPanel,
    "render_expanded_commit_header",
  );
  const outlineFooter = functionBody(outlinePanel, "render_filter_footer");
  const collabHeader = functionBody(collabPanel, "render_panel_header");
  const collabDisabled = functionBody(
    collabPanel,
    "render_disabled_by_organization",
  );
  const collabSignedOut = functionBody(collabPanel, "render_signed_out");
  const collabSignedIn = functionBody(collabPanel, "render_signed_in");

  assert.ok(
    !dock.includes('format!("{id_prefix}-split-side-panel")'),
    "shared dock helper must not expose a visible split-control id",
  );
  assert.ok(
    dock.includes('format!("{id_prefix}-close-side-panel")'),
    "shared dock helper must create a stable close-control id",
  );
  assert.match(dock, /IconName::Close/);
  const sidePanelHeaderControls = functionBody(dock, "side_panel_header_controls");
  assert.doesNotMatch(sidePanelHeaderControls, /IconName::SplitAlt/);
  assert.doesNotMatch(sidePanelHeaderControls, /let can_split = workspace/);
  assert.doesNotMatch(sidePanelHeaderControls, /Open another panel to split/);
  assert.match(dock, /let panel_is_registered = workspace[\s\S]*?\.upgrade\(\)[\s\S]*?\.is_some_and/);
  assert.doesNotMatch(dock, /\.disabled\(!can_split\)|\.disabled\(!panel_is_registered\)/);
  assert.match(dock, /Panel is not available/);
  assert.doesNotMatch(
    sidePanelHeaderControls,
    /workspace\.split_side_panel_by_id\(panel_id, window, cx\)/,
  );
  assert.match(dock, /workspace\.close_side_panel_by_id\(panel_id, window, cx\)/);
  assert.match(dock, /pub fn can_split_panel_by_id/);
  assert.match(dock, /pub fn contains_panel_id/);
  assert.match(workspace, /pub fn split_side_panel_by_id/);
  assert.match(workspace, /pub fn close_side_panel_by_id/);
  assert.match(workspace, /pub fn can_split_side_panel_by_id/);
  assert.match(workspace, /pub fn contains_side_panel_by_id/);

  for (const [source, name] of [
    [projectPanel, "project panel"],
    [gitPanel, "git panel"],
    [outlinePanel, "outline panel"],
    [collabPanel, "collab panel"],
  ] as const) {
    assert.match(
      source,
      /side_panel_header_controls[\s\S]*?self\.workspace\.clone\(\)[\s\S]*?cx\.entity\(\)\.entity_id\(\)[\s\S]*?cx,/,
      `${name} must target its own panel entity for close controls`,
    );
    assert.doesNotMatch(
      source,
      /workspace::SplitActiveSidePanel|workspace::CloseActiveSidePanel/,
      `${name} header controls must not depend on active side-panel focus`,
    );
  }

  assert.doesNotMatch(projectPanel, /fn render_panel_header/);
  assert.match(projectPanel, /render_side_panel_header_controls\("dx-explorer", cx\)/);
  assert.doesNotMatch(projectPanel, /side_panel_header_controls\(\s*"project-panel-sticky",/);
  assert.doesNotMatch(emptyProjectWrapper, /render_panel_header\(cx\)/);
  assert.match(
    projectSelectionToolbar,
    /side_panel_header_controls\(\s*"project-panel-selection",/,
  );
  assert.match(gitTabBar, /render_side_panel_header_controls\(cx\)/);
  assert.match(
    gitExpandedCommitHeader,
    /render_side_panel_header_controls\(cx\)/,
  );
  assert.match(outlineFooter, /side_panel_header_controls\(\s*"outline-panel",/);
  assert.match(collabHeader, /side_panel_header_controls\(\s*"collab-panel",/);
  assert.match(collabDisabled, /self\.render_panel_header\(cx\)/);
  assert.match(collabSignedOut, /self\.render_panel_header\(cx\)/);
  assert.match(collabSignedIn, /self\.render_panel_header\(cx\)/);
});

test("panel headers keep titles flexible and side actions fixed", () => {
  assert.match(
    dock,
    /\.id\(format!\("\{id_prefix\}-side-panel-controls"\)\)[\s\S]*?\.flex_none\(\)/,
    "shared side-panel actions should not shrink in narrow stacked panels",
  );

  const gitTabBar = functionBody(gitPanel, "render_tab_bar");
  const outlineFooter = functionBody(outlinePanel, "render_filter_footer");
  const collabHeader = functionBody(collabPanel, "render_panel_header");
  const collabSignedIn = functionBody(collabPanel, "render_signed_in");

  assert.doesNotMatch(projectPanel, /fn render_panel_header/);
  assert.match(gitTabBar, /\.h_full\(\)[\s\S]*?\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(outlineFooter, /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(outlineFooter, /\.items_center\(\)[\s\S]*?\.gap_0p5\(\)[\s\S]*?\.flex_none\(\)/);
  assert.match(collabHeader, /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(
    collabSignedIn,
    /div\(\)[\s\S]*?\.flex_1\(\)[\s\S]*?\.min_w_0\(\)[\s\S]*?render_filter_input/,
  );

  for (const [source, actionId, label] of [
    [iconPicker, "side_panel_header_controls", "Icons"],
    [fontPanel, "side_panel_header_controls", "Fonts"],
    [mediaPanel, "side_panel_header_controls", "Media"],
    [uiPanel, "side_panel_header_controls", "UI"],
    [stylePanel, "side_panel_header_controls", "Style"],
  ] as const) {
    assert.match(source, new RegExp(actionId));
    assert.match(sourceWindow(source, `Label::new("${label}")`), /\.truncate\(\)/);
    assert.match(
      sourceWindow(source, `Label::new("${label}")`),
      /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/,
    );
  }
});

test("recent tool panels use professional visible copy", () => {
  assert.match(titleBar, /Tooltip::text\("More"\)/);
  assert.doesNotMatch(titleBar, /More Hidden Features/);

  assert.match(stylePanel, /Label::new\("Style"\)/);
  assert.match(stylePanel, /id\("dx-style-panel-web-preview-host"\)/);
  assert.match(stylePanel, /id\("dx-style-panel-web-preview"\)/);
  assert.match(stylePanel, /Label::new\("Web Preview"\)/);
  assert.match(stylePanel, /Label::new\("Styles"\)/);
  assert.doesNotMatch(stylePanel, /section_label\("Contracts"\)/);
  assert.doesNotMatch(stylePanel, /Style Generators|Readiness Contracts/);

  assert.match(dxStylePanelCards, /metric\("Web Preview", web_preview_state\(snapshot\)\)/);
  assert.match(dxStylePanelCards, /metric\(\s*"Generators",/);
  assert.match(dxStylePanelCards, /\{\} declared/);
  assert.match(dxStylePanelCards, /"Open Style Controls"/);
  assert.match(dxStylePanelCards, /"Ready"/);
  assert.match(dxStylePanelCards, /"Bridge available"/);
  assert.match(dxStylePanelCards, /"Bridge missing"/);
  assert.match(dxStylePanelCards, /apply_gate_state_label/);
  assert.match(dxStylePanelCards, /metric\("Apply", apply_gate_state_label\(&gate\.state\)\)/);
  assert.doesNotMatch(
    dxStylePanelCards,
    /"Host"|"Open Web Preview Controls"|"Open Web Preview Generators"|generator bridge ready|host present|host missing|controls ready|host connected|host unavailable|cataloged controls/,
  );

  assert.match(iconPicker, /icon_history_count_label\(self\.recent_icon_actions\.len\(\), "action", "actions"\)/);
  assert.match(iconPicker, /icon_history_count_label\(self\.pinned_icon_actions\.len\(\), "pinned", "pinned"\)/);
  assert.match(iconPicker, /\{count\} \{plural\}/);
  assert.doesNotMatch(iconPicker, /saved|\{count\} ready|"1 ready"/);

  assert.match(fontPanel, /" in Web Preview"/);
  assert.match(fontPanel, /"Selected "/);
  assert.match(fontPanel, /"Opening preview for "/);
  assert.match(fontPanel, /font_history_count_label\(self\.recent_font_actions\.len\(\), "action", "actions"\)/);
  assert.match(fontPanel, /font_history_count_label\(self\.pinned_font_actions\.len\(\), "pinned", "pinned"\)/);
  assert.doesNotMatch(fontPanel, /Previewing |saved| in WebPreview|\{count\} ready|"1 ready"/);

  assert.match(mediaPanel, /media_history_availability_label/);
  assert.match(mediaPanel, /"Opening preview for "/);
  assert.match(mediaPanel, /IconButton::new\(\s*"media-panel-remove-missing-recent",\s*IconName::ListX/);
  assert.match(mediaPanel, /IconButton::new\(\s*"media-panel-remove-missing-pinned",\s*IconName::ListX/);
  assert.match(mediaPanel, /IconButton::new\("media-panel-clear-recent", IconName::Trash\)/);
  assert.match(mediaPanel, /IconButton::new\("media-panel-clear-pinned", IconName::Trash\)/);
  assert.match(mediaPanel, /"media-panel-remove-missing-history"/);
  assert.match(mediaPanel, /\{available\} available/);
  assert.match(mediaPanel, /\{available\} available, \{missing\} missing/);
  assert.match(mediaPanel, /No missing \{section\} entries/);
  assert.match(mediaPanel, /open remote sources/);
  assert.match(mediaPanel, /Clear recent media entries/);
  assert.match(mediaPanel, /Pinned media and search results stay/);
  assert.match(mediaPanel, /Pin to pinned media/);
  assert.match(mediaPanel, /Remove this entry from history/);
  assert.doesNotMatch(mediaPanel, /Previewing |"Clean"|use Clean|use Remove|CLEAN_STALE|remove-stale|\bstale\b|No-key|No no-key|no-key|recent media actions|local index|remote cache|working set|\{ready\} ready|available \//);

  assert.match(uiPanel, /"Preview in Web Preview"/);
  assert.match(uiPanel, /ui_history_availability_label/);
  assert.match(uiPanel, /"Opening preview for "/);
  assert.match(uiPanel, /IconButton::new\(\s*"shadcn-ui-remove-missing-recent",\s*IconName::ListX/);
  assert.match(uiPanel, /IconButton::new\(\s*"shadcn-ui-remove-missing-pinned",\s*IconName::ListX/);
  assert.match(uiPanel, /IconButton::new\("shadcn-ui-clear-recent", IconName::Trash\)/);
  assert.match(uiPanel, /IconButton::new\("shadcn-ui-clear-pinned", IconName::Trash\)/);
  assert.match(uiPanel, /"shadcn-ui-remove-missing-history"/);
  assert.match(uiPanel, /\{available\} available/);
  assert.match(uiPanel, /\{available\} available, \{missing\} missing/);
  assert.match(uiPanel, /No missing \{section\} entries/);
  assert.match(uiPanel, /Changes queued/);
  assert.match(uiPanel, /UI registry preview/);
  assert.match(uiPanel, /Clear recent UI entries/);
  assert.match(uiPanel, /Component source is missing/);
  assert.match(uiPanel, /Install missing component files without overwriting existing files/);
  assert.match(uiPanel, /Open component source/);
  assert.match(uiPanel, /Remove this entry from history/);
  assert.doesNotMatch(uiPanel, /Previewing |"Clean"|use Clean|use Remove|CLEAN_STALE|remove-stale|\bstale\b|recent UI action|pinned UI action|Recent UI actions|registry files|registry manifest|primary target|No editor paste|Open this registry source|working set|Saved changes|The UI registry is ready|Preview in WebPreview|\{ready\} ready|available \//);
});

test("item project-handle collections cap visited items before pushing handles", () => {
  assert.match(item, /const MAX_PROJECT_ITEMS_PER_ITEM: usize = 512;/);

  const itemHandleImpl = item.slice(
    item.indexOf("impl<T: Item> ItemHandle for Entity<T>"),
  );
  for (const name of [
    "project_entry_ids",
    "project_paths",
    "project_item_model_ids",
  ]) {
    const body = functionBody(itemHandleImpl, name);
    assertBefore({
      body,
      before: "if !should_collect_project_item",
      after: "result.push(",
      message: `${name} must cap project-item visits before pushing handles`,
    });
    assert.match(body, /log_project_item_collection_truncated/);
  }

  const helper = functionBody(item, "should_collect_project_item");
  assertBefore({
    body: helper,
    before: "*visited >= MAX_PROJECT_ITEMS_PER_ITEM",
    after: "*visited += 1;",
    message: "project-item collection helper must check cap before incrementing visits",
  });
});

test("workspace pane cycling uses checked target pane lookups", () => {
  const nextPane = functionBody(workspace, "activate_next_pane");
  assert.doesNotMatch(
    nextPane,
    /panes\s*\[\s*next_ix\s*\]/,
    "next-pane activation must not directly index stale pane snapshots",
  );
  assert.match(
    nextPane,
    /panes\s*\.get\(\s*next_ix\s*\)/,
    "next-pane activation must check the target pane still exists",
  );

  const previousPane = functionBody(workspace, "activate_previous_pane");
  assert.doesNotMatch(
    previousPane,
    /panes\s*\[\s*prev_ix\s*\]/,
    "previous-pane activation must not directly index stale pane snapshots",
  );
  assert.match(
    previousPane,
    /panes\s*\.get\(\s*prev_ix\s*\)/,
    "previous-pane activation must check the target pane still exists",
  );
});

test("workspace pane tab materialization checks stale item indexes", () => {
  const renderTab = functionBody(pane, "render_tab");
  assert.doesNotMatch(
    renderTab,
    /self\.items\s*\[\s*ix\s*\]/,
    "render_tab must use its provided item handle instead of directly indexing self.items[ix]",
  );

  const renderTabBar = functionBody(pane, "render_tab_bar");
  assert.doesNotMatch(
    renderTabBar,
    /self\.items\s*\[\s*ix\s*\]/,
    "render_tab_bar must not directly index stale visible item indexes",
  );
  assert.match(
    renderTabBar,
    /self\.items\s*\.get\(\s*ix\s*\)/,
    "render_tab_bar must check visible indexes before cloning item handles",
  );
});

test("pane item removal checks stale item indexes before mutation side effects", () => {
  const removeItem = functionBody(pane, "_remove_item");
  assert.doesNotMatch(
    removeItem,
    /self\.items\s*\[\s*item_index\s*\]/,
    "_remove_item must not directly index self.items[item_index]",
  );
  assertBefore({
    body: removeItem,
    before:
      /let\s+Some\(item_id\)\s*=\s*self\.items\s*\.get\(\s*item_index\s*\)\s*\.map\(\|item\|\s*item\.item_id\(\)\)\s*else/,
    after: /self\.activation_history\s*[\s\S]*?\.retain\(/,
    message:
      "_remove_item must verify item_index before retaining activation history",
  });
  assertBefore({
    body: removeItem,
    before:
      /self\.items\s*\.get\(\s*item_index\s*\)\s*\.map\(\|item\|\s*item\.item_id\(\)\)\s*!=\s*Some\(item_id\)/,
    after: /self\.items\s*\.remove\(\s*item_index\s*\)/,
    message: "_remove_item must recheck item_index before removing",
  });
});

test("pinned tab movement checks current source and destination indexes", () => {
  const pinnedDrop = functionBody(pane, "handle_pinned_tab_bar_drop");
  assertBefore({
    body: pinnedDrop,
    before:
      /if\s+actual_ix\s*>=\s*items_len\s*\|\|\s*destination_ix\s*>=\s*items_len/,
    after: /this\.items\.remove\(\s*actual_ix\s*\)/,
    message:
      "pinned tab movement must verify source and destination before removing",
  });
  assertBefore({
    body: pinnedDrop,
    before:
      /if\s+actual_ix\s*>=\s*items_len\s*\|\|\s*destination_ix\s*>=\s*items_len/,
    after: /this\.items\.insert\(\s*destination_ix\s*,\s*item\s*\)/,
    message:
      "pinned tab movement must verify source and destination before inserting",
  });
});
