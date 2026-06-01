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
  assertBefore({
    body: updateVisibleEntries,
    before: ".take(MAX_PROJECT_PANEL_VISIBLE_WORKTREES)",
    after: ".collect()",
    message: "visible worktrees must be capped before snapshot vector collection",
  });
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
  const record = functionBody(undo, "record");

  assert.match(utils, /pub\(crate\) const MAX_PROJECT_PANEL_DISPLAY_LABEL_CHARS: usize = 1_024;/);
  assert.match(utils, /pub\(crate\) fn bounded_project_panel_label\(/);
  assert.match(source, /const MAX_PROJECT_PANEL_STICKY_PARENTS: usize = 128;/);
  assert.match(undo, /const MAX_PROJECT_PANEL_UNDO_BATCH_CHANGES: usize = 4_096;/);
  assert.match(detailsForEntry, /utils::bounded_project_panel_label\(filename\)/);
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

test("project panel media preview is lazy, bounded, and preserves normal tree rows", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const media = read("crates/project_panel/src/media_preview.rs");
  const detailsForEntry = functionBody(source, "details_for_entry");
  const renderEntry = functionBody(source, "render_entry");
  const renderProjectPanel = functionBody(source, "render");
  const updateVisibleEntries = functionBody(source, "update_visible_entries");
  const activeMediaFolderForSelection = functionBody(source, "active_media_folder_for_selection");
  const activeFolderMediaPreview = functionBody(source, "active_folder_media_preview");

  assert.match(source, /mod media_preview;/);
  assert.match(source, /struct ActiveMediaFolder/);
  assert.match(source, /folder_media_previews:\s*RefCell<HashMap<\(WorktreeId, ProjectEntryId\), Option<media_preview::FolderMediaPreview>>>/);
  assert.match(source, /media_preview:\s*Option<media_preview::FolderMediaPreview>/);
  assert.match(source, /fn active_media_folder_for_selection\(/);
  assert.match(source, /fn active_folder_media_preview\(/);

  assert.match(media, /pub\(crate\) const MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN: usize = 512;/);
  assert.match(media, /pub\(crate\) const MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS: usize = 12;/);
  assert.match(media, /pub\(crate\) const MAX_PROJECT_PANEL_MEDIA_INLINE_CARDS: usize = 4;/);
  assert.match(media, /pub\(crate\) const PROJECT_PANEL_MEDIA_GALLERY_COLUMNS: u16 = 3;/);
  assert.match(media, /pub\(crate\) enum MediaPreviewKind/);
  assert.match(media, /Image/);
  assert.match(media, /Video/);
  assert.match(media, /Audio/);
  assert.match(media, /fn video_preview_frame_path/);
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
  assert.match(media, /video_frame_path:\s*Option<PathBuf>/);
  assert.match(media, /duration_label:\s*Option<String>/);
  assert.match(media, /size:\s*u64/);
  assert.match(
    source,
    /let preview = media_preview::build_folder_media_preview\(parent_abs_path, children\);[\s\S]*insert\(cache_key, preview\.clone\(\)\);[\s\S]*preview/,
    "media preview cache must store both populated previews and no-media misses",
  );

  assertBefore({
    body: detailsForEntry,
    before: /entry\.kind\.is_dir\(\)\s*&&\s*is_expanded/,
    after: /self\.folder_media_preview\(/,
    message: "media previews must be built only after confirming an expanded directory",
  });
  const mediaPreviewBranch = detailsForEntry.match(
    /let media_preview = if entry\.kind\.is_dir\(\) && is_expanded \{[\s\S]*?\n        \} else \{\n            None\n        \};/,
  );
  assert.ok(
    mediaPreviewBranch,
    "details_for_entry must isolate media probing inside the expanded-directory branch",
  );
  assert.match(mediaPreviewBranch[0], /self\.folder_media_preview\(/);
  assert.doesNotMatch(
    detailsForEntry.replace(mediaPreviewBranch[0], ""),
    /self\.folder_media_preview\(/,
    "details_for_entry must not probe media outside the expanded-directory branch",
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
    /self\.folder_media_preview\([\s\S]*active_media_folder\.worktree_id[\s\S]*active_media_folder\.entry_id[\s\S]*&active_media_folder\.absolute_path[\s\S]*children[\s\S]*\)/,
    "active media shelf must reuse the cached snapshot-derived media preview",
  );
  assert.match(
    updateVisibleEntries,
    /let active_media_folder_for_visibility = self[\s\S]*active_media_folder_for_selection\(cx\)[\s\S]*map\(\|folder\| \(folder\.worktree_id, folder\.entry_id\)\)/,
    "visible-entry derivation must know which active folder is represented by the bottom media shelf",
  );
  assert.match(
    updateVisibleEntries,
    /entry_is_active_media_shelf_child[\s\S]*media_preview::is_media_path\([\s\S]*entry\.path\.as_std_path\(\)[\s\S]*entry\.path\.parent\(\)[\s\S]*parent\.id == active_folder_id/,
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
  assertBefore({
    body: renderEntry,
    before: /let media_preview = \(!is_sticky\)\s*\.then\(\|\| details\.media_preview\.clone\(\)\)\s*\.flatten\(\);/,
    after: /media_preview::render_folder_media_preview/,
    message: "render_entry must use the cached media preview instead of probing from render code",
  });
  assertBefore({
    body: renderEntry,
    before: /\.end_slot::<AnyElement>/,
    after: /media_preview::render_folder_media_preview/,
    message: "media previews must render inside the existing uniform-height row end slot",
  });
  assert.doesNotMatch(
    renderEntry,
    /\.when\(!is_sticky && kind\.is_dir\(\) && is_expanded[\s\S]*media_preview::render_folder_media_preview/,
    "media previews must not add variable-height children under uniform_list rows",
  );
  assert.match(renderEntry, /block_mouse_except_scroll\(\)/);
  assertBefore({
    body: renderProjectPanel,
    before: /let active_media_preview = has_worktree/,
    after: /let project = self\.project\.read\(cx\);\s*let panel_settings = ProjectPanelSettings::get_global\(cx\);/,
    message: "active media shelf lookup must finish before the long project render borrow",
  });
  assertBefore({
    body: renderProjectPanel,
    before: /uniform_list\("entries"/,
    after: /media_preview::render_folder_media_shelf/,
    message: "media shelf must render under the virtualized tree rather than inside a row",
  });
  assert.match(
    renderProjectPanel,
    /active_media_folder\.worktree_id[\s\S]*active_media_folder\.selected_media_entry_id/,
    "media shelf must receive real worktree and selected media entry state",
  );
  assertBefore({
    body: renderProjectPanel,
    before: /media_preview::render_folder_media_shelf/,
    after: /id\("project-panel-blank-area"\)/,
    message: "media shelf must occupy the project-panel bottom area before the blank drop zone",
  });
});

test("project panel media preview renders direct image previews and video frames when available", () => {
  const media = read("crates/project_panel/src/media_preview.rs");
  const metadata = read("crates/project_panel/src/media_preview/metadata.rs");
  const renderFolderMediaPreview = functionBody(media, "render_folder_media_preview");
  const renderFolderMediaGallery = functionBody(media, "render_folder_media_gallery");
  const renderFolderMediaShelf = functionBody(media, "render_folder_media_shelf");
  const renderMediaShelfCard = functionBody(media, "render_media_shelf_card");
  const renderMediaPreviewCard = functionBody(media, "render_media_preview_card");
  const renderMediaGalleryCard = functionBody(media, "render_media_gallery_card");
  const mediaGalleryCardContainer = functionBody(media, "media_gallery_card_container");
  const mediaPreviewCardTooltipMeta = functionBody(media, "media_preview_card_tooltip_meta");
  const audioGradientBackground = functionBody(media, "audio_gradient_background");
  const buildFolderMediaPreview = functionBody(media, "build_folder_media_preview");
  const buildMediaMetadataIndex = functionBody(metadata, "build_media_metadata_index");
  const readBoundedMediaMetadataManifest = functionBody(
    metadata,
    "read_bounded_media_metadata_manifest",
  );
  const collectMediaMetadataRecord = functionBody(metadata, "collect_media_metadata_record");
  const mediaDurationLabelFromRecord = functionBody(metadata, "media_duration_label_from_record");
  const videoPreviewFramePath = functionBody(media, "video_preview_frame_path");
  const videoFrameCandidateRank = functionBody(media, "video_frame_candidate_rank");

  assert.match(media, /mod metadata;/);
  assert.match(metadata, /pub\(super\) struct MediaMetadataIndex/);
  assert.match(metadata, /pub\(super\) const MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES/);

  assertBefore({
    body: buildFolderMediaPreview,
    before: /children\.take\(MAX_PROJECT_PANEL_MEDIA_CHILD_SCAN \+ 1\)/,
    after: /scanned_cap_hit/,
    message: "media child scans must be capped before classification work",
  });
  assertBefore({
    body: buildFolderMediaPreview,
    before: /items\.sort_by/,
    after: /items\.truncate\(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS\)/,
    message: "media preview candidates must be ordered before the bounded render set is selected",
  });
  assertBefore({
    body: buildFolderMediaPreview,
    before: /items\.truncate\(MAX_PROJECT_PANEL_MEDIA_PREVIEW_ITEMS\)/,
    after: /for item in &mut items/,
    message: "media preview items must be capped before render data receives video frame paths",
  });
  assert.match(
    buildFolderMediaPreview,
    /size:\s*child\.size/,
    "media preview items must carry snapshot file sizes for hover details",
  );
  assert.match(
    buildFolderMediaPreview,
    /entry_id:\s*child\.id/,
    "media preview items must carry project entry ids for card selection/opening",
  );
  assert.match(
    buildFolderMediaPreview,
    /let media_metadata = metadata::build_media_metadata_index\(parent_abs_path, &child_entries\);/,
    "media preview items must derive optional duration/frame metadata from the bounded child snapshot",
  );
  assert.match(
    buildFolderMediaPreview,
    /duration_label:\s*media_metadata\.duration_label_for_path\(&absolute_path\)/,
    "media preview items must carry manifest duration labels when present",
  );
  assert.match(
    buildFolderMediaPreview,
    /media_metadata[\s\S]*\.video_frame_for_path\(&item\.absolute_path\)[\s\S]*\.or_else\(\|\| video_preview_frame_path/,
    "video media cards must prefer manifest-declared center frames before heuristic sidecar frames",
  );
  assertBefore({
    body: renderFolderMediaPreview,
    before: /preview\s*\.items\s*\.iter\(\)/,
    after: /take\(MAX_PROJECT_PANEL_MEDIA_INLINE_CARDS\)/,
    message: "folder media preview must cap inline cards before render mapping",
  });
  assertBefore({
    body: renderFolderMediaPreview,
    before: /take\(MAX_PROJECT_PANEL_MEDIA_INLINE_CARDS\)/,
    after: /render_media_preview_card/,
    message: "folder media preview must render from bounded preview items",
  });
  assert.match(renderFolderMediaPreview, /\.h_6\(\)/);
  assert.match(renderFolderMediaPreview, /\.overflow_hidden\(\)/);
  assert.match(renderFolderMediaPreview, /Tooltip::with_meta/);
  assert.match(renderFolderMediaPreview, /PopoverMenu::new\(gallery_id\)/);
  assert.match(
    renderFolderMediaPreview,
    /IconButton::new\("project-panel-media-gallery-trigger", IconName::Blocks\)/,
    "folder media preview must expose a real grid affordance from the compact row",
  );
  assert.match(
    renderFolderMediaGallery,
    /\.grid\(\)[\s\S]*\.grid_cols\(PROJECT_PANEL_MEDIA_GALLERY_COLUMNS\)/,
    "folder media gallery must use a bounded three-column grid",
  );
  assert.match(
    renderFolderMediaShelf,
    /\.border_t_1\(\)[\s\S]*\.grid\(\)[\s\S]*\.grid_cols\(PROJECT_PANEL_MEDIA_GALLERY_COLUMNS\)/,
    "folder media shelf must render as a bottom three-column media grid",
  );
  assert.match(
    renderFolderMediaShelf,
    /Label::new\("Media"\)[\s\S]*format!\("\{visible_count\} shown \/ \{summary\}"\)/,
    "folder media shelf must expose a concise real count summary",
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
    /SelectedEntry[\s\S]*worktree_id[\s\S]*entry_id[\s\S]*PreviewTabsSettings::get_global\(cx\)[\s\S]*panel\.open_entry/,
    "media shelf cards must select and open real project entries",
  );
  assert.match(
    renderMediaShelfCard,
    /MouseButton::Right[\s\S]*panel\.deploy_context_menu\(event\.position, entry_id, window, cx\)/,
    "media shelf cards must use the real project-panel context menu on right click",
  );
  assert.match(
    mediaGalleryCardContainer,
    /is_selected[\s\S]*colors\.border_focused[\s\S]*colors\.element_selected/,
    "media shelf cards must show selected state through the shared card container",
  );
  assert.match(
    renderMediaPreviewCard,
    /MediaPreviewKind::Image[\s\S]*img\(item\.absolute_path\.clone\(\)\)[\s\S]*object_fit\(ObjectFit::Cover\)/,
    "image media cards must render direct visual previews from local paths",
  );
  assert.match(
    renderMediaPreviewCard,
    /MediaPreviewKind::Video[\s\S]*item\.video_frame_path\.as_ref\(\)[\s\S]*img\(frame_path\.clone\(\)\)[\s\S]*IconName::PlayOutlined/,
    "video media cards must use representative frame images when available and keep a play affordance",
  );
  assert.match(
    renderMediaPreviewCard,
    /MediaPreviewKind::Video[\s\S]*Icon::new\(IconName::PlayOutlined\)/,
    "video media cards need a lightweight fallback when no preview frame exists",
  );
  assert.match(
    renderMediaPreviewCard,
    /let tooltip_title = item\.name\.clone\(\);[\s\S]*MediaPreviewKind::Audio[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*Label::new\(item\.name\.clone\(\)\)[\s\S]*Tooltip::with_meta\(tooltip_title\.clone\(\), None, tooltip_meta\.clone\(\), cx\)/,
    "audio media cards must render stable gradient rectangles with centered truncated filenames",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Image[\s\S]*img\(item\.absolute_path\.clone\(\)\)[\s\S]*object_fit\(ObjectFit::Cover\)/,
    "gallery image cards must render direct visual previews",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Video[\s\S]*item\.video_frame_path\.as_ref\(\)[\s\S]*img\(frame_path\.clone\(\)\)[\s\S]*IconName::PlayOutlined/,
    "gallery video cards must use available representative frame images and keep a play affordance",
  );
  assert.match(
    mediaGalleryCardContainer,
    /MediaPreviewKind::Audio[\s\S]*audio_gradient_background\(&item\.name\)[\s\S]*Label::new\(item\.name\.clone\(\)\)[\s\S]*buffer_font\(cx\)[\s\S]*truncate\(\)/,
    "gallery audio cards must use deterministic color rectangles with centered truncated filenames",
  );
  assert.match(
    mediaPreviewCardTooltipMeta,
    /let size_label = media_size_label\(item\.size\);[\s\S]*item\.duration_label[\s\S]*Duration unavailable/,
    "media hover details must include snapshot size and manifest duration with an honest unavailable state",
  );
  assert.match(
    audioGradientBackground,
    /audio_gradient_colors\(name\)/,
    "audio card gradients must be deterministic from the audio filename",
  );
  assert.match(
    videoPreviewFramePath,
    /video_frame_candidate_rank\(&video_stem, stem\)[\s\S]*\.min_by_key/,
    "video preview frame matching should rank sidecar images instead of accepting the first candidate",
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
    readBoundedMediaMetadataManifest,
    /fs::File::open[\s\S]*take\(MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES \+ 1\)[\s\S]*String::from_utf8/,
    "media metadata manifest reads must use a sentinel-byte bound before UTF-8 and JSON parsing",
  );
  assert.match(
    collectMediaMetadataRecord,
    /media_duration_label_from_record[\s\S]*center_frame[\s\S]*middle_frame/,
    "media metadata records must support center-frame paths and duration labels",
  );
  assert.match(
    mediaDurationLabelFromRecord,
    /duration_label[\s\S]*duration_seconds[\s\S]*format_media_duration_seconds/,
    "duration metadata must accept explicit labels or numeric seconds",
  );
  assert.doesNotMatch(
    `${media}\n${metadata}`,
    /path\.is_file\(\)|std::fs::metadata|fs::metadata/,
    "media preview classification must stay snapshot-derived and avoid UI-path filesystem metadata probes",
  );
});

test("project panel marquee drag selection is real and bounded", () => {
  const source = read("crates/project_panel/src/project_panel.rs");
  const startMarqueeSelection = functionBody(source, "start_marquee_selection");
  const updateMarqueeSelection = functionBody(source, "update_marquee_selection");
  const finishMarqueeSelection = functionBody(source, "finish_marquee_selection");
  const applyMarqueeSelection = functionBody(source, "apply_marquee_selection");
  const marqueeEntryRange = functionBody(source, "project_panel_marquee_entry_range");
  const marqueeDecorationCompute = functionBody(source, "compute");
  const renderProjectPanel = functionBody(source, "render");

  assert.match(source, /const MAX_PROJECT_PANEL_MARQUEE_SELECTION_ENTRIES: usize = 20_000;/);
  assert.match(source, /const PROJECT_PANEL_MARQUEE_MIN_DRAG_DISTANCE: Pixels = px\(4\.\);/);
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
    /event\.dragging\(\)[\s\S]*PROJECT_PANEL_MARQUEE_MIN_DRAG_DISTANCE[\s\S]*marquee\.active = true[\s\S]*self\.apply_marquee_selection\(cx\)/,
    "marquee selection must wait for a drag threshold before applying real selection",
  );
  assert.match(
    finishMarqueeSelection,
    /self\.marquee_selection\.take\(\)[\s\S]*let was_active = marquee\.active[\s\S]*self\.mouse_down = false[\s\S]*cx\.notify\(\)[\s\S]*cx\.stop_propagation\(\)/,
    "finishing an active marquee drag must clear state and suppress the trailing click",
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
    /project_panel_marquee_bounds\(selection\)\.intersect\(&layout\.bounds\)[\s\S]*visible_range\.start[\s\S]*visible_range\.end/,
    "marquee row mapping must clip to the actual uniform-list bounds and visible range",
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
