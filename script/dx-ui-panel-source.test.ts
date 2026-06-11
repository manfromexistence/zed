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
  const renderEmptyRow = functionBody(uiPanel, "render_empty_row");
  const renderFilterTabs = functionBody(uiPanel, "render_filter_tabs");
  const renderRecentUiSection = functionBody(uiPanel, "render_recent_ui_section");
  const renderPinnedUiSection = functionBody(uiPanel, "render_pinned_ui_section");
  const renderItemRow = functionBody(uiPanel, "render_item_row");
  const renderUiHistoryRow = functionBody(uiPanel, "render_ui_history_row");

  assert.match(uiPanel, /use ui::\{[\s\S]*ListHeader,[\s\S]*ListItem,[\s\S]*ListItemSpacing/);
  assert.match(render, /let status = self\.status\.clone\(\);/);
  assert.match(render, /usize::from\(status\.is_some\(\)\)/);
  assert.match(render, /self\.render_status_row\(status, cx\)\.into_any_element\(\)/);
  assert.match(render, /self\.render_empty_row\(\)/);
  assert.doesNotMatch(render, /h_full\(\)\.flex\(\)\.items_center\(\)\.justify_center\(\)/);
  assert.match(renderStatusRow, /ListItem::new\("shadcn-ui-status-row"\)/);
  assert.match(renderStatusRow, /\.inset\(true\)/);
  assert.match(renderStatusRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderStatusRow, /\.selectable\(false\)/);
  assert.match(renderStatusRow, /Tooltip::text\(status\.clone\(\)\)/);
  assert.match(renderFilterTabs, /let filter_scroll_offset = self\.filter_scroll_handle\.offset\(\)\.x;/);
  assert.match(renderFilterTabs, /let filter_scroll_max = self\.filter_scroll_handle\.max_offset\(\)\.x;/);
  assert.match(renderFilterTabs, /let filter_tabs_scrollable = filter_scroll_max > px\(2\.\);/);
  assert.match(renderFilterTabs, /let can_scroll_filter_tabs_back = filter_tabs_scrollable && filter_scroll_offset < px\(0\.\);/);
  assert.match(
    renderFilterTabs,
    /let can_scroll_filter_tabs_forward =\s*filter_tabs_scrollable && filter_scroll_offset > -filter_scroll_max;/,
  );
  assert.match(renderFilterTabs, /IconButton::new\("ui-panel-filter-prev", IconName::ChevronLeft\)[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderFilterTabs, /IconButton::new\("ui-panel-filter-prev", IconName::ChevronLeft\)[\s\S]*\.disabled\(!can_scroll_filter_tabs_back\)/);
  assert.match(renderFilterTabs, /IconButton::new\("ui-panel-filter-next", IconName::ChevronRight\)[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderFilterTabs, /IconButton::new\("ui-panel-filter-next", IconName::ChevronRight\)[\s\S]*\.disabled\(!can_scroll_filter_tabs_forward\)/);
  assert.match(render, /IconButton::new\(\s*"shadcn-ui-refresh-catalog",\s*IconName::RotateCw[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(render, /IconButton::new\(\s*"shadcn-ui-remove-missing-history",\s*IconName::Trash[\s\S]*\.style\(ButtonStyle::Subtle\)/);
  assert.match(renderEmptyRow, /ListItem::new\("shadcn-ui-empty-row"\)/);
  assert.match(renderEmptyRow, /\.inset\(true\)/);
  assert.match(renderEmptyRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderEmptyRow, /\.selectable\(false\)/);
  assert.match(renderEmptyRow, /Icon::new\(IconName::Info\)/);
  assert.match(renderEmptyRow, /\.size\(IconSize::Small\)/);
  assert.match(renderEmptyRow, /Label::new\("No matching components"\)/);
  assert.match(renderEmptyRow, /\.size\(LabelSize::Small\)/);
  assert.match(renderEmptyRow, /\.color\(Color::Muted\)/);
  assert.match(renderEmptyRow, /Tooltip::text\("No matching components"\)/);
  assert.match(renderRecentUiSection, /ListHeader::new\("Recent"\)/);
  assert.match(renderRecentUiSection, /\.start_slot\(Icon::new\(IconName::Clock\)\.size\(IconSize::Small\)\)/);
  assert.match(renderPinnedUiSection, /ListHeader::new\("Pinned"\)/);
  assert.match(renderPinnedUiSection, /\.start_slot\(Icon::new\(IconName::Star\)\.size\(IconSize::Small\)\)/);
  assert.match(renderItemRow, /let install_plan_id = shadcn_element_id\("shadcn-install-plan-", item\.id\.as_ref\(\)\)/);
  assert.match(renderItemRow, /let metadata = v_flex\(\)/);
  assert.match(renderItemRow, /let actions = h_flex\(\)/);
  assert.match(renderItemRow, /ListItem::new\(row_id\)/);
  assert.match(renderItemRow, /\.inset\(true\)/);
  assert.match(renderItemRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderItemRow, /\.start_slot\(shadcn_thumbnail\(&item, image_url, cx\)\)/);
  assert.match(renderItemRow, /\.end_slot\(metadata\)/);
  assert.match(renderItemRow, /\.end_slot_on_hover\(actions\)/);
  assert.match(renderItemRow, /\.when\(can_drag,[\s\S]*\.on_drag\(payload,/);
  assert.match(renderItemRow, /ListItem::new\(install_plan_id\)/);
  assert.match(renderItemRow, /Icon::new\(IconName::Info\)[\s\S]*\.size\(IconSize::Small\)[\s\S]*\.color\(Color::Accent\)/);
  assert.match(renderItemRow, /Label::new\(install_plan\)[\s\S]*\.size\(LabelSize::Small\)[\s\S]*\.color\(Color::Muted\)/);
  assert.match(renderItemRow, /Tooltip::text\(\s*"Install the source package before inserting",\s*\)/);
  assert.doesNotMatch(
    renderItemRow,
    /\.p_1\(\)|\.p_2\(\)|\.rounded_sm\(\)|\.border_1\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.(?:elevated_surface_background|element_background)\)/,
  );
  assert.match(renderItemRow, /Button::new\(insert_id, primary_action\)/);
  assert.match(renderItemRow, /Button::new\(insert_id, primary_action\)[\s\S]*\.tab_index\(0_isize\)/);
  assert.match(renderItemRow, /\.occlude\(\)/);
  assert.match(renderItemRow, /gpui::MouseButton::Left/);
  assert.match(renderItemRow, /cx\.stop_propagation\(\);/);
  assert.match(renderItemRow, /IconButton::new\(copy_id, IconName::Copy\)/);
  assert.match(renderItemRow, /IconButton::new\(preview_id, IconName::Eye\)/);
  assert.match(renderItemRow, /IconButton::new\(docs_id, IconName::ArrowUpRight\)/);
  assert.match(renderItemRow, /IconButton::new\(pin_id, IconName::Pin\)/);
  assert.match(renderItemRow, /IconButton::new\(copy_id, IconName::Copy\)[\s\S]*\.tab_index\(0_isize\)/);
  assert.match(renderItemRow, /IconButton::new\(preview_id, IconName::Eye\)[\s\S]*\.tab_index\(0_isize\)/);
  assert.match(renderItemRow, /IconButton::new\(docs_id, IconName::ArrowUpRight\)[\s\S]*\.tab_index\(0_isize\)/);
  assert.match(renderItemRow, /IconButton::new\(pin_id, IconName::Pin\)[\s\S]*\.tab_index\(0_isize\)/);
  assert.match(renderItemRow, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.match(renderItemRow, /\.icon_size\(IconSize::Small\)/);
  assert.doesNotMatch(
    renderItemRow,
    /Button::new\((?:copy_id|preview_id|docs_id|pin_id), (?:copy_action|"Preview"|"Docs"|"Pin")\)|\.flex_wrap\(\)/,
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
  assert.match(renderUiHistoryRow, /\.occlude\(\)/);
  assert.match(renderUiHistoryRow, /gpui::MouseButton::Left/);
  assert.match(renderUiHistoryRow, /cx\.stop_propagation\(\);/);
  assert.match(renderUiHistoryRow, /Button::new\(primary_id, primary_action\)/);
  assert.match(renderUiHistoryRow, /IconButton::new\(copy_id, IconName::Copy\)/);
  assert.match(renderUiHistoryRow, /IconButton::new\(preview_id, IconName::Eye\)/);
  assert.match(renderUiHistoryRow, /IconButton::new\(remove_id, IconName::Trash\)/);
  assert.match(renderUiHistoryRow, /IconButton::new\(docs_id, IconName::ArrowUpRight\)/);
  assert.match(renderUiHistoryRow, /IconButton::new\(pin_id, pin_icon\)/);
  assert.match(renderUiHistoryRow, /\.shape\(ui::IconButtonShape::Square\)/);
  assert.match(renderUiHistoryRow, /\.icon_size\(IconSize::Small\)/);
  assert.match(renderUiHistoryRow, /panel\.copy_item_code/);
  assert.match(renderUiHistoryRow, /panel\.preview_item/);
  assert.match(renderUiHistoryRow, /panel\.open_item_docs/);
  assert.match(renderUiHistoryRow, /panel\.pin_ui_action/);
  assert.match(renderUiHistoryRow, /panel\.unpin_ui_action/);
  assert.match(renderUiHistoryRow, /panel\.remove_ui_history_entry/);
  assert.doesNotMatch(renderUiHistoryRow, /\.flex_wrap\(\)/);
  assert.doesNotMatch(
    renderUiHistoryRow,
    /(^|[^A-Za-z0-9_])Button::new\((?:copy_id|preview_id|remove_id|docs_id|pin_id),/,
  );
  assert.match(renderUiHistoryRow, /\.tooltip\(Tooltip::text\(row_tooltip\)\)/);
  assert.doesNotMatch(
    renderUiHistoryRow,
    /\.border_1\(\)|\.rounded_sm\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)/,
  );
});
