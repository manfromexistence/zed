import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;
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

test("DX launch receipts keep IO, paths, fields, freshness, and summaries focused", () => {
  const parentPath = "crates/agent_ui/src/dx_launch_receipts.rs";
  const fieldsPath = "crates/agent_ui/src/dx_launch_receipts/fields.rs";
  const freshnessPath = "crates/agent_ui/src/dx_launch_receipts/freshness.rs";
  const pathsPath = "crates/agent_ui/src/dx_launch_receipts/paths.rs";
  const receiptIoPath = "crates/agent_ui/src/dx_launch_receipts/receipt_io.rs";
  const summaryPath = "crates/agent_ui/src/dx_launch_receipts/summary.rs";

  assert.ok(existsSync(fieldsPath), "missing focused launch-receipts field module");
  assert.ok(existsSync(freshnessPath), "missing focused launch-receipts freshness module");
  assert.ok(existsSync(pathsPath), "missing focused launch-receipts path module");
  assert.ok(existsSync(receiptIoPath), "missing focused launch-receipts IO module");
  assert.ok(existsSync(summaryPath), "missing focused launch-receipts summary module");

  const parent = read(parentPath);
  const fields = read(fieldsPath);
  const freshness = read(freshnessPath);
  const paths = read(pathsPath);
  const receiptIo = read(receiptIoPath);
  const summary = read(summaryPath);
  const launchSnapshotPaths = functionBody(paths, "launch_snapshot_paths");
  const scanLaunchReceipts = functionBody(parent, "scan_launch_receipts");

  assert.match(parent, /^mod fields;$/m);
  assert.match(parent, /^mod freshness;$/m);
  assert.match(parent, /^mod paths;$/m);
  assert.match(parent, /^mod receipt_io;$/m);
  assert.match(parent, /^mod summary;$/m);
  assert.match(parent, /use self::freshness::launch_receipt_operator_summary;/);
  assert.match(parent, /use self::paths::\{launch_snapshot_paths, now_ms\};/);
  assert.match(parent, /use crate::dx_launch_receipt_roots::active_launch_receipt_root;/);
  assert.match(parent, /pub\(crate\) fn launch_receipt_review_snapshot_for_roots/);
  assert.match(parent, /let root = active_launch_receipt_root\(workspace_roots\);/);
  assert.match(parent, /cached_root == &root/);
  assert.doesNotMatch(parent, /DX_LAUNCH_RECEIPT_ROOT/);
  assert.doesNotMatch(parent, /fn read_json_receipt\(/);
  assert.doesNotMatch(parent, /fn launch_snapshot_paths\(/);
  assert.doesNotMatch(parent, /fn freshness_state\(/);
  assert.doesNotMatch(parent, /fn optional_string_field\(/);
  assert.doesNotMatch(parent, /impl DxLaunchReceiptSummary/);
  assert.match(fields, /pub\(super\) fn optional_string_field/);
  assert.match(fields, /fn render_safe_string/);
  assert.match(freshness, /pub\(super\) fn freshness_state/);
  assert.match(freshness, /pub\(super\) fn launch_receipt_operator_summary/);
  assert.match(paths, /const MAX_LAUNCH_SNAPSHOT_DIR_ENTRIES: usize = 512;/);
  assert.match(paths, /const MAX_LAUNCH_SNAPSHOT_PATHS: usize = 128;/);
  assert.match(paths, /pub\(super\) struct LaunchSnapshotPaths/);
  assert.match(paths, /pub\(super\) fn launch_snapshot_paths/);
  assert.match(
    launchSnapshotPaths,
    /receipt_order_ms\(&path\)[\s\S]*candidates\.push\(\(order_ms, path\)\)[\s\S]*candidates\.sort_by/,
    "launch snapshot candidates must be timestamp-ranked before retaining the bounded newest set",
  );
  assert.doesNotMatch(
    launchSnapshotPaths,
    /\.take\(\s*MAX_LAUNCH_SNAPSHOT_PATHS\s*\)[\s\S]*sort_by/,
    "launch receipt enumeration must not truncate the arbitrary read_dir order before freshness ordering",
  );
  assert.match(
    launchSnapshotPaths,
    /scan_truncated = true/,
    "launch receipt enumeration must expose when a directory or retained-candidate cap was hit",
  );
  assert.match(parent, /snapshot_scan_truncated: bool/);
  assert.match(
    scanLaunchReceipts,
    /let snapshot_paths = launch_snapshot_paths\(&root\);[\s\S]*let snapshot_scan_truncated = snapshot_paths\.scan_truncated;[\s\S]*snapshot_paths[\s\S]*\.paths[\s\S]*snapshot_scan_truncated:/,
    "launch receipt review snapshots must preserve scan-cap visibility",
  );
  assert.match(
    freshness,
    /snapshot_scan_truncated: bool[\s\S]*Launch receipts warning: snapshot scan capped/,
    "operator summaries must not present capped receipt scans as complete truth",
  );
  assert.match(paths, /pub\(super\) fn now_ms/);
  assert.match(receiptIo, /pub\(super\) fn read_json_receipt/);
  assert.match(receiptIo, /MAX_RECEIPT_BYTES/);
  assert.match(receiptIo, /take\(MAX_RECEIPT_BYTES \+ 1\)/);
  assert.match(receiptIo, /if buffer\.len\(\) as u64 > MAX_RECEIPT_BYTES/);
  assert.match(receiptIo, /serde_json::from_slice\(&buffer\)/);
  assert.doesNotMatch(receiptIo, /read_to_string/);
  assert.match(summary, /impl DxLaunchReceiptSummary/);
  assert.match(summary, /pub\(super\) fn from_path/);

  assert.ok(lineCount(parentPath) < 250, "dx_launch_receipts.rs should stay focused on snapshot assembly");
  assert.ok(lineCount(fieldsPath) < 45, "launch-receipts field module should stay small");
  assert.ok(lineCount(freshnessPath) < 60, "launch-receipts freshness module should stay small");
  assert.ok(lineCount(pathsPath) < 95, "launch-receipts path module should stay small");
  assert.ok(lineCount(receiptIoPath) < 55, "launch-receipts IO module should stay small");
  assert.ok(lineCount(summaryPath) < 95, "launch-receipts summary module should stay small");
});
