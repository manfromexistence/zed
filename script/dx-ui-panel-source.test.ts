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
  const renderItemRow = functionBody(uiPanel, "render_item_row");
  const renderUiHistoryRow = functionBody(uiPanel, "render_ui_history_row");
  const installPlanStart = renderItemRow.indexOf(".when_some(install_plan");
  assert.ok(installPlanStart >= 0, "expected install plan branch");
  const installPlanRemainder = renderItemRow.slice(installPlanStart);
  const actionChildMatch = installPlanRemainder.match(/\.child\(\r?\n\s+h_flex\(\)/);
  assert.ok(actionChildMatch?.index, "expected install plan branch to end before actions");
  const installPlanChrome = installPlanRemainder.slice(0, actionChildMatch.index);

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
  assert.match(renderItemRow, /let install_plan_id = shadcn_element_id\("shadcn-install-plan-", item\.id\.as_ref\(\)\)/);
  assert.match(installPlanChrome, /ListItem::new\(install_plan_id\)/);
  assert.match(installPlanChrome, /\.inset\(true\)/);
  assert.match(installPlanChrome, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(installPlanChrome, /\.selectable\(false\)/);
  assert.match(installPlanChrome, /Icon::new\(IconName::Info\)[\s\S]*\.size\(IconSize::Small\)[\s\S]*\.color\(Color::Accent\)/);
  assert.match(installPlanChrome, /Label::new\(install_plan\)[\s\S]*\.size\(LabelSize::Small\)[\s\S]*\.color\(Color::Muted\)/);
  assert.match(installPlanChrome, /Tooltip::text\("Install the source package before inserting"\)/);
  assert.doesNotMatch(
    installPlanChrome,
    /\.p_1\(\)|\.rounded_sm\(\)|\.border_1\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.elevated_surface_background\)|IconSize::XSmall|LabelSize::XSmall/,
  );
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
