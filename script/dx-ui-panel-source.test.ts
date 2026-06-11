import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const uiPanel = read("crates/shadcn_ui_panel/src/shadcn_ui_panel.rs");

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

test("UI panel history rows use shared GPUI list primitives", () => {
  const render = functionBody(uiPanel, "render");
  const renderStatusRow = functionBody(uiPanel, "render_status_row");
  const renderRecentUiSection = functionBody(uiPanel, "render_recent_ui_section");
  const renderPinnedUiSection = functionBody(uiPanel, "render_pinned_ui_section");
  const renderUiHistoryRow = functionBody(uiPanel, "render_ui_history_row");

  assert.match(uiPanel, /use ui::\{[\s\S]*ListHeader,[\s\S]*ListItem,[\s\S]*ListItemSpacing/);
  assert.match(render, /let status = self\.status\.clone\(\);/);
  assert.match(render, /usize::from\(status\.is_some\(\)\)/);
  assert.match(render, /self\.render_status_row\(status, cx\)\.into_any_element\(\)/);
  assert.match(renderStatusRow, /ListItem::new\("shadcn-ui-status-row"\)/);
  assert.match(renderStatusRow, /\.inset\(true\)/);
  assert.match(renderStatusRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderStatusRow, /\.selectable\(false\)/);
  assert.match(renderStatusRow, /Tooltip::text\(status\.clone\(\)\)/);
  assert.match(renderRecentUiSection, /ListHeader::new\("Recent"\)/);
  assert.match(renderRecentUiSection, /\.start_slot\(Icon::new\(IconName::Clock\)\.size\(IconSize::Small\)\)/);
  assert.match(renderPinnedUiSection, /ListHeader::new\("Pinned"\)/);
  assert.match(renderPinnedUiSection, /\.start_slot\(Icon::new\(IconName::Star\)\.size\(IconSize::Small\)\)/);
  const historySectionChrome = `${renderRecentUiSection}\n${renderPinnedUiSection}`;
  assert.match(historySectionChrome, /\.end_slot\([\s\S]*Label::new\(availability_label\)/);
  assert.match(
    historySectionChrome,
    /IconButton::new\(\s*"shadcn-ui-remove-missing-recent",\s*IconName::ListX/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\(\s*"shadcn-ui-remove-missing-pinned",\s*IconName::ListX/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\("shadcn-ui-clear-recent", IconName::Trash\)/,
  );
  assert.match(
    historySectionChrome,
    /IconButton::new\("shadcn-ui-clear-pinned", IconName::Trash\)/,
  );
  assert.match(historySectionChrome, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.doesNotMatch(
    historySectionChrome,
    /Button::new\("shadcn-ui-(?:remove-missing|clear)-(?:recent|pinned)", "(?:Remove|Clear)"\)/,
  );
  assert.doesNotMatch(historySectionChrome, /LabelSize::XSmall|IconSize::XSmall|\.justify_between\(\)/);
  assert.match(renderUiHistoryRow, /ListItem::new\(row_id\)/);
  assert.match(renderUiHistoryRow, /\.inset\(true\)/);
  assert.match(renderUiHistoryRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderUiHistoryRow, /\.selectable\(false\)/);
  assert.match(renderUiHistoryRow, /\.start_slot\(/);
  assert.match(renderUiHistoryRow, /\.end_slot\(/);
  assert.match(renderUiHistoryRow, /IconName::Ellipsis/);
  assert.match(renderUiHistoryRow, /\.end_slot_on_hover\(/);
  assert.match(renderUiHistoryRow, /\.tooltip\(Tooltip::text\(row_tooltip\)\)/);
  assert.doesNotMatch(
    renderUiHistoryRow,
    /\.border_1\(\)|\.rounded_sm\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)/,
  );
});
