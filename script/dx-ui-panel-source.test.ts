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
  const renderUiHistoryRow = functionBody(uiPanel, "render_ui_history_row");

  assert.match(uiPanel, /use ui::\{[\s\S]*ListItem,[\s\S]*ListItemSpacing/);
  assert.match(renderUiHistoryRow, /ListItem::new\(row_id\)/);
  assert.match(renderUiHistoryRow, /\.inset\(true\)/);
  assert.match(renderUiHistoryRow, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(renderUiHistoryRow, /\.selectable\(false\)/);
  assert.match(renderUiHistoryRow, /\.start_slot\(/);
  assert.match(renderUiHistoryRow, /\.end_slot\(/);
  assert.match(renderUiHistoryRow, /\.tooltip\(Tooltip::text\(row_tooltip\)\)/);
  assert.doesNotMatch(
    renderUiHistoryRow,
    /\.border_1\(\)|\.rounded_sm\(\)|\.bg\(cx\.theme\(\)\.colors\(\)\.element_background\)/,
  );
});
