import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/icon_picker/src/icon_picker.rs";
const editorSourcePath = "crates/editor/src/items.rs";
const uiIconPathsSourcePath = "crates/ui/src/dx_icon_paths.rs";

const productionSource = (source: string) =>
  source.split(/\r?\n#\[cfg\(test\)\]\r?\nmod tests\s*\{/)[0] ?? source;

const source = productionSource(readFileSync(sourcePath, "utf8"));
const editorSource = productionSource(readFileSync(editorSourcePath, "utf8"));
const uiIconPathsSource = productionSource(readFileSync(uiIconPathsSourcePath, "utf8"));

function functionBody(source: string, name: string): string {
  const start = source.search(new RegExp(`fn\\s+${name}(?:\\s*<[^>]+>)?\\s*\\(`));
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
}

function assertBefore(
  haystack: string,
  before: string | RegExp,
  after: string | RegExp,
  message: string,
) {
  const beforeIndex =
    typeof before === "string"
      ? haystack.indexOf(before)
      : haystack.match(before)?.index ?? -1;
  const afterIndex =
    typeof after === "string"
      ? haystack.indexOf(after)
      : haystack.match(after)?.index ?? -1;
  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
}

test("icon picker bounds TSV sample columns before pack summaries store them", () => {
  assert.match(source, /const MAX_ICON_PACK_SAMPLE_NAMES: usize = 2;/);

  const sampleHelper = functionBody(source, "icon_pack_sample_names");
  assert.match(sampleHelper, /Vec::with_capacity\(MAX_ICON_PACK_SAMPLE_NAMES\)/);
  assertBefore(
    sampleHelper,
    ".filter(|name| !name.is_empty())",
    ".take(MAX_ICON_PACK_SAMPLE_NAMES)",
    "empty sample columns should be discarded before the sample cap is applied",
  );
  assert.match(sampleHelper, /\.map\(SharedString::from\)/);

  const staticSummaries = functionBody(source, "static_icon_pack_summaries");
  assert.match(staticSummaries, /let sample_names = icon_pack_sample_names\(columns\);/);
  assert.doesNotMatch(
    staticSummaries,
    /Vec::with_capacity\(columns\.size_hint\(\)\.0\)/,
    "TSV row width must not drive sample-name allocation",
  );
});

test("icon picker representative loops share the sample-column cap", () => {
  const representativeSummaries = functionBody(
    source,
    "representative_icons_from_pack_summaries",
  );
  const externalCatalog = functionBody(source, "load_external_icon_catalog");

  assert.match(representativeSummaries, /for index in 0\.\.MAX_ICON_PACK_SAMPLE_NAMES/);
  assert.match(externalCatalog, /for index in 0\.\.MAX_ICON_PACK_SAMPLE_NAMES/);
  assert.doesNotMatch(
    `${representativeSummaries}\n${externalCatalog}`,
    /for index in 0\.\.2/,
    "representative rendering should stay tied to the bounded TSV sample count",
  );
});

test("icon picker tile selection is explicit and non-mutating", () => {
  const selectIcon = functionBody(source, "select_icon");
  const renderIconTile = functionBody(source, "render_icon_tile");
  const renderSelectedIconActions = functionBody(source, "render_selected_icon_actions");
  const insertSelectedIcon = functionBody(source, "insert_selected_icon");
  const insertIcon = functionBody(source, "insert_icon");
  const copySelectedIconName = functionBody(source, "copy_selected_icon_name");

  assert.match(selectIcon, /self\.selected_icon = Some\(icon\);/);
  assert.match(selectIcon, /icon_status_label\("Selected "/);
  assert.doesNotMatch(
    selectIcon,
    /insert_icon|insert_selected_icon|copy_icon_name|record_recent_icon_action|active_item_as::<Editor>|insert_icon_asset/,
    "selecting a tile must not insert, copy, or mutate editor contents",
  );

  assert.match(renderIconTile, /\.h\(px\(66\.\)\)/);
  assert.match(
    renderIconTile,
    /\.on_click\([\s\S]*panel\.select_icon\(icon\.clone\(\), cx\);[\s\S]*\)/,
    "tile clicks must select only; insertion lives in the selected action strip",
  );
  assert.doesNotMatch(
    renderIconTile,
    /panel\.(insert_icon|insert_selected_icon|copy_icon_name|copy_selected_icon_name)/,
    "tile click handlers must not perform selected-icon actions implicitly",
  );

  assert.match(source, /selected_icon:\s*Option<PickerIcon>/);
  assert.match(renderSelectedIconActions, /\.id\("icon-picker-selected-icon-actions"\)/);
  assert.match(
    renderSelectedIconActions,
    /IconButton::new\("icon-picker-copy-selected"[\s\S]*copy_selected_icon_name/,
  );
  assert.match(
    renderSelectedIconActions,
    /IconButton::new\("icon-picker-insert-selected"[\s\S]*insert_selected_icon/,
  );
  assert.match(
    renderSelectedIconActions,
    /IconButton::new\("icon-picker-pin-selected"[\s\S]*pin_selected_icon/,
  );
  assert.match(insertSelectedIcon, /self\.insert_icon\(icon, window, cx\)/);
  assert.match(insertIcon, /active_item_as::<Editor>[\s\S]*editor\.insert_icon_asset/);
  assert.match(copySelectedIconName, /self\.copy_icon_name\(icon, cx\)/);
});

test("icon insertion guards supported editors and safe React asset names", () => {
  const insertIconAsset = functionBody(editorSource, "insert_icon_asset");
  const insertReactIconAsset = functionBody(editorSource, "insert_react_icon_asset");
  const insertIconAssetOnDrop = functionBody(editorSource, "insert_icon_asset_on_drop");
  const insertReactIconAssetOnDrop = functionBody(editorSource, "insert_react_icon_asset_on_drop");
  const isReactEditorPath = functionBody(editorSource, "is_react_editor_path");
  const isBasicIconEditorPath = functionBody(editorSource, "is_basic_icon_editor_path");
  const iconAssetFileName = functionBody(editorSource, "icon_asset_file_name");
  const reactIconComponentName = functionBody(editorSource, "react_icon_component_name");

  assert.match(insertIconAsset, /target_file_abs_path_for_app\(self, cx\)/);
  assertBefore(
    insertIconAsset,
    /is_react_editor_path\(&active_path\)/,
    /insert_react_icon_asset\(/,
    "React icon insertion must be gated by TSX or JSX paths",
  );
  assertBefore(
    insertIconAsset,
    /!is_basic_icon_editor_path\(&active_path\)/,
    /self\.insert\(&svg, window, cx\)/,
    "unsupported file paths must error before raw SVG insertion",
  );
  assert.match(
    insertIconAsset,
    /TSX, JSX, HTML, SVG, Markdown, MDX, and untitled editors/,
  );
  assertBefore(
    insertIconAssetOnDrop,
    /!is_basic_icon_editor_path\(&active_path\)/,
    /self\.insert_text_on_drop\(svg, cx\)/,
    "unsupported drag/drop paths must error before raw SVG insertion",
  );

  assert.match(isReactEditorPath, /Some\("tsx" \| "jsx"\)/);
  assert.match(isBasicIconEditorPath, /Some\("html" \| "htm" \| "svg" \| "md" \| "mdx"\)/);
  assert.match(
    insertReactIconAsset,
    /asset_dir\.join\(icon_asset_file_name\(icon\.stem\.as_ref\(\)\)\)/,
  );
  assert.match(
    insertReactIconAssetOnDrop,
    /asset_dir\.join\(icon_asset_file_name\(icon\.stem\.as_ref\(\)\)\)/,
  );
  assert.doesNotMatch(
    `${insertReactIconAsset}\n${insertReactIconAssetOnDrop}`,
    /asset_dir\.join\(format!\("\{\}\.svg"/,
    "React icon asset paths must not be built from raw stem filenames",
  );

  assert.match(iconAssetFileName, /ch\.is_ascii_alphanumeric\(\)/);
  assert.match(iconAssetFileName, /file_name\.push\('-'\)/);
  assert.match(iconAssetFileName, /while file_name\.ends_with\('-'\)/);
  assert.match(iconAssetFileName, /file_name\.push_str\("icon"\)/);
  assert.match(iconAssetFileName, /file_name\.push_str\("\.svg"\)/);

  assert.match(reactIconComponentName, /push_react_component_name_segment/);
  assert.match(reactIconComponentName, /name\.insert_str\(0, "Svg"\)/);
  assert.match(reactIconComponentName, /name\.push_str\("Icon"\)/);
});

test("DX icon data resolves through the shared DX icon source order", () => {
  const candidates = functionBody(uiIconPathsSource, "dx_icon_data_dir_candidates");
  const resolver = functionBody(uiIconPathsSource, "resolve_dx_icon_data_dir");
  const pickerDataDir = functionBody(source, "external_icon_data_dir");
  const iconifySvgSource = functionBody(editorSource, "iconify_svg_source");

  assertBefore(candidates, "DX_ICON_INDEX_ENV", "DX_ICON_DATA_ENV", "DX_ICON_INDEX should win first");
  assertBefore(candidates, "DX_ICON_DATA_ENV", "DX_ICON_ROOT_ENV", "DX_ICON_DATA should win before root");
  assertBefore(
    candidates,
    "DX_ICON_ROOT_ENV",
    "LEGACY_DX_ICONS_DATA_DIR_ENV",
    "DX_ICON_ROOT should win before the legacy env",
  );
  assertBefore(
    candidates,
    "LEGACY_DX_ICONS_DATA_DIR_ENV",
    "DX_HOME_ENV",
    "legacy env should stay only as compatibility after DX source vars",
  );
  assertBefore(candidates, "DX_HOME_ENV", "SHARED_DX_ICON_ROOT", "DX_HOME icon should win before shared fallback");
  assertBefore(candidates, "SHARED_DX_ICON_ROOT", "USERPROFILE_ENV", "shared DX checkout should win before user fallback");

  assert.match(
    resolver,
    /file_name\(\)\s*\.is_some_and\(\|name\| name == OsStr::new\("data"\)\)/,
  );
  assert.match(resolver, /candidate\.join\("data"\)/);
  assert.match(resolver, /OsStr::new\("index"\)/);
  assert.match(resolver, /root\.join\("data"\)/);
  assert.match(pickerDataDir, /dx_icon_data_dir\(\)/);
  assert.match(iconifySvgSource, /dx_icon_data_dir\(\)\.join\(format!\("\{pack\}\.json"\)\)/);

  const combinedProduction = `${source}\n${editorSource}\n${uiIconPathsSource}`;
  assert.doesNotMatch(combinedProduction, /G:\/Assets\/icon\/data|DX_ICON_DATA_DIR/);
});

test("icon picker source guard is focused on production editor code", () => {
  assert.equal(sourcePath, "crates/icon_picker/src/icon_picker.rs");
  assert.equal(editorSourcePath, "crates/editor/src/items.rs");
  assert.equal(uiIconPathsSourcePath, "crates/ui/src/dx_icon_paths.rs");
  assert.doesNotMatch(sourcePath, /test/i);
  assert.doesNotMatch(editorSourcePath, /test/i);
  assert.doesNotMatch(uiIconPathsSourcePath, /test/i);
  assert.doesNotMatch(
    source,
    /#\[cfg\(test\)\]/,
    "source guard should only inspect production icon picker code",
  );
  assert.doesNotMatch(
    editorSource,
    /#\[cfg\(test\)\]/,
    "source guard should only inspect production editor insertion code",
  );
  assert.doesNotMatch(
    uiIconPathsSource,
    /#\[cfg\(test\)\]/,
    "source guard should only inspect production DX icon path code",
  );
});
