import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;
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
    typeof before === "string" ? body.indexOf(before) : body.search(before);
  const afterIndex =
    typeof after === "string" ? body.indexOf(after) : body.search(after);
  assert.ok(beforeIndex >= 0, `${message}: missing before marker`);
  assert.ok(afterIndex >= 0, `${message}: missing after marker`);
  assert.ok(beforeIndex < afterIndex, message);
};

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
  const renderSectionShell = functionBody(view, "render_section_shell");
  const sectionCountLabel = functionBody(view, "section_count_label");
  const renderPanel = functionBody(view, "render");
  const setActiveTab = functionBody(view, "set_active_tab");
  const openWorkspacePath = functionBody(view, "open_workspace_path");
  const renderSections = functionBody(view, "render_sections");
  const renderActiveTabSections = functionBody(view, "render_active_tab_sections");
  const section = functionBody(rows, "section");
  const sectionRow = functionBody(rows, "section_row");
  const noticeRow = functionBody(rows, "notice_row");
  const quickFixRow = functionBody(rows, "quick_fix_row");
  const adapterPlanRow = functionBody(rows, "adapter_plan_row");
  const webAuditRow = functionBody(rows, "web_audit_row");
  const overflowRow = functionBody(rows, "overflow_row");
  const outcomeLabel = functionBody(rows, "outcome_label");
  const sectionStatusColor = functionBody(rows, "section_status_color");
  const checkTab = functionBody(tabs, "check_tab");

  assert.match(renderHeader, /h_flex\(\)/);
  assert.match(renderHeader, /\.id\("dx-check-panel-header"\)/);
  assert.match(renderHeader, /\.h\(Tab::container_height\(cx\)\)/);
  assert.match(renderHeader, /Label::new\("Check"\)/);
  assert.match(renderHeader, /side_panel_header_controls/);
  assert.match(renderStatusStrip, /ListItem::new\("dx-check-status"\)/);
  assert.match(renderStatusStrip, /let focus_handle = self\.focus_handle\(cx\);/);
  assert.match(renderStatusStrip, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderStatusStrip, /\.selectable\(false\)/);
  assert.match(renderStatusStrip, /\.end_slot\(/);
  assert.match(renderStatusStrip, /\.id\("dx-check-status-actions"\)/);
  assert.match(renderStatusStrip, /\.gap_0p5\(\)/);
  assert.match(renderStatusStrip, /\.occlude\(\)/);
  assert.match(renderStatusStrip, /gpui::MouseButton::Left/);
  assert.match(renderStatusStrip, /cx\.stop_propagation\(\);/);
  assert.match(
    renderStatusStrip,
    /receipt_path\.is_absolute\(\) && receipt_path\.exists\(\)/,
  );
  assert.match(
    renderStatusStrip,
    /IconButton::new\(\s*"dx-check-open-receipt",\s*IconName::FileTextOutlined,\s*\)/,
  );
  assert.match(renderStatusStrip, /IconButton::new\("dx-check-refresh", IconName::RotateCw\)/);
  assert.match(
    renderStatusStrip,
    /IconButton::new\(\s*"dx-check-open-receipt",\s*IconName::FileTextOutlined,\s*\)[\s\S]*\.tab_index\(0_isize\)[\s\S]*\.track_focus\(&focus_handle\)[\s\S]*\.disabled\(!receipt_enabled\)/,
  );
  assert.match(
    renderStatusStrip,
    /IconButton::new\("dx-check-refresh", IconName::RotateCw\)[\s\S]*\.tab_index\(0_isize\)[\s\S]*\.track_focus\(&focus_handle\)/,
  );
  assert.match(
    renderStatusStrip,
    /IconButton::new\("dx-check-refresh", IconName::RotateCw\)[\s\S]*\.style\(ButtonStyle::Subtle\)/,
  );
  assert.match(renderStatusStrip, /\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderStatusStrip, /\.disabled\(!receipt_enabled\)/);
  assert.match(renderStatusStrip, /Tooltip::text\(tooltip\)/);
  assert.doesNotMatch(renderStatusStrip, /status_label\(/);
  assert.match(setActiveTab, /self\.scroll_handle\.set_offset\(point\(px\(0\.\), px\(0\.\)\)\);/);
  assert.match(openWorkspacePath, /if !path\.is_absolute\(\) \|\| !path\.exists\(\) \{/);
  assert.doesNotMatch(view, /fn render_toolbar\(/);
  assert.doesNotMatch(view, /\.id\("dx-check-toolbar"\)|self\.render_toolbar/);
  assert.doesNotMatch(renderPanel, /self\.render_toolbar/);
  assert.doesNotMatch(renderStatusStrip, /(^|[^A-Za-z0-9_])Button::new\("dx-check-open-receipt"/);
  assert.doesNotMatch(renderStatusStrip, /\.start_icon\(/);
  assert.match(tabs, /TabBar::new\(\("dx-check-tab-bar", panel_id\)\)/);
  assert.match(tabs, /snapshot\.adapter_plans\.len\(\)/);
  assert.match(checkTab, /Tab::new\(id\)/);
  assert.match(checkTab, /\.fill_available_width\(\)/);
  assert.match(checkTab, /\.position\(tab_position\(tab, active_tab\)\)/);
  assert.match(checkTab, /\.selected_bottom_border\(true\)/);
  assert.match(checkTab, /\.end_slot\(/);
  assert.match(checkTab, /Label::new\(count\.to_string\(\)\)/);
  assert.match(checkTab, /cx\.stop_propagation\(\);/);
  assert.doesNotMatch(checkTab, /SharedString::from\(format!\("\{label\} \(\{count\}\)"\)\)/);
  assert.match(view, /render_tab_bar\([\s\S]*panel_id[\s\S]*panel\.clone\(\),[\s\S]*cx,/);
  assert.match(view, /\.id\("dx-check-panel-scroll-host"\)/);
  assert.match(view, /\.vertical_scrollbar_for\(&self\.scroll_handle, window, cx\)/);
  assert.match(view, /\.track_scroll\(&self\.scroll_handle\)/);
  assert.match(section, /v_flex\(\)\.id\(id\)/);
  assert.match(section, /ListHeader::new\(title\)/);
  assert.match(section, /\.toggle\(Some\(is_open\)\)/);
  assert.match(rows, /count_label: Option<SharedString>/);
  assert.match(section, /\.when_some\(count_label, \|this, count_label\|/);
  assert.match(section, /\.end_slot\([\s\S]*Label::new\(count_label\)[\s\S]*\.size\(LabelSize::Small\)[\s\S]*\.color\(Color::Muted\)[\s\S]*\.truncate\(\)/);
  assert.match(renderSectionShell, /section_count_label\(section_kind, snapshot\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::Sections => snapshot\.sections\.len\(\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::WebAudit => snapshot\.web_audits\.len\(\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::AdapterPlans => snapshot\.adapter_plans\.len\(\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::Notices => snapshot\.blockers\.len\(\) \+ snapshot\.warnings\.len\(\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::QuickFixes => snapshot\.quick_fixes\.len\(\)/);
  assert.match(sectionCountLabel, /DxCheckPanelSectionKind::Commands =>[\s\S]*detail_command\.as_ref\(\)\.is_some\(\)/);
  assert.doesNotMatch(section, /Open|Closed|end_slot\(status_/);
  assert.match(renderSections, /for \(index, section\) in snapshot\.sections\.iter\(\)\.take\(MAX_SECTION_ROWS\)\.enumerate\(\)/);
  assert.match(renderSections, /section_row\(index, section\)/);
  assert.match(rows, /fn section_row\(index: usize, section: &DxCheckPanelSection\)/);
  assert.match(sectionRow, /"dx-check-section-score-\{index\}-\{\}"/);
  assertBefore({
    body: renderActiveTabSections,
    before: "self.render_notices(snapshot, panel.clone(), cx)",
    after: "self.render_web_audits(snapshot, panel.clone(), cx)",
    message: "Findings should show blocker/warning notices before web audits",
  });
  assertBefore({
    body: renderActiveTabSections,
    before: "self.render_quick_fixes(snapshot, panel.clone(), cx)",
    after: "self.render_adapter_plans(snapshot, panel, cx)",
    message: "Findings should keep quick fixes ahead of adapter plans",
  });
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
  assert.match(view, /const MAX_WEB_AUDIT_ROWS: usize = 4;/);
  assert.match(
    view,
    /collapsed_sections:\s*\[[\s\S]*DxCheckPanelSectionKind::AdapterPlans/,
    "Adapter Plans should default collapsed so the Check surface opens on the actionable rows",
  );
  assert.match(view, /overflow_row\(\s*"dx-check-section-overflow"/);
  assert.match(view, /overflow_row\(\s*"dx-check-web-audit-overflow"/);
  assert.match(noticeRow, /Tooltip::text\(tooltip\)/);
  for (const infoRow of [noticeRow, quickFixRow, adapterPlanRow, webAuditRow]) {
    assert.match(infoRow, /\.selectable\(false\)/);
    assert.match(infoRow, /\.spacing\(ListItemSpacing::Sparse\)/);
    assert.match(infoRow, /Tooltip::text\(tooltip\)/);
    assert.doesNotMatch(infoRow, /let mut content = v_flex|\.child\(\s*v_flex\(\)/);
  }
  assert.match(quickFixRow, /ListItem::new\(SharedString::from\(format!\("dx-check-quick-fix-\{index\}"\)\)\)/);
  assert.match(quickFixRow, /\.end_slot\([\s\S]*fix\.next_action[\s\S]*\.truncate_start\(\)/);
  assert.match(adapterPlanRow, /ListItem::new\(SharedString::from\(format!\("dx-check-adapter-plan-\{index\}"\)\)\)/);
  assert.match(adapterPlanRow, /\.end_slot\([\s\S]*plan\.target[\s\S]*plan\.command[\s\S]*\.truncate_start\(\)/);
  assert.match(webAuditRow, /let normalized_status = audit\.status\.to_ascii_lowercase\(\);/);
  assert.match(webAuditRow, /match normalized_status\.as_str\(\)/);
  assert.match(webAuditRow, /Label::new\(audit\.status\.clone\(\)\)/);
  assert.match(webAuditRow, /\.end_slot\([\s\S]*Label::new\(audit\.status\.clone\(\)\)[\s\S]*Label::new\(source\.to_string\(\)\)/);
  assert.doesNotMatch(webAuditRow, /\.end_slot\(\s*Icon::new\(icon\)/);
  assert.match(overflowRow, /ListItem::new\(id\.into\(\)\)/);
  assert.match(overflowRow, /IconName::Ellipsis/);
  assert.match(overflowRow, /Receipt tab or run the detail command for the full list/);
  assert.match(sectionStatusColor, /let status = status\.to_ascii_lowercase\(\);/);
  assert.match(sectionStatusColor, /match status\.as_str\(\)/);
  assert.match(outcomeLabel, /Counts unavailable/);
  assert.match(outcomeLabel, /check_count_label\(pass_count\)/);
  assert.doesNotMatch(outcomeLabel, /unwrap_or\(0\)/);
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
