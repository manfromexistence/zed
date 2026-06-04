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
  assert.match(fetchRemoteMediaAssets, /if !result\.assets\.is_empty\(\)/);
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

  assert.match(panelSource, /fn render_status_row\(/);
  assert.match(render, /let status = self\.status\.clone\(\);/);
  assert.match(render, /render_status_row\(status, cx\)/);
  assert.match(
    matchingRemoteAssets,
    /if !query_terms\.is_empty\(\) && !remote_media_search_matches\(asset, query_terms\)/,
  );
});
