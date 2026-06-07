import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

const functionBody = (source: string, name: string) => {
  const start = source.indexOf(`fn ${name}(`);
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = source.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const character = source[index];
    if (character === "{") {
      depth += 1;
    } else if (character === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
};

test("DX source sets keep receipt IO and JSON field helpers in focused modules", () => {
  const parentPath = "crates/agent_ui/src/dx_source_sets.rs";
  const attachmentSummaryPath =
    "crates/agent_ui/src/dx_source_sets/attachment_summary.rs";
  const cachePath = "crates/agent_ui/src/dx_source_sets/cache.rs";
  const formattingPath = "crates/agent_ui/src/dx_source_sets/formatting.rs";
  const receiptsPath = "crates/agent_ui/src/dx_source_sets/receipts.rs";
  const fieldsPath = "crates/agent_ui/src/dx_source_sets/receipt_fields.rs";
  const restorePath = "crates/agent_ui/src/dx_source_sets/restore.rs";

  assert.ok(
    existsSync(attachmentSummaryPath),
    "missing focused source-set attachment summary module",
  );
  assert.ok(existsSync(cachePath), "missing focused source-set cache module");
  assert.ok(existsSync(formattingPath), "missing focused source-set formatting module");
  assert.ok(existsSync(receiptsPath), "missing focused source-set receipt IO module");
  assert.ok(existsSync(fieldsPath), "missing focused source-set JSON field helper module");
  assert.ok(existsSync(restorePath), "missing focused source-set restore warning module");

  const parent = read(parentPath);
  const attachmentSummary = read(attachmentSummaryPath);
  const cache = read(cachePath);
  const formatting = read(formattingPath);
  const receipts = read(receiptsPath);
  const fields = read(fieldsPath);
  const restore = read(restorePath);

  assert.match(parent, /^mod attachment_summary;$/m);
  assert.match(parent, /^mod cache;$/m);
  assert.match(parent, /^mod formatting;$/m);
  assert.match(parent, /^mod receipt_fields;$/m);
  assert.match(parent, /^mod receipts;$/m);
  assert.match(parent, /^mod restore;$/m);
  assert.match(parent, /pub\(crate\) use self::attachment_summary::DxSourceAttachmentSummary;/);
  assert.match(parent, /pub\(crate\) use self::cache::invalidate_source_set_snapshot_cache;/);
  assert.match(parent, /use self::cache::\{cached_source_set_snapshot, store_source_set_snapshot\};/);
  assert.match(
    parent,
    /use self::formatting::\{display_name, format_bytes, short_hash, source_set_status\};/,
  );
  assert.match(parent, /use self::receipt_fields::\{/);
  assert.match(parent, /use self::receipts::\{ReceiptCandidate, latest_receipts, read_receipt_json\};/);
  assert.match(parent, /use self::restore::forge_restore_warnings;/);
  assert.doesNotMatch(parent, /pub\(crate\) struct DxSourceAttachmentSummary/);
  assert.doesNotMatch(parent, /fn attachment_summary\(/);
  assert.doesNotMatch(parent, /fn latest_receipts\(/);
  assert.doesNotMatch(parent, /fn read_receipt_json\(/);
  assert.doesNotMatch(parent, /fn value_at</);
  assert.doesNotMatch(parent, /fn format_bytes\(/);
  assert.doesNotMatch(parent, /fn source_set_status\(/);
  assert.doesNotMatch(parent, /fn forge_restore_warnings\(/);
  assert.match(attachmentSummary, /pub\(crate\) struct DxSourceAttachmentSummary/);
  assert.match(attachmentSummary, /impl DxSourceSetSnapshot/);
  assert.match(attachmentSummary, /pub\(crate\) fn attachment_summary/);
  assert.match(formatting, /pub\(super\) fn display_name/);
  assert.match(formatting, /pub\(super\) fn format_bytes/);
  assert.match(formatting, /pub\(super\) fn source_set_status/);
  assert.match(receipts, /pub\(super\) struct ReceiptCandidate/);
  assert.match(receipts, /pub\(super\) fn latest_receipts/);
  assert.match(receipts, /pub\(super\) fn read_receipt_json/);
  assert.match(receipts, /const LATEST_RECEIPT_ROOT_ENTRY_LIMIT: usize = 128;/);
  assert.match(receipts, /entries\.flatten\(\)\.take\(LATEST_RECEIPT_ROOT_ENTRY_LIMIT\)/);
  assert.match(fields, /pub\(super\) fn string_at/);
  assert.match(fields, /pub\(super\) fn array_strings_at/);
  assert.match(restore, /pub\(super\) fn forge_restore_warnings/);
  assert.match(restore, /target_mutation_applied/);
  assert.match(cache, /SOURCE_SET_CACHE_TTL/);
  assert.match(cache, /pub\(super\) fn cached_source_set_snapshot/);
  assert.match(cache, /pub\(super\) fn store_source_set_snapshot/);
  assert.match(cache, /pub\(crate\) fn invalidate_source_set_snapshot_cache/);

  assert.ok(lineCount(parentPath) < 420, "dx_source_sets.rs should stay a coordinator");
  assert.ok(
    lineCount(attachmentSummaryPath) < 55,
    "source-set attachment summary module should stay small",
  );
  assert.ok(lineCount(cachePath) < 60, "source-set cache module should stay small");
  assert.ok(lineCount(formattingPath) < 55, "source-set formatting module should stay small");
  assert.ok(lineCount(receiptsPath) < 90, "source-set receipt IO module should stay small");
  assert.ok(lineCount(fieldsPath) < 70, "source-set field helper module should stay small");
  assert.ok(lineCount(restorePath) < 50, "source-set restore warning module should stay small");
});

test("DX source sets surface receipt-backed proof and missing media honestly", () => {
  const sourceSets = read("crates/agent_ui/src/dx_source_sets.rs");
  const attachments = read("crates/agent/src/dx_source_attachment.rs");
  const metasearchSource = functionBody(sourceSets, "metasearch_source_from_receipt");
  const mediaSourceRows = functionBody(sourceSets, "media_sources_from_receipt");
  const mediaAttachments = functionBody(attachments, "media_sources_from_receipt");

  assert.match(metasearchSource, /proofs: vec!\[format!\("Source-pack receipt \{\}", receipt\.label\)\]/);

  assert.match(mediaSourceRows, /let file_exists = Path::new\(&open_path\)\.is_file\(\);/);
  assert.match(mediaSourceRows, /let receipt_declared_exists = bool_at\(file, &\["exists"\]\);/);
  assert.match(mediaSourceRows, /if !file_exists \{/);
  assert.match(mediaSourceRows, /Produced file is missing; generation cannot be claimed from this receipt\./);
  assert.match(mediaSourceRows, /Receipt declared the file existed, but the file is missing on disk\./);
  assert.doesNotMatch(
    mediaSourceRows,
    /bool_at\(file, &\["exists"\]\)\.unwrap_or_else\(\|\| Path::new\(&open_path\)\.is_file\(\)\)/,
  );
  assert.doesNotMatch(mediaSourceRows, /if !exists \{\s*return None;\s*\}/);

  assert.match(mediaAttachments, /let file_exists = Path::new\(&path\)\.is_file\(\);/);
  assert.match(mediaAttachments, /attach_as: if file_exists \{ "file" \} else \{ "receipt" \}/);
  assert.match(mediaAttachments, /missing output; receipt retained for review/);
  assert.doesNotMatch(mediaAttachments, /if !exists \{\s*return None;\s*\}/);
});

test("DX source-set bounded readers reject files larger than their parse limits", () => {
  const receiptsPath = "crates/agent_ui/src/dx_source_sets/receipts.rs";
  const toolchainPath = "crates/agent_ui/src/dx_source_sets/dx_editor_toolchain.rs";
  const agentAttachmentPath = "crates/agent/src/dx_source_attachment.rs";
  const contextAdapterPath = "crates/agent/src/dx_metasearch_context_adapter.rs";

  const receipts = read(receiptsPath);
  const toolchain = read(toolchainPath);
  const agentAttachment = read(agentAttachmentPath);
  const contextAdapter = read(contextAdapterPath);

  assert.match(receipts, /\.take\(MAX_RECEIPT_BYTES \+ 1\)/);
  assert.match(receipts, /buffer\.len\(\) as u64 > MAX_RECEIPT_BYTES/);
  assert.doesNotMatch(receipts, /\.take\(MAX_RECEIPT_BYTES\)/);

  assert.match(toolchain, /\.take\(MAX_DX_CONFIG_BYTES \+ 1\)/);
  assert.match(toolchain, /buffer\.len\(\) as u64 > MAX_DX_CONFIG_BYTES/);
  assert.match(toolchain, /let config = read_bounded_utf8\(&config_path\)\?;/);
  assert.doesNotMatch(toolchain, /\.take\(MAX_DX_CONFIG_BYTES\)/);
  assert.doesNotMatch(toolchain, /read_bounded_utf8\(&config_path\)\.unwrap_or_default\(\)/);

  assert.match(agentAttachment, /\.take\(MAX_RECEIPT_BYTES \+ 1\)/);
  assert.match(agentAttachment, /buffer\.len\(\) as u64 > MAX_RECEIPT_BYTES/);
  assert.doesNotMatch(agentAttachment, /\.take\(MAX_RECEIPT_BYTES\)/);

  assert.match(contextAdapter, /\.take\(MAX_ATTACHMENT_RECEIPT_BYTES \+ 1\)/);
  assert.match(contextAdapter, /buffer\.len\(\) as u64 > MAX_ATTACHMENT_RECEIPT_BYTES/);
  assert.doesNotMatch(contextAdapter, /\.take\(MAX_ATTACHMENT_RECEIPT_BYTES\)/);
});
