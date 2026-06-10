import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/agent_ui/src/conversation_view/thread_view.rs";
const source = readFileSync(sourcePath, "utf8");
const workflowNodeIconSource = readFileSync("crates/agent_ui/src/workflow_node_icons.rs", "utf8");

const cycleThinkingEffort = sliceBetween(
  "fn cycle_thinking_effort(",
  "\n    fn toggle_thinking_effort_menu(",
);

test("thread view thinking effort cycling checks stale next indexes before reading the effort", () => {
  assert.doesNotMatch(
    cycleThinkingEffort,
    /effort_levels\s*\[\s*next_index\s*\]/,
    "thinking effort cycling must not directly index effort_levels with next_index",
  );
  assert.match(
    cycleThinkingEffort,
    /let\s+Some\(\w+\)\s*=\s*effort_levels\.get\(next_index\)\s*else\s*\{\s*return;\s*\};/s,
    "thinking effort cycling must use a checked lookup and return early for stale next indexes",
  );
  assertBefore(
    cycleThinkingEffort,
    "effort_levels.get(next_index)",
    "thread.update(cx, |thread, cx| {",
    "the checked effort lookup must happen before mutating thread/settings state",
  );
});

test("thread view source guard stays scoped to production thread view code", () => {
  assert.equal(sourcePath, "crates/agent_ui/src/conversation_view/thread_view.rs");
  assert.doesNotMatch(sourcePath, /test/i);
  assert.doesNotMatch(cycleThinkingEffort, /#\[cfg\(test\)\]/);
});

test("configured workflow-node plugins render above the chat input from bridge receipts", () => {
  const renderMessageEditor = sliceBetween(
    "pub(crate) fn render_message_editor(",
    "\n    fn render_profile_option_slots(",
  );
  const pluginStrip = sliceBetween(
    "fn render_configured_plugin_strip(",
    "\n    fn insert_configured_plugin_prompt(",
  );
  const promptInsert = sliceBetween(
    "fn insert_configured_plugin_prompt(",
    "\n    fn render_profile_option_slots(",
  );

  assert.match(renderMessageEditor, /self\.render_configured_plugin_strip\(cx\)/);
  assertBefore(
    renderMessageEditor,
    "self.render_configured_plugin_strip(cx)",
    "self.message_editor.clone()",
    "configured plugins must appear above the chat input",
  );
  assert.match(pluginStrip, /dx_agent_bridge_snapshot_for_roots/);
  assert.match(pluginStrip, /workflow_node_catalog\.configured_plugins/);
  assert.match(pluginStrip, /MAX_VISIBLE_CONFIGURED_PLUGIN_OPTIONS/);
  assert.match(pluginStrip, /ButtonLike::new/);
  assert.match(pluginStrip, /render_configured_plugin_menu/);
  assert.match(pluginStrip, /configured_plugin_icon/);
  assert.match(source, /workflow_node_icon_asset_for/);
  assert.match(workflowNodeIconSource, /WorkflowNodeIconAsset/);
  assert.match(workflowNodeIconSource, /Icon::from_external_svg_with_original_colors/);
  assert.match(workflowNodeIconSource, /dx_icon_data_dir/);
  assert.match(workflowNodeIconSource, /dx_icon\(DxUiIcon::Plugins\)/);
  assert.match(promptInsert, /message_editor\.update\(cx, \|editor, cx\|/);
  assert.match(promptInsert, /Use configured plugin/);
  assert.match(promptInsert, /action_id/);
  assert.match(promptInsert, /receipt_id/);
  assert.doesNotMatch(promptInsert, /run_command/);
  assert.doesNotMatch(pluginStrip, /list_agent_plugins|inspect_agent_plugin_runtime_status|prepare_agent_plugin_runtime/);
  assert.doesNotMatch(pluginStrip, /api_key|access_token|refresh_token|client_secret|password/i);
  assert.doesNotMatch(promptInsert, /api_key|access_token|refresh_token|client_secret|password/i);
});

function sliceBetween(start: string, end: string): string {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `expected ${start}`);

  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `expected ${end} after ${start}`);

  return source.slice(startIndex, endIndex);
}

function assertBefore(
  haystack: string,
  before: string,
  after: string,
  message: string,
) {
  const beforeIndex = haystack.indexOf(before);
  const afterIndex = haystack.indexOf(after);

  assert.notEqual(beforeIndex, -1, `expected ${before}`);
  assert.notEqual(afterIndex, -1, `expected ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
}
