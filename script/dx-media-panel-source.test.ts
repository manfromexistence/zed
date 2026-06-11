import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const readIfExists = (path: string) => (existsSync(path) ? read(path) : "");

const workspaceManifest = read("Cargo.toml");
const lockfile = read("Cargo.lock");
const panelManifest = read("crates/media_panel/Cargo.toml");
const panelSource = read("crates/media_panel/src/media_panel.rs");
const bridgeSource = readIfExists("crates/media_panel/src/dx_media_bridge.rs");

function functionBody(sourceText: string, name: string): string {
  const fnIndex = sourceText.search(new RegExp(`fn\\s+${name}(?:<[^>]+>)?\\s*\\(`));
  assert.ok(fnIndex >= 0, `expected ${name}`);

  const bodyStart = sourceText.indexOf("{", fnIndex);
  assert.ok(bodyStart > fnIndex, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < sourceText.length; index += 1) {
    const char = sourceText[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return sourceText.slice(fnIndex, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
}

function assertBefore(
  haystack: string,
  before: string | RegExp,
  after: string | RegExp,
  message: string,
) {
  const beforeIndex =
    typeof before === "string" ? haystack.indexOf(before) : haystack.match(before)?.index ?? -1;
  const afterIndex =
    typeof after === "string" ? haystack.indexOf(after) : haystack.match(after)?.index ?? -1;

  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
}

function lockPackageBlock(name: string): string {
  const start = lockfile.indexOf(`name = "${name}"`);
  assert.ok(start >= 0, `expected lock package ${name}`);

  const blockStart = lockfile.lastIndexOf("[[package]]", start);
  const nextBlock = lockfile.indexOf("[[package]]", start + name.length);
  return lockfile.slice(blockStart, nextBlock >= 0 ? nextBlock : undefined);
}

test("media panel depends on the dx-media crate without taking CLI defaults", () => {
  assert.match(
    workspaceManifest,
    /dx_media = \{ package = "dx-media", path = "\.\.\/media", default-features = false \}/,
  );
  assert.match(panelManifest, /^dx_media\.workspace = true$/m);
  assert.match(lockfile, /\[\[package\]\]\r?\nname = "dx-media"\r?\nversion = "1\.0\.0"/);
  assert.match(lockfile, /name = "media_panel"[\s\S]*"dx-media"[\s\S]*"gpui_tokio"/);

  const dxMediaLock = lockPackageBlock("dx-media");
  assert.doesNotMatch(dxMediaLock, /"clap"/);
  assert.doesNotMatch(dxMediaLock, /"colored"/);
  assert.doesNotMatch(dxMediaLock, /"console/);
  assert.doesNotMatch(dxMediaLock, /"indicatif"/);
  assert.doesNotMatch(dxMediaLock, /"tracing-subscriber"/);
});

test("media panel routes remote search through the dx-media bridge first", () => {
  assert.match(panelSource, /^mod dx_media_bridge;$/m);
  assert.match(panelManifest, /^gpui_tokio\.workspace = true$/m);

  const fetchRemoteMediaAssets = functionBody(panelSource, "fetch_remote_media_assets");
  assert.match(fetchRemoteMediaAssets, /cx: &mut AsyncApp/);
  assert.doesNotMatch(fetchRemoteMediaAssets, /cx: &mut AsyncWindowContext/);
  assert.match(fetchRemoteMediaAssets, /dx_media_bridge::fetch_panel_media\(/);
  assert.match(fetchRemoteMediaAssets, /dx_media_bridge::PanelMediaSearchRequest::new\(/);
  assert.match(fetchRemoteMediaAssets, /gpui_tokio::Tokio::spawn_result\(cx/);
  assert.match(fetchRemoteMediaAssets, /if !result\.assets\.is_empty\(\) && result\.warning\.is_none\(\)/);
  assert.match(fetchRemoteMediaAssets, /assets\.extend\(result\.assets\)/);
  assert.match(fetchRemoteMediaAssets, /DX Media: no panel-renderable rows/);
  assert.match(panelSource, /RemoteMediaAsset::from/);
  assertBefore(
    fetchRemoteMediaAssets,
    /dx_media_bridge::fetch_panel_media\(/,
    /fetches\.push\(remote_media_fetch\(/,
    "dx-media must be the primary remote search path before panel-local provider fallbacks",
  );
});

test("dx-media bridge preserves query, type, page, count, and provider evidence", () => {
  assert.match(bridgeSource, /use dx_media::\{DxMedia, MediaAsset, MediaType, SearchMode, SearchQuery, SearchResult\};/);

  const buildSearchQueries = functionBody(bridgeSource, "build_search_queries");
  assert.match(buildSearchQueries, /media_types_for_filter\(request\.filter\)/);
  assert.match(buildSearchQueries, /build_search_query\(request, None\)/);
  assert.match(buildSearchQueries, /build_search_query\(request, Some\(\*media_type\)\)/);

  const buildSearchQuery = functionBody(bridgeSource, "build_search_query");
  assert.match(buildSearchQuery, /SearchQuery::new\(request\.query\.clone\(\)\)/);
  assert.match(buildSearchQuery, /\.count\(request\.count\)/);
  assert.match(buildSearchQuery, /\.page\(request\.page\)/);
  assert.match(buildSearchQuery, /\.mode\(SearchMode::Quality\)/);
  assert.match(buildSearchQuery, /query\.media_type\(media_type\)/);

  const resultMapping = functionBody(bridgeSource, "from_search_results");
  assert.match(resultMapping, /providers_searched\.extend\(result\.providers_searched\)/);
  assert.match(resultMapping, /provider_errors\.extend\(result\.provider_errors\)/);
  assert.match(resultMapping, /total_count \+= result\.total_count/);
});

test("dx-media bridge maps rich media assets into panel-supported media kinds", () => {
  const mediaTypesForFilter = functionBody(bridgeSource, "media_types_for_filter");
  assert.match(mediaTypesForFilter, /PanelMediaKindFilter::Images => &\[MediaType::Image, MediaType::Gif, MediaType::Vector\]/);

  const panelKindMapping = functionBody(bridgeSource, "panel_kind_for_media_type");
  assert.match(panelKindMapping, /MediaType::Image \| MediaType::Gif \| MediaType::Vector/);
  assert.match(panelKindMapping, /MediaType::Video/);
  assert.match(panelKindMapping, /MediaType::Audio/);
  assert.match(panelKindMapping, /=> None/);

  const assetMapping = functionBody(bridgeSource, "panel_asset_from_media_asset");
  assert.match(assetMapping, /id: format!\("\{\}:\{\}", asset\.provider, asset\.id\)/);
  assert.match(assetMapping, /label: clean_panel_label\(&asset\.title\)/);
  assert.match(assetMapping, /provider: clean_panel_label\(&asset\.provider\)/);
  assert.match(assetMapping, /url: asset\.download_url/);
  assert.match(assetMapping, /thumbnail_url: asset\.preview_url/);
  assert.match(assetMapping, /license: clean_panel_label\(asset\.license\.as_str\(\)\)/);
  assert.match(assetMapping, /tags: asset\.tags\.join\(", "\)/);

  const renderableDownload = functionBody(bridgeSource, "is_panel_renderable_download");
  assert.match(renderableDownload, /DownloadUrlKind::DirectFile \| DownloadUrlKind::PreviewDerivative => true/);
  assert.match(renderableDownload, /DownloadUrlKind::Unknown => has_media_type_evidence\(asset\)/);
  assert.match(renderableDownload, /DownloadUrlKind::AssetManifest \| DownloadUrlKind::LandingPage/);

  const mediaTypeEvidence = functionBody(bridgeSource, "has_media_type_evidence");
  assert.match(mediaTypeEvidence, /matches_mime\(mime_type\)/);
  assert.match(mediaTypeEvidence, /matches_extension\(extension\)/);
});

test("media panel renders bridge state and filters fetched remote rows by query", () => {
  const render = panelSource.slice(panelSource.indexOf("impl Render for MediaPanel"));
  const matchingRemoteAssets = functionBody(panelSource, "matching_remote_assets");
  const remoteSignature = functionBody(panelSource, "media_remote_signature");
  const renderKindFilters = functionBody(panelSource, "render_kind_filters");
  const renderRemoteBrowserRow = functionBody(panelSource, "render_remote_browser_row");
  const renderRemoteWarningRow = functionBody(panelSource, "render_remote_warning_row");
  const renderStatusRow = functionBody(panelSource, "render_status_row");
  const renderRemoteHealthRow = functionBody(panelSource, "render_remote_health_row");
  const renderRemoteLoadingRow = functionBody(panelSource, "render_remote_loading_row");
  const renderUrlInsert = functionBody(panelSource, "render_url_insert");
  const renderAssetRow = functionBody(panelSource, "render_asset_row");
  const renderRemoteAssetRow = functionBody(panelSource, "render_remote_asset_row");
  const renderRecentMediaSection = functionBody(panelSource, "render_recent_media_section");
  const renderPinnedMediaSection = functionBody(panelSource, "render_pinned_media_section");
  const renderMediaHistoryRow = functionBody(panelSource, "render_media_history_row");
  const remotePanelRows = [
    renderRemoteBrowserRow,
    renderRemoteWarningRow,
    renderStatusRow,
    renderRemoteHealthRow,
    renderRemoteLoadingRow,
  ].join("\n");

  assert.match(panelSource, /use ui::\{[\s\S]*ListHeader,[\s\S]*ListItem,[\s\S]*ListItemSpacing/);
  assert.match(panelSource, /fn render_status_row\(/);
  assert.match(render, /let status = self\.status\.clone\(\);/);
  assert.match(render, /render_status_row\(status, cx\)/);
  assert.match(renderRecentMediaSection, /ListHeader::new\("Recent"\)/);
  assert.match(renderRecentMediaSection, /\.start_slot\(Icon::new\(IconName::Clock\)\.size\(IconSize::Small\)\)/);
  assert.match(renderPinnedMediaSection, /ListHeader::new\("Pinned"\)/);
  assert.match(renderPinnedMediaSection, /\.start_slot\(Icon::new\(IconName::Star\)\.size\(IconSize::Small\)\)/);
  const historySectionChrome = `${renderRecentMediaSection}\n${renderPinnedMediaSection}`;
  assert.match(historySectionChrome, /\.end_slot\([\s\S]*Label::new\(availability_label\)/);
  assert.match(
    historySectionChrome,
    /IconButton::new\(\s*"media-panel-remove-missing-recent",\s*IconName::ListX/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\(\s*"media-panel-remove-missing-pinned",\s*IconName::ListX/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\("media-panel-clear-recent", IconName::Trash\)/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\("media-panel-clear-pinned", IconName::Trash\)/,
  );
  assert.match(historySectionChrome, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.doesNotMatch(
    historySectionChrome,
    /Button::new\("media-panel-(?:remove-missing|clear)-(?:recent|pinned)", "(?:Remove|Clear)"\)/,
  );
  assert.doesNotMatch(historySectionChrome, /LabelSize::XSmall|IconSize::XSmall|\.justify_between\(\)/);
  assert.match(renderRemoteBrowserRow, /ListItem::new\("media-panel-remote-browser-row"\)/);
  assert.match(renderRemoteWarningRow, /ListItem::new\("media-panel-remote-warning-row"\)/);
  assert.match(
    renderRemoteBrowserRow,
    /IconButton::new\("media-panel-browse-remote-row", IconName::ArrowUpRight\)/,
  );
  assert.match(renderRemoteBrowserRow, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.match(renderRemoteBrowserRow, /\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderRemoteBrowserRow, /\.icon_size\(IconSize::Small\)/);
  assert.match(renderRemoteBrowserRow, /Tooltip::text\("Open remote provider browser"\)/);
  assert.match(
    renderRemoteWarningRow,
    /IconButton::new\("media-panel-retry-remote-warning", IconName::RotateCw\)/,
  );
  assert.match(renderRemoteWarningRow, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.match(renderRemoteWarningRow, /\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderRemoteWarningRow, /\.icon_size\(IconSize::Small\)/);
  assert.match(renderRemoteWarningRow, /Tooltip::text\("Retry remote media search"\)/);
  assert.doesNotMatch(
    `${renderRemoteBrowserRow}\n${renderRemoteWarningRow}`,
    /Button::new\("media-panel-(?:browse-remote-row|retry-remote-warning)", "(?:Open|Retry)"\)/,
  );
  assert.match(renderStatusRow, /ListItem::new\("media-panel-status-row"\)/);
  assert.match(renderRemoteHealthRow, /ListItem::new\("media-panel-remote-health-row"\)/);
  assert.match(renderRemoteLoadingRow, /ListItem::new\("media-panel-remote-loading-row"\)/);
  assert.match(renderUrlInsert, /ListItem::new\("media-panel-url-insert-row"\)/);
  assert.match(renderUrlInsert, /\.inset\(true\)/);
  assert.match(renderUrlInsert, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderUrlInsert, /\.selectable\(false\)/);
  assert.match(renderUrlInsert, /\.start_slot\(Icon::new\(media_kind_icon\(kind\)\)\.size\(IconSize::Small\)\)/);
  assert.match(renderUrlInsert, /\.end_slot\([\s\S]*IconName::Ellipsis/);
  assert.match(renderUrlInsert, /\.end_slot_on_hover\(/);
  assert.match(renderUrlInsert, /\.occlude\(\)/);
  assert.match(renderUrlInsert, /gpui::MouseButton::Left/);
  assert.match(renderUrlInsert, /cx\.stop_propagation\(\);/);
  assert.match(renderUrlInsert, /IconButton::new\("media-panel-preview-url", IconName::Eye\)/);
  assert.match(renderUrlInsert, /IconButton::new\("media-panel-copy-url", IconName::Copy\)/);
  assert.match(renderUrlInsert, /IconButton::new\("media-panel-pin-url", IconName::Pin\)/);
  assert.match(renderUrlInsert, /IconButton::new\("media-panel-insert-url", IconName::Plus\)/);
  assert.match(renderUrlInsert, /\.style\(ButtonStyle::Filled\)/);
  assert.match(renderUrlInsert, /\.tooltip\(Tooltip::text\(row_tooltip\)\)/);
  assert.match(renderAssetRow, /let drag_payload = payload\.clone\(\);/);
  assert.match(renderAssetRow, /ListItem::new\(row_id\)/);
  assert.match(renderAssetRow, /\.inset\(true\)/);
  assert.match(renderAssetRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderAssetRow, /\.start_slot\(thumbnail\)/);
  assert.match(renderAssetRow, /\.end_slot\([\s\S]*IconName::Ellipsis/);
  assert.match(renderAssetRow, /\.end_slot_on_hover\(actions\)/);
  assert.doesNotMatch(
    renderAssetRow,
    /\.p_2\(\)|\.border_1\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)|\.hover\(\|style\| style\.bg\(cx\.theme\(\)\.colors\(\)\.element_hover\)\)/,
  );
  assert.match(renderRemoteAssetRow, /ListItem::new\(row_id\)/);
  assert.match(renderRemoteAssetRow, /\.inset\(true\)/);
  assert.match(renderRemoteAssetRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderRemoteAssetRow, /\.start_slot\(thumbnail\)/);
  assert.match(renderRemoteAssetRow, /\.end_slot\([\s\S]*IconName::Ellipsis/);
  assert.match(renderRemoteAssetRow, /\.end_slot_on_hover\(actions\)/);
  assert.doesNotMatch(
    renderRemoteAssetRow,
    /\.p_2\(\)|\.border_1\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)/,
  );
  assert.match(renderRemoteHealthRow, /IconButton::new\("media-panel-refresh-remote-health", IconName::RotateCw\)[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  for (const assetRow of [renderAssetRow, renderRemoteAssetRow]) {
    assert.match(assetRow, /\.occlude\(\)/);
    assert.match(assetRow, /gpui::MouseButton::Left/);
    assert.match(assetRow, /cx\.stop_propagation\(\);/);
    assert.match(assetRow, /\.shape\(ui::IconButtonShape::Square\)/);
    assert.match(assetRow, /\.icon_size\(IconSize::Small\)/);
    assert.match(assetRow, /IconButton::new\(preview_id, IconName::Eye\)/);
    assert.match(assetRow, /IconButton::new\(copy_id, IconName::Copy\)/);
    assert.match(assetRow, /IconButton::new\(pin_id, IconName::Pin\)/);
  }
  assert.match(renderAssetRow, /\.on_drag\(drag_payload,/);
  assert.match(renderRemoteAssetRow, /IconButton::new\(insert_id, IconName::Plus\)/);
  assert.match(renderRemoteAssetRow, /\.style\(ButtonStyle::Filled\)/);
  assert.doesNotMatch(
    `${renderAssetRow}\n${renderRemoteAssetRow}`,
    /Button::new\((?:preview_id|copy_id|pin_id|insert_id), "(?:Preview|Copy|Pin|Insert URL)"\)/,
  );
  assert.doesNotMatch(
    renderUrlInsert,
    /Button::new\("media-panel-(?:preview|copy|pin|insert)-url", "(?:Preview|Copy|Pin|Insert URL)"\)/,
  );
  assert.doesNotMatch(
    renderUrlInsert,
    /\.border_1\(\)|\.rounded_sm\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)|IconSize::XSmall/,
  );
  assert.match(renderMediaHistoryRow, /ListItem::new\(row_id\)/);
  assert.match(renderMediaHistoryRow, /\.inset\(true\)/);
  assert.match(renderMediaHistoryRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderMediaHistoryRow, /\.end_slot\(/);
  assert.match(renderMediaHistoryRow, /IconName::Ellipsis/);
  assert.match(renderMediaHistoryRow, /\.end_slot_on_hover\(/);
  assert.match(renderMediaHistoryRow, /\.occlude\(\)/);
  assert.match(renderMediaHistoryRow, /gpui::MouseButton::Left/);
  assert.match(renderMediaHistoryRow, /cx\.stop_propagation\(\);/);
  assert.match(renderMediaHistoryRow, /IconButton::new\(preview_id, IconName::Eye\)/);
  assert.match(renderMediaHistoryRow, /IconButton::new\(copy_id, IconName::Copy\)/);
  assert.match(renderMediaHistoryRow, /IconButton::new\(insert_id, IconName::Plus\)/);
  assert.match(renderMediaHistoryRow, /IconButton::new\(remove_id, IconName::Trash\)/);
  assert.match(renderMediaHistoryRow, /IconButton::new\(pin_id, pin_icon\)/);
  assert.match(renderMediaHistoryRow, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.match(renderMediaHistoryRow, /\.icon_size\(IconSize::Small\)/);
  assert.match(renderMediaHistoryRow, /\.style\(ButtonStyle::Filled\)/);
  assert.match(renderMediaHistoryRow, /panel\.preview_media_asset/);
  assert.match(renderMediaHistoryRow, /panel\.preview_media_url/);
  assert.match(renderMediaHistoryRow, /panel\.copy_media_source/);
  assert.match(renderMediaHistoryRow, /panel\.insert_media\(/);
  assert.match(renderMediaHistoryRow, /panel\.insert_media_url/);
  assert.match(renderMediaHistoryRow, /panel\.pin_media/);
  assert.match(renderMediaHistoryRow, /panel\.unpin_media/);
  assert.match(renderMediaHistoryRow, /panel\.remove_media_history_entry/);
  assert.doesNotMatch(renderMediaHistoryRow, /\.flex_wrap\(\)/);
  assert.doesNotMatch(
    renderMediaHistoryRow,
    /Button::new\((?:preview_id|copy_id|insert_id|remove_id|pin_id), "(?:Preview|Copy|Insert|Insert URL|Remove|Pin|Unpin)"\)/,
  );
  assert.match(renderMediaHistoryRow, /\.tooltip\(Tooltip::text\(row_tooltip\)\)/);
  assert.doesNotMatch(renderMediaHistoryRow, /\.border_1\(\)|\.rounded_sm\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)/);
  assert.match(remotePanelRows, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(remotePanelRows, /\.selectable\(false\)/);
  assert.doesNotMatch(remotePanelRows, /\.border_1\(\)|\.rounded\(/);
  assert.match(renderKindFilters, /IconButton::new\(\s*"media-panel-kind-prev",\s*IconName::ChevronLeft[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderKindFilters, /let kind_scroll_offset = self\.kind_scroll_handle\.offset\(\)\.x;/);
  assert.match(renderKindFilters, /let kind_scroll_max = self\.kind_scroll_handle\.max_offset\(\)\.x;/);
  assert.match(renderKindFilters, /let kind_tabs_scrollable = kind_scroll_max > px\(2\.\);/);
  assert.match(renderKindFilters, /let can_scroll_kind_tabs_back = kind_tabs_scrollable && kind_scroll_offset < px\(0\.\);/);
  assert.match(
    renderKindFilters,
    /let can_scroll_kind_tabs_forward =\s*kind_tabs_scrollable && kind_scroll_offset > -kind_scroll_max;/,
  );
  assert.match(renderKindFilters, /IconButton::new\(\s*"media-panel-kind-prev",\s*IconName::ChevronLeft[\s\S]*\.disabled\(!can_scroll_kind_tabs_back\)/);
  assert.match(renderKindFilters, /IconButton::new\(\s*"media-panel-kind-next",\s*IconName::ChevronRight[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderKindFilters, /IconButton::new\(\s*"media-panel-kind-next",\s*IconName::ChevronRight[\s\S]*\.disabled\(!can_scroll_kind_tabs_forward\)/);
  assert.match(render, /IconButton::new\(\s*"media-panel-refresh-remote",\s*IconName::RotateCw[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(render, /IconButton::new\(\s*"media-panel-remove-missing-history",\s*IconName::Trash[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(
    matchingRemoteAssets,
    /if !query_terms\.is_empty\(\) && !remote_media_search_matches\(asset, query_terms\)/,
  );
  assert.match(remoteSignature, /push_lowercase\(&mut signature, query\.trim\(\)\)/);
});
