import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/font_panel/src/font_panel.rs";
const metadataPath = "crates/font_panel/src/font_metadata.rs";

const productionSource = (source: string) =>
  source.split(/\r?\n#\[cfg\(test\)\]\r?\nmod tests\s*\{/)[0] ?? source;

const readIfExists = (path: string) => (existsSync(path) ? readFileSync(path, "utf8") : "");

const source = productionSource(readIfExists(sourcePath));
const metadataSource = productionSource(readIfExists(metadataPath));

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

test("font panel declares focused font materialization bounds", () => {
  assert.match(source, /const MAX_CUSTOM_WEB_FONT_NAME_WORDS: usize = 6;/);
  assert.match(source, /const MAX_CUSTOM_WEB_FONT_WORD_CHARS: usize = 24;/);
  assert.match(source, /const MAX_CUSTOM_WEB_FONT_NAME_CHARS: usize = 96;/);
  assert.match(source, /const MAX_FONT_ELEMENT_ID_VALUE_CHARS: usize = 96;/);
  assert.match(source, /const MAX_FONT_PREVIEW_FILE_STEM_CHARS: usize = 96;/);
});

test("custom web font names are bounded before CSS, status, preview, and row materialization", () => {
  const customWebFontName = functionBody(source, "custom_web_font_name");
  const pushCustomWord = functionBody(source, "push_custom_web_font_word");
  const specByName = functionBody(source, "web_font_spec_by_name");

  assert.match(
    customWebFontName,
    /String::with_capacity\(MAX_CUSTOM_WEB_FONT_NAME_CHARS\)/,
  );
  assert.match(customWebFontName, /\.take\(MAX_CUSTOM_WEB_FONT_NAME_WORDS\)/);
  assertBefore(
    customWebFontName,
    "push_custom_web_font_word(&mut name, word, &mut name_chars);",
    "(!name.is_empty()).then_some(name)",
    "custom query names must be bounded before becoming a FontEntry/WebFontSpec name",
  );
  assert.match(pushCustomWord, /MAX_CUSTOM_WEB_FONT_NAME_CHARS/);
  assert.match(pushCustomWord, /MAX_CUSTOM_WEB_FONT_WORD_CHARS/);
  assert.match(pushCustomWord, /to_uppercase\(\)/);
  assertBefore(
    specByName,
    ".or_else(|| custom_web_font_name(name))?",
    "family_query: google_font_family_query(&name)",
    "web font specs must use the bounded custom name before CSS query materialization",
  );
});

test("preview file stems and element IDs compact oversized font names", () => {
  const elementId = functionBody(source, "font_element_id");
  const previewStem = functionBody(source, "font_preview_file_stem");

  assert.match(elementId, /MAX_FONT_ELEMENT_ID_VALUE_CHARS/);
  assert.match(elementId, /stable_text_hash\(id\)/);
  assert.match(elementId, /\.take\(MAX_FONT_ELEMENT_ID_VALUE_CHARS\)/);
  assert.match(previewStem, /MAX_FONT_PREVIEW_FILE_STEM_CHARS/);
  assert.match(previewStem, /stable_text_hash\(font_name\)/);
  assert.match(previewStem, /\.take\(MAX_FONT_PREVIEW_FILE_STEM_CHARS\)/);
});

test("font panel warms the system font cache before first render", () => {
  const newPanel = functionBody(source, "new");
  const ensureSystemFontsLoading = functionBody(source, "ensure_system_fonts_loading");
  const spawnSystemFontsLoading = functionBody(source, "spawn_system_fonts_loading");
  const render = source.slice(source.indexOf("impl Render for FontPanel"));

  assertBefore(
    newPanel,
    /let cached_fonts = Self::cached_fonts\(cx, selected_font\.clone\(\)\);/,
    /let loading_fonts = cached_fonts\.needs_live_refresh;/,
    "font panel construction must derive the loading flag from the global font cache or durable metadata state",
  );
  assertBefore(
    newPanel,
    /let loading_fonts = cached_fonts\.needs_live_refresh;/,
    /Self::spawn_system_fonts_loading\(cx\);/,
    "font panel construction must start system font cache warmup when the cache is cold",
  );
  assertBefore(
    newPanel,
    /Self::spawn_system_fonts_loading\(cx\);/,
    /loading_fonts,/,
    "the panel state must remember the construction-time warmup task",
  );
  assert.match(
    ensureSystemFontsLoading,
    /if self\.fonts_loaded \|\| self\.loading_fonts[\s\S]*return;/,
    "render-time warmup fallback must avoid duplicate font prefetch tasks",
  );
  assert.match(ensureSystemFontsLoading, /Self::spawn_system_fonts_loading\(cx\);/);
  assert.match(spawnSystemFontsLoading, /FontFamilyCache::global\(cx\)/);
  assert.match(spawnSystemFontsLoading, /font_family_cache\.prefetch\(cx\)\.await;/);
  assert.match(spawnSystemFontsLoading, /panel\.loading_fonts = false;/);
  assertBefore(
    render,
    /self\.ensure_system_fonts_loading\(cx\);/,
    /self\.refresh_fonts_if_needed\(cx\);/,
    "render should keep the fallback warmup before cache refresh",
  );
});

test("font panel uses durable bounded system font metadata while live cache warms", () => {
  const newPanel = functionBody(source, "new");
  const cachedFonts = functionBody(source, "cached_fonts");
  const spawnSystemFontsLoading = functionBody(source, "spawn_system_fonts_loading");
  const refreshFontsIfNeeded = functionBody(source, "refresh_fonts_if_needed");
  const loadSystemFontMetadata = functionBody(metadataSource, "load_system_font_metadata");
  const parseSystemFontMetadata = functionBody(metadataSource, "parse_system_font_metadata");
  const persistSystemFontMetadata = functionBody(metadataSource, "persist_system_font_metadata");
  const boundedSystemFontName = functionBody(metadataSource, "bounded_system_font_name");

  assert.match(source, /mod font_metadata;/);
  assert.match(source, /struct CachedSystemFonts/);
  assert.match(source, /enum SystemFontListSource/);
  assert.match(metadataSource, /const DX_SYSTEM_FONT_METADATA_KEY: &str = "dx_font_panel_system_fonts";/);
  assert.match(metadataSource, /const DX_SYSTEM_FONT_METADATA_SCHEMA_VERSION: u32 = 1;/);
  assert.match(metadataSource, /const MAX_SYSTEM_FONT_METADATA_ENTRIES: usize = 4096;/);
  assert.match(metadataSource, /const MAX_SYSTEM_FONT_METADATA_NAME_CHARS: usize = 160;/);
  assert.match(metadataSource, /const MAX_SYSTEM_FONT_METADATA_JSON_BYTES: usize = 512 \* 1024;/);
  assert.match(metadataSource, /struct SerializedSystemFontMetadataArtifact/);

  assertBefore(
    cachedFonts,
    /FontFamilyCache::global\(cx\)\.try_list_font_families\(\)/,
    /font_metadata::load_system_font_metadata\(cx\)/,
    "live system font cache must be preferred before durable metadata",
  );
  assert.match(cachedFonts, /SystemFontListSource::LiveCache/);
  assert.match(cachedFonts, /SystemFontListSource::DurableMetadata/);
  assert.match(cachedFonts, /SystemFontListSource::CurrentSelection/);
  assert.match(cachedFonts, /needs_live_refresh: true/);

  assertBefore(
    newPanel,
    /let cached_fonts = Self::cached_fonts\(cx, selected_font\.clone\(\)\);/,
    /let loading_fonts = cached_fonts\.needs_live_refresh;/,
    "construction must derive loading state from durable-metadata freshness",
  );
  assert.match(newPanel, /cached_fonts\.source == SystemFontListSource::LiveCache/);
  assert.match(newPanel, /font_metadata::persist_system_font_metadata\(&cached_fonts\.fonts, cx\);/);

  assert.match(
    spawnSystemFontsLoading,
    /let fonts = Self::sort_fonts\(fonts\);[\s\S]*font_metadata::persist_system_font_metadata\(&fonts, cx\);[\s\S]*panel\.fonts = fonts;/,
    "background live-cache warmup must persist the sorted real font list before replacing panel state",
  );
  assert.match(
    refreshFontsIfNeeded,
    /font_metadata::persist_system_font_metadata\(&fonts, cx\);/,
    "render fallback refresh must also update durable metadata when the live cache becomes available",
  );

  assert.match(loadSystemFontMetadata, /KeyValueStore::global\(cx\)[\s\S]*read_kvp\(DX_SYSTEM_FONT_METADATA_KEY\)/);
  assert.match(parseSystemFontMetadata, /json\.len\(\) > MAX_SYSTEM_FONT_METADATA_JSON_BYTES/);
  assert.match(parseSystemFontMetadata, /schema_version != DX_SYSTEM_FONT_METADATA_SCHEMA_VERSION/);
  assert.match(parseSystemFontMetadata, /\.take\(MAX_SYSTEM_FONT_METADATA_ENTRIES\)/);
  assert.match(parseSystemFontMetadata, /bounded_system_font_name/);
  assert.match(persistSystemFontMetadata, /\.take\(MAX_SYSTEM_FONT_METADATA_ENTRIES\)/);
  assert.match(persistSystemFontMetadata, /serde_json::to_string/);
  assert.match(persistSystemFontMetadata, /write_kvp\(DX_SYSTEM_FONT_METADATA_KEY\.to_string\(\), json\)/);
  assert.match(boundedSystemFontName, /MAX_SYSTEM_FONT_METADATA_NAME_CHARS/);
});

test("font panel source guard stays scoped to worker-owned files", () => {
  assert.deepEqual(
    [sourcePath, metadataPath],
    ["crates/font_panel/src/font_panel.rs", "crates/font_panel/src/font_metadata.rs"],
  );
  assert.doesNotMatch(`${sourcePath}\n${metadataPath}`, /DX\.md|todo\.txt|changelog\.txt/);
  assert.doesNotMatch(
    `${source}\n${metadataSource}`,
    /#\[cfg\(test\)\]/,
    "source guard should only inspect production font panel code",
  );
});
