import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const functionBody = (source: string, name: string) => {
  const start = source.search(new RegExp(`fn\\s+${name}(?:<[^>]+>)?\\s*\\(`));
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
  const beforeIndex =
    typeof before === "string" ? body.indexOf(before) : body.match(before)?.index ?? -1;
  const afterIndex =
    typeof after === "string" ? body.indexOf(after) : body.match(after)?.index ?? -1;
  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
};

test("project panel visible tree materialization has named caps before collection", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");

  assert.match(source, /const MAX_PROJECT_PANEL_VISIBLE_WORKTREES: usize = 256;/);
  assert.match(source, /const MAX_PROJECT_PANEL_VISIBLE_ENTRIES: usize = 200_000;/);
  assert.match(source, /const MAX_PROJECT_PANEL_VISIBLE_ENTRIES_PER_WORKTREE: usize = 50_000;/);
  assert.match(source, /fn project_panel_cap_hit\(boundary: &'static str, cap: usize\)/);
  assert.match(
    updateVisibleEntries,
    /let visible_worktrees: Vec<_> = visible_worktree_iter[\s\S]*\.take\(MAX_PROJECT_PANEL_VISIBLE_WORKTREES\)[\s\S]*\.map\(\|worktree\| worktree\.read\(cx\)\.snapshot\(\)\)[\s\S]*\.collect\(\);/,
    "visible worktrees must be capped before snapshot vector collection",
  );
  assertBefore({
    body: updateVisibleEntries,
    before:
      /visible_worktree_entries\.len\(\)\s*>=\s*MAX_PROJECT_PANEL_VISIBLE_ENTRIES_PER_WORKTREE/,
    after: "visible_worktree_entries.push(entry.to_owned())",
    message: "per-worktree entries must be capped before visible row pushes",
  });
  assertBefore({
    body: updateVisibleEntries,
    before: "visible_entries_total >= MAX_PROJECT_PANEL_VISIBLE_ENTRIES",
    after: "new_state.visible_entries.push(VisibleEntriesForWorktree",
    message: "total visible entries must be capped before state materialization",
  });
});

test("project panel DX Explorer header is source-backed and action-wired", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const projectPanelUiSources = [
    source,
    read("crates/project_panel/src/media_preview.rs"),
    read("crates/project_panel/src/storage_roots_view.rs"),
  ].join("\n");
  const dxExplorerSummary = functionBody(source, "dx_explorer_summary");
  const renderDxExplorerMetric = functionBody(source, "render_dx_explorer_metric");
  const renderDxExplorerHeader = functionBody(source, "render_dx_explorer_header");
  const renderSidePanelHeaderControls = functionBody(
    source,
    "render_side_panel_header_controls",
  );
  const updateVisibleEntries = functionBody(source, "update_visible_entries");

  assert.match(source, /struct DxExplorerSummary/);
  assert.match(source, /struct DxExplorerVisibleSummary/);
  assert.match(source, /enum DxExplorerSourceKind/);
  assert.match(source, /fn from_project\(project: &Project, cx: &App\) -> Self/);
  assert.match(source, /fn label\(self\) -> &'static str/);
  assert.match(source, /LocalWorkspace/);
  assert.match(source, /WslWorkspace/);
  assert.match(source, /RemoteWorkspace/);
  assert.match(source, /ReadOnlyWorkspace/);
  assert.match(source, /dx_explorer_visible_summary:\s*DxExplorerVisibleSummary/);
  assert.match(source, /fn record_entry\(&mut self, entry: &Entry\)/);
  assert.match(source, /fn record_entry_kind\(&mut self, kind: EntryKind, size: u64\)/);
  assert.match(source, /fn record_skipped_entry\(&mut self\)/);
  assert.match(dxExplorerSummary, /source_kind,/);
  assert.match(dxExplorerSummary, /worktree_count: self\.state\.visible_entries\.len\(\)/);
  assert.match(dxExplorerSummary, /let visible_summary = self\.state\.dx_explorer_visible_summary;/);
  assert.match(
    dxExplorerSummary,
    /visible_entry_count: visible_summary\.entry_count/,
    "DX Explorer visible entry counts must read the cached visible summary",
  );
  assert.match(
    dxExplorerSummary,
    /skipped_entry_count: visible_summary\.skipped_entry_count/,
    "DX Explorer skipped entry counts must read the cached visible summary",
  );
  assert.match(
    dxExplorerSummary,
    /visible_file_count: visible_summary\.file_count/,
    "DX Explorer file counts must read the cached visible summary",
  );
  assert.match(
    dxExplorerSummary,
    /visible_folder_count: visible_summary\.folder_count/,
    "DX Explorer folder counts must read the cached visible summary",
  );
  assert.match(
    dxExplorerSummary,
    /visible_file_bytes: visible_summary\.file_bytes/,
    "DX Explorer storage counts must read the cached visible summary",
  );
  assert.doesNotMatch(dxExplorerSummary, /selected_entry_count,/);
  assert.match(
    dxExplorerSummary,
    /expanded_dir_count:[\s\S]*self[\s\S]*\.state[\s\S]*expanded_dir_ids[\s\S]*\.sum\(\)/,
  );
  assert.match(
    dxExplorerSummary,
    /cached_media_folder_count = folder_media_previews[\s\S]*\.values\(\)[\s\S]*\.filter\(\|preview\| preview\.is_some\(\)\)[\s\S]*\.count\(\)/,
    "DX Explorer media folder counts must skip negative preview cache entries",
  );
  assert.match(
    dxExplorerSummary,
    /cached_media_item_count = folder_media_previews[\s\S]*\.values\(\)[\s\S]*\.filter_map\(\|preview\| preview\.as_ref\(\)\)[\s\S]*\.map\(\|preview\| preview\.total_count\)[\s\S]*\.sum\(\)/,
    "DX Explorer media item counts must come from cached folder media previews",
  );
  assert.doesNotMatch(dxExplorerSummary, /\bread_dir\(|\bFile::open\(|read_to_string|cx\.spawn/);
  assert.doesNotMatch(
    dxExplorerSummary,
    /for visible_worktree|for entry in/,
    "DX Explorer header summary must not walk all visible entries during render",
  );
  assertBefore({
    body: updateVisibleEntries,
    before: /\.dx_explorer_visible_summary[\s\S]*\.record_entry\(entry\.entry\)/,
    after: "visible_worktree_entries.push(entry.to_owned())",
    message: "visible-tree summary must be recorded as rows are materialized, before the row push",
  });
  assertBefore({
    body: updateVisibleEntries,
    before: /\.dx_explorer_visible_summary[\s\S]*\.record_entry_kind\(new_entry_kind, 0\)/,
    after: "Self::create_new_git_entry",
    message: "temporary new-entry rows must update visible summary without counting parent bytes",
  });

  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-header"\)/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-title-row"\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /\.id\("dx-explorer-summary-row"\)/);
  assert.match(renderDxExplorerHeader, /ProjectPanelSettings::get_global\(cx\)/);
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::Project\)/);
  assert.match(
    renderDxExplorerHeader,
    /Icon::new\(dx_icon\(DxUiIcon::Project\)\)[\s\S]*\.color\(Color::Muted\)/,
    "Project header icon should stay quiet and native to the panel chrome",
  );
  assert.match(renderDxExplorerHeader, /let source_label = summary\.source_kind\.label\(\);/);
  assert.doesNotMatch(renderDxExplorerHeader, /source_label = if is_read_only/);
  assert.match(source, /Self::LocalWorkspace => "Local"/);
  assert.match(source, /Self::WslWorkspace => "WSL"/);
  assert.match(source, /Self::RemoteWorkspace => "Remote"/);
  assert.match(source, /Self::ReadOnlyWorkspace => "Read-only"/);
  assert.doesNotMatch(source, /(?:Local|WSL|Remote|Read-only) source/);
  assert.doesNotMatch(
    projectPanelUiSources,
    /\b(?:IconName::Settings|IconName::Sliders|DxUiIcon::Settings|DxUiIcon::Style)\b/,
    "Project Panel should not use settings/sliders glyphs for explorer chrome",
  );
  assert.match(renderDxExplorerHeader, /summary\.skipped_entry_count/);
  assert.match(renderDxExplorerHeader, /summary\.visible_file_count/);
  assert.match(renderDxExplorerHeader, /summary\.visible_folder_count/);
  assert.match(
    renderDxExplorerHeader,
    /let header_summary_meta = \[/,
    "full DX Explorer summary should move into a tooltip instead of crowding the visible header",
  );
  assert.match(
    renderDxExplorerHeader,
    /format!\(\s*"Size: \{\}"[\s\S]*storage::format_file_size\(\s*summary\.visible_file_bytes\s*\)/,
  );
  assert.match(renderDxExplorerHeader, /summary\.cached_media_item_count/);
  assert.match(renderDxExplorerHeader, /Tooltip::with_meta\([\s\S]*"Project summary"[\s\S]*header_summary_meta\.clone\(\)[\s\S]*cx[\s\S]*\)/);
  assert.match(
    renderDxExplorerHeader,
    /\.id\("dx-explorer-title-row"\)[\s\S]*Icon::new\(dx_icon\(DxUiIcon::Project\)\)[\s\S]*Label::new\("Project"\)[\s\S]*\.child\(header_controls\)/,
    "DX Explorer top chrome should keep title and actions in a readable Git-panel-style title row",
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /dx-explorer-summary-row|header_metrics/,
    "DX Explorer metrics should stay in the title tooltip instead of creating a cramped visible strip",
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /Label::new\("DX Explorer"\)/,
    "DX Explorer should use the product-facing Project title in the top row",
  );
  assert.match(
    renderDxExplorerMetric,
    /Label::new\(label\)[\s\S]*\.size\(LabelSize::Small\)[\s\S]*\.color\(Color::Muted\)[\s\S]*\.truncate\(\)/,
    "DX Explorer metric text should stay small, muted, and truncating",
  );
  assert.doesNotMatch(
    renderDxExplorerMetric,
    /Chip::new|ButtonLike::new|\.border_|\.bg\(/,
    "DX Explorer metric text should not grow local chrome",
  );
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-source-controls"\)/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-edit-controls"\)/);
  assert.match(renderDxExplorerHeader, /SharedString::from\(format!\("dx-explorer-project-options-menu-\{panel_id:\?\}"\)\)/);
  assert.match(renderDxExplorerHeader, /PopoverMenu::new\(project_options_menu_id\)/);
  assert.match(renderDxExplorerHeader, /action_checked_with_disabled\(/);
  assert.doesNotMatch(renderDxExplorerHeader, /\.id\("dx-explorer-filter-controls"\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /\.id\("dx-explorer-view-controls"\)/);
  assert.equal(
    renderDxExplorerHeader.match(/Divider::vertical\(\)\.color\(ui::DividerColor::BorderFaded\)/g)
      ?.length,
    1,
    "DX Explorer header should keep only the primary action divider and move secondary controls into overflow",
  );
  assert.match(
    renderSidePanelHeaderControls,
    /side_panel_header_controls\(\s*id_prefix,[\s\S]*self\.workspace\.clone\(\),[\s\S]*cx\.entity\(\)\.entity_id\(\),[\s\S]*cx,/,
    "Project Panel side-panel chrome must stay routed through Zed's shared Dock controls",
  );
  assert.match(renderSidePanelHeaderControls, /div\(\)\.pr_0p5\(\)\.child\(side_panel_header_controls/);
  assert.match(
    renderDxExplorerHeader,
    /self\.render_side_panel_header_controls\("dx-explorer", cx\)/,
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /(?<!render_)side_panel_header_controls\(\s*"dx-explorer"/,
  );
  assert.equal(
    source.match(/\bself\.render_side_panel_header_controls\(/g)?.length,
    2,
    "Project Panel should reuse the side-panel chrome helper only for the Project header and selection controls",
  );
  assert.equal(
    source.match(/\bside_panel_header_controls\(/g)?.length,
    1,
    "Project Panel should call the shared Dock controls only from the helper",
  );
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::OpenProject\)/);
  assert.match(dxIcons, /DxUiIcon::OpenProject => IconName::FolderOpenAdd/);
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::OpenFile\)/);
  assert.match(dxIcons, /DxUiIcon::OpenFile => IconName::MagnifyingGlass/);
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /"dx-explorer-open-file",\s*dx_icon\(DxUiIcon::Search\)/,
    "Open File must use its own DX semantic icon instead of the broader Search icon",
  );
  assert.match(renderDxExplorerHeader, /let header_focus_handle = self\.focus_handle\(cx\);/);
  assert.match(
    renderDxExplorerHeader,
    /"dx-explorer-open-project"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&open_project_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Open project",\s*&workspace::Open::default\(\),\s*&open_project_focus_handle,[\s\S]*cx/,
    "Open Project should use the shared focused IconButton plus action-aware tooltip",
  );
  assert.match(
    renderDxExplorerHeader,
    /"dx-explorer-open-file"[\s\S]*\.when\(has_worktree,[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&open_file_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Open file",\s*&ToggleFileFinder::default\(\),\s*&open_file_tooltip_focus_handle,[\s\S]*cx/,
    "Open File should only enter tab order when enabled and should expose its action keybinding",
  );
  assert.match(
    renderDxExplorerHeader,
    /IconButton::new\(project_options_button_id,[\s\S]*IconName::Ellipsis[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&project_options_focus_handle\)[\s\S]*Tooltip::text\("Project options"\)/,
    "Secondary Project controls should move behind a focus-tracked overflow menu",
  );
  assert.match(
    renderDxExplorerHeader,
    /action_checked_with_disabled\([\s\S]*if show_ignored_entries[\s\S]*"Ignored files visible"[\s\S]*"Ignored files hidden"[\s\S]*ToggleHideGitIgnore\.boxed_clone\(\),[\s\S]*show_ignored_entries,[\s\S]*!has_worktree/,
    "Ignored-files toggle should stay action-backed inside the Project options menu",
  );
  assert.match(
    renderDxExplorerHeader,
    /action_checked_with_disabled\([\s\S]*if show_hidden_entries[\s\S]*"Hidden files visible"[\s\S]*"Hidden files hidden"[\s\S]*ToggleHideHidden\.boxed_clone\(\),[\s\S]*show_hidden_entries,[\s\S]*!has_worktree/,
    "Hidden-files toggle should stay action-backed inside the Project options menu",
  );
  assert.match(
    renderDxExplorerHeader,
    /action_disabled_when\([\s\S]*!has_worktree,[\s\S]*"Project symbols",[\s\S]*ToggleProjectSymbols\.boxed_clone\(\)/,
    "Project Symbols should stay visibly disabled when no worktree is open",
  );
  assert.match(
    renderDxExplorerHeader,
    /action_disabled_when\([\s\S]*!has_worktree,[\s\S]*"Collapse folders",[\s\S]*CollapseAllEntries\.boxed_clone\(\)/,
    "Collapse All should stay visibly disabled when no worktree is open",
  );
  assert.match(
    renderDxExplorerHeader,
    /"dx-explorer-new-file"[\s\S]*\.when\(!is_read_only && has_worktree,[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&new_file_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"New file",\s*&NewFile,[\s\S]*&new_file_tooltip_focus_handle,[\s\S]*cx/,
    "New File should be focus-tracked only when writable and expose its action keybinding",
  );
  assert.match(
    renderDxExplorerHeader,
    /"dx-explorer-new-folder"[\s\S]*\.when\(!is_read_only && has_worktree,[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&new_folder_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"New folder",\s*&NewDirectory,[\s\S]*&new_folder_tooltip_focus_handle,[\s\S]*cx/,
    "New Folder should be focus-tracked only when writable and expose its action keybinding",
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /"dx-explorer-open-project"[\s\S]*Tooltip::text\("Open project"\)/,
    "Open Project should not keep a plain tooltip when the action binding is exact",
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /"dx-explorer-open-file"[\s\S]*Tooltip::text\("Open file"\)/,
    "Open File should not keep a plain tooltip when the action binding is exact",
  );
  assert.doesNotMatch(
    renderDxExplorerHeader,
    /Tooltip::text\((?:if show_ignored_entries|if show_hidden_entries|"New file"|"New folder")/,
    "Visible DX Explorer header action buttons should use action-aware tooltips instead of plain text",
  );
  assert.doesNotMatch(renderDxExplorerHeader, /IconName::ListX/);
  assert.doesNotMatch(renderDxExplorerHeader, /IconName::ListFilter/);
  assert.match(renderDxExplorerHeader, /workspace::Open::default\(\)\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleFileFinder::default\(\)\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleHideGitIgnore\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleHideHidden\.boxed_clone\(\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /selected_style\(ButtonStyle::Tinted\(TintColor::Accent\)\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /toggle_state\(show_ignored_entries\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /toggle_state\(show_hidden_entries\)/);
  assert.match(renderDxExplorerHeader, /ToggleProjectSymbols\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /this\.new_file\(&NewFile, window, cx\)/);
  assert.match(renderDxExplorerHeader, /this\.new_directory\(&NewDirectory, window, cx\)/);
  assert.match(
    renderDxExplorerHeader,
    /action_disabled_when\([\s\S]*"Collapse folders"[\s\S]*CollapseAllEntries\.boxed_clone\(\)/,
  );
  assert.match(renderDxExplorerHeader, /\.disabled\(is_read_only \|\| !has_worktree\)/);
  assert.match(renderDxExplorerHeader, /\.min_w_0\(\)[\s\S]*\.overflow_hidden\(\)/);
  assert.doesNotMatch(renderDxExplorerHeader, /\bread_dir\(|\bFile::open\(|read_to_string|cx\.spawn/);

  assert.match(
    source,
    /let dx_explorer_source_kind = DxExplorerSourceKind::from_project\(&project, cx\);/,
  );
  assert.match(
    source,
    /let dx_explorer_summary =\s*self\.dx_explorer_summary\(dx_explorer_source_kind\);/,
  );
  assert.match(
    source,
    /self\.render_dx_explorer_header\([\s\S]*dx_explorer_summary,[\s\S]*has_worktree,[\s\S]*is_read_only,[\s\S]*cx/,
    "DX Explorer header must receive the current source mode instead of guessing from path text",
  );
  const headerMounts = source.match(/\.child\(self\.render_dx_explorer_header\(/g) ?? [];
  assert.equal(headerMounts.length, 2, "DX Explorer header should mount in tree and empty states");
  assertBefore({
    body: source,
    before: ".child(self.render_dx_explorer_header(",
    after: /\.map\(\|this\| \{\s*if let Some\(toolbar\) = selected_entries_toolbar/,
    message: "DX Explorer header should render before the selected-entry toolbar and tree",
  });
});

test("project panel expansion and selection fanout is bounded", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const expandAllForEntry = functionBody(source, "expand_all_for_entry");
  const collapseAllForEntry = functionBody(source, "collapse_all_for_entry");
  const renderEntry = functionBody(source, "render_entry");
  const pushExpandedDir = functionBody(source, "push_project_panel_expanded_dir");

  assert.match(source, /const MAX_PROJECT_PANEL_EXPANDED_DIRS_PER_WORKTREE: usize = 50_000;/);
  assert.match(source, /const MAX_PROJECT_PANEL_SELECTION_RANGE_ENTRIES: usize = 20_000;/);
  assert.match(source, /fn push_project_panel_expanded_dir\(/);
  assert.match(
    expandAllForEntry,
    /push_project_panel_expanded_dir\(expanded_dir_ids, entry\.id\)/,
    "entry expansion must use the capped insert helper",
  );
  assertBefore({
    body: pushExpandedDir,
    before: "expanded_dir_ids.len() >= MAX_PROJECT_PANEL_EXPANDED_DIRS_PER_WORKTREE",
    after: "expanded_dir_ids.insert",
    message: "expanded directory ids must check the cap before sorted insertion",
  });
  assertBefore({
    body: collapseAllForEntry,
    before:
      /dirs_to_collapse\.len\(\)\s*>=\s*MAX_PROJECT_PANEL_EXPANDED_DIRS_PER_WORKTREE/,
    after: "dirs_to_collapse.push(child.id)",
    message: "recursive collapse worklists must be capped before push",
  });
  assertBefore({
    body: renderEntry,
    before: "MAX_PROJECT_PANEL_SELECTION_RANGE_ENTRIES",
    after: "for_each_visible_entry",
    message: "shift range selection must be capped before visible-entry materialization",
  });
});

test("project panel previous selection uses checked visible-entry lookups", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const selectPrevious = functionBody(source, "select_previous");

  assert.doesNotMatch(
    selectPrevious,
    /visible_entries\s*\[\s*worktree_ix\s*\]/,
    "select_previous must not index visible_entries with a stale worktree_ix",
  );
  assert.doesNotMatch(
    selectPrevious,
    /entries\s*\[\s*entry_ix\s*\]/,
    "select_previous must not index entries with a stale entry_ix",
  );
  assertBefore({
    body: selectPrevious,
    before: /\.visible_entries\s*\.\s*get\(\s*worktree_ix\s*\)/,
    after: "let selection = SelectedEntry",
    message: "select_previous must check the target worktree before creating a selection",
  });
  assertBefore({
    body: selectPrevious,
    before: /entries\.get\(\s*entry_ix\s*\)/,
    after: "let selection = SelectedEntry",
    message: "select_previous must check the target entry before creating a selection",
  });
});

test("project panel active indent guide uses checked visible-entry lookups", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const findActiveIndentGuide = functionBody(source, "find_active_indent_guide");

  assert.doesNotMatch(
    findActiveIndentGuide,
    /visible_entries\s*\[\s*worktree_ix\s*\]/,
    "active indent guide lookup must not index visible_entries with a stale worktree_ix",
  );
  assertBefore({
    body: findActiveIndentGuide,
    before: /\.visible_entries\s*\.\s*get\(\s*worktree_ix\s*\)/,
    after: "let child_paths =",
    message: "active indent guide lookup must check the target worktree before reading entries",
  });
});

test("project panel visible-entry range materialization skips stale ranges", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const iterVisibleEntries = functionBody(source, "iter_visible_entries");
  const forEachVisibleEntry = functionBody(source, "for_each_visible_entry");

  assert.doesNotMatch(
    iterVisibleEntries,
    /visible\.entries\s*\[\s*entry_range\s*\]/,
    "iter_visible_entries must not directly slice a potentially stale visible entry range",
  );
  assertBefore({
    body: iterVisibleEntries,
    before: /visible\.entries\s*\.\s*get\(\s*entry_range\s*\)/,
    after: "for (i, entry)",
    message: "iter_visible_entries must check the entry range before iterating it",
  });
  assert.doesNotMatch(
    forEachVisibleEntry,
    /visible\.entries\s*\[\s*entry_range\s*\]/,
    "for_each_visible_entry must not directly slice a potentially stale visible entry range",
  );
  assertBefore({
    body: forEachVisibleEntry,
    before: /visible\.entries\s*\.\s*get\(\s*entry_range\s*\)/,
    after: "let status =",
    message: "for_each_visible_entry must check the entry range before materializing details",
  });
});

test("project panel edit-state display handles stale ancestor relations", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const forEachVisibleEntry = functionBody(source, "for_each_visible_entry");

  assert.doesNotMatch(
    forEachVisibleEntry,
    /\.expect\(\s*"Edited sub-entry should be an ancestor of selected leaf entry"\s*\)/,
    "edit-state display must not panic when a stale leaf no longer contains the edited ancestor",
  );
  assertBefore({
    body: forEachVisibleEntry,
    before:
      /if let Some\(position\)\s*=\s*ancestors\s*\.\s*ancestors\s*\.\s*iter\(\)\s*\.\s*position\(\|entry_id\|\s*\*entry_id\s*==\s*edit_state\.entry_id\)/,
    after: "let all_components = ancestors.ancestors.len();",
    message: "edit-state display must check the edited ancestor position before deriving path components",
  });
  assertBefore({
    body: forEachVisibleEntry,
    before:
      /if let Some\(position\)\s*=\s*ancestors\s*\.\s*ancestors\s*\.\s*iter\(\)\s*\.\s*position\(\|entry_id\|\s*\*entry_id\s*==\s*edit_state\.entry_id\)/,
    after: /details\s*\.\s*filename\s*\.\s*push_str\(\s*processing_filename\.as_unix_str\(\)\s*\)/,
    message: "edit-state display must fall back to the processing filename when ancestor lookup is stale",
  });
});

test("project panel drag, drop, and download materialization is bounded", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const dropExternalFiles = functionBody(source, "drop_external_files");
  const dragOnto = functionBody(source, "drag_onto");
  const createPastePath = functionBody(source, "create_paste_path");
  const paste = functionBody(source, "paste");
  const downloadFromRemote = functionBody(source, "download_from_remote");
  const moveEntry = functionBody(source, "move_entry");
  const moveWorktreeRoot = functionBody(source, "move_worktree_root");
  const renderEntry = functionBody(source, "render_entry");
  const renderFolderElements = functionBody(source, "render_folder_elements");
  const renderEntryPathSeparator = functionBody(source, "render_entry_path_separator");

  assert.match(source, /const MAX_PROJECT_PANEL_EXTERNAL_DROP_PATHS: usize = 4_096;/);
  assert.match(source, /const MAX_PROJECT_PANEL_DRAG_SELECTION_ENTRIES: usize = 4_096;/);
  assert.match(source, /const MAX_PROJECT_PANEL_DOWNLOAD_FILES: usize = 10_000;/);
  assert.match(
    moveEntry,
    /entry_is_worktree_root\(entry_to_move, cx\)[\s\S]*self\.move_worktree_root\(entry_to_move, destination, cx\)[\s\S]*None/,
    "worktree-root moves must stay routed through the dedicated worktree move path",
  );
  assert.match(
    moveWorktreeRoot,
    /worktree_for_entry\(entry_to_move, cx\)[\s\S]*worktree_for_entry\(destination, cx\)[\s\S]*move_worktree\(worktree_id, destination_id, cx\)/,
    "worktree-root moving must use Project worktree ordering rather than entry rename semantics",
  );
  assertBefore({
    body: dropExternalFiles,
    before: ".take(MAX_PROJECT_PANEL_EXTERNAL_DROP_PATHS)",
    after: "paths_to_replace.push",
    message: "external drops must be bounded before replacement and copy vectors",
  });
  assert.doesNotMatch(createPastePath, /RelPath::unix\([^)]*\)\.unwrap\(\)/);
  assert.doesNotMatch(dropExternalFiles, /RelPath::unix\(name\)\.unwrap\(\)/);
  assert.match(
    dropExternalFiles,
    /let Ok\(name\) = RelPath::unix\(name\) else \{[\s\S]*continue;[\s\S]*\};[\s\S]*target_directory\.join\(name\)/,
    "external drop filename conversion must skip invalid rel-path names instead of panicking",
  );
  assertBefore({
    body: dragOnto,
    before: "cap_project_panel_entry_set(",
    after: "copy_tasks.push(task)",
    message: "drag selections must be bounded before copy task fanout",
  });
  assert.match(
    dragOnto,
    /if folded_selection_info\.is_empty\(\) \{[\s\S]*let mut last_moved_entry = None;[\s\S]*last_moved_entry = Some\(SelectedEntry[\s\S]*entry_id: new_entry\.id[\s\S]*this\.selection = Some\(selection\);[\s\S]*this\.expand_entry\(selection\.worktree_id, selection\.entry_id, cx\);[\s\S]*this\.update_visible_entries\(\s*Some\(\(selection\.worktree_id, selection\.entry_id\)\),\s*false,\s*true,\s*window,\s*cx/,
    "successful plain drag moves must keep the moved entry visible and selected",
  );
  assertBefore({
    body: paste,
    before: ".take(MAX_PROJECT_PANEL_DRAG_SELECTION_ENTRIES)",
    after: "paste_tasks.push(task)",
    message: "paste selections must be bounded before task fanout",
  });
  assertBefore({
    body: downloadFromRemote,
    before: "files_to_download.len() >= MAX_PROJECT_PANEL_DOWNLOAD_FILES",
    after: "files_to_download.push",
    message: "remote download lists must be bounded before recursive file collection",
  });
  assert.match(
    renderEntry,
    /move \|this, selections: &DraggedSelection, window, cx\|[\s\S]*this\.drag_onto\(selections, entry_id, kind\.is_file\(\), window, cx\);[\s\S]*cx\.stop_propagation\(\);/,
    "internal row drops must stop propagation after move/copy handling",
  );
  assert.match(
    renderFolderElements,
    /move \|this, selections: &DraggedSelection, window, cx\|[\s\S]*this\.drag_onto\([\s\S]*target_entry_id,[\s\S]*is_file,[\s\S]*window,[\s\S]*cx,[\s\S]*\);[\s\S]*cx\.stop_propagation\(\);/,
    "folded path component drops must stop propagation after move/copy handling",
  );
  assert.match(
    renderEntryPathSeparator,
    /move \|this, selections: &DraggedSelection, window, cx\|[\s\S]*this\.drag_onto\(selections, target_entry_id, is_file, window, cx\);[\s\S]*cx\.stop_propagation\(\);/,
    "folded path separator drops must stop propagation after move/copy handling",
  );
});

test("project panel selection toolbar exposes file-browser operation state", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const operationStatus = read("crates/project_panel/src/operation_status.rs");
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const clipboardOperationSummary = functionBody(source, "clipboard_operation_summary");
  const renderSelectedEntriesToolbar = functionBody(source, "render_selected_entries_toolbar");
  const paste = functionBody(source, "paste");
  const createMovePath = functionBody(source, "create_move_path");
  const moveWorktreeEntry = functionBody(source, "move_worktree_entry");
  const dragMoveEntries = functionBody(source, "drag_move_entries");
  const dragOnto = functionBody(source, "drag_onto");
  const clipboardEntryIsCut = functionBody(source, "is_cut");

  assert.match(source, /mod operation_status;/);
  assert.match(operationStatus, /pub\(crate\) enum ClipboardOperationMode/);
  assert.match(operationStatus, /Copy/);
  assert.match(operationStatus, /Move/);
  assert.match(operationStatus, /pub\(crate\) struct ClipboardOperationSummary/);
  assert.match(operationStatus, /pub\(crate\) fn new\(mode: ClipboardOperationMode, item_count: usize\) -> Option<Self>/);
  assert.match(operationStatus, /\(item_count > 0\)\.then_some/);
  assert.match(operationStatus, /pub\(crate\) fn status_label\(self\) -> String/);
  assert.match(operationStatus, /pub\(crate\) fn paste_tooltip\(self\) -> &'static str/);
  assert.doesNotMatch(operationStatus, /\b(?:prototype|dummy|magic|slop|v1)\b/i);

  assert.match(
    clipboardOperationSummary,
    /let clipboard = self\.clipboard\.as_ref\(\)\?/,
    "clipboard operation status must derive from the panel clipboard state",
  );
  assert.match(
    clipboardOperationSummary,
    /clipboard\.is_cut\(\)[\s\S]*ClipboardOperationMode::Move[\s\S]*ClipboardOperationMode::Copy/,
    "cut clipboard entries must be presented as move operations",
  );
  assert.match(
    clipboardOperationSummary,
    /ClipboardOperationSummary::new\(mode, clipboard\.items\(\)\.len\(\)\)/,
    "empty clipboard state must not render an operation chip",
  );
  assert.match(
    clipboardEntryIsCut,
    /matches!\(self, Self::Cut\(_\)\)/,
    "cut clipboard detection must match the tuple variant shape",
  );

  assert.match(renderSelectedEntriesToolbar, /\.id\("project-panel-clipboard-operation-status"\)/);
  assert.match(
    renderSelectedEntriesToolbar,
    /\.id\("project-panel-selection-toolbar"\)[\s\S]*\.gap_1\(\)[\s\S]*\.px_1\(\)[\s\S]*\.py_0p5\(\)/,
    "selection toolbar should keep compact Zed panel spacing",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /Chip::new\(operation\.status_label\(\)\)[\s\S]*\.icon\(icon\)[\s\S]*\.icon_color\(Color::Muted\)[\s\S]*\.label_color\(Color::Muted\)[\s\S]*\.truncate\(\)/,
  );
  assert.doesNotMatch(
    renderSelectedEntriesToolbar,
    /project-panel-clipboard-operation-status[\s\S]*(?:Label::new\(operation\.status_label\(\)|Icon::new\(icon\)|\.border_1\(\)|border_variant|element_background|\.rounded_sm\(\)|\.py_0p5\(\))/,
    "clipboard operation status must use the shared Chip component instead of local pill chrome",
  );
  assert.match(renderSelectedEntriesToolbar, /operation\.status_label\(\)/);
  assert.match(renderSelectedEntriesToolbar, /dx_icon\(DxUiIcon::Copy\)/);
  assert.match(renderSelectedEntriesToolbar, /dx_icon\(DxUiIcon::Move\)/);
  assert.match(renderSelectedEntriesToolbar, /dx_icon\(DxUiIcon::Duplicate\)/);
  assert.match(renderSelectedEntriesToolbar, /dx_icon\(DxUiIcon::PasteInto\)/);
  assert.match(
    renderSelectedEntriesToolbar,
    /IconButton::new\([\s\S]*"project-panel-paste-selection-target"/,
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /let clipboard_operation = self\.clipboard_operation_summary\(\);/,
    "selection toolbar status must derive from cached Project Panel clipboard state",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /let can_paste_to_selection = clipboard_operation_for_paste\.is_some\(\);/,
    "toolbar paste visibility must follow the internal Project Panel clipboard without reading the OS clipboard during render",
  );
  assert.doesNotMatch(
    renderSelectedEntriesToolbar,
    /has_external_paste_paths|external_paths_from_system_clipboard|read_from_clipboard/,
    "selection toolbar render must not query the OS clipboard",
  );
  assert.match(renderSelectedEntriesToolbar, /operation\.mode\.paste_tooltip\(\)/);
  assert.match(renderSelectedEntriesToolbar, /unwrap_or\("Paste Here"\)/);
  assert.match(renderSelectedEntriesToolbar, /\.when\(!is_read_only && can_paste_to_selection/);
  assert.match(renderSelectedEntriesToolbar, /this\.paste\(&Paste \{\}, window, cx\)/);
  assert.match(
    renderSelectedEntriesToolbar,
    /let toolbar_focus_handle = self\.focus_handle\(cx\);/,
    "selection toolbar should share the Project Panel focus handle across command buttons",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-copy-selection"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&copy_selection_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Copy selected",\s*&Copy \{\},\s*&copy_selection_tooltip_focus_handle,[\s\S]*cx/,
    "Copy selected should stay keyboard reachable and expose its Project Panel action keybinding",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-cut-selection"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&cut_selection_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Cut selected",\s*&Cut \{\},\s*&cut_selection_tooltip_focus_handle,[\s\S]*cx/,
    "Cut selected should be keyboard reachable and expose its Project Panel action keybinding",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-duplicate-selection"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&duplicate_selection_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Duplicate selected",\s*&Duplicate \{\},\s*&duplicate_selection_tooltip_focus_handle,[\s\S]*cx/,
    "Duplicate selected should be keyboard reachable and expose its Project Panel action keybinding",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-paste-selection-target"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&paste_selection_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*paste_tooltip,\s*&Paste \{\},\s*&paste_selection_tooltip_focus_handle,[\s\S]*cx/,
    "Paste Here should be keyboard reachable and expose its Project Panel action keybinding",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-trash-selection"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&trash_selection_focus_handle\)[\s\S]*Tooltip::for_action_in\(\s*"Trash selected",\s*&Trash \{ skip_prompt: false \},\s*&trash_selection_tooltip_focus_handle,[\s\S]*cx/,
    "Trash selected should be keyboard reachable and expose its Project Panel action keybinding",
  );
  assert.match(
    renderSelectedEntriesToolbar,
    /"project-panel-clear-selection"[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&clear_selection_focus_handle\)[\s\S]*Tooltip::text\("Clear selection"\)/,
    "Clear selection should remain keyboard reachable even though it is local toolbar state",
  );
  assert.doesNotMatch(
    renderSelectedEntriesToolbar,
    /Tooltip::text\("Cut selected"\)|Tooltip::text\("Duplicate selected"\)|Tooltip::text\(paste_tooltip\)|Tooltip::text\("Trash selected"\)/,
    "selection toolbar action buttons should use action-aware tooltips instead of plain text",
  );
  assert.doesNotMatch(
    source,
    /let has_external_paste_paths = self\.external_paths_from_system_clipboard\(cx\)\.is_some\(\);[\s\S]*render_selected_entries_toolbar\(/,
    "Project Panel render must not compute external paste availability from the OS clipboard",
  );
  assert.match(
    paste,
    /if let Some\(external_paths\) = self\.external_paths_from_system_clipboard\(cx\)[\s\S]*self\.drop_external_files\(external_paths\.paths\(\), entry_id, window, cx\);[\s\S]*return;/,
    "paste must give external OS file paths precedence before reading the internal clipboard",
  );
  assertBefore({
    body: paste,
    before: /if let Some\(external_paths\) = self\.external_paths_from_system_clipboard\(cx\)/,
    after: /let \(worktree, entry\) = self\.selected_entry_handle\(cx\)\?/,
    message: "external file paste must return before the Project Panel clipboard path is read",
  });
  assert.doesNotMatch(
    renderSelectedEntriesToolbar,
    /IconName::(?:Copy|Scissors|BookCopy)/,
    "selection toolbar operation icons must use DX semantic mappings",
  );
  assertBefore({
    body: renderSelectedEntriesToolbar,
    before: "project-panel-clipboard-operation-status",
    after: "project-panel-paste-selection-target",
    message: "selection toolbar should show clipboard operation state before the paste target action",
  });

  assert.match(paste, /let clip_is_cut = clipboard_entries\.is_cut\(\)/);
  assert.match(
    paste,
    /let original_cut_entries = clip_is_cut\.then\(\|\| clipboard_entries\.items\(\)\.clone\(\)\);/,
    "cut paste must capture the exact source cut set before async tasks run",
  );
  assert.match(
    paste,
    /let completed_cut_paste = clip_is_cut && !changes\.is_empty\(\);/,
    "cut paste must only convert clipboard state after at least one successful rename",
  );
  assert.match(
    paste,
    /if completed_cut_paste[\s\S]*original_cut_entries\.as_ref\(\)\.filter[\s\S]*Some\(ClipboardEntry::Cut\(current_entries\)\)[\s\S]*current_entries == \*cut_entries[\s\S]*this\.clipboard = Some\(ClipboardEntry::Copied\(cut_entries\.clone\(\)\)\)[\s\S]*cx\.notify\(\);/,
    "successful cut paste must only convert the original still-current cut clipboard to copy state",
  );
  assert.doesNotMatch(
    paste,
    /this\.clipboard\.take\(\)|ClipboardEntry::into_copy_entry/,
    "async cut paste completion must not mutate a newer clipboard entry",
  );

  assert.match(
    createMovePath,
    /destination_path\.worktree_id == source_path\.worktree_id[\s\S]*new_path\.as_rel_path\(\) == source_path\.path\.as_ref\(\)[\s\S]*return None;/,
    "drag moves into the same parent must stay a no-op instead of creating a copy-suffixed rename",
  );
  assert.match(
    createMovePath,
    /while destination_worktree\.entry_for_path\(&new_path\)\.is_some\(\)[\s\S]*let disambiguation = " copy";[\s\S]*ix \+= 1;/,
    "drag moves into conflicting destinations must disambiguate with the same copy suffix pattern as paste",
  );
  assertBefore({
    body: createMovePath,
    before: /return None;/,
    after: /while destination_worktree\.entry_for_path\(&new_path\)\.is_some\(\)/,
    message: "same-parent drag moves must be checked before conflict disambiguation",
  });
  assert.match(
    moveWorktreeEntry,
    /Self::create_move_path\([\s\S]*source_entry,[\s\S]*&source_path,[\s\S]*&destination_path,[\s\S]*destination_is_file,[\s\S]*&destination_worktree/,
    "normal drag moves must compute conflict-safe destination paths through the move helper",
  );
  assert.match(
    moveWorktreeEntry,
    /project\.rename_entry\([\s\S]*entry_to_move,[\s\S]*\(destination_worktree_id, new_path\)\.into\(\),[\s\S]*cx/,
    "normal drag moves must still dispatch through the real project rename operation",
  );

  assert.match(
    dragMoveEntries,
    /project\.entry_is_worktree_root\(entry\.entry_id, cx\)[\s\S]*BTreeSet::from\(\[entry\]\)/,
    "single worktree-root drags must reach move_entry for root reordering",
  );
  assert.match(
    dragOnto,
    /Self::is_copy_modifier_set\(&window\.modifiers\(\)\)[\s\S]*self\.disjoint_entries\(resolved_selections, cx\)[\s\S]*self\.drag_move_entries\(resolved_selections, cx\)/,
    "copy drags must keep the normal file sanitizer while move drags preserve root reorder intent",
  );

  assert.match(dxIcons, /DxUiIcon::Copy => IconName::Copy/);
  assert.match(dxIcons, /DxUiIcon::Move => IconName::ArrowRightLeft/);
  assert.match(dxIcons, /DxUiIcon::Duplicate => IconName::BookCopy/);
  assert.match(dxIcons, /DxUiIcon::PasteInto => IconName::ReplyArrowRight/);
});

test("project panel display strings, sticky rows, and undo batches are bounded", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const utils = read("crates/project_panel/src/utils.rs");
  const undo = read("crates/project_panel/src/undo.rs");
  const detailsForEntry = functionBody(source, "details_for_entry");
  const renderStickyEntries = functionBody(source, "render_sticky_entries");
  const renderEntry = functionBody(source, "render_entry");
  const record = functionBody(undo, "record");

  assert.match(utils, /pub\(crate\) const MAX_PROJECT_PANEL_DISPLAY_LABEL_CHARS: usize = 1_024;/);
  assert.match(utils, /pub\(crate\) fn bounded_project_panel_label\(/);
  assert.match(source, /const MAX_PROJECT_PANEL_STICKY_PARENTS: usize = 128;/);
  assert.match(undo, /const MAX_PROJECT_PANEL_UNDO_BATCH_CHANGES: usize = 4_096;/);
  assert.match(detailsForEntry, /utils::bounded_project_panel_label\(filename\)/);
  assert.doesNotMatch(
    renderEntry,
    /render_side_panel_header_controls\(\s*"project-panel-sticky"/,
    "sticky tree rows must not inject side-panel close controls into the file tree",
  );
  assertBefore({
    body: renderStickyEntries,
    before: "sticky_parents.len() >= MAX_PROJECT_PANEL_STICKY_PARENTS",
    after: "sticky_parents.push",
    message: "sticky parent rows must be capped before vector push",
  });
  assertBefore({
    body: record,
    before: ".take(MAX_PROJECT_PANEL_UNDO_BATCH_CHANGES + 1)",
    after: "UndoMessage::Changed(changes)",
    message: "undo batches must be capped before sending to the manager task",
  });
});

test("project panel folder storage summaries are cache-only on the visible-row path", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const storage = read("crates/project_panel/src/storage.rs");
  const detailsForEntry = functionBody(source, "details_for_entry");
  const renderEntry = functionBody(source, "render_entry");
  const renderEntryInfoBadge = functionBody(source, "render_entry_info_badge");
  const storageOverview = functionBody(storage, "storage_overview");
  const storageDrilldownItems = functionBody(storage, "storage_folder_items");
  const renderStorageDrilldown = functionBody(source, "render_dx_explorer_storage_drilldown");
  const renderDxExplorerHeader = functionBody(source, "render_dx_explorer_header");
  const renderProjectPanel = functionBody(source, "render");
  const renderStorageDrilldownRow = functionBody(
    source,
    "render_dx_explorer_storage_drilldown_row",
  );
  const storageHeatLevel = functionBody(storage, "storage_heat_level");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");
  const cachedFolderStorageSummary = functionBody(source, "cached_folder_storage_summary");

  assert.match(
    source,
    /const MAX_PROJECT_PANEL_BACKGROUND_FOLDER_STORAGE_DIRS: usize = 4_096;/,
    "folder storage warming must have a named background cap",
  );
  assert.match(
    storage,
    /pub\(crate\) const MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS: usize = 5;/,
    "storage drilldown must stay visually bounded",
  );
  assert.match(
    source,
    /const MAX_PROJECT_PANEL_FOLDER_STORAGE_SUMMARY_CACHE: usize = 4_096;/,
    "folder storage summaries must have a persistent session cache cap",
  );
  assert.match(
    source,
    /const MAX_PROJECT_PANEL_FOLDER_STORAGE_CHILD_FILES: usize = 512;/,
    "direct-child folder storage warming must be capped per folder",
  );
  assert.match(
    source,
    /storage_details_visible:\s*bool/,
    "Project Panel storage intelligence must have explicit visible state",
  );
  assert.match(
    source,
    /storage_details_visible:\s*false/,
    "largest folder storage intelligence must be opt-in by default",
  );
  assert.match(
    renderDxExplorerHeader,
    /IconButton::new\(storage_details_button_id,\s*dx_icon\(DxUiIcon::Storage\)\)[\s\S]*\.tooltip\(Tooltip::text\(if storage_details_visible[\s\S]*Show storage details[\s\S]*this\.toggle_storage_details_visible\(window, cx\)/,
    "Project Panel header must expose a real storage-details toggle without rendering largest folders by default",
  );
  assert.match(
    renderStorageDrilldown,
    /if !self\.storage_details_visible[\s\S]*return None;/,
    "Folder Storage drilldown must render only after the user asks for storage details",
  );
  assert.match(storage, /pub\(crate\) struct FolderStorageSummary/);
  assert.match(storage, /pub\(crate\) struct StorageFolderItem/);
  assert.match(storage, /pub path_label: String/);
  assert.match(storage, /pub\(crate\) fn from_entry\(/);
  assert.match(storage, /pub\(crate\) fn has_recorded_files\(&self\) -> bool/);
  assert.match(source, /dx_explorer_storage_drilldown:\s*Vec<storage::StorageFolderItem>/);
  assert.match(storage, /pub\(crate\) fn record_file\(&mut self, entry: &Entry\)/);
  assert.match(source, /fn cached_folder_storage_summary\(/);
  assert.match(source, /fn visible_storage_entries\(/);
  assert.doesNotMatch(source, /fn dx_explorer_storage_drilldown_items\(/);
  assert.doesNotMatch(source, /fn dx_explorer_storage_overview\(/);
  assert.match(source, /fn render_dx_explorer_storage_drilldown\(/);
  assert.match(source, /fn render_dx_explorer_storage_drilldown_row\(/);
  assert.doesNotMatch(source, /fn dx_explorer_storage_heat_level\(/);
  assert.match(renderEntryInfoBadge, /\.visible_on_hover\("list_item"\)/);
  assert.match(
    renderEntryInfoBadge,
    /Chip::new\(label\)[\s\S]*?\.label_color\(Color::Muted\)[\s\S]*?\.truncate\(\)/,
  );
  assert.doesNotMatch(renderEntryInfoBadge, /\.ml_1\(\)/);
  assert.match(renderEntry, /\.end_slot::<AnyElement>\([\s\S]*h_flex\(\)[\s\S]*\.gap_0p5\(\)[\s\S]*\.pr_0p5\(\)[\s\S]*\.child\(hover_badge\)/);
  assert.doesNotMatch(
    renderEntry,
    /\.end_slot::<AnyElement>\([\s\S]*h_flex\(\)[\s\S]*\.gap_1\(\)[\s\S]*\.pr_1\(\)[\s\S]*\.child\(hover_badge\)/,
  );
  assert.match(source, /fn render_dx_explorer_storage_heat_indicator\(/);
  assert.match(source, /fn dx_explorer_storage_heat_indicator_width\(/);
  assert.match(source, /fn dx_explorer_storage_heat_color\(/);
  assert.match(renderEntryInfoBadge, /\.max_w\(rems\(9\.\)\)/);
  assert.match(renderEntryInfoBadge, /\.overflow_hidden\(\)/);
  assert.match(
    cachedFolderStorageSummary,
    /folder_storage_summaries[\s\S]*get\(&cache_key\)[\s\S]*cloned\(\)/,
    "render-facing folder storage lookup must be cache-only",
  );
  assert.match(
    detailsForEntry,
    /let folder_storage_summary = if self\.storage_details_visible && entry\.kind\.is_dir\(\)[\s\S]*self\.cached_folder_storage_summary\(worktree_id, entry\.id\)[\s\S]*\} else \{[\s\S]*None[\s\S]*\};/,
    "details_for_entry must keep folder-size chips opt-in and cache-only on the visible-row path",
  );
  assert.doesNotMatch(
    detailsForEntry,
    /folder_storage_summaries[\s\S]*borrow_mut\(\)|child_entries_with_options/,
    "details_for_entry must not mutate folder storage caches or scan child entries",
  );
  assert.match(
    renderEntryInfoBadge,
    /let Some\(summary\) = folder_storage_summary else \{[\s\S]*return div\(\)\.into_any_element\(\);[\s\S]*\};/,
    "cold folder storage badges must not display a misleading zero count",
  );
  assert.match(
    renderEntryInfoBadge,
    /summary\.file_count[\s\S]*summary\.file_bytes[\s\S]*format_file_size\(summary\.file_bytes\)/,
    "folder hover badges must show direct child count plus cached storage bytes when available",
  );
  assert.match(
    renderEntryInfoBadge,
    /Chip::new\(label\)[\s\S]*\.label_color\(Color::Muted\)[\s\S]*\.truncate\(\)/,
  );
  assert.match(renderEntryInfoBadge, /let tooltip = label\.clone\(\);/);
  assert.match(renderEntryInfoBadge, /\.tooltip\(Tooltip::text\(tooltip\)\)/);
  assert.doesNotMatch(
    renderEntryInfoBadge,
    /(?:cx:\s*&App|Label::new\(label\)|\.size\(LabelSize::XSmall\)|\.border_1\(\)|border_variant|element_background|\.rounded_sm\(\)|\.px_1\(\)|\.py_0p5\(\)|cx\.theme\(\))/,
    "entry storage badges must use the shared Chip component instead of custom badge chrome",
  );
  assert.match(
    updateVisibleEntries,
    /let cached_folder_storage_summary_keys = cached_folder_storage_summaries[\s\S]*keys\(\)[\s\S]*collect::<HashSet<_>>\(\);/,
    "visible-entry refresh must snapshot folder storage cache state before background warming",
  );
  assert.match(
    updateVisibleEntries,
    /let visible_folder_storage_summary_keys = if storage_details_visible[\s\S]*Self::visible_folder_storage_summary_keys\(&self\.state\)[\s\S]*HashSet::default\(\)[\s\S]*let cached_folder_storage_summaries =[\s\S]*self\.cached_folder_storage_summaries_for_keys\(&visible_folder_storage_summary_keys\);/,
    "visible-entry refresh must snapshot cache entries only when storage details are visible",
  );
  assert.match(
    source,
    /fn cached_folder_storage_summaries_for_keys\([\s\S]*visible_folder_keys: &HashSet<\(WorktreeId, ProjectEntryId\)>[\s\S]*filter_map\(\|key\|[\s\S]*folder_storage_summaries[\s\S]*get\(key\)[\s\S]*cloned\(\)/,
    "folder storage cache snapshots must be filtered by visible folder keys before background work",
  );
  assert.match(
    updateVisibleEntries,
    /let mut folder_storage_summary_updates = Vec::new\(\);/,
    "folder storage warming must collect background results separately from visible rows",
  );
  assert.match(
    updateVisibleEntries,
    /folder_storage_summary_updates\.len\(\)[\s\S]*MAX_PROJECT_PANEL_BACKGROUND_FOLDER_STORAGE_DIRS[\s\S]*let mut summary = storage::FolderStorageSummary::default\(\)[\s\S]*let mut child_file_count = 0usize;[\s\S]*child_entries_with_options[\s\S]*include_files: true[\s\S]*include_dirs: false[\s\S]*child_file_count[\s\S]*MAX_PROJECT_PANEL_FOLDER_STORAGE_CHILD_FILES[\s\S]*summary\.record_file\(child\)[\s\S]*child_file_count \+= 1;[\s\S]*folder_storage_summary_updates\.push\(\(cache_key, summary\)\)/,
    "background folder storage warming must count direct file children, bytes, and mtimes under a named cap",
  );
  assert.match(
    updateVisibleEntries,
    /folder_storage_summaries\.entry\(cache_key\)\.or_insert\(summary\)/,
    "background folder storage results must populate cache misses without overwriting fresher summaries",
  );
  assert.match(
    updateVisibleEntries,
    /let storage_details_visible = self\.storage_details_visible;/,
    "visible-entry refresh must snapshot the storage-details toggle before background work",
  );
  assert.match(
    updateVisibleEntries,
    /let visible_folder_storage_summary_keys = if storage_details_visible[\s\S]*Self::visible_folder_storage_summary_keys\(&self\.state\)[\s\S]*HashSet::default\(\)/,
    "folder storage cache keys must stay empty while storage details are hidden",
  );
  assert.match(
    updateVisibleEntries,
    /if storage_details_visible\s*&&\s*entry_is_visible\s*&&\s*entry\.kind\.is_dir\(\)/,
    "background folder storage warming must be disabled while storage details are hidden",
  );
  assert.match(
    detailsForEntry,
    /if self\.storage_details_visible && entry\.kind\.is_dir\(\)[\s\S]*self\.cached_folder_storage_summary\(worktree_id, entry\.id\)[\s\S]*None/,
    "tree rows must not show folder-size chips until storage details are enabled",
  );
  assert.match(
    updateVisibleEntries,
    /Self::retain_visible_folder_storage_summaries\([\s\S]*&mut folder_storage_summaries,[\s\S]*&visible_folder_storage_summary_keys[\s\S]*\);/,
    "folder storage summary cache must be pruned back to visible folder keys after refresh",
  );
  assert.match(
    storageDrilldownItems,
    /folder_storage_summaries: &HashMap<\(WorktreeId, ProjectEntryId\), FolderStorageSummary>/,
    "storage drilldown must receive the warmed cache from the visible-entry refresh job",
  );
  assert.match(
    storageDrilldownItems,
    /for \(worktree_id, entry\) in visible_entries[\s\S]*!entry\.kind\.is_dir\(\)[\s\S]*folder_storage_summaries\.get\(&cache_key\)/,
    "storage drilldown must derive candidate folders from materialized visible rows and cached summaries",
  );
  assert.match(
    storageDrilldownItems,
    /rank_storage_folder_items\(items, storage_sort_mode\)/,
    "storage drilldown must rank cached folders through the storage domain helper",
  );
  assert.match(
    storageDrilldownItems,
    /items\.truncate\(MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS\)/,
    "storage drilldown must stay capped before render",
  );
  assert.match(
    storageDrilldownItems,
    /StorageFolderItem::from_entry\(worktree_id, entry, summary\)/,
    "storage drilldown must precompute heat-map levels outside render rows",
  );
  assert.match(
    storageOverview,
    /visible_file_bytes,[\s\S]*for \(worktree_id, entry\) in visible_entries[\s\S]*overview\.record_visible_file\(entry\)[\s\S]*summary\.has_recorded_files\(\)/,
    "storage overview projection must live in the storage domain and use visible entry candidates",
  );
  assert.doesNotMatch(
    storageDrilldownItems,
    /\bread_dir\(|\bFile::open\(|read_to_string|child_entries_with_options|cx\.spawn/,
    "storage drilldown item building must stay cache/materialized-state only",
  );
  assertBefore({
    body: updateVisibleEntries,
    before:
      /folder_storage_summary_cache[\s\S]*\.entry\(\*cache_key\)[\s\S]*\.or_insert_with\(\|\| summary\.clone\(\)\)/,
    after:
      /new_state\.dx_explorer_storage_overview =[\s\S]*storage::storage_overview/,
    message: "storage drilldown must merge fresh background summaries before ranking cached folders",
  });
  assertBefore({
    body: updateVisibleEntries,
    before:
      /new_state\.dx_explorer_storage_drilldown =[\s\S]*storage::storage_folder_items\([\s\S]*Self::visible_storage_entries\(&new_state\),[\s\S]*&folder_storage_summary_cache,[\s\S]*storage_sort_mode/,
    after: /\(new_state, media_preview_updates, folder_storage_summary_updates\)\s*\}\)\s*\.await;/,
    message: "storage drilldown must be ranked in the background job before state is installed",
  });
  assert.match(renderStorageDrilldown, /\.id\("dx-explorer-storage-drilldown"\)/);
  assert.match(
    renderStorageDrilldown,
    /\.id\("dx-explorer-storage-drilldown"\)[\s\S]*\.gap_0p5\(\)[\s\S]*\.px_1\(\)[\s\S]*\.py_0p5\(\)/,
    "storage drilldown shell should stay compact in stacked side panels",
  );
  assert.match(renderStorageDrilldown, /dx_icon\(DxUiIcon::Storage\)/);
  assert.match(renderStorageDrilldown, /ListHeader::new\("Folder Storage"\)/);
  assert.match(
    renderStorageDrilldown,
    /ListHeader::new\("Folder Storage"\)[\s\S]*\.start_slot\([\s\S]*Icon::new\(dx_icon\(DxUiIcon::Storage\)\)/,
  );
  assert.match(
    renderStorageDrilldown,
    /ListHeader::new\("Folder Storage"\)[\s\S]*\.end_slot(?:::<[^>]+>)?\([\s\S]*sort_mode\.status_label\(\)[\s\S]*PopoverMenu::new\(storage_sort_menu_id\)/,
  );
  assert.match(
    renderStorageDrilldown,
    /let storage_sort_tooltip = if metrics\.is_empty\(\) \{[\s\S]*format!\("Sort by \{\}", sort_mode\.label\(\)\)[\s\S]*metrics\.join\("\\n"\)/,
  );
  assert.match(renderStorageDrilldown, /Tooltip::text\(storage_sort_tooltip\)/);
  assert.doesNotMatch(
    renderStorageDrilldown,
    /\.children\(metrics\)/,
    "storage drilldown header should keep volatile metrics in tooltip metadata, not visible end-slot labels",
  );
  assert.match(
    renderStorageDrilldown,
    /IconButton::new\(\s*storage_sort_button_id,[\s\S]*IconName::ListFilter,[\s\S]*\)[\s\S]*\.shape\(IconButtonShape::Square\)[\s\S]*\.style\(ButtonStyle::Subtle\)[\s\S]*\.icon_size\(IconSize::Small\)[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&self\.focus_handle\(cx\)\)/,
  );
  assert.match(
    renderStorageDrilldown,
    /div\(\)[\s\S]*\.flex_none\(\)[\s\S]*\.child\([\s\S]*PopoverMenu::new\(storage_sort_menu_id\)/,
  );
  assert.doesNotMatch(
    renderStorageDrilldown,
    /Label::new\("Folder Storage"\)/,
    "storage drilldown section header should use the shared ListHeader component",
  );
  assert.match(renderStorageDrilldown, /\.children\(rows\)/);
  assert.doesNotMatch(renderStorageDrilldown, /\bread_dir\(|\bFile::open\(|child_entries/);
  assert.match(
    renderStorageDrilldownRow,
    /SelectedEntry \{[\s\S]*worktree_id: item\.worktree_id,[\s\S]*entry_id: item\.entry_id/,
    "storage drilldown rows must target real project entries",
  );
  const renderStorageHeatIndicator = functionBody(source, "render_dx_explorer_storage_heat_indicator");
  const storageHeatIndicatorWidth = functionBody(source, "dx_explorer_storage_heat_indicator_width");
  const storageHeatColor = functionBody(source, "dx_explorer_storage_heat_color");

  assert.match(
    renderStorageDrilldownRow,
    /render_dx_explorer_storage_heat_indicator\(item\.heat_level\)/,
  );
  assert.match(renderStorageHeatIndicator, /\.flex_none\(\)/);
  assert.match(renderStorageHeatIndicator, /Indicator::bar\(\)\.color\(dx_explorer_storage_heat_color\(heat_level\)\)/);
  assert.match(renderStorageHeatIndicator, /dx_explorer_storage_heat_indicator_width\(heat_level\)/);
  assert.match(storageHeatIndicatorWidth, /f32::from\(heat_level\.max\(1\)\) \* 8\./);
  assert.match(storageHeatColor, /4 => Color::Warning/);
  assert.match(storageHeatColor, /3 => Color::Accent/);
  assert.match(storageHeatColor, /2 => Color::Info/);
  assert.match(storageHeatColor, /_ => Color::Muted/);
  assert.doesNotMatch(
    renderStorageDrilldownRow,
    /\.h\(px\(4\.\)\)[\s\S]*\.rounded_sm\(\)[\s\S]*\.bg\(heat_color\.opacity\(0\.8\)\)/,
    "storage heat marks should use the shared Indicator bar component",
  );
  assert.match(renderStorageDrilldownRow, /format_file_size\(item\.file_bytes\)/);
  assert.match(renderStorageDrilldownRow, /item\.path_label/);
  assert.match(renderStorageDrilldownRow, /this\.expand_entry\(target\.worktree_id, target\.entry_id, cx\)/);
  assert.match(
    renderStorageDrilldownRow,
    /this\.update_visible_entries\([\s\S]*Some\(\(target\.worktree_id, target\.entry_id\)\)[\s\S]*true,[\s\S]*window,[\s\S]*cx/,
    "storage drilldown row clicks must select and scroll to the real folder",
  );
  assert.match(storageHeatLevel, /u128::from\(file_bytes\) \* 4/);
  assert.match(storageHeatLevel, /scaled\.clamp\(1, 4\) as u8/);
  assert.match(storage, /item\.heat_level = storage_heat_level\(item\.file_bytes, max_file_bytes\)/);
  assertBefore({
    body: renderProjectPanel,
    before: /media_preview::render_folder_media_shelf/,
    after: /self\.render_dx_explorer_storage_drilldown\(cx\)/,
    message: "media shelf should render before storage drilldown and tree rows",
  });
});

test("project panel storage overview and root shortcuts stay cached and professionally named", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const storage = read("crates/project_panel/src/storage.rs");
  const storageRoots = read("crates/project_panel/src/storage_roots.rs");
  const storageRootsView = read("crates/project_panel/src/storage_roots_view.rs");
  const listItem = read("crates/ui/src/components/list/list_item.rs");
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const media = read("crates/project_panel/src/media_preview.rs");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");
  const storageOverview = functionBody(storage, "storage_overview");
  const storageDrilldownItems = functionBody(storage, "storage_folder_items");
  const renderStorageDrilldown = functionBody(source, "render_dx_explorer_storage_drilldown");
  const renderStorageDrilldownRow = functionBody(
    source,
    "render_dx_explorer_storage_drilldown_row",
  );
  const renderRootStripCall = functionBody(source, "render_dx_explorer_storage_root_strip");
  const renderRootStrip = functionBody(storageRootsView, "render_storage_root_strip");
  const renderRootStripRow = functionBody(storageRootsView, "render_storage_root_strip_row");
  const refreshStorageRoots = functionBody(source, "refresh_dx_explorer_storage_roots");
  const refreshStorageRootsAfterInterval = functionBody(
    source,
    "refresh_dx_explorer_storage_roots_after_interval",
  );
  const focusIn = functionBody(source, "focus_in");
  const openStorageRoot = functionBody(source, "open_dx_explorer_storage_root");
  const collectStorageRootShortcuts = functionBody(storageRoots, "collect_storage_root_shortcuts");
  const knownRootShortcuts = functionBody(storageRoots, "known_root_shortcuts");
  const knownRootShortcut = functionBody(storageRoots, "known_root_shortcut");
  const rootStatusLabel = functionBody(storageRoots, "status_label");
  const capacityUsedBytes = functionBody(storageRoots, "used_bytes");
  const driveCapacityProgress = functionBody(storageRootsView, "drive_capacity_progress");
  const compareBySize = functionBody(storage, "compare_by_size");
  const compareByFileCount = functionBody(storage, "compare_by_file_count");
  const compareByModified = functionBody(storage, "compare_by_modified");
  const compareLargestFiles = functionBody(storage, "compare_largest_files");
  const cmpWorktreeEntries = functionBody(source, "cmp_worktree_entries");
  const sortWorktreeEntries = functionBody(source, "sort_worktree_entries");
  const parSortWorktreeEntries = functionBody(source, "par_sort_worktree_entries");
  const renderFolderMediaShelf = functionBody(media, "render_folder_media_shelf");
  const mediaPreviewCountLabel = functionBody(media, "media_preview_count_label");

  assert.match(source, /mod storage;/);
  assert.match(source, /mod storage_roots;/);
  assert.match(source, /mod storage_roots_view;/);
  assert.match(storage, /pub\(crate\) const MAX_PROJECT_PANEL_STORAGE_LARGEST_FILES: usize = 3;/);
  assert.match(storage, /pub\(crate\) const MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS: usize = 5;/);
  assert.match(storageRoots, /pub\(crate\) const MAX_PROJECT_PANEL_STORAGE_ROOT_STRIP_ITEMS: usize = 16;/);
  assert.match(storage, /pub\(crate\) enum StorageSortMode/);
  assert.match(storage, /StorageSortMode::Size/);
  assert.match(storage, /StorageSortMode::FileCount/);
  assert.match(storage, /StorageSortMode::Modified/);
  assert.match(storage, /pub\(crate\) fn status_label\(self\) -> String/);
  assert.match(storage, /pub\(crate\) fn menu_label\(self, current: Self\) -> String/);
  assert.doesNotMatch(storage, /pub\(crate\) fn heat_label\(heat_level: u8\) -> &'static str/);
  assert.match(storage, /pub\(crate\) fn storage_heat_level\(file_bytes: u64, max_file_bytes: u64\) -> u8/);
  assert.match(storage, /pub\(crate\) fn format_file_size\(bytes: u64\) -> String/);
  assert.match(storage, /pub\(crate\) struct FolderStorageSummary/);
  assert.match(storage, /largest_files:\s*Vec<FolderStorageFile>/);
  assert.match(storage, /latest_modified_at:\s*Option<MTime>/);
  assert.match(storage, /pub\(crate\) fn record_file\(&mut self, entry: &Entry\)/);
  assert.match(storage, /pub\(crate\) fn has_recorded_files\(&self\) -> bool/);
  assert.match(storage, /pub path_label: String/);
  assert.match(storage, /fn folder_path_label\(entry: &Entry\) -> String/);
  assert.match(storage, /utils::bounded_project_panel_label/);
  assert.match(storage, /timestamp_for_user\(\)/);
  assert.doesNotMatch(storage, /\b(?:prototype|dummy|magic|slop|v1)\b/i);

  assert.match(source, /folder_storage_summaries:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), storage::FolderStorageSummary>>/);
  assert.match(source, /storage_root_shortcuts:\s*Vec<storage_roots::StorageRootShortcut>/);
  assert.match(source, /storage_root_refresh_generation:\s*Cell<u64>/);
  assert.match(source, /storage_root_refresh_requested_at:\s*Cell<Option<Instant>>/);
  assert.match(source, /storage_root_refresh_task:\s*Task<\(\)>/);
  assert.match(source, /storage_sort_mode:\s*storage::StorageSortMode/);
  assert.match(source, /folder_storage_cache_generation:\s*Cell<u64>/);
  assert.match(source, /fn bump_folder_storage_cache_generation\(&self\)/);
  assert.match(source, /dx_explorer_storage_overview:\s*storage::StorageOverview/);

  assert.match(
    updateVisibleEntries,
    /let storage_sort_mode = self\.storage_sort_mode;/,
    "visible-entry refresh must snapshot the storage sort mode outside render",
  );
  assert.match(
    updateVisibleEntries,
    /let folder_storage_cache_generation = self\.folder_storage_cache_generation\.get\(\);/,
    "visible-entry refresh must snapshot folder storage generation before background warming",
  );
  assert.match(
    updateVisibleEntries,
    /let mut summary = storage::FolderStorageSummary::default\(\)[\s\S]*summary\.record_file\(child\)/,
    "background folder storage warming must record direct child entries with size and mtime",
  );
  assert.match(
    updateVisibleEntries,
    /let mut new_state = new_state;[\s\S]*let\s+folder_storage_cache_current\s*=\s*this\.folder_storage_cache_generation\.get\(\)\s*==\s*folder_storage_cache_generation;/,
    "foreground install must compare the current folder-storage generation before accepting warmed data",
  );
  assert.match(
    updateVisibleEntries,
    /if folder_storage_cache_current[\s\S]*for \(cache_key, summary\) in folder_storage_summary_updates[\s\S]*folder_storage_summaries\.entry\(cache_key\)\.or_insert\(summary\)[\s\S]*else[\s\S]*new_state\.dx_explorer_storage_overview = Default::default\(\);[\s\S]*new_state\.dx_explorer_storage_drilldown\.clear\(\);/,
    "stale folder-storage jobs must not merge warmed summaries or install stale storage projections",
  );
  assert.match(
    storageOverview,
    /visible_file_count,[\s\S]*visible_file_bytes,/,
    "storage overview must read visible summary bytes",
  );
  assert.match(
    storageOverview,
    /let cache_key = \(worktree_id, entry\.id\);[\s\S]*folder_storage_summaries\.get\(&cache_key\)/,
    "storage overview cached-folder totals must stay scoped to visible drilldown candidates",
  );
  assert.match(
    storageOverview,
    /summary\.has_recorded_files\(\)/,
    "storage overview should use the storage-domain predicate for cached folder totals",
  );
  assert.doesNotMatch(
    storageOverview,
    /folder_storage_summaries\.values\(\)/,
    "storage overview must not aggregate stale warmed folders outside the visible tree",
  );
  assertBefore({
    body: updateVisibleEntries,
    before:
      /folder_storage_summary_cache[\s\S]*\.entry\(\*cache_key\)[\s\S]*\.or_insert_with\(\|\| summary\.clone\(\)\)/,
    after:
      /new_state\.dx_explorer_storage_overview =[\s\S]*storage::storage_overview/,
    message: "storage overview must use merged cached folder summaries",
  });
  assert.match(
    updateVisibleEntries,
    /new_state\.dx_explorer_storage_drilldown =[\s\S]*storage::storage_folder_items\([\s\S]*Self::visible_storage_entries\(&new_state\),[\s\S]*&folder_storage_summary_cache,[\s\S]*storage_sort_mode/,
    "storage drilldown ranking must run in the background with the chosen sort mode",
  );

  assert.match(
    storageDrilldownItems,
    /folder_storage_summaries: &HashMap<\(WorktreeId, ProjectEntryId\), FolderStorageSummary>/,
  );
  assert.match(storageDrilldownItems, /StorageFolderItem/);
  assert.match(storageDrilldownItems, /StorageFolderItem::from_entry/);
  assert.match(storageDrilldownItems, /rank_storage_folder_items\(items, storage_sort_mode\)/);
  assert.match(renderStorageDrilldown, /ListHeader::new\("Folder Storage"\)/);
  assert.match(renderStorageDrilldown, /StorageSortMode::ALL/);
  assert.match(renderStorageDrilldown, /SharedString::from\(format!\("dx-explorer-storage-sort-menu-\{panel_id:\?\}"\)\)/);
  assert.match(renderStorageDrilldown, /IconButton::new\(\s*storage_sort_button_id,\s*IconName::ListFilter/);
  assert.match(renderStorageDrilldown, /sort_mode\.status_label\(\)/);
  assert.match(renderStorageDrilldown, /mode\.menu_label\(sort_mode\)/);
  assert.doesNotMatch(
    renderStorageDrilldown,
    /Folder files|cached child file|cached child files|Sorted by/,
    "Folder storage header should use polished product text instead of implementation/cache wording",
  );
  assert.match(
    renderStorageDrilldown,
    /this\.storage_sort_mode = mode;[\s\S]*this\.update_visible_entries\(\s*None,\s*false,\s*false,[\s\S]*cx\.notify\(\);/,
    "storage sort menu must immediately refresh visible storage projections",
  );
  assert.match(renderStorageDrilldownRow, /format_file_size\(item\.file_bytes\)/);
  assert.match(renderStorageDrilldownRow, /storage::format_modified_label\(item\.latest_modified_at\)/);
  assert.doesNotMatch(renderStorageDrilldownRow, /storage::heat_label\(item\.heat_level\)/);
  assert.match(renderStorageDrilldownRow, /item\.path_label/);
  assert.match(renderStorageDrilldownRow, /Tooltip::with_meta\("Folder"/);
  assert.match(renderStorageDrilldownRow, /Largest files:/);
  assert.doesNotMatch(
    renderStorageDrilldownRow,
    /Folder file summary|Largest:|\{heat_label\}/,
    "Folder storage tooltips should avoid generated-summary and heat-label debug wording",
  );
  assert.match(renderStorageDrilldownRow, /item\s*\.\s*largest_files/);
  assert.match(
    renderStorageDrilldownRow,
    /ListItem::new\(SharedString::from\(format!\([\s\S]*"dx-explorer-storage-drilldown-\{\}-\{\}"[\s\S]*item\.worktree_id\.to_usize\(\),[\s\S]*item\.entry_id\.to_usize\(\)[\s\S]*\)\)\)/,
    "storage drilldown rows must use stable real-entry ListItem ids",
  );
  assert.match(renderStorageDrilldownRow, /\.spacing\(ListItemSpacing::Dense\)/);
  assert.match(renderStorageDrilldownRow, /\.toggle_state\(is_selected\)/);
  assert.match(renderStorageDrilldownRow, /\.tab_index\(0(?:_isize)?\)/);
  assert.match(renderStorageDrilldownRow, /\.track_focus\(&self\.focus_handle\(cx\)\)/);
  assert.match(renderStorageDrilldownRow, /\.start_slot::<AnyElement>\(/);
  assert.match(renderStorageDrilldownRow, /\.end_slot::<AnyElement>\(/);
  assert.match(
    renderStorageDrilldownRow,
    /let target_is_current_dir = \{[\s\S]*worktree_for_id\(target\.worktree_id, cx\)[\s\S]*entry_for_id\(target\.entry_id\)[\s\S]*entry\.is_dir\(\)[\s\S]*unwrap_or\(false\)[\s\S]*if !target_is_current_dir \{[\s\S]*return;/,
    "storage drilldown row clicks must fail closed when cached folder targets go stale",
  );
  assert.match(
    renderStorageDrilldownRow,
    /\.on_click\(cx\.listener\(move \|this, _, window, cx\|[\s\S]*this\.focus_handle\(cx\)\.focus\(window, cx\)[\s\S]*this\.expand_entry\(target\.worktree_id, target\.entry_id, cx\)[\s\S]*this\.update_visible_entries\([\s\S]*Some\(\(target\.worktree_id, target\.entry_id\)\)[\s\S]*true,[\s\S]*window,[\s\S]*cx/,
    "storage drilldown ListItem clicks must focus, expand, select, and scroll to the real folder",
  );
  assert.match(renderStorageDrilldownRow, /\.start_slot::<AnyElement>\([\s\S]*render_dx_explorer_storage_heat_indicator\(item\.heat_level\)[\s\S]*\.into_any_element\(\)/);
  assert.doesNotMatch(renderStorageDrilldownRow, /Label::new\(heat_label\)/);
  assert.match(renderStorageDrilldownRow, /\.child\([\s\S]*Label::new\(item\.label\)[\s\S]*\.truncate\(\)/);
  assert.match(renderStorageDrilldownRow, /\.end_slot::<AnyElement>\([\s\S]*Label::new\(format!\("\{file_count\} \/ \{storage_label\}"\)\)/);
  assert.doesNotMatch(renderStorageDrilldownRow, /Label::new\(largest_files\.join/);
  assert.doesNotMatch(
    renderStorageDrilldownRow,
    /ButtonLike::new|\.selected_style\(ButtonStyle::Tinted\(TintColor::Accent\)\)|\.size\(ButtonSize::None\)|\.full_width\(\)|cursor_pointer\(\)|\.hover\(/,
    "storage drilldown rows must use Zed ListItem chrome instead of custom ButtonLike row styling",
  );
  assert.match(
    listItem,
    /tab_index:\s*Option<isize>/,
    "Zed ListItem must support explicit tab stops for focusable Project Panel action rows",
  );
  assert.match(
    listItem,
    /focus_handle:\s*Option<FocusHandle>/,
    "Zed ListItem must track focus when used as an action row",
  );
  assert.match(listItem, /pub fn tab_index\(mut self, tab_index: isize\) -> Self/);
  assert.match(listItem, /pub fn track_focus\(mut self, focus_handle: &FocusHandle\) -> Self/);

  assert.match(renderRootStrip, /\.id\("dx-explorer-storage-root-strip"\)/);
  assert.match(
    renderRootStripCall,
    /storage_roots_view::render_storage_root_strip\([\s\S]*self\.storage_root_shortcuts\.clone\(\),[\s\S]*cx\.entity\(\)\.downgrade\(\),[\s\S]*self\.focus_handle\(cx\),[\s\S]*cx/,
    "Project Panel should delegate storage-root strip rendering to the focused view module",
  );
  assert.match(
    renderRootStripCall,
    /if !self\.storage_root_shortcuts_allowed\(cx\) \{[\s\S]*return None;[\s\S]*\}/,
    "local storage-root shortcuts must not render for read-only or remote-only project contexts",
  );
  assert.match(source, /fn storage_root_shortcuts_allowed\(&self, cx: &mut Context<Self>\) -> bool \{[\s\S]*if !self\.storage_details_visible \{[\s\S]*return false;[\s\S]*\}[\s\S]*!project\.is_read_only\(cx\)[\s\S]*project\.is_local\(\) \|\| project\.is_via_wsl_with_host_interop\(cx\)/);
  assert.match(source, /fn toggle_storage_details_visible\(&mut self, window: &mut Window, cx: &mut Context<Self>\) \{[\s\S]*self\.storage_details_visible = !self\.storage_details_visible;[\s\S]*if self\.storage_details_visible \{[\s\S]*self\.refresh_dx_explorer_storage_roots\(cx\);[\s\S]*\} else \{[\s\S]*self\.storage_root_shortcuts\.clear\(\);[\s\S]*\}[\s\S]*self\.update_visible_entries\(None, false, false, window, cx\);[\s\S]*cx\.notify\(\);[\s\S]*\}/);
  assert.match(source, /if this\.storage_root_shortcuts_allowed\(cx\) \{[\s\S]*this\.refresh_dx_explorer_storage_roots\(cx\);[\s\S]*\}/);
  assert.match(source, /fn refresh_dx_explorer_storage_roots\(&mut self, cx: &mut Context<Self>\)[\s\S]*if !self\.storage_root_shortcuts_allowed\(cx\) \{[\s\S]*self\.storage_root_shortcuts\.clear\(\);[\s\S]*return;[\s\S]*\}/);
  assert.match(source, /if !this\.storage_root_shortcuts_allowed\(cx\) \{[\s\S]*this\.storage_root_shortcuts\.clear\(\);[\s\S]*cx\.notify\(\);[\s\S]*return;[\s\S]*\}/);
  assert.match(source, /fn open_dx_explorer_storage_root\([\s\S]*if !self\.storage_root_shortcuts_allowed\(cx\) \{[\s\S]*return;[\s\S]*\}/);
  assert.doesNotMatch(source, /render_dx_explorer_storage_root_strip\(is_local_or_wsl, is_read_only, cx\)/);
  assert.match(renderRootStrip, /ListHeader::new\("Storage"\)/);
  assert.match(
    renderRootStrip,
    /\.id\("dx-explorer-storage-root-strip"\)[\s\S]*\.gap_0p5\(\)[\s\S]*\.px_1\(\)[\s\S]*\.py_0p5\(\)/,
    "storage root strip should stay compact and aligned with the storage drilldown chrome",
  );
  assert.match(
    renderRootStrip,
    /\.id\("dx-explorer-storage-root-strip-scroll"\)[\s\S]*h_flex\(\)\.gap_0p5\(\)\.children\(rows\)/,
    "storage root shortcut row spacing should be owned by the shared strip chrome",
  );
  assert.match(
    renderRootStrip,
    /ListHeader::new\("Storage"\)[\s\S]*\.start_slot\(Icon::new\(dx_icon\(DxUiIcon::Storage\)\)\.size\(IconSize::Small\)\)/,
  );
  assert.match(
    renderRootStrip,
    /shortcuts[\s\S]*\.map\(\|shortcut\|[\s\S]*render_storage_root_strip_row\(shortcut, panel\.clone\(\), focus_handle\.clone\(\), cx\)[\s\S]*\)/,
  );
  assert.doesNotMatch(source, /fn render_dx_explorer_storage_root_strip_row\(/);
  assert.match(renderRootStripRow, /storage_roots::StorageRootKind::Drive/);
  assert.match(storageRootsView, /ButtonLike/);
  assert.match(storageRootsView, /ButtonSize/);
  assert.match(storageRootsView, /ButtonStyle/);
  assert.match(storageRootsView, /ProgressBar/);
  assert.match(renderRootStripRow, /ButtonLike::new\(/);
  assert.match(renderRootStripRow, /\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderRootStripRow, /\.size\(ButtonSize::Compact\)/);
  assert.match(renderRootStripRow, /\.max_w\(rems\(18\.\)\)/);
  assert.doesNotMatch(renderRootStripRow, /\.width\(rems\(18\.\)\)/);
  assert.match(
    renderRootStripRow,
    /\.when\(available,[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&row_focus_handle\)/,
  );
  assert.doesNotMatch(
    renderRootStripRow,
    /\.size\(ButtonSize::Compact\)\s*\.tab_index\(0(?:_isize)?\)\s*\.track_focus\(&focus_handle\)/,
    "unavailable storage roots must not remain in keyboard tab order",
  );
  assert.match(renderRootStripRow, /Icon::new\(icon\)/);
  assert.match(renderRootStripRow, /let status_label = shortcut\.status_label\(\);/);
  assert.match(renderRootStripRow, /Tooltip::with_meta\("Storage", None, format!\("\{tooltip\}\\n\{status_label\}"\), cx\)/);
  assert.doesNotMatch(renderRootStripRow, /Label::new\(status_label\)/);
  assert.match(renderRootStripRow, /Label::new\(shortcut\.label\)[\s\S]*\.truncate\(\)/);
  assert.match(renderRootStripRow, /\.disabled\(!available\)/);
  assert.match(
    renderRootStripRow,
    /StorageRootKind::OneDrive => dx_icon\(DxUiIcon::CloudStorage\)[\s\S]*StorageRootKind::GoogleDrive => dx_icon\(DxUiIcon::DriveProvider\)[\s\S]*StorageRootKind::Dropbox => dx_icon\(DxUiIcon::DropboxProvider\)/,
    "cloud storage variants must stay tied to provider-specific buttons",
  );
  assert.match(renderRootStripRow, /dx_icon\(DxUiIcon::CloudStorage\)/);
  assert.match(renderRootStripRow, /dx_icon\(DxUiIcon::DriveProvider\)/);
  assert.match(renderRootStripRow, /dx_icon\(DxUiIcon::DropboxProvider\)/);
  assert.match(renderRootStripRow, /let row_focus_handle = focus_handle\.clone\(\);/);
  assert.match(renderRootStripRow, /let click_focus_handle = row_focus_handle\.clone\(\);/);
  assert.match(renderRootStripRow, /window\.focus\(&click_focus_handle, cx\)/);
  assert.match(renderRootStripRow, /this\.open_dx_explorer_storage_root\(path\.clone\(\), window, cx\)/);
  assert.match(renderRootStripRow, /let status_label = shortcut\.status_label\(\);/);
  assert.match(
    capacityUsedBytes,
    /self\.total_bytes\s*\.saturating_sub\(self\.available_bytes\.min\(self\.total_bytes\)\)/,
    "drive capacity progress must derive used bytes from real disk capacity",
  );
  assert.match(
    driveCapacityProgress,
    /matches!\(shortcut\.kind, storage_roots::StorageRootKind::Drive\)[\s\S]*shortcut\.capacity\.as_ref\(\)[\s\S]*capacity\.used_bytes\(\) as f32[\s\S]*capacity\.total_bytes as f32[\s\S]*100\.0[\s\S]*\.clamp\(0\.0, 100\.0\)/,
    "storage root capacity progress must be real drive-only capacity derived from cached sysinfo data",
  );
  assert.match(
    renderRootStripRow,
    /ProgressBar::new\([\s\S]*capacity_progress\.id[\s\S]*capacity_progress\.used_percent[\s\S]*100\.0_f32[\s\S]*cx,?\s*\)/,
    "storage root capacity should render a shared ProgressBar component",
  );
  assert.doesNotMatch(
    renderRootStripRow,
    /storage::format_file_size|format_file_size\(capacity\.|capacity\.capacity_label\(\)|format!\([\s\S]{0,120}(?:free|quota|GB|MB|TB)/i,
    "storage root capacity copy must come from the storage-root domain, not inline panel formatting",
  );
  assert.doesNotMatch(
    source,
    /(?:pub(?:\([^)]*\))?\s+)?fn\s+format_file_size\s*\(/,
    "storage byte formatting must live in the storage domain instead of the large panel file",
  );
  assert.match(
    renderRootStripRow,
    /let available = shortcut\.is_available\(\);[\s\S]*\.when\(available,[\s\S]*\.on_click\(/,
    "storage root rows must only wire activation for available absolute roots",
  );
  assert.match(
    renderRootStripRow,
    /\.disabled\(!available\)/,
    "unavailable storage roots must use the shared disabled ButtonLike state instead of custom row chrome",
  );
  assert.doesNotMatch(
    renderRootStripRow,
    /\.border_1\(\)|border_variant|element_background|cursor_not_allowed\(\)|opacity\(0\.55\)/,
    "storage root shortcuts must use Zed ButtonLike chrome instead of custom row styling",
  );
  assert.match(openStorageRoot, /open_workspace_for_paths\([\s\S]*OpenMode::Activate,[\s\S]*vec!\[path\]/);
  assertBefore({
    body: openStorageRoot,
    before: /if !path\.is_absolute\(\) \|\| !path\.is_dir\(\)[\s\S]*return;/,
    after: /open_workspace_for_paths\([\s\S]*OpenMode::Activate,[\s\S]*vec!\[path\]/,
    message: "storage root activation must reject relative, missing, or file paths before opening",
  });
  assert.doesNotMatch(openStorageRoot, /path\.exists\(\)/);
  assert.match(refreshStorageRoots, /storage_root_refresh_generation/);
  assert.match(refreshStorageRoots, /self\.storage_root_refresh_requested_at[\s\S]*\.set\(Some\(Instant::now\(\)\)\)/);
  assert.match(refreshStorageRoots, /background_spawn\(async move \{[\s\S]*storage_roots::collect_storage_root_shortcuts\(\)/);
  assert.match(refreshStorageRoots, /this\.storage_root_refresh_generation\.get\(\) != generation/);
  assert.match(
    refreshStorageRootsAfterInterval,
    /PROJECT_PANEL_STORAGE_ROOT_REFRESH_INTERVAL[\s\S]*self\.refresh_dx_explorer_storage_roots\(cx\)/,
    "focus refresh must reuse the existing generation-guarded storage-root refresh path",
  );
  assert.match(
    focusIn,
    /self\.refresh_dx_explorer_storage_roots_after_interval\(cx\)/,
    "Project Panel focus must throttle-refresh drive capacity labels for long sessions",
  );

  assert.match(storageRoots, /sysinfo::Disks::new_with_refreshed_list\(\)/);
  assert.match(storageRoots, /MAX_PROJECT_PANEL_STORAGE_ROOT_STRIP_ITEMS/);
  assert.match(storageRoots, /use crate::storage;/);
  assert.match(storageRoots, /pub\(crate\) fn capacity_label\(&self\) -> String/);
  assert.match(storageRoots, /pub\(crate\) fn used_bytes\(&self\) -> u64/);
  assert.match(storageRoots, /pub\(crate\) fn status_label\(&self\) -> String/);
  assert.doesNotMatch(
    knownRootShortcut,
    /capacity:\s*Some|DriveCapacity|available_bytes|total_bytes/,
    "named cloud roots must not invent quota or capacity values",
  );
  assert.doesNotMatch(
    storageRoots,
    /\b(?:cloud quota|free tier|quota|15\s*GB|2\s*GB|1\s*TB)\b/i,
    "storage roots must not encode fake provider quota copy",
  );
  assert.match(
    collectStorageRootShortcuts,
    /let known_roots = known_root_shortcuts\(\);[\s\S]*saturating_sub\(known_roots\.len\(\)\)[\s\S]*collect_drive_shortcuts\(&mut shortcuts, drive_limit\);[\s\S]*shortcuts\.extend\(known_roots\);[\s\S]*dedupe_and_cap\(shortcuts\)/,
    "drive shortcuts must reserve strip slots for DX/cloud roots before capping",
  );
  assert.match(
    knownRootShortcuts,
    /StorageRootKind::OneDrive[\s\S]*"OneDrive"[\s\S]*"onedrive"[\s\S]*OneDriveConsumer[\s\S]*StorageRootKind::GoogleDrive[\s\S]*"Google Drive"[\s\S]*"google-drive"[\s\S]*GOOGLE_DRIVE_ROOT[\s\S]*StorageRootKind::Dropbox[\s\S]*"Dropbox"[\s\S]*"dropbox"[\s\S]*DROPBOX_ROOT/,
    "cloud drive buttons must keep named provider shortcuts",
  );
  assert.match(
    knownRootShortcut,
    /find_map\(valid_env_dir_path\)[\s\S]*fallbacks\.iter\(\)\.find\(\|path\| path\.is_dir\(\)\)\.cloned\(\)[\s\S]*fallbacks\.first\(\)\.cloned\(\)[\s\S]*let available = path\.is_absolute\(\) && path\.is_dir\(\);/,
    "known roots must prefer valid env directory paths before falling back to user-profile roots",
  );
  assert.match(
    storageRoots,
    /fn valid_env_dir_path\(name: &&str\) -> Option<PathBuf>[\s\S]*env_path\(name\)[\s\S]*\.filter\(\|path\| path\.is_absolute\(\) && path\.is_dir\(\)\)/,
    "cloud-drive env roots must reject stale, relative, or file paths before suppressing fallbacks",
  );
  assert.match(
    rootStatusLabel,
    /Some\(capacity\)[\s\S]*capacity\.capacity_label\(\)[\s\S]*self\.is_available\(\)[\s\S]*"Available"[\s\S]*"Unavailable"/,
    "storage root rows must show honest capacity/available/unavailable status",
  );
  assert.match(storageRoots, /DX_HOME/);
  assert.match(storageRoots, /OneDriveConsumer/);
  assert.match(storageRoots, /GOOGLE_DRIVE_ROOT/);
  assert.match(storageRoots, /DROPBOX_ROOT/);
  assert.doesNotMatch(storageRoots, /\bread_dir\(|walkdir|glob|recursive|home.*scan/i);
  assert.doesNotMatch(source, /sysinfo::Disks|GetDiskFreeSpaceExW/);

  assert.match(dxIcons, /CloudStorage/);
  assert.match(dxIcons, /DriveProvider/);
  assert.match(dxIcons, /DropboxProvider/);
  assert.match(dxIcons, /DxUiIcon::CloudStorage => IconName::CloudDownload/);
  assert.match(dxIcons, /DxUiIcon::DriveProvider => IconName::DxForgeProviderDrive/);
  assert.match(dxIcons, /DxUiIcon::DropboxProvider => IconName::DxForgeProviderDropbox/);
  assert.match(renderFolderMediaShelf, /dx_icon\(DxUiIcon::Media\)/);
  assert.doesNotMatch(renderFolderMediaShelf, /IconName::Blocks/);

  assert.match(
    compareBySize,
    /right[\s\S]*\.file_bytes[\s\S]*\.cmp\(&left\.file_bytes\)[\s\S]*right\.file_count\.cmp\(&left\.file_count\)[\s\S]*compare_mtime_desc\(left\.latest_modified_at, right\.latest_modified_at\)[\s\S]*left\.label\.cmp\(&right\.label\)/,
    "size sorting must tie-break by file count, modified time, then label",
  );
  assert.match(
    compareByFileCount,
    /right[\s\S]*\.file_count[\s\S]*\.cmp\(&left\.file_count\)[\s\S]*right\.file_bytes\.cmp\(&left\.file_bytes\)[\s\S]*compare_mtime_desc\(left\.latest_modified_at, right\.latest_modified_at\)[\s\S]*left\.label\.cmp\(&right\.label\)/,
    "file-count sorting must tie-break by size, modified time, then label",
  );
  assert.match(
    compareByModified,
    /compare_mtime_desc\(left\.latest_modified_at, right\.latest_modified_at\)[\s\S]*right\.file_bytes\.cmp\(&left\.file_bytes\)[\s\S]*right\.file_count\.cmp\(&left\.file_count\)[\s\S]*left\.label\.cmp\(&right\.label\)/,
    "modified sorting must tie-break by size, file count, then label",
  );
  assert.match(
    compareLargestFiles,
    /right[\s\S]*\.file_bytes[\s\S]*\.cmp\(&left\.file_bytes\)[\s\S]*compare_mtime_desc\(left\.modified_at, right\.modified_at\)[\s\S]*left\.label\.cmp\(&right\.label\)/,
    "largest-file chips must keep deterministic size, modified, and label ordering",
  );

  for (const body of [
    storageOverview,
    storageDrilldownItems,
    renderStorageDrilldown,
    renderStorageDrilldownRow,
    renderRootStrip,
    renderRootStripRow,
  ]) {
    assert.doesNotMatch(
      body,
      /\bread_dir\(|\bFile::open\(|read_to_string|std::fs::|fs::metadata|canonicalize\(|child_entries_with_options|cx\.spawn|background_spawn|sysinfo::/,
      "render-facing storage helpers must stay cache/materialized-state only",
    );
  }

  for (const body of [cmpWorktreeEntries, sortWorktreeEntries, parSortWorktreeEntries]) {
    assert.doesNotMatch(
      body,
      /storage_sort_mode|StorageSortMode|folder_storage|FolderStorage|file_bytes|file_count|mtime|latest_modified_at|size/,
      "tree sorting must remain path/kind based until a separate tree-sort contract exists",
    );
  }
});

test("project panel media preview is lazy, bounded, and preserves normal tree rows", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const media = read("crates/project_panel/src/media_preview.rs");
  const detailsForEntry = functionBody(source, "details_for_entry");
  const renderEntry = functionBody(source, "render_entry");
  const renderProjectPanel = functionBody(source, "render");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");
  const activeMediaFolderForSelection = functionBody(source, "active_media_folder_for_selection");
  const activeFolderMediaPreview = functionBody(source, "active_folder_media_preview");
  const cachedFolderMediaPreview = functionBody(source, "cached_folder_media_preview");
  const selectNext = functionBody(source, "select_next");
  const selectPrevious = functionBody(source, "select_previous");
  const selectMediaShelfEntry = functionBody(source, "select_media_shelf_entry");
  const renderFolderMediaGallery = functionBody(media, "render_folder_media_gallery");
  const renderFolderMediaShelf = functionBody(media, "render_folder_media_shelf");
  const mediaPreviewCountLabel = functionBody(media, "media_preview_count_label");
  const renderMediaShelfOverflowCard = functionBody(media, "render_media_shelf_overflow_card");

  assert.match(source, /mod media_preview;/);
  assert.match(source, /const MAX_PROJECT_PANEL_BACKGROUND_MEDIA_PREVIEW_FOLDERS: usize = 256;/);
  assert.match(source, /struct ActiveMediaFolder/);
  assert.match(source, /enum MediaShelfNavigationDirection/);
  assert.match(source, /folder_media_previews:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), Option<media_preview::FolderMediaPreview>>>/);
  assert.match(source, /media_preview:\s*Option<media_preview::FolderMediaPreview>/);
  assert.match(source, /fn cached_folder_media_preview\(/);
  assert.match(source, /fn active_media_folder_for_selection\(/);
  assert.match(source, /fn active_folder_media_preview\(/);
  assert.match(source, /fn select_media_shelf_entry\(/);

  assert.match(media, /pub\(crate\) const MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN: usize = 512;/);
  assert.match(media, /pub\(crate\) const MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS: usize = 12;/);
  assert.match(media, /pub\(crate\) const PROJECT_PANEL_MEDIA_GALLERY_COLUMNS: u16 = 3;/);
  assert.match(media, /pub\(crate\) const PROJECT_PANEL_MEDIA_SHELF_COLUMNS: u16 = 4;/);
  assert.match(media, /pub\(crate\) enum MediaPreviewKind/);
  assert.match(media, /Image/);
  assert.match(media, /Video/);
  assert.match(media, /Audio/);
  assert.match(media, /fn video_preview_frame/);
  assert.match(media, /fn media_stem_key/);
  assert.match(media, /fn media_preview_card_tooltip_meta/);
  assert.match(media, /fn render_folder_media_gallery/);
  assert.match(media, /fn render_folder_media_shelf/);
  assert.match(media, /fn render_media_shelf_overflow_card\([\s\S]*focus_handle: FocusHandle/);
  assert.match(media, /fn render_media_shelf_card/);
  assert.match(media, /fn render_media_gallery_card/);
  assert.match(media, /fn audio_media_label/);
  assert.match(media, /fn audio_gradient_background/);
  assert.match(media, /pub\(crate\) fn is_media_path/);
  assert.match(media, /fn media_kind_sort_rank/);
  assert.match(media, /entry_id:\s*ProjectEntryId/);
  assert.match(media, /pub\(crate\) enum VideoFramePreviewKind/);
  assert.match(media, /pub\(crate\) struct VideoFramePreview/);
  assert.match(media, /video_frame_preview:\s*Option<VideoFramePreview>/);
  assert.match(media, /duration_label:\s*Option<String>/);
  assert.match(media, /size:\s*u64/);
  assert.match(media, /use ui::\{[\s\S]*ListHeader/);
  assert.match(
    renderFolderMediaGallery,
    /let header_count_label = media_preview_count_label\(visible_count, preview\);[\s\S]*ListHeader::new\("Media"\)[\s\S]*\.start_slot\(Icon::new\(dx_icon\(DxUiIcon::Media\)\)[\s\S]*\.end_slot(?:::<AnyElement>)?\([\s\S]*Label::new\(header_count_label\)/,
    "media gallery popover should use the shared ListHeader component with DX media icon and summary slot",
  );
  assert.match(
    renderFolderMediaShelf,
    /ListHeader::new\("Media"\)[\s\S]*\.start_slot\(Icon::new\(dx_icon\(DxUiIcon::Media\)\)[\s\S]*\.end_slot\(header_controls\)/,
    "top media shelf should use the shared ListHeader component and keep compact header controls in the end slot",
  );
  assert.match(renderFolderMediaShelf, /let visible_media_count = media_card_limit\.min\(preview\.items\.len\(\)\);/);
  assert.match(renderFolderMediaShelf, /let header_count_label = media_preview_count_label\(visible_media_count, preview\);/);
  assert.doesNotMatch(renderFolderMediaShelf, /shelf_cards\.len\(\)\.min\(preview\.total_count\)/);
  assert.match(mediaPreviewCountLabel, /preview\.scanned_cap_hit/);
  assert.match(mediaPreviewCountLabel, /format!\("\{visible_media_count\} of \{\}\+"/);
  assert.match(mediaPreviewCountLabel, /format!\("\{visible_media_count\} of \{\}"/);
  assert.match(renderFolderMediaShelf, /\.when_some\(panel_controls, \|this, controls\| this\.child\(controls\)\)/);
  assert.match(
    renderFolderMediaShelf,
    /render_media_shelf_overflow_card\([\s\S]*preview,[\s\S]*focus_handle\.clone\(\),[\s\S]*cx/,
    "top media shelf should pass the Project Panel focus handle into the overflow trigger",
  );
  assert.match(
    renderMediaShelfOverflowCard,
    /PopoverMenu::new\(menu_id\)[\s\S]*\.trigger_with_tooltip\([\s\S]*ButtonLike::new\(trigger_id\)[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&focus_handle\)[\s\S]*Tooltip::with_meta\("More media"/,
    "media overflow should use a focusable ButtonLike popover trigger with a tooltip",
  );
  assert.doesNotMatch(
    `${renderFolderMediaGallery}\n${renderFolderMediaShelf}`,
    /Label::new\("Media"\)/,
    "media preview headers should not rebuild title chrome with raw labels",
  );
  assert.match(
    cachedFolderMediaPreview,
    /folder_media_previews[\s\S]*get\(&cache_key\)[\s\S]*cloned\(\)[\s\S]*flatten\(\)/,
    "render-facing media preview lookup must be cache-only",
  );
  assert.match(
    updateVisibleEntries,
    /let cached_media_preview_keys = self[\s\S]*folder_media_previews[\s\S]*keys\(\)[\s\S]*collect::<HashSet<_>>\(\);[\s\S]*let generated_media_metadata = self\.generated_media_metadata\.borrow\(\)\.clone\(\);/,
    "visible-entry refresh must snapshot media cache state before background media preview warming",
  );
  assert.match(
    updateVisibleEntries,
    /let media_preview_cache_generation = self\.media_preview_cache_generation\.get\(\);/,
    "visible-entry refresh must capture the current media cache generation before async warming",
  );
  assert.match(
    updateVisibleEntries,
    /background_spawn\(async move \{[\s\S]*let mut active_media_shelf_entry_ids = active_media_shelf_entry_ids;[\s\S]*let mut media_preview_updates = Vec::new\(\);[\s\S]*let is_active_media_folder =[\s\S]*active_media_folder_for_visibility == Some\(cache_key\);[\s\S]*media_preview_enabled[\s\S]*is_active_media_folder[\s\S]*MAX_PROJECT_PANEL_BACKGROUND_MEDIA_PREVIEW_FOLDERS[\s\S]*match generated_media_metadata\.get\(&cache_key\)[\s\S]*build_folder_media_preview_with_generated_metadata\([\s\S]*&absolute_path,[\s\S]*children,[\s\S]*Some\(generated_metadata\)[\s\S]*\)[\s\S]*build_folder_media_preview\([\s\S]*&absolute_path,[\s\S]*children,[\s\S]*\)[\s\S]*active_media_shelf_entry_ids\.extend\([\s\S]*preview\.items\.iter\(\)\.map\(\|item\| item\.entry_id\)[\s\S]*media_preview_updates\.push\(\(cache_key, preview\)\)[\s\S]*\(new_state, media_preview_updates, folder_storage_summary_updates\)/,
    "media preview cache misses must be warmed inside the visible-entry background task",
  );
  assert.match(
    updateVisibleEntries,
    /this\.media_preview_cache_generation\.get\(\) == media_preview_cache_generation[\s\S]*folder_media_previews\.entry\(cache_key\)\.or_insert\(preview\)/,
    "background media preview results must populate cache misses only if the media cache generation is still current",
  );

  assertBefore({
    body: detailsForEntry,
    before: /entry\.kind\.is_dir\(\)\s*&&\s*is_expanded/,
    after: /self\.cached_folder_media_preview\(/,
    message: "media previews must be read only after confirming an expanded directory",
  });
  const mediaPreviewBranch = detailsForEntry.match(
    /let media_preview = if entry\.kind\.is_dir\(\) && is_expanded \{\s*self\.cached_folder_media_preview\(worktree_id, entry\.id\)\s*\} else \{\s*None\s*\};/,
  );
  assert.ok(
    mediaPreviewBranch,
    "details_for_entry must isolate media preview lookup inside the expanded-directory branch",
  );
  assert.match(mediaPreviewBranch[0], /self\.cached_folder_media_preview\(/);
  assert.doesNotMatch(
    detailsForEntry.replace(mediaPreviewBranch[0], ""),
    /self\.cached_folder_media_preview\(/,
    "details_for_entry must not look up media previews outside the expanded-directory branch",
  );
  assert.doesNotMatch(
    detailsForEntry,
    /build_folder_media_preview_with_generated_metadata|read_bounded_media_metadata_manifest|File::open|fs::File::open/,
    "details_for_entry must not build media previews or read media manifests on the visible-row path",
  );
  assert.doesNotMatch(
    mediaPreviewBranch[0],
    /child_entries_with_options/,
    "details_for_entry media preview branch must not scan children on the visible-row path",
  );
  assert.match(
    activeMediaFolderForSelection,
    /entry\.is_file\(\)[\s\S]*entry\.path\.parent\(\)\?[\s\S]*entry = worktree\.entry_for_path\(parent_path\)\?/,
    "active media shelf must resolve selected files to their parent folder",
  );
  assert.match(
    activeMediaFolderForSelection,
    /selected_media_entry_id[\s\S]*media_preview::is_media_path\(entry\.path\.as_std_path\(\)\)/,
    "active media folder must remember when a selected media file is represented by a card",
  );
  assert.match(
    activeFolderMediaPreview,
    /expanded_dir_ids[\s\S]*binary_search\(&active_media_folder\.entry_id\)/,
    "active media shelf must only render for an expanded real folder",
  );
  assert.match(
    activeFolderMediaPreview,
    /self\.cached_folder_media_preview\([\s\S]*active_media_folder\.worktree_id[\s\S]*active_media_folder\.entry_id[\s\S]*\)/,
    "active media shelf must use an already warmed cached media preview",
  );
  assert.doesNotMatch(
    activeFolderMediaPreview,
    /build_folder_media_preview_with_generated_metadata|child_entries_with_options|read_bounded_media_metadata_manifest|File::open|fs::File::open/,
    "active media shelf lookup must not build previews, scan children, or read manifests from render",
  );
  assertBefore({
    body: selectNext,
    before: /self\.select_media_shelf_entry\([\s\S]*MediaShelfNavigationDirection::Next,[\s\S]*window\.modifiers\(\)\.shift,[\s\S]*cx,[\s\S]*\)/,
    after: /self\.index_for_selection\(selection\)/,
    message: "select-next must give the active media shelf a keyboard navigation chance before visible-row fallback",
  });
  assertBefore({
    body: selectPrevious,
    before: /self\.select_media_shelf_entry\([\s\S]*MediaShelfNavigationDirection::Previous,[\s\S]*window\.modifiers\(\)\.shift,[\s\S]*cx,[\s\S]*\)/,
    after: /self\.index_for_selection\(selection\)/,
    message: "select-previous must give the active media shelf a keyboard navigation chance before visible-row fallback",
  });
  assert.match(
    selectMediaShelfEntry,
    /self\.active_folder_media_preview\(cx\)[\s\S]*active_media_folder\.selected_media_entry_id[\s\S]*selection\.entry_id == active_media_folder\.entry_id/,
    "media shelf keyboard navigation must only activate from the folder row or an already selected media card",
  );
  assert.match(
    selectMediaShelfEntry,
    /MediaShelfNavigationDirection::Next[\s\S]*preview[\s\S]*\.items[\s\S]*\.iter\(\)[\s\S]*\.position\(\|item\| item\.entry_id == selected_media_entry_id\)[\s\S]*MediaShelfNavigationDirection::Previous[\s\S]*preview[\s\S]*\.items[\s\S]*\.iter\(\)[\s\S]*\.position\(\|item\| item\.entry_id == selected_media_entry_id\)/,
    "media shelf keyboard navigation must move through bounded real media preview items",
  );
  assert.match(
    selectMediaShelfEntry,
    /SelectedEntry \{[\s\S]*worktree_id: active_media_folder\.worktree_id[\s\S]*entry_id: item\.entry_id[\s\S]*self\.selection = Some\(selection\)/,
    "media shelf keyboard navigation must select the real underlying project entry",
  );
  assert.match(
    selectMediaShelfEntry,
    /entry_id: active_media_folder\.entry_id[\s\S]*self\.selection = Some\(selection\)/,
    "media shelf previous navigation must be able to return to the owning folder row",
  );
  assert.match(
    updateVisibleEntries,
    /let media_preview_enabled =[\s\S]*project\.is_local\(\) \|\| project\.is_via_wsl_with_host_interop\(cx\)[\s\S]*let active_media_folder_for_visibility =[\s\S]*media_preview_enabled[\s\S]*\.then[\s\S]*active_media_folder_for_selection\(cx\)[\s\S]*map\(\|folder\| \(folder\.worktree_id, folder\.entry_id\)\)/,
    "visible-entry derivation must know which active folder is represented by the bottom media shelf",
  );
  assert.match(
    updateVisibleEntries,
    /let active_media_shelf_entry_ids: HashSet<ProjectEntryId> =[\s\S]*active_media_folder_for_visibility[\s\S]*folder_media_previews[\s\S]*preview[\s\S]*items[\s\S]*into_iter\(\)[\s\S]*map\(\|item\| item\.entry_id\)[\s\S]*collect\(\)[\s\S]*unwrap_or_default\(\);/,
    "visible-entry derivation must hide only the bounded media entries actually represented by shelf cards",
  );
  assert.match(
    updateVisibleEntries,
    /entry_is_active_media_shelf_child[\s\S]*active_media_shelf_entry_ids\.contains\(&entry\.id\)[\s\S]*media_preview::is_media_path\([\s\S]*entry\.path\.as_std_path\(\)[\s\S]*entry\.path\.parent\(\)[\s\S]*parent\.id == active_folder_id/,
    "direct media children represented by the active shelf must be detected from snapshot paths",
  );
  assert.match(
    updateVisibleEntries,
    /if entry_is_visible && !entry_is_active_media_shelf_child[\s\S]*visible_worktree_entries\.push\(entry\.to_owned\(\)\)/,
    "active shelf media children must not also render as full-width tree rows",
  );
  assertBefore({
    body: updateVisibleEntries,
    before: /if entry_is_active_media_shelf_child/,
    after: /let \(depth, chars\)/,
    message: "active shelf media children must leave the row path before width estimation",
  });
  assert.doesNotMatch(
    renderEntry,
    /media_preview::render_folder_media_preview/,
    "media previews must not render as row badges inside uniform_list rows",
  );
  assert.match(renderEntry, /\.end_slot::<AnyElement>[\s\S]*\.ml_auto\(\)[\s\S]*\.child\(hover_badge\)/);
  assert.match(renderEntry, /block_mouse_except_scroll\(\)/);
  assert.match(
    renderProjectPanel,
    /let \([\s\S]*is_read_only,[\s\S]*is_remote,[\s\S]*is_local,[\s\S]*is_local_or_wsl,[\s\S]*is_via_remote_server,[\s\S]*dx_explorer_source_kind,[\s\S]*\) = \{[\s\S]*let project = self\.project\.read\(cx\);[\s\S]*let dx_explorer_source_kind = DxExplorerSourceKind::from_project\(&project, cx\);[\s\S]*project\.is_read_only\(cx\)[\s\S]*project\.is_remote\(\)[\s\S]*project\.is_local\(\)[\s\S]*project\.is_local\(\) \|\| project\.is_via_wsl_with_host_interop\(cx\)[\s\S]*project\.is_via_remote_server\(\)[\s\S]*dx_explorer_source_kind,[\s\S]*\};/,
    "render must snapshot project flags before media shelf lookup so no long project borrow crosses cx-using closures",
  );
  assertBefore({
    body: renderProjectPanel,
    before: /let active_media_preview = \(has_worktree && is_local_or_wsl\)/,
    after: /let panel_settings = ProjectPanelSettings::get_global\(cx\);/,
    message: "active media shelf lookup must finish before panel rendering settings are applied",
  });
  assert.match(
    renderProjectPanel,
    /\(has_worktree && is_local_or_wsl\)[\s\S]*self\.top_folder_media_preview\(cx\)/,
    "media shelf thumbnails must only render for local or WSL-backed worktrees",
  );
  assertBefore({
    body: renderProjectPanel,
    before: /media_preview::render_folder_media_shelf/,
    after: /uniform_list\("entries"/,
    message: "media shelf must render above the virtualized tree rather than inside or below a row",
  });
  assert.match(
    renderProjectPanel,
    /id\("project-panel-media-shelf-scroll-proxy"\)[\s\S]*on_scroll_wheel[\s\S]*base_handle\.set_offset\(new_offset\)/,
    "media shelf must forward wheel scrolling to the file tree scroll handle",
  );
  assert.match(
    renderProjectPanel,
    /active_media_folder\.worktree_id[\s\S]*active_media_folder\.selected_media_entry_id/,
    "media shelf must receive real worktree and selected media entry state",
  );
  assert.match(
    renderProjectPanel,
    /media_preview::render_folder_media_shelf\([\s\S]*active_media_folder\.selected_media_entry_id,[\s\S]*self\.focus_handle\(cx\)/,
    "media shelf must receive the Project Panel focus handle for keyboard-reachable controls",
  );
  assertBefore({
    body: renderProjectPanel,
    before: /media_preview::render_folder_media_shelf/,
    after: /id\("project-panel-blank-area"\)/,
    message: "media shelf must remain ahead of the blank drop zone",
  });
});

test("project panel media preview renders direct image previews and video frames when available", () => {
  const media = read("crates/project_panel/src/media_preview.rs");
  const metadata = read("crates/project_panel/src/media_preview/metadata.rs");
  const generatedMetadata = read(
    "crates/project_panel/src/media_preview/generated_metadata.rs",
  );
  const generatedVideoFrame = read(
    "crates/project_panel/src/media_preview/generated_video_frame.rs",
  );
  const metadataProbe = read("crates/project_panel/src/media_preview/metadata_probe.rs");
  const projectPanelCargo = read("crates/project_panel/Cargo.toml");
  const projectPanel = read("crates/project_panel/src/project_panel.rs");
  const renderFolderMediaGallery = functionBody(media, "render_folder_media_gallery");
  const renderFolderMediaShelf = functionBody(media, "render_folder_media_shelf");
  const renderMediaShelfCard = functionBody(media, "render_media_shelf_card");
  const renderMediaGalleryCard = functionBody(media, "render_media_gallery_card");
  const mediaShelfCardContainer = functionBody(media, "media_shelf_card_container");
  const mediaGalleryCardContainer = functionBody(media, "media_gallery_card_container");
  const renderMediaShelfCardBody = functionBody(media, "render_media_shelf_card_body");
  const mediaCardImageFallback = functionBody(media, "media_card_image_fallback");
  const mediaPreviewCardTooltipMeta = functionBody(media, "media_preview_card_tooltip_meta");
  const audioMediaLabel = functionBody(media, "audio_media_label");
  const audioGradientBackground = functionBody(media, "audio_gradient_background");
  const buildFolderMediaPreview = functionBody(media, "build_folder_media_preview");
  const buildFolderMediaPreviewWithGeneratedMetadata = functionBody(
    media,
    "build_folder_media_preview_with_generated_metadata",
  );
  const selectBalancedMediaPreviewItems = functionBody(
    media,
    "select_balanced_media_preview_items",
  );
  const pushMediaPreviewItemIfMissing = functionBody(
    media,
    "push_media_preview_item_if_missing",
  );
  const buildMediaMetadataIndex = functionBody(metadata, "build_media_metadata_index");
  const buildGeneratedMediaMetadataJobBatch = functionBody(
    generatedMetadata,
    "build_generated_media_metadata_job_batch",
  );
  const collectGeneratedMediaMetadata = functionBody(
    generatedMetadata,
    "collect_generated_media_metadata",
  );
  const isSafeGeneratedMediaMetadataJob = functionBody(
    generatedMetadata,
    "is_safe_managed_job",
  );
  const safeGeneratedMediaMetadataJobs = functionBody(
    generatedMetadata,
    "safe_jobs",
  );
  const generateVideoCenterFrame = functionBody(
    generatedVideoFrame,
    "generate_video_center_frame",
  );
  const isSafeGeneratedVideoSource = functionBody(
    generatedVideoFrame,
    "is_safe_generated_video_source",
  );
  const managedVideoFrameCachePath = functionBody(
    generatedVideoFrame,
    "managed_video_frame_cache_path",
  );
  const stableVideoFrameCacheKey = functionBody(
    generatedVideoFrame,
    "stable_video_frame_cache_key",
  );
  const probeVideoDurationSeconds = functionBody(
    generatedVideoFrame,
    "probe_video_duration_seconds",
  );
  const extractVideoCenterFrame = functionBody(
    generatedVideoFrame,
    "extract_video_center_frame",
  );
  const runMediaCommandOutput = functionBody(
    generatedVideoFrame,
    "run_media_command_output",
  );
  const mediaBinaryIsShell = functionBody(
    generatedVideoFrame,
    "media_binary_is_shell",
  );
  const audioDurationSecondsForPath = functionBody(
    generatedMetadata,
    "audio_duration_seconds_for_path",
  );
  const mergeGeneratedMediaMetadata = functionBody(
    metadata,
    "merge_generated_media_metadata",
  );
  const activeFolderMediaPreview = functionBody(projectPanel, "active_folder_media_preview");
  const ensureGeneratedMediaMetadata = functionBody(
    projectPanel,
    "ensure_generated_media_metadata",
  );
  const clearDxExplorerMediaCaches = functionBody(projectPanel, "clear_dx_explorer_media_caches");
  const clearDxExplorerMediaAndStorageCaches = functionBody(
    projectPanel,
    "clear_dx_explorer_media_and_storage_caches",
  );
  const readBoundedMediaMetadataManifest = functionBody(
    metadata,
    "read_bounded_media_metadata_manifest",
  );
  const buildMediaMetadataProbePlan = functionBody(
    metadataProbe,
    "build_media_metadata_probe_plan",
  );
  const collectMediaMetadataManifest = functionBody(metadata, "collect_media_metadata_manifest");
  const collectMediaMetadataRecord = functionBody(metadata, "collect_media_metadata_record");
  const mediaDurationLabelFromRecord = functionBody(metadata, "media_duration_label_from_record");
  const resolveMetadataMediaPath = functionBody(metadata, "resolve_metadata_media_path");
  const confinedMediaPreviewPath = functionBody(metadata, "confined_media_preview_path");
  const normalizeMediaPreviewPath = functionBody(metadata, "normalize_media_preview_path");
  const generatedVideoFramePreview = functionBody(metadata, "generated_video_frame_preview");
  const resolveGeneratedVideoFramePath = functionBody(
    metadata,
    "resolve_generated_video_frame_path",
  );
  const videoPreviewFrame = functionBody(media, "video_preview_frame");
  const videoFramePreviewLabel = functionBody(media, "video_frame_preview_label");
  const videoFrameCandidateRank = functionBody(media, "video_frame_candidate_rank");

  assert.match(media, /mod metadata;/);
  assert.match(media, /mod generated_metadata;/);
  assert.match(media, /mod generated_video_frame;/);
  assert.match(media, /mod metadata_probe;/);
  assert.match(media, /pub\(crate\) use metadata::GeneratedMediaMetadataIndex;/);
  assert.match(
    media,
    /pub\(crate\) use generated_metadata::\{[\s\S]*build_generated_media_metadata_job_batch[\s\S]*collect_generated_media_metadata[\s\S]*\};/,
    "media preview must expose the bounded generated-metadata builder and collector to ProjectPanel",
  );
  assert.match(metadata, /pub\(super\) struct MediaMetadataIndex/);
  assert.match(metadata, /pub\(crate\) struct GeneratedMediaMetadataIndex/);
  assert.match(metadata, /pub\(crate\) struct GeneratedMediaMetadataRecord/);
  assert.match(metadata, /const MAX_GENERATED_MEDIA_METADATA_RECORDS: usize = 256;/);
  assert.match(
    generatedMetadata,
    /pub\(crate\) const GENERATED_MEDIA_METADATA_RUNNER_SCHEMA: &str =\s*"zed\.project_panel\.generated_media_metadata_runner";/,
    "generated metadata background work must have a source-owned runner schema",
  );
  assert.match(generatedMetadata, /const MAX_GENERATED_MEDIA_METADATA_JOBS: usize = 8;/);
  assert.match(
    generatedMetadata,
    /const MAX_GENERATED_MEDIA_METADATA_FILE_BYTES: u64 = 64 \* 1024 \* 1024;/,
    "automatic generated media metadata must stay bounded for busy low-end machines",
  );
  assert.match(
    generatedMetadata,
    /const MAX_GENERATED_MEDIA_METADATA_PATH_TEXT_BYTES: usize = 4096;/,
    "generated media metadata jobs must bound path text before cache keys or tool handoff",
  );
  assert.match(generatedMetadata, /pub\(crate\) struct GeneratedMediaMetadataJobBatch/);
  assert.match(generatedMetadata, /struct GeneratedMediaMetadataJob/);
  assert.match(
    projectPanelCargo,
    /rodio\.workspace = true/,
    "project panel generated audio duration extraction must use the existing workspace rodio dependency",
  );
  assert.match(
    projectPanelCargo,
    /paths\.workspace = true/,
    "project panel generated video frames must write only under Zed's managed cache/data paths",
  );
  assert.match(metadata, /pub\(super\) const MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES/);
  assert.match(
    metadata,
    /const MEDIA_METADATA_LIST_FIELDS: &\[&str\] = &\["items", "media", "entries", "assets", "files"\];/,
    "media metadata manifest list fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MEDIA_METADATA_PATH_FIELDS: &\[&str\] = &\[[\s\S]*"path",[\s\S]*"file",[\s\S]*"name",[\s\S]*"source",[\s\S]*"media_source",[\s\S]*"relative_path",[\s\S]*\];/,
    "media metadata record path fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MEDIA_METADATA_DURATION_LABEL_FIELDS: &\[&str\] =[\s\S]*&\["duration_label", "duration", "time", "length"\];/,
    "media metadata duration-label fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MEDIA_METADATA_DURATION_SECONDS_FIELDS: &\[&str\] = &\[[\s\S]*"duration_seconds",[\s\S]*"duration_secs",[\s\S]*"seconds",[\s\S]*"length_seconds",[\s\S]*\];/,
    "media metadata numeric duration fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MEDIA_METADATA_CENTER_FRAME_FIELDS: &\[&str\] = &\["center_frame", "middle_frame"\];/,
    "media metadata center-frame fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MEDIA_METADATA_PREVIEW_FRAME_FIELDS: &\[&str\] = &\[[\s\S]*"frame_path",[\s\S]*"thumbnail_path",[\s\S]*"poster_path",[\s\S]*"preview_path",[\s\S]*"frame",[\s\S]*"thumbnail",[\s\S]*"poster",[\s\S]*"preview",[\s\S]*\];/,
    "media metadata preview-frame fields must be a named source-owned contract",
  );
  assert.match(
    metadata,
    /const MAX_MEDIA_METADATA_DURATION_LABEL_CHARS: usize = 32;/,
    "media metadata duration labels must have a named bounded display cap",
  );
  assert.match(media, /const PROJECT_PANEL_MEDIA_SHELF_CARD_MIN_WIDTH: f32 = 96\.;/);
  assert.match(media, /const PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT: f32 = 96\.;/);
  assert.match(
    media,
    /metadata_probe_plan:\s*Option<metadata_probe::MediaMetadataProbePlan>/,
    "folder media previews must carry a bounded non-executing metadata probe plan",
  );
  assert.match(
    media,
    /pub\(crate\) fn build_folder_media_preview_with_generated_metadata[\s\S]*generated_metadata:\s*Option<&GeneratedMediaMetadataIndex>/,
    "folder media preview building must expose a generated-metadata merge seam",
  );
  assert.match(
    buildFolderMediaPreview,
    /build_folder_media_preview_with_generated_metadata\(parent_abs_path, children, None\)/,
    "default media preview building must preserve existing behavior when no generated cache exists",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /metadata::build_media_metadata_index[\s\S]*merge_generated_media_metadata/,
    "generated metadata must merge into the same index before preview items are populated",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /let metadata_probe_plan =\s*metadata_probe::build_media_metadata_probe_plan\(parent_abs_path, &items\);/,
    "probe planning must see generated metadata results before deciding what is still missing",
  );
  assert.match(
    buildGeneratedMediaMetadataJobBatch,
    /items\s*\{[\s\S]*jobs\.len\(\) >= MAX_GENERATED_MEDIA_METADATA_JOBS[\s\S]*MediaPreviewKind::Audio[\s\S]*duration_label\.is_none\(\)[\s\S]*item\.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES[\s\S]*let job = GeneratedMediaMetadataJob[\s\S]*job\.is_safe_managed_job\(\)[\s\S]*jobs\.push\(job\)/,
    "generated metadata jobs must be bounded, select missing audio durations that are small enough to inspect, and pass the final safe-job gate",
  );
  assert.doesNotMatch(
    buildGeneratedMediaMetadataJobBatch,
    /items\.iter\(\)\.take\(MAX_GENERATED_MEDIA_METADATA_JOBS \+ 1\)/,
    "generated metadata job selection must filter all already-bounded preview items before applying the job cap",
  );
  assert.match(
    buildGeneratedMediaMetadataJobBatch,
    /MediaPreviewKind::Video[\s\S]*video_frame_preview\.is_none\(\)\s*\|\|\s*item\.duration_label\.is_none\(\)[\s\S]*item\.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES/,
    "generated metadata jobs must select videos missing either representative frames or duration labels",
  );
  assert.match(
    isSafeGeneratedMediaMetadataJob,
    /self\.size > 0[\s\S]*self\.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES[\s\S]*self\.path\.is_absolute\(\)[\s\S]*!self\.path_text\.is_empty\(\)[\s\S]*self\.path_text\.len\(\) <= MAX_GENERATED_MEDIA_METADATA_PATH_TEXT_BYTES[\s\S]*!matches!\(self\.kind, MediaPreviewKind::Image\)/,
    "generated metadata jobs must fail closed for empty, oversized, relative, path-text-abusive, or image-only work",
  );
  assert.match(
    safeGeneratedMediaMetadataJobs,
    /\.take\(MAX_GENERATED_MEDIA_METADATA_JOBS\)[\s\S]*\.filter\(GeneratedMediaMetadataJob::is_safe_managed_job\)/,
    "generated metadata collection must reapply the job cap and safe-job gate at the final execution boundary",
  );
  assert.match(
    collectGeneratedMediaMetadata,
    /Vec::with_capacity\(batch\.jobs\.len\(\)\.min\(MAX_GENERATED_MEDIA_METADATA_JOBS\)\)[\s\S]*for job in batch\.safe_jobs\(\)/,
    "generated metadata collection must not trust the builder before opening files or delegating video extraction",
  );
  assert.match(
    collectGeneratedMediaMetadata,
    /batch\.safe_jobs\(\)[\s\S]*audio_duration_seconds_for_path\(&job\.path, &executor\)[\s\S]*\.await[\s\S]*GeneratedMediaMetadataRecord[\s\S]*duration_seconds: Some\(duration_seconds\)[\s\S]*GeneratedMediaMetadataIndex::from_records/,
    "generated metadata collection must turn successful timeout-backed audio duration reads into generated metadata records",
  );
  assert.match(
    collectGeneratedMediaMetadata,
    /generate_video_center_frame\(&job\.path, &job\.path_text, job\.size, &executor\)[\s\S]*\.await[\s\S]*GeneratedMediaMetadataRecord[\s\S]*duration_seconds: video_frame_metadata\.duration_seconds[\s\S]*center_frame_path: Some\(video_frame_metadata\.center_frame_path\)/,
    "generated metadata collection must turn successful background video extraction into center-frame and duration metadata records",
  );
  assert.match(
    generatedMetadata,
    /pub\(crate\) async fn collect_generated_media_metadata/,
    "generated metadata collection must be async so external video tooling never blocks the ProjectPanel entity task",
  );
  assert.match(
    audioDurationSecondsForPath,
    /executor\.spawn\(async move[\s\S]*audio_duration_seconds_for_path_sync\(&path\)[\s\S]*executor\.timer\(GENERATED_AUDIO_DURATION_PROBE_TIMEOUT\)[\s\S]*select\(duration_task, timeout\)\.await[\s\S]*Either::Left\(\(duration_seconds, _\)\) => duration_seconds[\s\S]*Either::Right\(\(_, _\)\) => None/,
    "audio duration extraction must use a timeout-backed background task rather than blocking the generated metadata collector indefinitely",
  );
  assert.match(
    generatedMetadata,
    /const GENERATED_AUDIO_DURATION_PROBE_TIMEOUT: Duration = Duration::from_secs\(3\);/,
    "automatic audio duration generation must have a named wall-clock timeout",
  );
  assert.match(
    generatedMetadata,
    /fn audio_duration_seconds_for_path_sync\(path: &Path\) -> Option<f64>[\s\S]*File::open\(path\)[\s\S]*BufReader::new\(file\)[\s\S]*Decoder::new\(reader\)[\s\S]*total_duration\(\)[\s\S]*as_secs_f64\(\)/,
    "audio duration extraction must keep the existing bounded Rust decoder metadata path rather than shelling out",
  );
  assert.doesNotMatch(
    generatedMetadata,
    /std::process|process::Command|Command::new|\.status\(|\.output\(|ffmpeg|ffprobe|fs::write|File::create|create_dir_all|remove_file|rename\(|copy\(/,
    "generated metadata job coordinator must delegate managed video extraction instead of embedding process or write logic",
  );
  assert.match(
    generatedVideoFrame,
    /const PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR: &str = "project-panel-media-frames";/,
    "generated video frames must live under a named app-owned cache directory",
  );
  assert.match(
    generatedVideoFrame,
    /const DX_FFMPEG_PATH_ENV: &str = "DX_FFMPEG_PATH";[\s\S]*const DX_FFPROBE_PATH_ENV: &str = "DX_FFPROBE_PATH";/,
    "generated video frame extraction must honor the same configurable ffmpeg/ffprobe environment contract as DX media tooling",
  );
  assert.match(
    generatedVideoFrame,
    /pub\(super\) struct GeneratedVideoFrameMetadata[\s\S]*center_frame_path: PathBuf[\s\S]*duration_seconds: Option<f64>/,
    "generated video frame extraction must return the generated center frame and any duration it already probed",
  );
  assert.match(
    generatedVideoFrame,
    /const GENERATED_VIDEO_DURATION_PROBE_TIMEOUT: Duration = Duration::from_secs\(3\);[\s\S]*const GENERATED_VIDEO_FRAME_EXTRACTION_TIMEOUT: Duration = Duration::from_secs\(8\);/,
    "generated video frame extraction must use named wall-clock timeouts for ffprobe and ffmpeg",
  );
  assert.match(
    generatedVideoFrame,
    /const MAX_GENERATED_VIDEO_SOURCE_BYTES: u64 = 64 \* 1024 \* 1024;[\s\S]*const MAX_GENERATED_VIDEO_PATH_TEXT_BYTES: usize = 4096;/,
    "generated video frame extraction must bound source file size and path text before invoking media tools",
  );
  assert.match(
    generateVideoCenterFrame,
    /is_safe_generated_video_source\(source_path, path_text, size\)[\s\S]*let modified_at = video_frame_cache_modified_at\(source_path\);[\s\S]*managed_video_frame_cache_path\(path_text, size, modified_at\)[\s\S]*probe_video_duration_seconds\(source_path, executor\)\.await[\s\S]*extract_video_center_frame\([\s\S]*source_path,[\s\S]*&temporary_output_path,[\s\S]*center_seconds,[\s\S]*executor,[\s\S]*\)[\s\S]*\.await/,
    "video center-frame generation must derive a managed cache path, probe duration, then extract the center timestamp",
  );
  assertBefore({
    body: generateVideoCenterFrame,
    before: /is_safe_generated_video_source\(source_path, path_text, size\)/,
    after: /video_frame_cache_modified_at\(source_path\)/,
    message: "generated video frame extraction must validate source bounds before metadata or cache work",
  });
  assert.match(
    isSafeGeneratedVideoSource,
    /size > 0[\s\S]*size <= MAX_GENERATED_VIDEO_SOURCE_BYTES[\s\S]*source_path\.is_absolute\(\)[\s\S]*!path_text\.is_empty\(\)[\s\S]*path_text\.len\(\) <= MAX_GENERATED_VIDEO_PATH_TEXT_BYTES/,
    "generated video frame extraction must fail closed for empty, oversized, relative, or path-text-abusive sources",
  );
  assertBefore({
    body: generateVideoCenterFrame,
    before: /video_frame_cache_modified_at\(source_path\)/,
    after: /managed_video_frame_cache_path\(path_text, size, modified_at\)/,
    message: "generated video frame cache keys must include source freshness before cache lookup",
  });
  assert.match(
    generateVideoCenterFrame,
    /let duration_seconds = probe_video_duration_seconds\(source_path, executor\)\.await;[\s\S]*if output_path\.is_file\(\) \{[\s\S]*duration_seconds,[\s\S]*\}[\s\S]*let duration_seconds = duration_seconds\?;/,
    "cached generated video frames must still try to carry bounded duration evidence for hover metadata",
  );
  assert.match(
    managedVideoFrameCachePath,
    /modified_at: u64[\s\S]*paths::temp_dir\(\)[\s\S]*PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR[\s\S]*stable_video_frame_cache_key\(path_text, size, modified_at\)/,
    "video frame cache paths must be app-owned and stable from source identity rather than written beside user files",
  );
  assert.match(
    stableVideoFrameCacheKey,
    /modified_at: u64[\s\S]*let size_bytes = size\.to_le_bytes\(\);[\s\S]*let modified_at_bytes = modified_at\.to_le_bytes\(\);[\s\S]*path_text[\s\S]*\.as_bytes\(\)[\s\S]*\.chain\(size_bytes\.iter\(\)\)[\s\S]*\.chain\(modified_at_bytes\.iter\(\)\)/,
    "video frame cache keys must include modified-time freshness so same-path same-size replacements do not reuse stale frames",
  );
  assert.match(
    generatedVideoFrame,
    /fn video_frame_cache_modified_at\(source_path: &Path\) -> u64 \{[\s\S]*fs::metadata\(source_path\)[\s\S]*\.modified\(\)[\s\S]*duration_since\(UNIX_EPOCH\)[\s\S]*unwrap_or_default\(\)[\s\S]*\}/,
    "video frame cache freshness must come from a bounded source metadata fingerprint",
  );
  assert.match(
    probeVideoDurationSeconds,
    /run_media_command_output\([\s\S]*ffprobe_binary\(\)[\s\S]*"-show_entries"[\s\S]*"format=duration"[\s\S]*source_path[\s\S]*GENERATED_VIDEO_DURATION_PROBE_TIMEOUT/,
    "video duration probing must use direct ffprobe arguments for center-frame timestamps",
  );
  assert.match(
    extractVideoCenterFrame,
    /run_media_command_output\([\s\S]*ffmpeg_binary\(\)[\s\S]*"-nostdin"[\s\S]*"-ss"[\s\S]*format_video_timestamp\(center_seconds\)[\s\S]*"-frames:v"[\s\S]*"1"[\s\S]*"-vf"[\s\S]*"scale=480:-2"[\s\S]*GENERATED_VIDEO_FRAME_EXTRACTION_TIMEOUT/,
    "video frame extraction must use direct ffmpeg arguments with no shell and a bounded preview scale",
  );
  assert.match(
    runMediaCommandOutput,
    /executor: &BackgroundExecutor[\s\S]*timeout:\s*Duration[\s\S]*util::command::new_command\(program\)[\s\S]*command\.stdin\(Stdio::null\(\)\)[\s\S]*command\.kill_on_drop\(true\)[\s\S]*let output = command\.output\(\);[\s\S]*executor\.timer\(timeout\)[\s\S]*select\(output, timeout\)\.await[\s\S]*Either::Left[\s\S]*output\.ok\(\)[\s\S]*Either::Right[\s\S]*None/,
    "media commands must use Zed's Windows-safe command wrapper, a wall-clock timeout, and kill-on-drop semantics",
  );
  assert.match(
    mediaBinaryIsShell,
    /"cmd"[\s\S]*"powershell"[\s\S]*"pwsh"[\s\S]*"sh"[\s\S]*"bash"[\s\S]*"zsh"/,
    "configured media tool binaries must reject shell executables before command execution",
  );
  assert.doesNotMatch(
    generatedVideoFrame,
    /std::process|Command::new|new_std_command|\.status\(|\.spawn\(|fs::write|File::create|copy\(/,
    "generated video frame extraction must avoid shells, std process spawning, and user-project writes",
  );
  assert.match(
    media,
    /let metadata_probe_plan =\s*metadata_probe::build_media_metadata_probe_plan\(parent_abs_path, &items\);[\s\S]*FolderMediaPreview[\s\S]*metadata_probe_plan/,
    "metadata probe planning must be built from cached preview items outside per-row rendering",
  );
  assert.match(
    metadataProbe,
    /pub\(super\) const PROJECT_PANEL_MEDIA_METADATA_PROBE_SCHEMA: &str =\s*"zed\.project_panel\.media_metadata_probe";/,
    "project-panel media metadata probe plans need a professional source-owned schema name",
  );
  assert.match(
    metadataProbe,
    /const MAX_PROJECT_PANEL_MEDIA_METADATA_PROBE_ACTIONS: usize = 8;/,
    "media metadata probe planning must be bounded",
  );
  assert.match(
    metadataProbe,
    /tool_execution_allowed: false/,
    "metadata probe plans must not imply project-panel render-path tool execution is allowed",
  );
  assert.match(
    metadataProbe,
    /writes_user_project_files: false/,
    "metadata probe plans must keep generated metadata out of user project files by default",
  );
  assert.match(
    buildMediaMetadataProbePlan,
    /MediaPreviewKind::Video[\s\S]*needs_center_frame[\s\S]*ffmpeg_center_frame_argument_template/,
    "video probe planning must identify missing center-frame evidence separately from duration",
  );
  assert.match(
    buildMediaMetadataProbePlan,
    /MediaPreviewKind::Audio[\s\S]*needs_duration[\s\S]*ffprobe_argument_vector/,
    "audio probe planning must identify missing duration evidence",
  );
  assert.doesNotMatch(
    metadataProbe,
    /std::process|Command::new|\.status\(|\.output\(|\.spawn\(/,
    "project-panel metadata probe planning must not execute ffmpeg, ffprobe, shells, or child processes",
  );

  assertBefore({
    body: buildFolderMediaPreviewWithGeneratedMetadata,
    before: /children\.take\(MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN \+ 1\)/,
    after: /scanned_cap_hit/,
    message: "media child scans must be capped before classification work",
  });
  assertBefore({
    body: buildFolderMediaPreviewWithGeneratedMetadata,
    before: /items\.sort_by\(media_preview_item_sort_order\)/,
    after: /select_balanced_media_preview_items\(items\)/,
    message: "media preview candidates must be ordered before the balanced bounded render set is selected",
  });
  assertBefore({
    body: buildFolderMediaPreviewWithGeneratedMetadata,
    before: /select_balanced_media_preview_items\(items\)/,
    after: /for item in &mut items/,
    message: "media preview items must be capped before render data receives video frame paths",
  });
  assert.match(
    selectBalancedMediaPreviewItems,
    /items\.len\(\) <= MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS[\s\S]*MediaPreviewKind::Image[\s\S]*MediaPreviewKind::Video[\s\S]*MediaPreviewKind::Audio[\s\S]*push_media_preview_item_if_missing[\s\S]*selected\.len\(\) >= MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS[\s\S]*selected\.sort_by\(media_preview_item_sort_order\)/,
    "bounded media selection must preserve image, video, and audio representation before filling remaining card slots",
  );
  assert.match(
    pushMediaPreviewItemIfMissing,
    /selected_item\.entry_id == item\.entry_id[\s\S]*selected\.push\(item\.clone\(\)\)/,
    "balanced media card selection must not duplicate the same project entry",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /size:\s*child\.size/,
    "media preview items must carry snapshot file sizes for hover details",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /entry_id:\s*child\.id/,
    "media preview items must carry project entry ids for card selection/opening",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /let mut media_metadata = metadata::build_media_metadata_index\(parent_abs_path, &child_entries\);/,
    "media preview items must derive optional duration/frame metadata from the bounded child snapshot",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /duration_label:\s*media_metadata\.duration_label_for_path\(&absolute_path\)/,
    "media preview items must carry manifest duration labels when present",
  );
  assert.match(
    buildFolderMediaPreviewWithGeneratedMetadata,
    /media_metadata[\s\S]*\.video_frame_for_path\(&item\.absolute_path\)[\s\S]*\.or_else\(\|\| video_preview_frame/,
    "video media cards must prefer manifest-declared center frames before heuristic sidecar frames",
  );
  assert.doesNotMatch(
    media,
    /fn render_folder_media_preview/,
    "folder media previews must not render as compact row chips",
  );
  assert.match(
    renderFolderMediaGallery,
    /\.grid\(\)[\s\S]*\.grid_cols\(PROJECT_PANEL_MEDIA_GALLERY_COLUMNS\)/,
    "folder media gallery must use a bounded three-column grid",
  );
  assert.match(
    renderFolderMediaShelf,
    /\.border_b_1\(\)[\s\S]*\.grid\(\)[\s\S]*\.grid_cols\(PROJECT_PANEL_MEDIA_SHELF_COLUMNS\)[\s\S]*\.children\(shelf_cards\)/,
    "folder media shelf must render as a top four-column media grid",
  );
  assert.match(
    renderFolderMediaShelf,
    /ListHeader::new\("Media"\)[\s\S]*\.end_slot\(header_controls\)/,
    "folder media shelf header must use shared GPUI chrome and keep compact header controls in the end slot",
  );
  assert.doesNotMatch(
    renderFolderMediaShelf,
    /format!\("\{visible_media_count\} shown \/ \{summary\}"\)|format!\("\{visible_count\} shown \/ \{summary\}"\)/,
    "folder media shelf header must avoid sticky top-right count text",
  );
  assertBefore({
    body: renderFolderMediaGallery,
    before: /take\(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS\)/,
    after: /render_media_gallery_card/,
    message: "folder media gallery must render only bounded preview items",
  });
  assert.match(
    renderFolderMediaGallery,
    /render_media_gallery_card\("project-panel-media-gallery-card", item, cx\)/,
    "folder media gallery cards must have a distinct element id prefix",
  );
  assert.match(
    renderFolderMediaShelf,
    /render_media_shelf_card\([\s\S]*item[\s\S]*worktree_id[\s\S]*selected_entry_id == Some\(item\.entry_id\)[\s\S]*cx/,
    "folder media shelf cards must have a distinct element id prefix",
  );
  assert.match(
    renderFolderMediaShelf,
    /render_media_shelf_card\([\s\S]*item,[\s\S]*worktree_id,[\s\S]*selected_entry_id == Some\(item\.entry_id\),[\s\S]*focus_handle\.clone\(\),[\s\S]*cx/,
    "folder media shelf cards must receive the Project Panel focus handle for keyboard focus tracking",
  );
  assert.match(
    renderMediaShelfCard,
    /media_shelf_card_container\([\s\S]*focus_handle[\s\S]*cx[\s\S]*SelectedEntry[\s\S]*worktree_id[\s\S]*entry_id[\s\S]*PreviewTabsSettings::get_global\(cx\)[\s\S]*panel\.open_entry/,
    "media shelf cards must select and open real project entries",
  );
  assert.match(
    renderMediaShelfCard,
    /\.on_right_click\([\s\S]*panel\.deploy_context_menu\(event\.position\(\), entry_id, window, cx\)/,
    "media shelf cards must use ButtonLike right-click handling and the real project-panel context menu",
  );
  assert.match(
    media,
    /use ui::\{[\s\S]*ButtonLike[\s\S]*TintColor/,
    "media shelf cards should use shared ButtonLike selection styling instead of custom div-only state",
  );
  assert.match(
    mediaShelfCardContainer,
    /\) -> ButtonLike/,
    "media shelf card containers should return ButtonLike instead of raw clickable Divs",
  );
  assert.match(
    mediaShelfCardContainer,
    /ButtonLike::new\(SharedString::from\(format!\([\s\S]*"\{id_prefix\}-\{:\?\}-\{:\?\}"[\s\S]*item\.kind,[\s\S]*item\.entry_id[\s\S]*\.full_width\(\)[\s\S]*\.height\(px\(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT\)\.into\(\)\)[\s\S]*\.style\(ButtonStyle::OutlinedGhost\)[\s\S]*\.selected_style\(ButtonStyle::Tinted\(TintColor::Accent\)\)[\s\S]*\.toggle_state\(is_selected\)[\s\S]*\.tab_index\(0(?:_isize)?\)[\s\S]*\.track_focus\(&focus_handle\)/,
    "media shelf cards must use a focusable ButtonLike with a visible outer focus border and Zed selected-state styling",
  );
  assert.match(
    mediaShelfCardContainer,
    /\.child\([\s\S]*\.min_w\(px\(PROJECT_PANEL_MEDIA_SHELF_CARD_MIN_WIDTH\)\)[\s\S]*\.h\(px\(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT\)\)[\s\S]*\.w_full\(\)[\s\S]*\.v_flex\(\)[\s\S]*\.p_0\(\)[\s\S]*\.overflow_hidden\(\)/,
    "media shelf cards must use fixed-height icon-panel-like tiles instead of full-width list rows",
  );
  assert.match(
    mediaShelfCardContainer,
    /format!\(\s*"\{id_prefix\}-\{:\?\}-\{:\?\}"[\s\S]*item\.kind, item\.entry_id/,
    "media shelf cards must derive element ids from real project entry identity rather than filename-only hashes",
  );
  assert.match(
    renderMediaShelfCardBody,
    /MediaPreviewKind::Image[\s\S]*w_full\(\)[\s\S]*flex_1\(\)[\s\S]*img\(item\.absolute_path\.clone\(\)\)[\s\S]*object_fit\(ObjectFit::Cover\)[\s\S]*with_fallback\(\|\| media_card_image_fallback\(MediaPreviewKind::Image\)\)[\s\S]*media_shelf_name_overlay\(&item\.name, cx\)/,
    "shelf image cards must fill the tile from the real image path and overlay a readable filename",
  );
  assert.match(
    renderMediaShelfCardBody,
    /MediaPreviewKind::Video[\s\S]*w_full\(\)[\s\S]*flex_1\(\)[\s\S]*item\.video_frame_preview\.as_ref\(\)[\s\S]*img\(preview\.path\.clone\(\)\)[\s\S]*with_fallback\(\|\| media_card_image_fallback\(MediaPreviewKind::Video\)\)[\s\S]*IconName::PlayOutlined[\s\S]*media_shelf_name_overlay\(&item\.name, cx\)/,
    "shelf video cards must fill the tile with the center frame, play affordance, fallback, and readable filename overlay",
  );
  assert.match(
    renderMediaShelfCardBody,
    /MediaPreviewKind::Audio[\s\S]*w_full\(\)[\s\S]*flex_1\(\)[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*items_center\(\)[\s\S]*justify_center\(\)[\s\S]*audio_media_label\(&item\.name, IconSize::Small, cx\)/,
    "shelf audio cards must use full-height deterministic gradient rectangles with a readable centered audio label",
  );
  assert.doesNotMatch(
    media,
    /fn render_media_preview_card/,
    "media previews must avoid row-level image/video card rendering in the virtualized tree",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Image[\s\S]*img\(item\.absolute_path\.clone\(\)\)[\s\S]*object_fit\(ObjectFit::Cover\)[\s\S]*with_fallback\(\|\| media_card_image_fallback\(MediaPreviewKind::Image\)\)/,
    "gallery image cards must render direct visual previews with a nonblank fallback",
  );
  assert.match(
    mediaGalleryCardContainer,
    /format!\(\s*"\{id_prefix\}-\{:\?\}-\{:\?\}"[\s\S]*item\.kind, item\.entry_id/,
    "media gallery cards must derive element ids from real project entry identity rather than filename-only hashes",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Video[\s\S]*item\.video_frame_preview\.as_ref\(\)[\s\S]*img\(preview\.path\.clone\(\)\)[\s\S]*with_fallback\(\|\| media_card_image_fallback\(MediaPreviewKind::Video\)\)[\s\S]*IconName::PlayOutlined/,
    "gallery video cards must use available representative frame images and keep a play affordance plus fallback",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Audio[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*audio_media_label\(&item\.name, IconSize::Small, cx\)/,
    "gallery audio cards must use deterministic color rectangles with readable centered audio labels",
  );
  assert.match(
    audioMediaLabel,
    /bg\(colors\.editor_background\.opacity\(0\.78\)\)[\s\S]*Icon::new\(IconName::AudioOn\)[\s\S]*\.color\(Color::Accent\)[\s\S]*Label::new\(name\.to_string\(\)\)[\s\S]*\.color\(Color::Default\)[\s\S]*\.truncate\(\)/,
    "audio labels must place text and glyphs on a high-contrast editor-background plate over the gradient",
  );
  assert.match(
    mediaPreviewCardTooltipMeta,
    /let size_label = media_size_label\(item\.size\);[\s\S]*Unknown duration[\s\S]*\{size_label\}/,
    "media hover details must include snapshot size and manifest duration with an honest unavailable state",
  );
  assert.match(
    mediaPreviewCardTooltipMeta,
    /No thumbnail/,
    "video hover details must not imply a background frame job exists when no frame preview is available",
  );
  assert.doesNotMatch(
    mediaPreviewCardTooltipMeta,
    /Frame pending/,
    "missing video frame previews must use non-promissory unavailable wording",
  );
  assert.match(
    audioGradientBackground,
    /audio_gradient_colors\(name\)/,
    "audio card gradients must be deterministic from the audio filename",
  );
  assert.match(
    mediaCardImageFallback,
    /MediaPreviewKind::Image => IconName::Image[\s\S]*MediaPreviewKind::Video => IconName::PlayOutlined[\s\S]*size_full\(\)[\s\S]*items_center\(\)[\s\S]*justify_center\(\)/,
    "media image/frame fallbacks must render nonblank centered media icons",
  );
  assert.match(
    videoPreviewFrame,
    /video_frame_candidate_rank\(&video_stem, stem\)[\s\S]*\.min_by_key[\s\S]*VideoFramePreview/,
    "video preview frame matching should rank sidecar images instead of accepting the first candidate",
  );
  assert.match(
    videoFramePreviewLabel,
    /VideoFramePreviewKind::Center[\s\S]*"Thumbnail"[\s\S]*VideoFramePreviewKind::Preview[\s\S]*"Thumbnail"/,
    "video hover details should use concise thumbnail language",
  );
  assert.match(
    videoFrameCandidateRank,
    /CENTER_FRAME_HINTS[\s\S]*PREVIEW_FRAME_HINTS/,
    "video sidecar matching must prefer center or middle frame hints before poster/thumb previews",
  );
  assert.match(
    buildMediaMetadataIndex,
    /MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES[\s\S]*MEDIA_METADATA_MANIFEST_NAMES[\s\S]*read_bounded_media_metadata_manifest/,
    "media metadata manifests must be size-bounded and opt-in by known file name before parsing",
  );
  assert.match(
    mergeGeneratedMediaMetadata,
    /generated[\s\S]*\.records\(\)[\s\S]*\.take\(MAX_GENERATED_MEDIA_METADATA_RECORDS\)[\s\S]*duration_label[\s\S]*duration_seconds[\s\S]*center_frame_path[\s\S]*preview_frame_path/,
    "generated metadata overlays must be bounded and carry duration plus center/preview frame fields",
  );
  assert.doesNotMatch(
    metadata,
    /fs::write|File::create|create_dir_all|remove_file|rename\(|copy\(|std::process|Command::new|\.status\(|\.output\(|\.spawn\(/,
    "generated media metadata merge code must not write files or execute tools",
  );
  assert.match(
    projectPanel,
    /generated_media_metadata:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), media_preview::GeneratedMediaMetadataIndex>>/,
    "project panel must own an in-memory generated media metadata cache seam",
  );
  assert.match(
    projectPanel,
    /media_preview_cache_generation:\s*Cell<u64>/,
    "project panel must carry a generation token for async media cache writes",
  );
  assert.match(
    projectPanel,
    /media_metadata_generation_tasks:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), Task<\(\)>>/,
    "project panel must track in-flight generated metadata work per folder",
  );
  assert.match(
    clearDxExplorerMediaCaches,
    /bump_media_preview_cache_generation\(\)[\s\S]*generated_media_metadata\.borrow_mut\(\)\.clear\(\)[\s\S]*media_metadata_generation_tasks\.borrow_mut\(\)\.clear\(\)[\s\S]*folder_media_previews\.borrow_mut\(\)\.clear\(\)/,
    "media cache invalidation must bump generation before clearing generated metadata, tasks, and folder previews",
  );
  assert.match(
    clearDxExplorerMediaAndStorageCaches,
    /folder_storage_summaries\.borrow_mut\(\)\.clear\(\)[\s\S]*clear_dx_explorer_media_caches\(\)/,
    "storage invalidation must also invalidate dependent media preview caches",
  );
  assert.match(
    projectPanel,
    /generated_media_metadata[\s\S]*retain\(\|\(worktree_id, _\), _\| \*worktree_id != \*id\)[\s\S]*bump_media_preview_cache_generation\(\)/,
    "generated media metadata cache entries must be retained and async media writes invalidated when a worktree is removed",
  );
  assert.match(
    projectPanel,
    /clear_dx_explorer_media_and_storage_caches\(\)[\s\S]*update_visible_entries/,
    "generated media metadata cache must clear through the generation-bumping helper before previews are rebuilt after worktree/settings changes",
  );
  assert.match(
    projectPanel,
    /build_folder_media_preview_with_generated_metadata\([\s\S]*generated_metadata/,
    "project panel folder preview cache must pass generated metadata into the media preview builder",
  );
  assert.match(
    activeFolderMediaPreview,
    /self\.ensure_generated_media_metadata\([\s\S]*active_media_folder\.worktree_id[\s\S]*active_media_folder\.entry_id[\s\S]*&preview[\s\S]*cx[\s\S]*\)/,
    "active media shelf rendering must schedule missing generated metadata without doing that work in the render body",
  );
  assert.match(ensureGeneratedMediaMetadata, /build_generated_media_metadata_job_batch\(&preview\.items\)/);
  assertBefore({
    body: ensureGeneratedMediaMetadata,
    before: /self\.project\.read\(cx\)\.is_remote\(\)/,
    after: /build_generated_media_metadata_job_batch\(&preview\.items\)/,
    message: "automatic generated media metadata must skip remote projects before local path decoding/tooling",
  });
  assert.match(ensureGeneratedMediaMetadata, /media_metadata_generation_tasks[\s\S]*contains_key\(&cache_key\)/);
  assert.match(
    ensureGeneratedMediaMetadata,
    /let media_preview_cache_generation = self\.media_preview_cache_generation\.get\(\);/,
    "generated metadata tasks must capture the media cache generation before background work starts",
  );
  assert.match(ensureGeneratedMediaMetadata, /cx\.spawn\(async move \|this, cx\|/);
  assert.match(
    ensureGeneratedMediaMetadata,
    /this\.update_in\(cx,\s*\|this,\s*window,\s*cx\|/,
    "generated metadata completion must use update_in so it can schedule visible-entry rebuilds",
  );
  assert.match(
    ensureGeneratedMediaMetadata,
    /let executor = cx\.background_executor\(\)\.clone\(\);[\s\S]*background_spawn\([\s\S]*async move \{[\s\S]*collect_generated_media_metadata\(batch, executor\)\.await/,
    "generated metadata work must run through a background task",
  );
  assert.match(
    ensureGeneratedMediaMetadata,
    /this\.media_preview_cache_generation\.get\(\) != media_preview_cache_generation[\s\S]*return;[\s\S]*let generated_metadata_has_records = !generated_metadata\.is_empty\(\);[\s\S]*if !generated_metadata_has_records \{[\s\S]*return;[\s\S]*\}[\s\S]*generated_media_metadata[\s\S]*borrow_mut\(\)[\s\S]*\.insert\(cache_key, generated_metadata\)/,
    "generated metadata results must update the ProjectPanel generated metadata cache only when the cache generation is current and generated records exist",
  );
  assert.doesNotMatch(
    ensureGeneratedMediaMetadata,
    /let generated_metadata_has_records = !generated_metadata\.is_empty\(\);[\s\S]*generated_media_metadata[\s\S]*\.insert\(cache_key, generated_metadata\);[\s\S]*if generated_metadata_has_records/,
    "empty generated metadata results must not be cached as terminal success because transient decoder/tool failures need a future retry",
  );
  assert.match(
    ensureGeneratedMediaMetadata,
    /folder_media_previews[\s\S]*borrow_mut\(\)[\s\S]*\.remove\(&cache_key\)/,
    "generated metadata results must invalidate the stale rendered preview cache",
  );
  assert.match(
    ensureGeneratedMediaMetadata,
    /this\.update_visible_entries\(None,\s*false,\s*false,\s*window,\s*cx\)/,
    "generated metadata cache invalidation must schedule visible-entry refresh without changing selection",
  );
  assertBefore({
    body: ensureGeneratedMediaMetadata,
    before: /collect_generated_media_metadata\(batch, executor\)/,
    after: /\.insert\(cache_key, generated_metadata\)/,
    message: "generated metadata must be collected before the cache is updated",
  });
  assertBefore({
    body: ensureGeneratedMediaMetadata,
    before: /\.insert\(cache_key, generated_metadata\)/,
    after: /folder_media_previews[\s\S]*\.remove\(&cache_key\)/,
    message: "preview cache invalidation must happen after generated metadata is stored",
  });
  assertBefore({
    body: ensureGeneratedMediaMetadata,
    before: /folder_media_previews[\s\S]*\.remove\(&cache_key\)/,
    after: /this\.update_visible_entries\(None,\s*false,\s*false,\s*window,\s*cx\)/,
    message: "visible entries must refresh after the stale media preview cache is invalidated",
  });
  assert.doesNotMatch(
    `${activeFolderMediaPreview}\n${renderFolderMediaShelf}\n${renderMediaShelfCard}\n${renderMediaShelfCardBody}\n${mediaGalleryCardContainer}`,
    /File::open|Decoder::new|std::process|Command::new|ffmpeg|ffprobe|fs::write|File::create/,
    "project-panel render paths must not perform generated metadata IO, decoding, tool execution, or writes",
  );
  assert.match(
    collectMediaMetadataManifest,
    /for key in MEDIA_METADATA_LIST_FIELDS[\s\S]*collect_media_metadata_record/,
    "media metadata manifest collection must use the named list-field contract",
  );
  assert.match(
    readBoundedMediaMetadataManifest,
    /fs::File::open[\s\S]*take\(MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES \+ 1\)[\s\S]*String::from_utf8/,
    "media metadata manifest reads must use a sentinel-byte bound before UTF-8 and JSON parsing",
  );
  assert.match(
    collectMediaMetadataRecord,
    /first_string_field\(\s*object,\s*MEDIA_METADATA_PATH_FIELDS\s*\)[\s\S]*media_duration_label_from_record[\s\S]*MEDIA_METADATA_CENTER_FRAME_FIELDS[\s\S]*MEDIA_METADATA_PREVIEW_FRAME_FIELDS/,
    "media metadata records must route path, duration, and frame aliases through named contracts",
  );
  assert.match(
    resolveMetadataMediaPath,
    /confined_media_preview_path\(parent_abs_path, &candidate\)/,
    "manifest-provided media paths must be confined before becoming renderable image paths",
  );
  assert.doesNotMatch(
    resolveMetadataMediaPath,
    /candidate\.is_absolute\(\)\s*\{[\s\S]*candidate[\s\S]*\}/,
    "manifest-provided absolute paths must not be accepted as-is",
  );
  assert.match(
    confinedMediaPreviewPath,
    /normalize_media_preview_path\(parent_abs_path\)[\s\S]*normalize_media_preview_path\(&parent_abs_path\.join\(candidate\)\)[\s\S]*candidate\.starts_with\(&parent_abs_path\)/,
    "metadata frame paths must stay inside the folder that owns the manifest",
  );
  assert.match(
    normalizeMediaPreviewPath,
    /Component::ParentDir[\s\S]*if !normalized\.pop\(\) \{[\s\S]*return None;/,
    "metadata path normalization must reject parent traversal past the root",
  );
  assert.match(
    generatedVideoFramePreview,
    /resolve_generated_video_frame_path\(path\)/,
    "generated video frame metadata must use the managed-cache provenance check",
  );
  assert.match(
    resolveGeneratedVideoFramePath,
    /generated_video_frame_cache_root\(\)\?[\s\S]*path\.starts_with\(&cache_root\)/,
    "generated video frame paths must stay under the Project Panel managed frame cache",
  );
  assert.match(
    metadata,
    /paths::temp_dir\(\)[\s\S]*PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR/,
    "generated frame cache provenance must share the named managed frame directory",
  );
  assert.match(
    mediaDurationLabelFromRecord,
    /MEDIA_METADATA_DURATION_LABEL_FIELDS[\s\S]*MEDIA_METADATA_DURATION_SECONDS_FIELDS[\s\S]*format_media_duration_seconds/,
    "duration metadata must accept explicit labels or numeric seconds",
  );
  assert.doesNotMatch(
    `${media}\n${metadata}`,
    /path\.is_file\(\)|std::fs::metadata|fs::metadata|ffmpeg|ffprobe/,
    "media preview classification must stay snapshot-derived and avoid UI-path filesystem metadata or decoder probes",
  );
});

test("project panel marquee drag selection is real and bounded", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const startMarqueeSelection = functionBody(source, "start_marquee_selection");
  const updateMarqueeSelection = functionBody(source, "update_marquee_selection");
  const finishMarqueeSelection = functionBody(source, "finish_marquee_selection");
  const applyMarqueeSelection = functionBody(source, "apply_marquee_selection");
  const updateMarqueeAutoscroll = functionBody(source, "update_marquee_autoscroll");
  const stopMarqueeAutoscroll = functionBody(source, "stop_marquee_autoscroll");
  const marqueeEntryRange = functionBody(source, "project_panel_marquee_entry_range");
  const marqueeAutoscrollAdjustment = functionBody(source, "project_panel_marquee_autoscroll_adjustment");
  const marqueeDecorationCompute = functionBody(source, "compute");
  const renderProjectPanel = functionBody(source, "render");

  assert.match(source, /const MAX_PROJECT_PANEL_MARQUEE_SELECTION_ENTRIES: usize = 20_000;/);
  assert.match(source, /const PROJECT_PANEL_MARQUEE_MIN_DRAG_DISTANCE: Pixels = px\(4\.\);/);
  assert.match(source, /const PROJECT_PANEL_MARQUEE_AUTOSCROLL_TICK: Duration = Duration::from_millis\(16\);/);
  assert.match(source, /const PROJECT_PANEL_MARQUEE_AUTOSCROLL_FAST_EDGE: f32 = 0\.05;/);
  assert.match(source, /const PROJECT_PANEL_MARQUEE_AUTOSCROLL_SLOW_EDGE: f32 = 0\.15;/);
  assert.match(source, /struct ProjectPanelMarqueeSelection/);
  assert.match(source, /struct ProjectPanelMarqueeLayout/);
  assert.match(source, /struct ProjectPanelMarqueeDecoration/);
  assert.match(source, /marquee_selection:\s*Option<ProjectPanelMarqueeSelection>/);
  assert.match(
    source,
    /marquee_layout:\s*Rc<RefCell<Option<ProjectPanelMarqueeLayout>>>/,
    "marquee selection must share list geometry without mutating the entity from decoration compute",
  );
  assert.match(source, /impl UniformListDecoration for ProjectPanelMarqueeDecoration/);

  assert.match(
    startMarqueeSelection,
    /event\.button != MouseButton::Left[\s\S]*event\.click_count != 1[\s\S]*state\.edit_state\.is_some/,
    "marquee selection must ignore non-left, multi-click, and edit-state starts",
  );
  assert.match(
    startMarqueeSelection,
    /base_selection: self\.selection[\s\S]*base_marked_entries: self\.marked_entries\.clone\(\)[\s\S]*additive: event\.modifiers\.secondary\(\)/,
    "marquee selection must preserve base selection for secondary-modifier additive drags",
  );

  assert.match(
    updateMarqueeSelection,
    /event\.dragging\(\)[\s\S]*PROJECT_PANEL_MARQUEE_MIN_DRAG_DISTANCE[\s\S]*marquee\.active = true[\s\S]*self\.update_marquee_autoscroll\(window, cx\)[\s\S]*self\.apply_marquee_selection\(cx\)/,
    "marquee selection must wait for a drag threshold before applying real selection",
  );
  assert.match(
    updateMarqueeSelection,
    /!event\.dragging\(\)[\s\S]*self\.marquee_selection = None[\s\S]*self\.stop_marquee_autoscroll\(\)/,
    "marquee selection must stop edge autoscroll when dragging stops",
  );
  assert.match(
    finishMarqueeSelection,
    /self\.marquee_selection\.take\(\)[\s\S]*let was_active = marquee\.active[\s\S]*self\.mouse_down = false[\s\S]*self\.stop_marquee_autoscroll\(\)[\s\S]*cx\.notify\(\)[\s\S]*cx\.stop_propagation\(\)/,
    "finishing an active marquee drag must clear state and suppress the trailing click",
  );
  assert.match(
    updateMarqueeAutoscroll,
    /self[\s\S]*\.marquee_selection[\s\S]*\.as_ref\(\)[\s\S]*self\.marquee_layout\.borrow\(\)\.clone\(\)[\s\S]*project_panel_marquee_autoscroll_adjustment\(marquee\.current, &layout\)[\s\S]*self\.hover_scroll_task = Some\(cx\.spawn_in\(window/,
    "marquee edge autoscroll must use the recorded uniform-list layout and existing hover scroll task",
  );
  assert.match(
    updateMarqueeAutoscroll,
    /handle\.base_handle\.set_offset\(offset \+ adjustment\)[\s\S]*this\.apply_marquee_selection\(cx\)/,
    "marquee edge autoscroll must update real scroll offset and reapply selection as rows move",
  );
  assert.match(
    stopMarqueeAutoscroll,
    /self\.hover_scroll_task\.take\(\)/,
    "marquee edge autoscroll must stop through the shared hover scroll task",
  );

  assert.match(
    applyMarqueeSelection,
    /project_panel_marquee_entry_range\(&marquee, &layout\)/,
    "marquee selection must use the recorded uniform-list geometry",
  );
  assert.match(
    applyMarqueeSelection,
    /base_marked_entries[\s\S]*take\(MAX_PROJECT_PANEL_MARQUEE_SELECTION_ENTRIES\)[\s\S]*self\.entry_at_index\(index\)[\s\S]*SelectedEntry[\s\S]*marked_entries = selected_entries/,
    "marquee selection must update real project-panel marked entries, not only paint an overlay",
  );
  assert.match(
    applyMarqueeSelection,
    /base_marked_entries[\s\S]*take\(MAX_PROJECT_PANEL_MARQUEE_SELECTION_ENTRIES\)[\s\S]*selected_entries\.len\(\) >= MAX_PROJECT_PANEL_MARQUEE_SELECTION_ENTRIES[\s\S]*project_panel_cap_hit\([\s\S]*"marquee-selection-range"/,
    "marquee selection must cap row materialization before it can fan out to file actions",
  );

  assert.match(
    marqueeEntryRange,
    /debug_assert!\(layout\.visible_range\.end <= layout\.item_count\)[\s\S]*let marquee_bounds = project_panel_marquee_bounds\(selection\);[\s\S]*let clipped_bounds = marquee_bounds\.intersect\(&layout\.bounds\)[\s\S]*let start = first\.min\(layout\.item_count\);[\s\S]*let end = last\.min\(layout\.item_count\);/,
    "marquee row mapping must use full drag height while clipping horizontal overlap to the list bounds",
  );
  assert.match(
    marqueeAutoscrollAdjustment,
    /layout\.bounds\.size\.height <= px\(0\.\)[\s\S]*position\.y - layout\.bounds\.origin\.y[\s\S]*PROJECT_PANEL_MARQUEE_AUTOSCROLL_FAST_EDGE[\s\S]*PROJECT_PANEL_MARQUEE_AUTOSCROLL_SLOW_EDGE[\s\S]*point\(px\(0\.\), px\(vertical_scroll_offset\)\)/,
    "marquee edge autoscroll must derive speed from the pointer's top/bottom edge region",
  );
  assert.match(
    marqueeDecorationCompute,
    /self\.layout\.replace\(Some\(ProjectPanelMarqueeLayout[\s\S]*project-panel-marquee-selection/,
    "marquee decoration must record list geometry and render the shaded selection rectangle",
  );
  assert.match(
    renderProjectPanel,
    /on_mouse_down\(\s*MouseButton::Left,\s*cx\.listener\(Self::start_marquee_selection\)[\s\S]*on_mouse_move\(cx\.listener\(Self::update_marquee_selection\)\)[\s\S]*on_mouse_up\(\s*MouseButton::Left,\s*cx\.listener\(Self::finish_marquee_selection\)[\s\S]*on_mouse_up_out\(\s*MouseButton::Left,\s*cx\.listener\(Self::finish_marquee_selection\)/,
    "project panel root must wire marquee mouse lifecycle handlers",
  );
  assert.match(
    renderProjectPanel,
    /with_decoration\(ProjectPanelMarqueeDecoration[\s\S]*layout: self\.marquee_layout\.clone\(\)[\s\S]*selection: self\.marquee_selection\.clone\(\)/,
    "project panel list must install the marquee decoration after row decorations",
  );
});
