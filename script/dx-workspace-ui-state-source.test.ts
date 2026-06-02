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
const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const dxLaunchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
const sidebar = read("crates/sidebar/src/sidebar.rs");
const threadItem = read("crates/ui/src/components/ai/thread_item.rs");
const projectPanel = read("crates/project_panel/src/project_panel.rs");
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
  assert.match(dxLaunchWorkspace, /muted_card\("No active subagents", cx\)/);
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
  assert.doesNotMatch(sourcesRail, /\.border_r_1\(\)/);
  assert.doesNotMatch(progressRail, /\.right_0\(\)/);
  assert.doesNotMatch(progressRail, /\.border_l_1\(\)/);
  assert.doesNotMatch(dxLaunchWorkspace, /sources", status\.source_sets\.total_sources/);
  assert.doesNotMatch(dxLaunchWorkspace, /tasks", status\.background_task_count/);
  assert.doesNotMatch(dxLaunchWorkspace, /Current Agent panel conversation state/);
  assert.doesNotMatch(dxLaunchWorkspace, /Background Agent work visible in the right rail/);
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
