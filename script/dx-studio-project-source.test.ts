import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (source: string) => source.split(/\r?\n/).length;
const productionSource = (source: string) =>
  source.split(/\r?\n#\[cfg\(test\)\]\r?\nmod tests\s*\{/)[0] ?? source;

const directReadToString = /\b(?:fs|std::fs)::read_to_string\s*\(/;

test("DX Studio project detection keeps marker and Cargo.toml reads bounded", () => {
  const source = read("crates/web_preview/src/dx_studio/project.rs");

  assert.ok(lineCount(source) < 290, "project detection should stay compact");
  assert.doesNotMatch(source, directReadToString);
  assert.match(source, /const MAX_DX_CARGO_TOML_SCAN_BYTES: u64 = 256 \* 1024;/);
  assert.match(source, /const MAX_DX_MARKER_SCAN_DIRS: usize = 128;/);
  assert.match(source, /const MAX_DX_MARKER_SCAN_VISITED_DIRS: usize = 256;/);
  assert.match(
    source,
    /use std::\{[\s\S]*collections::HashSet,[\s\S]*path::\{Path, PathBuf\},[\s\S]*\}/,
    "DX Studio marker scan must keep an explicit visited-directory set",
  );
  assert.match(
    source,
    /struct DxMarkerScanBudget \{[\s\S]*files_left: usize,[\s\S]*dirs_left: usize,[\s\S]*visited_dirs: HashSet<PathBuf>,[\s\S]*\}/,
    "DX Studio marker scan must share file, directory, and visited-directory caps across source roots",
  );
  assert.match(
    source,
    /fn try_visit_dir\(&mut self, dir: &Path\) -> bool \{[\s\S]*self\.dirs_left == 0[\s\S]*self\.visited_dirs\.len\(\) >= MAX_DX_MARKER_SCAN_VISITED_DIRS[\s\S]*fs::canonicalize\(dir\)[\s\S]*self\.visited_dirs\.insert\(dir_key\)[\s\S]*self\.dirs_left -= 1/,
    "DX Studio marker scan must cap visited directories and dedupe canonical directory identities",
  );
  assert.match(
    source,
    /fn read_bounded_utf8_file\(path: &Path, max_bytes: u64\) -> Option<String>/,
  );
  assert.match(source, /\.take\(max_bytes \+ 1\)/);
  assert.match(source, /\.read_to_end\(&mut bytes\)/);
  assert.match(source, /bytes\.len\(\) as u64 > max_bytes/);
  assert.match(source, /cargo_toml_contains_dx_www_marker\(&cargo_toml\)/);
  assert.match(
    source,
    /read_bounded_utf8_file\(path, MAX_DX_MARKER_SCAN_BYTES\)/,
  );
  assert.match(
    source,
    /read_bounded_utf8_file\(path, MAX_DX_CARGO_TOML_SCAN_BYTES\)/,
  );
  assert.match(
    source,
    /let mut budget = DxMarkerScanBudget::new\(\);[\s\S]*dx_marker_source_dir_contains_marker\(&source_root, &mut budget\)/,
    "DX Studio marker scan must reuse one bounded budget across app, pages, components, src, and launch-template roots",
  );
  assert.match(
    source,
    /fn dx_marker_source_dir_contains_marker\(root: &Path, budget: &mut DxMarkerScanBudget\) -> bool[\s\S]*if !budget\.try_visit_dir\(&dir\) \{[\s\S]*continue;[\s\S]*\}[\s\S]*fs::read_dir\(&dir\)/,
    "DX Studio marker scan must spend directory budget before reading each directory",
  );
  assert.match(
    source,
    /file_type\.is_file\(\) && is_dx_marker_source_file\(&path\)[\s\S]*if !budget\.take_file\(\) \{[\s\S]*return false;[\s\S]*\}[\s\S]*dx_marker_source_file_contains_marker\(&path\)/,
    "DX Studio marker scan must spend file budget before opening candidate source files",
  );
  assert.doesNotMatch(
    source,
    /let mut files_left = MAX_DX_MARKER_SCAN_FILES|dx_marker_source_dir_contains_marker\(&source_root, &mut files_left\)/,
    "DX Studio marker scan must not use a file-only budget that leaves directory traversal unbounded",
  );
});

test("DX Studio source edits bound source file reads before UTF-8 decoding", () => {
  const fullSource = read("crates/web_preview/src/dx_studio_source_edit.rs");
  const source = productionSource(fullSource);

  assert.ok(lineCount(fullSource) < 580, "source edit root should stay compact");
  assert.doesNotMatch(source, directReadToString);
  assert.match(source, /fn read_source_file_for_edit\(source: &Path\) -> Result<String>/);
  assert.match(source, /\.take\(DX_STUDIO_MAX_SOURCE_FILE_BYTES \+ 1\)/);
  assert.match(source, /\.read_to_end\(&mut bytes\)/);
  assert.match(source, /bytes\.len\(\) as u64 > DX_STUDIO_MAX_SOURCE_FILE_BYTES/);
  assert.match(source, /String::from_utf8\(bytes\)/);
  assert.match(source, /read_source_file_for_edit\(&source\)\?/);

  const sizeValidation = source.indexOf(
    "ensure_source_file_size_allows_edit(&source, metadata.len())?",
  );
  const boundedRead = source.indexOf("read_source_file_for_edit(&source)?");
  const overLimitCheck = source.indexOf(
    "bytes.len() as u64 > DX_STUDIO_MAX_SOURCE_FILE_BYTES",
  );
  const utf8Decode = source.indexOf("String::from_utf8(bytes)");

  assert.ok(sizeValidation >= 0, "source edit should validate metadata size first");
  assert.ok(
    boundedRead > sizeValidation,
    "source edit should read the source only after metadata size validation",
  );
  assert.ok(overLimitCheck >= 0, "source edit should reject sentinel over-limit reads");
  assert.ok(
    utf8Decode > overLimitCheck,
    "source edit should reject over-limit reads before UTF-8 decoding",
  );
});
