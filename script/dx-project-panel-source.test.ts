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
  const dxExplorerSummary = functionBody(source, "dx_explorer_summary");
  const renderDxExplorerHeader = functionBody(source, "render_dx_explorer_header");
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
  assert.match(dxExplorerSummary, /selected_entry_count,/);
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
  assert.match(renderDxExplorerHeader, /ProjectPanelSettings::get_global\(cx\)/);
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::Project\)/);
  assert.match(renderDxExplorerHeader, /Label::new\("DX Explorer"\)/);
  assert.match(renderDxExplorerHeader, /let source_label = summary\.source_kind\.label\(\);/);
  assert.doesNotMatch(renderDxExplorerHeader, /source_label = if is_read_only/);
  assert.match(source, /Self::LocalWorkspace => "Local source"/);
  assert.match(source, /Self::WslWorkspace => "WSL source"/);
  assert.match(source, /Self::RemoteWorkspace => "Remote source"/);
  assert.match(source, /Self::ReadOnlyWorkspace => "Read-only source"/);
  assert.match(renderDxExplorerHeader, /summary\.skipped_entry_count/);
  assert.match(renderDxExplorerHeader, /summary\.visible_file_count/);
  assert.match(renderDxExplorerHeader, /summary\.visible_folder_count/);
  assert.match(renderDxExplorerHeader, /format_file_size\(summary\.visible_file_bytes\)/);
  assert.match(renderDxExplorerHeader, /summary\.cached_media_item_count/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-source-controls"\)/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-filter-controls"\)/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-view-controls"\)/);
  assert.match(renderDxExplorerHeader, /\.id\("dx-explorer-edit-controls"\)/);
  assert.match(renderDxExplorerHeader, /side_panel_header_controls\(\s*"dx-explorer"/);
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::Source\)/);
  assert.match(renderDxExplorerHeader, /dx_icon\(DxUiIcon::Search\)/);
  assert.match(renderDxExplorerHeader, /workspace::Open::default\(\)\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleFileFinder::default\(\)\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleHideGitIgnore\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /ToggleHideHidden\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /selected_style\(ButtonStyle::Tinted\(TintColor::Accent\)\)/);
  assert.match(renderDxExplorerHeader, /toggle_state\(show_ignored_entries\)/);
  assert.match(renderDxExplorerHeader, /toggle_state\(show_hidden_entries\)/);
  assert.match(renderDxExplorerHeader, /ToggleProjectSymbols\.boxed_clone\(\)/);
  assert.match(renderDxExplorerHeader, /this\.new_file\(&NewFile, window, cx\)/);
  assert.match(renderDxExplorerHeader, /this\.new_directory\(&NewDirectory, window, cx\)/);
  assert.match(
    renderDxExplorerHeader,
    /this\.collapse_all_entries\([\s\S]*&CollapseAllEntries,[\s\S]*window,[\s\S]*cx[\s\S]*\)/,
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
    /let dx_explorer_summary =\s*self\.dx_explorer_summary\(selected_entry_count, dx_explorer_source_kind\);/,
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
    after: ".map(|this| {\n                            if let Some(toolbar) = selected_entries_toolbar",
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
  const paste = functionBody(source, "paste");
  const downloadFromRemote = functionBody(source, "download_from_remote");

  assert.match(source, /const MAX_PROJECT_PANEL_EXTERNAL_DROP_PATHS: usize = 4_096;/);
  assert.match(source, /const MAX_PROJECT_PANEL_DRAG_SELECTION_ENTRIES: usize = 4_096;/);
  assert.match(source, /const MAX_PROJECT_PANEL_DOWNLOAD_FILES: usize = 10_000;/);
  assertBefore({
    body: dropExternalFiles,
    before: ".take(MAX_PROJECT_PANEL_EXTERNAL_DROP_PATHS)",
    after: "paths_to_replace.push",
    message: "external drops must be bounded before replacement and copy vectors",
  });
  assertBefore({
    body: dragOnto,
    before: "cap_project_panel_entry_set(",
    after: "copy_tasks.push(task)",
    message: "drag selections must be bounded before copy task fanout",
  });
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
  assert.match(renderEntry, /is_sticky && sticky_index == Some\(0\)/);
  assert.match(renderEntry, /side_panel_header_controls\(\s*"project-panel-sticky",/);
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
  const detailsForEntry = functionBody(source, "details_for_entry");
  const renderEntryInfoBadge = functionBody(source, "render_entry_info_badge");
  const storageDrilldownItems = functionBody(source, "dx_explorer_storage_drilldown_items");
  const renderStorageDrilldown = functionBody(source, "render_dx_explorer_storage_drilldown");
  const renderStorageDrilldownRow = functionBody(
    source,
    "render_dx_explorer_storage_drilldown_row",
  );
  const storageHeatLevel = functionBody(source, "dx_explorer_storage_heat_level");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");
  const cachedFolderStorageSummary = functionBody(source, "cached_folder_storage_summary");

  assert.match(
    source,
    /const MAX_PROJECT_PANEL_BACKGROUND_FOLDER_STORAGE_DIRS: usize = 4_096;/,
    "folder storage warming must have a named background cap",
  );
  assert.match(
    source,
    /const MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS: usize = 5;/,
    "storage drilldown must stay visually bounded",
  );
  assert.match(source, /struct FolderStorageSummary/);
  assert.match(source, /struct DxExplorerStorageDrilldownItem/);
  assert.match(source, /dx_explorer_storage_drilldown:\s*Vec<DxExplorerStorageDrilldownItem>/);
  assert.match(source, /fn record_file\(&mut self, size: u64\)/);
  assert.match(source, /fn cached_folder_storage_summary\(/);
  assert.match(source, /fn dx_explorer_storage_drilldown_items\(/);
  assert.match(source, /fn render_dx_explorer_storage_drilldown\(/);
  assert.match(source, /fn render_dx_explorer_storage_drilldown_row\(/);
  assert.match(source, /fn dx_explorer_storage_heat_level\(/);
  assert.match(source, /fn dx_explorer_storage_heat_color\(/);
  assert.match(
    cachedFolderStorageSummary,
    /folder_storage_summaries[\s\S]*get\(&cache_key\)[\s\S]*copied\(\)/,
    "render-facing folder storage lookup must be cache-only",
  );
  assert.match(
    detailsForEntry,
    /let folder_storage_summary = entry[\s\S]*kind[\s\S]*is_dir\(\)[\s\S]*then\(\|\| self\.cached_folder_storage_summary\(worktree_id, entry\.id\)\)[\s\S]*flatten\(\);/,
    "details_for_entry must not warm folder storage cache misses on the visible-row path",
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
    updateVisibleEntries,
    /let cached_folder_storage_summary_keys = cached_folder_storage_summaries[\s\S]*keys\(\)[\s\S]*collect::<HashSet<_>>\(\);/,
    "visible-entry refresh must snapshot folder storage cache state before background warming",
  );
  assert.match(
    updateVisibleEntries,
    /let cached_folder_storage_summaries = self\.folder_storage_summaries\.borrow\(\)\.clone\(\);/,
    "visible-entry refresh must snapshot the storage cache before moving work to the background job",
  );
  assert.match(
    updateVisibleEntries,
    /let mut folder_storage_summary_updates = Vec::new\(\);/,
    "folder storage warming must collect background results separately from visible rows",
  );
  assert.match(
    updateVisibleEntries,
    /folder_storage_summary_updates\.len\(\)[\s\S]*MAX_PROJECT_PANEL_BACKGROUND_FOLDER_STORAGE_DIRS[\s\S]*let mut summary = FolderStorageSummary::default\(\)[\s\S]*child_entries_with_options[\s\S]*include_files: true[\s\S]*include_dirs: false[\s\S]*summary\.record_file\(child\.size\)[\s\S]*folder_storage_summary_updates\.push\(\(cache_key, summary\)\)/,
    "background folder storage warming must count direct file children and bytes under a named cap",
  );
  assert.match(
    updateVisibleEntries,
    /folder_storage_summaries\.entry\(cache_key\)\.or_insert\(summary\)/,
    "background folder storage results must populate cache misses without overwriting fresher summaries",
  );
  assert.match(
    storageDrilldownItems,
    /folder_storage_summaries: &HashMap<\(WorktreeId, ProjectEntryId\), FolderStorageSummary>/,
    "storage drilldown must receive the warmed cache from the visible-entry refresh job",
  );
  assert.match(
    storageDrilldownItems,
    /for visible_worktree in &state\.visible_entries[\s\S]*for entry in &visible_worktree\.entries[\s\S]*!entry\.kind\.is_dir\(\)[\s\S]*folder_storage_summaries\.get\(&cache_key\)\.copied\(\)/,
    "storage drilldown must derive candidate folders from materialized visible rows and cached summaries",
  );
  assert.match(
    storageDrilldownItems,
    /items\.sort_by\(\|left, right\|[\s\S]*right[\s\S]*\.file_bytes[\s\S]*\.cmp\(&left\.file_bytes\)[\s\S]*right\.file_count\.cmp\(&left\.file_count\)/,
    "storage drilldown must list biggest cached folders first",
  );
  assert.match(
    storageDrilldownItems,
    /items\.truncate\(MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS\)/,
    "storage drilldown must stay capped before render",
  );
  assert.match(
    storageDrilldownItems,
    /item\.heat_level = dx_explorer_storage_heat_level\(item\.file_bytes, max_file_bytes\)/,
    "storage drilldown must precompute heat-map levels outside render rows",
  );
  assert.doesNotMatch(
    storageDrilldownItems,
    /\bread_dir\(|\bFile::open\(|read_to_string|child_entries_with_options|cx\.spawn/,
    "storage drilldown item building must stay cache/materialized-state only",
  );
  assertBefore({
    body: updateVisibleEntries,
    before:
      /folder_storage_summary_cache[\s\S]*\.entry\(\*cache_key\)[\s\S]*\.or_insert\(\*summary\)/,
    after:
      /new_state\.dx_explorer_storage_drilldown =[\s\S]*Self::dx_explorer_storage_drilldown_items\([\s\S]*&new_state,[\s\S]*&folder_storage_summary_cache/,
    message: "storage drilldown must merge fresh background summaries before ranking cached folders",
  });
  assertBefore({
    body: updateVisibleEntries,
    before:
      /new_state\.dx_explorer_storage_drilldown =[\s\S]*Self::dx_explorer_storage_drilldown_items\([\s\S]*&new_state,[\s\S]*&folder_storage_summary_cache/,
    after: /\(new_state, media_preview_updates, folder_storage_summary_updates\)/,
    message: "storage drilldown must be ranked in the background job before state is installed",
  });
  assert.match(renderStorageDrilldown, /\.id\("dx-explorer-storage-drilldown"\)/);
  assert.match(renderStorageDrilldown, /dx_icon\(DxUiIcon::Storage\)/);
  assert.match(renderStorageDrilldown, /Label::new\("Folder storage"\)/);
  assert.match(renderStorageDrilldown, /\.children\(rows\)/);
  assert.doesNotMatch(renderStorageDrilldown, /\bread_dir\(|\bFile::open\(|child_entries/);
  assert.match(
    renderStorageDrilldownRow,
    /SelectedEntry \{[\s\S]*worktree_id: item\.worktree_id,[\s\S]*entry_id: item\.entry_id/,
    "storage drilldown rows must target real project entries",
  );
  assert.match(renderStorageDrilldownRow, /dx_explorer_storage_heat_color\(item\.heat_level, cx\)/);
  assert.match(renderStorageDrilldownRow, /format_file_size\(item\.file_bytes\)/);
  assert.match(renderStorageDrilldownRow, /this\.expand_entry\(target\.worktree_id, target\.entry_id, cx\)/);
  assert.match(
    renderStorageDrilldownRow,
    /this\.update_visible_entries\([\s\S]*Some\(\(target\.worktree_id, target\.entry_id\)\)[\s\S]*true,[\s\S]*window,[\s\S]*cx/,
    "storage drilldown row clicks must select and scroll to the real folder",
  );
  assert.match(storageHeatLevel, /u128::from\(file_bytes\) \* 4/);
  assert.match(storageHeatLevel, /scaled\.clamp\(1, 4\) as u8/);
  assertBefore({
    body: source,
    before: /self\.render_dx_explorer_storage_drilldown\(cx\)/,
    after: /media_preview::render_folder_media_shelf/,
    message: "storage drilldown should render before the media shelf and tree rows",
  });
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
  assert.match(media, /fn render_media_shelf_card/);
  assert.match(media, /fn render_media_gallery_card/);
  assert.match(media, /fn audio_gradient_background/);
  assert.match(media, /pub\(crate\) fn is_media_path/);
  assert.match(media, /fn media_kind_sort_rank/);
  assert.match(media, /entry_id:\s*ProjectEntryId/);
  assert.match(media, /pub\(crate\) enum VideoFramePreviewKind/);
  assert.match(media, /pub\(crate\) struct VideoFramePreview/);
  assert.match(media, /video_frame_preview:\s*Option<VideoFramePreview>/);
  assert.match(media, /duration_label:\s*Option<String>/);
  assert.match(media, /size:\s*u64/);
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
    /background_spawn\(async move \{[\s\S]*let mut active_media_shelf_entry_ids = active_media_shelf_entry_ids;[\s\S]*let mut media_preview_updates = Vec::new\(\);[\s\S]*let is_active_media_folder =[\s\S]*active_media_folder_for_visibility == Some\(cache_key\);[\s\S]*media_preview_enabled[\s\S]*is_active_media_folder[\s\S]*MAX_PROJECT_PANEL_BACKGROUND_MEDIA_PREVIEW_FOLDERS[\s\S]*match generated_media_metadata\.get\(&cache_key\)[\s\S]*build_folder_media_preview_with_generated_metadata\([\s\S]*&absolute_path,[\s\S]*children,[\s\S]*Some\(generated_metadata\)[\s\S]*\)[\s\S]*build_folder_media_preview\([\s\S]*&absolute_path,[\s\S]*children,[\s\S]*\)[\s\S]*active_media_shelf_entry_ids\.extend\([\s\S]*preview\.items\.iter\(\)\.map\(\|item\| item\.entry_id\)[\s\S]*media_preview_updates\.push\(\(cache_key, preview\)\)[\s\S]*\(new_state, media_preview_updates, folder_storage_summary_updates\)/,
    "media preview cache misses must be warmed inside the visible-entry background task",
  );
  assert.match(
    updateVisibleEntries,
    /folder_media_previews\.entry\(cache_key\)\.or_insert\(preview\)/,
    "background media preview results must populate cache misses without overwriting fresher cache entries",
  );

  assertBefore({
    body: detailsForEntry,
    before: /entry\.kind\.is_dir\(\)\s*&&\s*is_expanded/,
    after: /self\.cached_folder_media_preview\(/,
    message: "media previews must be read only after confirming an expanded directory",
  });
  const mediaPreviewBranch = detailsForEntry.match(
    /let media_preview = if entry\.kind\.is_dir\(\) && is_expanded \{[\s\S]*?\n        \} else \{\n            None\n        \};/,
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
  const generateVideoCenterFrame = functionBody(
    generatedVideoFrame,
    "generate_video_center_frame",
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
    /items\s*\{[\s\S]*jobs\.len\(\) >= MAX_GENERATED_MEDIA_METADATA_JOBS[\s\S]*MediaPreviewKind::Audio[\s\S]*duration_label\.is_none\(\)[\s\S]*item\.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES/,
    "generated metadata jobs must be bounded and select missing audio durations that are small enough to inspect",
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
    collectGeneratedMediaMetadata,
    /batch\.jobs[\s\S]*audio_duration_seconds_for_path[\s\S]*GeneratedMediaMetadataRecord[\s\S]*duration_seconds: Some\(duration_seconds\)[\s\S]*GeneratedMediaMetadataIndex::from_records/,
    "generated metadata collection must turn successful background audio duration reads into generated metadata records",
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
    /File::open\(path\)[\s\S]*BufReader::new\(file\)[\s\S]*Decoder::new\(reader\)[\s\S]*total_duration\(\)[\s\S]*as_secs_f64\(\)/,
    "audio duration extraction must use bounded Rust decoder metadata off the render path",
  );
  assert.doesNotMatch(
    generatedMetadata,
    /std::process|Command::new|\.status\(|\.output\(|\.spawn\(|ffmpeg|ffprobe|fs::write|File::create|create_dir_all|remove_file|rename\(|copy\(/,
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
    generateVideoCenterFrame,
    /let modified_at = video_frame_cache_modified_at\(source_path\);[\s\S]*managed_video_frame_cache_path\(path_text, size, modified_at\)[\s\S]*probe_video_duration_seconds\(source_path, executor\)\.await[\s\S]*extract_video_center_frame\([\s\S]*source_path,[\s\S]*&temporary_output_path,[\s\S]*center_seconds,[\s\S]*executor,[\s\S]*\)[\s\S]*\.await/,
    "video center-frame generation must derive a managed cache path, probe duration, then extract the center timestamp",
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
    /Label::new\("Media"\)(?![\s\S]*format!\("\{visible_media_count\} shown \/ \{summary\}"\))/,
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
    renderMediaShelfCard,
    /media_shelf_card_container\([\s\S]*cursor_pointer\(\)[\s\S]*SelectedEntry[\s\S]*worktree_id[\s\S]*entry_id[\s\S]*PreviewTabsSettings::get_global\(cx\)[\s\S]*panel\.open_entry/,
    "media shelf cards must select and open real project entries",
  );
  assert.match(
    renderMediaShelfCard,
    /MouseButton::Right[\s\S]*panel\.deploy_context_menu\(event\.position, entry_id, window, cx\)/,
    "media shelf cards must use the real project-panel context menu on right click",
  );
  assert.match(
    mediaShelfCardContainer,
    /is_selected[\s\S]*colors\.border_focused[\s\S]*colors\.element_selected/,
    "media shelf cards must show selected state through the shelf card container",
  );
  assert.match(
    mediaShelfCardContainer,
    /\.h\(px\(PROJECT_PANEL_MEDIA_SHELF_CARD_TOTAL_HEIGHT\)\)[\s\S]*\.w_full\(\)[\s\S]*\.v_flex\(\)[\s\S]*\.p_0\(\)[\s\S]*\.overflow_hidden\(\)/,
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
    /MediaPreviewKind::Audio[\s\S]*w_full\(\)[\s\S]*flex_1\(\)[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*items_center\(\)[\s\S]*justify_center\(\)[\s\S]*Label::new\(item\.name\.clone\(\)\)[\s\S]*truncate\(\)/,
    "shelf audio cards must use full-height deterministic gradient rectangles with centered truncated filenames",
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
    /MediaPreviewKind::Audio[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*Label::new\(item\.name\.clone\(\)\)[\s\S]*buffer_font\(cx\)[\s\S]*truncate\(\)/,
    "gallery audio cards must use deterministic color rectangles with centered truncated filenames",
  );
  assert.match(
    mediaPreviewCardTooltipMeta,
    /let size_label = media_size_label\(item\.size\);[\s\S]*Duration unavailable[\s\S]*Size: \{size_label\}/,
    "media hover details must include snapshot size and manifest duration with an honest unavailable state",
  );
  assert.match(
    mediaPreviewCardTooltipMeta,
    /Thumbnail unavailable/,
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
    /VideoFramePreviewKind::Center[\s\S]*"Center thumbnail"[\s\S]*VideoFramePreviewKind::Preview[\s\S]*"Thumbnail"/,
    "video hover details must label center/middle frames separately from generic preview frames",
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
    /media_metadata_generation_tasks:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), Task<\(\)>>/,
    "project panel must track in-flight generated metadata work per folder",
  );
  assert.match(
    projectPanel,
    /generated_media_metadata[\s\S]*retain\(\|\(worktree_id, _\), _\| \*worktree_id != \*id\)/,
    "generated media metadata cache entries must be retained correctly when a worktree is removed",
  );
  assert.match(
    projectPanel,
    /generated_media_metadata\.borrow_mut\(\)\.clear\(\);[\s\S]*folder_media_previews\.borrow_mut\(\)\.clear\(\);/,
    "generated media metadata cache must clear before media previews are rebuilt after worktree/settings changes",
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
    /generated_media_metadata[\s\S]*borrow_mut\(\)[\s\S]*\.insert\(cache_key, generated_metadata\)/,
    "generated metadata results must update the ProjectPanel generated metadata cache",
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
