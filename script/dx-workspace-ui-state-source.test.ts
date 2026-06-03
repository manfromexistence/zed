import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const dock = read("crates/workspace/src/dock.rs");
const agentSettings = read("crates/agent_settings/src/agent_settings.rs");
const historyManager = read("crates/workspace/src/history_manager.rs");
const item = read("crates/workspace/src/item.rs");
const pane = read("crates/workspace/src/pane.rs");
const workspace = read("crates/workspace/src/workspace.rs");
const multiWorkspace = read("crates/workspace/src/multi_workspace.rs");
const titleBar = read("crates/title_bar/src/title_bar.rs");
const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const dxLaunchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
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
  assert.match(dockRender, /let visible_panels = self\s*\.visible_entries\(\)/);
  assert.match(dockRender, /\.when\(is_stacked, \|this\| this\.flex\(\)\.flex_col\(\)\)/);
  assert.match(dockRender, /\.flex_1\(\)/);
  assert.match(dockRender, /\.border_t_1\(\)/);
  assert.match(dockRender, /dock\.activate_panel\(panel_ix, window, cx\);/);
  assertBefore({
    body: canSplitPanel,
    before: /self\.panel_index_for_id\(panel_id\)\.is_none\(\)/,
    after: /self\.first_stack_candidate_for\(panel_id, cx\)\.is_none\(\)/,
    message:
      "split controls must only enable for panels contained in the current dock",
  });

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
  assert.match(agentPanel, /"agent-panel-split-side-panel"/);
  assert.match(agentPanel, /"agent-panel-close-side-panel"/);
  assert.match(iconPicker, /"icon-picker-split-side-panel"/);
  assert.match(fontPanel, /"font-panel-split-side-panel"/);
  assert.match(mediaPanel, /"media-panel-split-side-panel"/);
  assert.match(uiPanel, /"shadcn-ui-split-side-panel"/);
  assert.match(stylePanel, /"dx-style-panel-split-side-panel"/);
  assert.doesNotMatch(dockRender, /"dock-panel-inline-split"/);
  assert.doesNotMatch(dockRender, /"dock-panel-inline-close"/);
  assert.doesNotMatch(dockRender, /"dock-panel-inline-control-mask"/);
  assert.doesNotMatch(panelButtonsRender, /"dock-panel-stack"/);
  assert.match(dockRender, /cursor_row_resize/);
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

test("core side panels expose dock split and close controls in visible headers", () => {
  const sidePanelHeaderControls = functionBody(dock, "side_panel_header_controls");
  const projectHeader = functionBody(projectPanel, "render_panel_header");
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

  assert.match(projectHeader, /side_panel_header_controls\(\s*"project-panel"/);
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
    "core side-panel split/close buttons must stay visible in narrow headers",
  );
  assert.doesNotMatch(
    sidePanelHeaderControls,
    /ButtonStyle::Subtle/,
    "core side-panel split/close buttons should use the same visible square treatment as tool panels",
  );
  assert.match(
    sidePanelHeaderControls,
    /IconButtonShape::Square/,
    "core side-panel split/close buttons should remain compact square header actions",
  );
  assert.match(
    sidePanelHeaderControls,
    /contains_side_panel_by_id\(panel_id, cx\)/,
    "close tooltip should still use real side-panel registration state",
  );
});

test("agent fullscreen uses agent rails while sidebar button remains dock-scoped", () => {
  const fullscreenCenter = functionBody(agentPanel, "render_fullscreen_agent_center");
  const messageEditor = functionBody(threadView, "render_message_editor");
  const toolbar = functionBody(agentPanel, "render_toolbar");
  assert.match(agentPanel, /"agent-toolbar-toggle-sources-rail"/);
  assert.match(agentPanel, /"agent-toolbar-toggle-progress-rail"/);
  assert.match(agentPanel, /fullscreen_sources_rail_open/);
  assert.match(agentPanel, /fullscreen_progress_rail_open/);
  assert.match(agentPanel, /fn render_fullscreen_agent_center\(/);
  assert.match(agentPanel, /fn render_toolbar_response_indicator\(/);
  assert.match(agentPanel, /fn toolbar_response_indicator_segment\(/);
  assert.match(toolbar, /\.id\("agent-panel-toolbar"\)[\s\S]*\.relative\(\)/);
  assert.match(toolbar, /render_toolbar_response_indicator\(cx\)/);
  assert.match(workspace, /zoomed_is_agent_panel: bool/);
  assert.match(workspace, /zoomed_is_agent_panel: false/);
  assert.match(workspace, /pub\(crate\) fn zoomed_is_agent_panel\(&self\) -> bool/);
  assert.match(workspace, /pub fn client_side_decorations_with_content_flush\(/);
  assert.match(workspace, /content_flush_tiling: Tiling/);
  assert.match(workspace, /let content_tiling = Tiling\s*\{/);
  assert.match(workspace, /\.when\(!content_tiling\.right/);
  assert.match(workspace, /WorkspaceSettings::get_global\(cx\)\.zoomed_padding\s*\|\|\s*self\.zoomed_is_agent_panel/);
  assert.match(multiWorkspace, /client_side_decorations_with_content_flush/);
  assert.match(multiWorkspace, /let agent_fullscreen_flush_right = workspace\.read\(cx\)\.zoomed_is_agent_panel\(\);/);
  assert.match(multiWorkspace, /right: agent_fullscreen_flush_right/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = panel\.is_agent_panel\(cx\)/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = panel\.read\(cx\)\.is_agent_panel\(\)/);
  assert.match(dock, /workspace\.zoomed_is_agent_panel = false/);
  assert.match(threadView, /struct AgentResponseAnchor/);
  assert.match(threadView, /ScrollPositionChanged/);
  assert.match(threadView, /pub\(crate\) fn response_anchors\(&self, cx: &App\) -> Vec<AgentResponseAnchor>/);
  assert.match(threadView, /pub\(crate\) fn scroll_to_response_anchor\(/);
  assert.match(threadView, /cx\.emit\(AcpThreadViewEvent::ScrollPositionChanged\)/);
  assert.match(agentPanel, /active_thread\.read\(cx\)\.response_anchors\(cx\)/);
  assert.match(agentPanel, /AcpThreadViewEvent::ScrollPositionChanged => \{\s*cx\.notify\(\);\s*\}/);
  assert.match(agentPanel, /Tooltip::with_meta\(label\.clone\(\), None, detail\.clone\(\), cx\)/);
  assert.match(agentPanel, /thread\.scroll_to_response_anchor\(entry_ix, cx\)/);
  assert.match(agentPanel, /\.w\(px\(2\.0\)\)/);
  assert.match(agentPanel, /\.h\(height\)/);
  assert.match(agentPanel, /"agent-fullscreen-center"/);
  assert.doesNotMatch(fullscreenCenter, /\.px_4\(\)/);
  assert.doesNotMatch(fullscreenCenter, /\.pb_3\(\)/);
  assert.doesNotMatch(fullscreenCenter, /max_content_width/);
  assert.match(messageEditor, /\.pt_0p5\(\)/);
  assert.match(messageEditor, /\.pb_2\(\)/);
  assert.match(messageEditor, /\.p_1p5\(\)/);
  assert.match(threadView, /EditorMode::AutoHeight\s*\{\s*min_lines: 2,\s*max_lines: Some\(2\),/s);
  assert.doesNotMatch(messageEditor, /render_composer_status_row/);
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
  assert.match(sidebar, /thread_sort_mode: SidebarThreadSortMode/);
  assert.match(sidebar, /thread_icon_overrides: HashMap<ThreadId, IconName>/);
  assert.match(sidebar, /struct DraggedSidebarThread/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*"dragged-sidebar-thread"/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*Label::new\(self\.label\.clone\(\)\)/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*\.w\(px\(236\.0\)\)/);
  assert.match(sidebar, /impl Render for DraggedSidebarThread[\s\S]*\.color\(Color::Default\)/);
  assert.match(sidebar, /let dragged_thread = DraggedSidebarThread \{[\s\S]*?subtitle: None,/);
  assert.match(sidebar, /subtitle: None,\s*action: SerializedSidebarGridAction::OpenThread/s);
  assert.match(sidebar, /matches!\(action, SidebarGridAction::OpenThread\(_\)\)/);
  assert.match(sidebar, /struct ThreadIconPickerMenu/);
  assert.match(
    sidebar,
    /thread_icon_picker_handles: RefCell<HashMap<ThreadId, PopoverMenuHandle<ThreadIconPickerMenu>>>/,
  );
  assert.match(sidebar, /"thread-icon-picker-grid"/);
  assert.match(sidebar, /"thread-icon-picker-grid-icons"/);
  assert.match(sidebar, /"thread-icon-picker-close"/);
  assert.match(sidebar, /cx\.emit\(DismissEvent\)/);
  assert.match(sidebar, /\.pr_5\(\)/);
  assert.match(sidebar, /"sidebar-chat-sort-\{label\}"/);
  assert.match(sidebar, /"thread-icon-picker"/);
  assert.match(sidebar, /\.with_handle\(icon_picker_handle\)/);
  assert.match(sidebar, /is_hovered \|\| is_icon_picker_open/);
  assert.match(sidebar, /IconButton::new\(\("thread-icon-picker", ix\), IconName::Sparkle\)/);
  assert.match(sidebar, /IconName::iter\(\)/);
  assert.match(sidebar, /SerializedThreadIconOverride/);
  assert.match(threadItem, /let timestamp_color = if self\.selected \|\| self\.hovered/);
  assert.match(threadItem, /Label::new\(timestamp\.clone\(\)\)[\s\S]*\.color\(timestamp_color\)/);
  assert.doesNotMatch(sidebar, /ContextMenuEntry::new\(format!\("\{icon_name:\?\}"\)\)/);
});

test("agent rails and project badges keep compact production layout", () => {
  const launchChrome = functionBody(dxLaunchWorkspace, "render_workspace_chrome");
  const sourcesRail = functionBody(dxLaunchWorkspace, "render_sources_rail");
  const progressRail = functionBody(dxLaunchWorkspace, "render_right_rail");
  const diagnosticsMenu = functionBody(dxLaunchWorkspace, "diagnostics_menu");
  const sourceRow = functionBody(dxLaunchSourceRows, "source_item_row");
  const sourceRowControls = functionBody(agentPanel, "render_dx_launch_source_row_controls");
  assert.match(dxLaunchWorkspace, /fn progress_summary\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn render_response_controller\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_indicator_segment\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_controller_pill\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn response_controller_tick\(/);
  assert.match(dxLaunchWorkspace, /enum DxLaunchRailSection/);
  assert.match(dxLaunchWorkspace, /struct DxLaunchRailControls/);
  assert.match(dxLaunchWorkspace, /fn rail_section\(/);
  assert.match(dxLaunchWorkspace, /Disclosure::new\(format!\("\{id\}-disclosure"\), is_open\)/);
  assert.match(dxLaunchWorkspace, /"dx-progress-summary-section"/);
  assert.match(dxLaunchWorkspace, /"dx-environment-section"/);
  assert.match(dxLaunchWorkspace, /"dx-subagents-section"/);
  assert.match(dxLaunchWorkspace, /"dx-source-summary-section"/);
  assert.match(dxLaunchWorkspace, /"dx-readiness-section"/);
  assert.match(dxLaunchWorkspace, /fn subagent_pixel_icon/);
  assert.match(dxLaunchWorkspace, /gpui::hsla\(210\.0 \/ 360\.0/);
  assert.match(dxLaunchWorkspace, /status\.agent_bridge\.automations\.iter\(\)\.take\(6\)/);
  assert.match(dxLaunchWorkspace, /muted_card\("No subagent activity", cx\)/);
  assert.match(agentPanel, /collapsed_dx_launch_rail_sections: HashSet<DxLaunchRailSection>/);
  assert.match(agentPanel, /default_collapsed_dx_launch_rail_sections/);
  assert.match(agentPanel, /DxLaunchRailSection::SourceTools/);
  assert.match(agentPanel, /DxLaunchRailSection::WorkspaceState/);
  assert.match(agentPanel, /DxLaunchRailSection::Readiness/);
  assert.match(agentPanel, /toggle_dx_launch_rail_section/);
  assert.match(agentPanel, /DxLaunchRailControls\s*\{/);
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
  assert.match(sourcesRail, /\.rounded_lg\(\)/);
  assert.match(sourcesRail, /\.shadow_md\(\)/);
  assert.match(sourcesRail, /\.occlude\(\)/);
  assert.match(progressRail, /\.absolute\(\)/);
  assert.match(progressRail, /\.right_2\(\)/);
  assert.match(progressRail, /\.top_2\(\)/);
  assert.match(progressRail, /\.bottom_2\(\)/);
  assert.match(progressRail, /\.w\(px\(300\.0\)\)/);
  assert.match(progressRail, /\.rounded_lg\(\)/);
  assert.match(progressRail, /\.border_1\(\)/);
  assert.match(progressRail, /\.shadow_md\(\)/);
  assert.match(progressRail, /\.occlude\(\)/);
  assert.match(diagnosticsMenu, /IconButton::new\("dx-launch-diagnostics-button", IconName::Sliders\)/);
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
  assert.match(dxLaunchStylePanel, /metric_row\("Style", snapshot\.status\.clone\(\)\)/);
  assert.match(dxLaunchStylePanel, /metric_row\(\s*"Generators",/);
  assert.match(dxLaunchStylePanel, /format!\("\{\} declared", snapshot\.visual_generator_count\)/);
  assert.match(dxLaunchStylePanel, /metric_row\(\s*"Web Preview",/);
  assert.match(dxLaunchStylePanel, /"Preview bridge ready"/);
  assert.match(dxLaunchStylePanel, /"Host source present"/);
  assert.match(dxLaunchStylePanel, /"Host source missing"/);
  assert.match(
    dxLaunchStylePanel,
    /Button::new\("dx-style-open-generator-preview", "Open Style Controls"\)/,
  );
  assert.match(dxLaunchStylePanel, /style_contract_row\(\s*"Control Catalog",/);
  assert.doesNotMatch(
    dxLaunchStylePanel,
    /Style Cockpit|Open Generator Workspace|Open Generator"|Open Style Generator|Generator Host|Generator Contract|Web Preview ready|Web Preview host present|Web Preview host missing|Readiness Contracts|Readiness Fixtures|controls ready|host connected|host unavailable|cataloged/,
  );
  assert.match(dxLaunchCheckPanel, /"Readiness score"/);
  assert.doesNotMatch(dxLaunchCheckPanel, /"Rail score"/);
  assert.match(dxLaunchWorkspace, /"Quality Gate"/);
  assert.match(dxLaunchWorkspace, /"Worktrees"/);
  assert.match(dxLaunchWorkspace, /"Fresh Receipts"/);
  assert.doesNotMatch(dxLaunchWorkspace, /No active subagents|Show \{\} more|is working/);
  assert.doesNotMatch(dxLaunchWorkspace, /source bridge wired|source bridge missing|No automation receipts|Fresh proof|worktree\(s\)|task\(s\)/);
  assert.match(dxLaunchWorkspace, /"Preview Bridge"/);
  assert.match(dxLaunchWorkspace, /"Attachable"/);
  assert.doesNotMatch(dxLaunchWorkspace, /section_title\("Token And Tool Slots"/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn token_meter_slots\(/);
  assert.doesNotMatch(dxLaunchWorkspace, /fn background_task_state\(/);

  const badgeSlot = projectPanel.slice(
    projectPanel.indexOf(".end_slot::<AnyElement>("),
    projectPanel.indexOf(".child(if let Some(icon)", projectPanel.indexOf(".end_slot::<AnyElement>(")),
  );
  assert.match(badgeSlot, /\.ml_auto\(\)/);
  assert.match(badgeSlot, /\.pr_1\(\)/);
  assert.match(badgeSlot, /\.justify_end\(\)/);
});

test("agent launch rails use professional operator-facing copy", () => {
  const sidebarActions = functionBody(agentPanel, "render_dx_launch_sidebar_actions");
  const sourceActions = functionBody(agentPanel, "render_dx_launch_source_actions");
  const guidedCards = functionBody(agentPanel, "render_dx_launch_guided_cards");
  const progressSummary = functionBody(dxLaunchWorkspace, "progress_summary");
  const environmentSummary = functionBody(dxLaunchWorkspace, "environment_summary");
  const subagentSummary = functionBody(dxLaunchWorkspace, "subagent_summary");
  const sourceSummary = functionBody(dxLaunchWorkspace, "source_summary");
  const styleState = functionBody(dxLaunchStylePanel, "dx_style_panel_state");
  const webPreviewState = functionBody(dxStylePanelCards, "web_preview_state");
  const sourceSetStatus = functionBody(dxSourceSetFormatting, "source_set_status");

  assert.match(sidebarActions, /"Review Receipts"/);
  assert.match(sourceActions, /"Review Source"/);
  assert.match(sourceActions, /"Review Deploy Readiness"/);
  assert.match(sourceActions, /"No source actions yet"/);
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

  assert.match(progressSummary, /"Preview Bridge"/);
  assert.match(progressSummary, /"Quality Gate"/);
  assert.match(environmentSummary, /"Fresh Receipts"/);
  assert.match(subagentSummary, /"Active Tasks"/);
  assert.match(subagentSummary, /"No subagent activity"/);
  assert.match(sourceSummary, /"Attachable"/);
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

  assert.match(styleState, /"Preview bridge ready"/);
  assert.match(styleState, /"Host source present"/);
  assert.match(styleState, /"Host source missing"/);
  assert.match(webPreviewState, /"preview bridge ready"/);
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

test("core left panels expose split and close controls in native headers", () => {
  const projectHeader = functionBody(projectPanel, "render_panel_header");
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
    dock.includes('format!("{id_prefix}-split-side-panel")'),
    "shared dock helper must create a stable split-control id",
  );
  assert.ok(
    dock.includes('format!("{id_prefix}-close-side-panel")'),
    "shared dock helper must create a stable close-control id",
  );
  assert.match(dock, /IconName::SplitAlt/);
  assert.match(dock, /IconName::Close/);
  assert.match(dock, /let can_split = workspace[\s\S]*?\.upgrade\(\)[\s\S]*?\.is_some_and/);
  assert.match(dock, /Open another panel to split/);
  assert.match(dock, /let panel_is_registered = workspace[\s\S]*?\.upgrade\(\)[\s\S]*?\.is_some_and/);
  assert.doesNotMatch(dock, /\.disabled\(!can_split\)|\.disabled\(!panel_is_registered\)/);
  assert.match(dock, /Panel is not available/);
  assert.match(dock, /workspace\.split_side_panel_by_id\(panel_id, window, cx\)/);
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
      `${name} must target its own panel entity for split/close controls`,
    );
    assert.doesNotMatch(
      source,
      /workspace::SplitActiveSidePanel|workspace::CloseActiveSidePanel/,
      `${name} header controls must not depend on active side-panel focus`,
    );
  }

  assert.match(projectHeader, /side_panel_header_controls\(\s*"project-panel",/);
  assert.match(
    emptyProjectWrapper,
    /\.child\(self\.render_panel_header\(cx\)\)[\s\S]*ProjectEmptyState::new/,
  );
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

  const projectHeader = functionBody(projectPanel, "render_panel_header");
  const gitTabBar = functionBody(gitPanel, "render_tab_bar");
  const outlineFooter = functionBody(outlinePanel, "render_filter_footer");
  const collabHeader = functionBody(collabPanel, "render_panel_header");
  const collabSignedIn = functionBody(collabPanel, "render_signed_in");

  assert.match(projectHeader, /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(gitTabBar, /\.h_full\(\)[\s\S]*?\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(outlineFooter, /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(outlineFooter, /\.items_center\(\)[\s\S]*?\.gap_0p5\(\)[\s\S]*?\.flex_none\(\)/);
  assert.match(collabHeader, /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/);
  assert.match(
    collabSignedIn,
    /div\(\)[\s\S]*?\.flex_1\(\)[\s\S]*?\.min_w_0\(\)[\s\S]*?render_filter_input/,
  );

  for (const [source, actionId, label] of [
    [iconPicker, "icon-picker-split-side-panel", "Icons"],
    [fontPanel, "font-panel-split-side-panel", "Fonts"],
    [mediaPanel, "media-panel-split-side-panel", "Media"],
    [uiPanel, "shadcn-ui-split-side-panel", "UI"],
    [stylePanel, "dx-style-panel-split-side-panel", "Style"],
  ] as const) {
    assert.match(sourceWindow(source, actionId, 3600, 300), /\.flex_none\(\)/);
    assert.match(sourceWindow(source, `Label::new("${label}")`), /\.truncate\(\)/);
    assert.match(
      sourceWindow(source, `Label::new("${label}")`),
      /\.flex_1\(\)[\s\S]*?\.min_w_0\(\)/,
    );
  }
});

test("recent tool panels use professional visible copy", () => {
  assert.match(titleBar, /Tooltip::text\("More Tools"\)/);
  assert.doesNotMatch(titleBar, /More Hidden Features/);

  assert.match(stylePanel, /Label::new\("Style"\)/);
  assert.match(stylePanel, /section_label\("Contracts"\)/);
  assert.doesNotMatch(stylePanel, /Style Generators|Readiness Contracts/);

  assert.match(dxStylePanelCards, /metric\("Web Preview", web_preview_state\(snapshot\)\)/);
  assert.match(dxStylePanelCards, /metric\(\s*"Generators",/);
  assert.match(dxStylePanelCards, /\{\} declared/);
  assert.match(dxStylePanelCards, /"Open Style Controls"/);
  assert.match(dxStylePanelCards, /"preview bridge ready"/);
  assert.match(dxStylePanelCards, /"host source present"/);
  assert.match(dxStylePanelCards, /"host source missing"/);
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
  assert.match(mediaPanel, /Button::new\("media-panel-remove-missing-recent", "Remove"\)/);
  assert.match(mediaPanel, /Button::new\("media-panel-remove-missing-pinned", "Remove"\)/);
  assert.match(mediaPanel, /"media-panel-remove-missing-history"/);
  assert.match(mediaPanel, /\{available\} available/);
  assert.match(mediaPanel, /\{available\} available, \{missing\} missing/);
  assert.match(mediaPanel, /No missing \{section\} entries/);
  assert.match(mediaPanel, /open remote sources/);
  assert.match(mediaPanel, /Clear recent media entries/);
  assert.match(mediaPanel, /Remove this entry from history/);
  assert.doesNotMatch(mediaPanel, /Previewing |"Clean"|use Clean|use Remove|CLEAN_STALE|remove-stale|\bstale\b|No-key|No no-key|no-key|recent media actions|\{ready\} ready|available \//);

  assert.match(uiPanel, /"Preview in Web Preview"/);
  assert.match(uiPanel, /ui_history_availability_label/);
  assert.match(uiPanel, /"Opening preview for "/);
  assert.match(uiPanel, /Button::new\("shadcn-ui-remove-missing-recent", "Remove"\)/);
  assert.match(uiPanel, /Button::new\("shadcn-ui-remove-missing-pinned", "Remove"\)/);
  assert.match(uiPanel, /"shadcn-ui-remove-missing-history"/);
  assert.match(uiPanel, /\{available\} available/);
  assert.match(uiPanel, /\{available\} available, \{missing\} missing/);
  assert.match(uiPanel, /No missing \{section\} entries/);
  assert.match(uiPanel, /Changes queued/);
  assert.match(uiPanel, /UI registry preview/);
  assert.match(uiPanel, /Clear recent UI entries/);
  assert.match(uiPanel, /Remove this entry from history/);
  assert.doesNotMatch(uiPanel, /Previewing |"Clean"|use Clean|use Remove|CLEAN_STALE|remove-stale|\bstale\b|recent UI action|pinned UI action|Saved changes|The UI registry is ready|Preview in WebPreview|\{ready\} ready|available \//);
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
