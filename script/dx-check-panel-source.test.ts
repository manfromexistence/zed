import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("DX Check panel stays split by reader and parser ownership", () => {
  const parent = read("crates/agent_ui/src/dx_check_panel.rs");
  const expectedModules = [
    "crates/agent_ui/src/dx_check_panel/parser.rs",
    "crates/agent_ui/src/dx_check_panel/reader.rs",
  ];

  for (const module of expectedModules) {
    assert.ok(existsSync(module), `expected focused DX Check panel module ${module}`);
  }

  assert.match(parent, /^mod parser;$/m);
  assert.match(parent, /^mod reader;$/m);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_check_panel.rs") < 450,
    "dx_check_panel.rs should stay a cache/type facade",
  );
});

test("DX Check panel delegates receipt IO and panel parsing", () => {
  const parent = read("crates/agent_ui/src/dx_check_panel.rs");
  const parser = read("crates/agent_ui/src/dx_check_panel/parser.rs");
  const reader = read("crates/agent_ui/src/dx_check_panel/reader.rs");

  assert.match(parent, /reader::read_latest_check_panel/);
  assert.doesNotMatch(parent, /fn read_check_receipt/);
  assert.doesNotMatch(parent, /fn panel_from_zed_value/);
  assert.match(reader, /pub\(super\) fn read_latest_check_panel/);
  assert.match(parser, /pub\(super\) fn panel_from_receipt_value/);
  assert.match(parser, /pub\(super\) fn missing_snapshot/);
  assert.match(parser, /pub\(super\) fn malformed_snapshot/);
  assert.match(reader, /use crate::dx_project_context::DxProjectContext;/);
  assert.doesNotMatch(parent, /DX_FALLBACK_CHECK_RECEIPT/);
  assert.match(
    reader,
    /DxProjectContext::check_receipt_candidates\(workspace_roots, fallback_check_receipt\(\)\)/,
  );
  assert.match(reader, /fn fallback_check_receipt\(\) -> PathBuf/);
  assert.match(
    reader,
    /DxProjectContext::receipt_root_for\(DxProjectContext::shared_fallback_root\(\), "check"\)/,
  );
  assert.doesNotMatch(reader, /deploy_root_key|push_unique_path/);
  assert.ok(lineCount("crates/agent_ui/src/dx_check_panel/reader.rs") < 140);
  assert.ok(lineCount("crates/agent_ui/src/dx_check_panel/parser.rs") < 620);
});

test("DX Check panel reader uses sentinel-byte bounded JSON reads", () => {
  const reader = read("crates/agent_ui/src/dx_check_panel/reader.rs");

  assert.match(reader, /File::open\(path\)/);
  assert.match(reader, /\.take\(MAX_RECEIPT_BYTES \+ 1\)\s*\.read_to_end\(&mut receipt\)/);
  assert.match(reader, /receipt\.len\(\) as u64 > MAX_RECEIPT_BYTES/);
  assert.match(reader, /serde_json::from_slice::<Value>\(&receipt\)/);
  assert.doesNotMatch(reader, /read_to_string/);
});

test("DX Check panel parser bounds user-controlled snapshot strings", () => {
  const parser = read("crates/agent_ui/src/dx_check_panel/parser.rs");

  assert.match(parser, /const MAX_PANEL_TEXT_CHARS: usize = \d+;/);
  assert.match(parser, /fn bounded_panel_text\(value: &str\) -> Option<String>/);
  assert.match(parser, /fn bounded_string_from\(value: Option<&Value>\) -> Option<String>/);
  assert.match(
    parser,
    /fn bounded_string_at<const N: usize>\(value: &Value, path: \[&str; N\]\) -> Option<String>/,
  );

  const boundedSnapshotFields = [
    /status: bounded_string_from\(zed\.get\("status"\)\)/,
    /weight_profile: bounded_string_from\(zed\.get\("weight_profile"\)\)/,
    /refresh_command: bounded_string_from\(zed\.get\("refresh_command"\)\)/,
    /detail_command: bounded_string_from\(zed\.get\("detail_command"\)\)/,
    /title: bounded_string_from\(view_model\.get\("title"\)\)/,
    /receipt_error: if status == "malformed" \{\s+bounded_string_from\(view_model\.get\("empty_state"\)\)/,
    /last_run_label\(\s+bounded_string_from\(view_model\.get\("last_run_label"\)\)/,
    /refresh_command: bounded_string_at\(view_model, \["primary_action", "command"\]\)/,
    /detail_command: bounded_string_at\(view_model, \["secondary_action", "command"\]\)/,
  ];

  for (const pattern of boundedSnapshotFields) {
    assert.match(parser, pattern);
  }

  const boundedRowPatterns = [
    /let title = bounded_string_from\(section\.get\("title"\)\)/,
    /status: bounded_string_from\(section\.get\("status"\)\)/,
    /let message = bounded_string_from\(notice\.get\("message"\)\)\?/,
    /code: bounded_string_from\(notice\.get\("code"\)\)/,
    /next_action: bounded_string_from\(notice\.get\("next_action"\)\)/,
    /let label = bounded_string_from\(fix\.get\("label"\)\)\?/,
    /let next_action = bounded_string_from\(fix\.get\("next_action"\)\)\?/,
    /let raw_command = string_from\(fix\.get\("command"\)\);/,
    /let command = raw_command\.and_then\(bounded_panel_text\);/,
    /risk_level: bounded_string_from\(fix\.get\("risk_level"\)\)/,
    /quick_fix_risk_level\(raw_command\)/,
    /quick_fix_requires_approval\(raw_command\)/,
    /quick_fix_writes_receipts\(raw_command\)/,
    /string_array\(value: Option<&Value>\).*bounded_panel_text/s,
  ];

  for (const pattern of boundedRowPatterns) {
    assert.match(parser, pattern);
  }
});

test("DX Check panel view uses shared panel primitives instead of badge chrome", () => {
  const view = read("crates/agent_ui/src/dx_check_panel_view.rs");
  const rows = read("crates/agent_ui/src/dx_check_panel_view/view_rows.rs");
  const tabs = read("crates/agent_ui/src/dx_check_panel_view/tabs.rs");

  const renderHeader = functionBody(view, "render_header");
  const renderStatusStrip = functionBody(view, "render_status_strip");
  const renderToolbar = functionBody(view, "render_toolbar");
  const section = functionBody(rows, "section");
  const checkTab = functionBody(tabs, "check_tab");

  assert.match(renderHeader, /h_flex\(\)/);
  assert.match(renderHeader, /\.id\("dx-check-panel-header"\)/);
  assert.match(renderHeader, /\.h\(Tab::container_height\(cx\)\)/);
  assert.match(renderHeader, /Label::new\("Check"\)/);
  assert.match(renderHeader, /side_panel_header_controls/);
  assert.match(renderStatusStrip, /ListItem::new\("dx-check-status"\)/);
  assert.match(renderStatusStrip, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderStatusStrip, /\.selectable\(false\)/);
  assert.match(renderStatusStrip, /Tooltip::text\(tooltip\)/);
  assert.doesNotMatch(renderStatusStrip, /status_label\(/);
  assert.match(renderToolbar, /Button::new\("dx-check-open-receipt", "Receipt"\)/);
  assert.match(renderToolbar, /\.start_icon\([\s\S]*Icon::new\(IconName::FileTextOutlined\)/);
  assert.match(renderToolbar, /\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderToolbar, /IconButton::new\("dx-check-refresh", IconName::RotateCw\)/);
  assert.match(tabs, /TabBar::new\("dx-check-tab-bar"\)/);
  assert.match(checkTab, /Tab::new\(id\)/);
  assert.match(checkTab, /\.position\(tab_position\(tab, active_tab\)\)/);
  assert.match(checkTab, /\.selected_bottom_border\(true\)/);
  assert.match(section, /v_flex\(\)\.id\(id\)/);
  assert.match(section, /ListHeader::new\(title\)/);
  assert.match(section, /\.toggle\(Some\(is_open\)\)/);
  assert.doesNotMatch(section, /Open|Closed|end_slot\(status_/);
  const statusColor = functionBody(rows, "status_color");
  assert.match(statusColor, /let status = snapshot\.status\.to_ascii_lowercase\(\);/);
  assert.match(statusColor, /check_status_is_failure\(&status\)/);
  assert.match(statusColor, /check_status_is_warning\(&status\)/);
  assert.match(
    statusColor,
    /check_status_is_success\(&status\) && check_snapshot_has_result_signal\(snapshot\)/,
  );
  assert.match(rows, /fn check_status_is_success\(status: &str\) -> bool/);
  assert.match(rows, /fn check_snapshot_has_result_signal\(snapshot: &DxCheckPanelSnapshot\) -> bool/);
  assert.match(rows, /snapshot\.score_value\.is_some\(\)/);
  assert.match(rows, /!\s*snapshot\.sections\.is_empty\(\)/);
  assert.doesNotMatch(tabs, /count_chip|Divider::vertical|border_b_1|ghost_element|editor_background/);
  assert.doesNotMatch(
    `${view}\n${rows}\n${tabs}`,
    /\b(?:Badge|Chip|Pill|Tag|StatusBadge|BadgeCluster)\b|fn\s+\w*(?:badge|chip|pill|tag|cluster)\w*\s*\(|LabelSize::XSmall|IconSize::XSmall|ListItemSpacing::ExtraDense/i,
  );
});

function functionBody(source: string, name: string): string {
  const signature = source.indexOf(`fn ${name}`);
  assert.notEqual(signature, -1, `missing function ${name}`);
  const bodyStart = source.indexOf("{", signature);
  assert.notEqual(bodyStart, -1, `missing function body for ${name}`);
  let depth = 0;
  for (let index = bodyStart; index < source.length; index++) {
    const char = source[index];
    if (char === "{") depth++;
    if (char === "}") {
      depth--;
      if (depth === 0) return source.slice(bodyStart, index + 1);
    }
  }
  assert.fail(`unterminated function body for ${name}`);
}
