import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/agent_ui/src/conversation_view/thread_view.rs";
const source = readFileSync(sourcePath, "utf8");
const agentPanelSource = readFileSync("crates/agent_ui/src/agent_panel.rs", "utf8");
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
  const threadViewStruct = sliceBetween(
    "pub struct ThreadView {",
    "pub(crate) struct AgentResponseAnchor",
  );
  const panelRefreshMethod = sliceBetweenIn(
    agentPanelSource,
    "fn refresh_configured_plugin_options(",
    "\n    fn configured_plugin_options_input(",
  );
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
  const configuredPluginPromptLeakPattern =
    /\b(?:run_command|configure_action|action_id|receipt_id|source_path|source_root_id|source_package_version|credential_types|api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret|password|private[_-]?key|authorization|bearer)\b/i;

  assert.match(renderMessageEditor, /self\.render_configured_plugin_strip\(cx\)/);
  assertBefore(
    renderMessageEditor,
    "self.render_configured_plugin_strip(cx)",
    "self.message_editor.clone()",
    "configured plugins must appear above the chat input",
  );
  assert.match(threadViewStruct, /configured_plugins: Vec<DxConfiguredPluginSummary>/);
  assert.match(source, /configured_plugins: Vec::new\(\)/);
  assert.match(source, /pub\(crate\) fn set_configured_plugin_options\(/);
  assert.match(source, /self\.configured_plugins = configured_plugins/);
  assert.match(agentPanelSource, /configured_plugin_options_cache: Option<DxConfiguredPluginOptionsCache>/);
  assert.match(agentPanelSource, /refresh_configured_plugin_options\(cx\)/);
  assert.match(agentPanelSource, /sync_configured_plugin_options_to_active_thread\(cx\)/);
  assert.match(panelRefreshMethod, /dx_agent_bridge_snapshot_from_settings_for_roots/);
  assert.match(panelRefreshMethod, /workflow_node_catalog\s*\.\s*configured_plugins/);
  assert.match(panelRefreshMethod, /trusted_tool_bridge\s*\.\s*trusted_tool_ids/);
  assert.match(agentPanelSource, /const DX_CONFIGURED_PLUGIN_ID_LIMIT: usize = 96;/);
  assert.match(agentPanelSource, /value\.len\(\) <= DX_CONFIGURED_PLUGIN_ID_LIMIT/);
  assert.match(agentPanelSource, /\.bytes\(\)[\s\S]*matches!\(byte/);
  for (const state of [
    "active",
    "authorized",
    "available",
    "configured",
    "connected",
    "enabled",
    "healthy",
    "not_required",
    "ready",
    "valid",
  ]) {
    assert.match(agentPanelSource, new RegExp(`"${state}"`));
  }
  assert.match(agentPanelSource, /plugin\.trust_policy == DX_TRUSTED_TOOL_POLICY/);
  assert.match(agentPanelSource, /plugin\.approved_by_trusted_bridge/);
  assert.match(agentPanelSource, /plugin\.writes_receipt/);
  assert.match(agentPanelSource, /!plugin\.secrets_exposed/);
  assert.doesNotMatch(pluginStrip, /dx_agent_bridge_snapshot_for_roots/);
  assert.doesNotMatch(pluginStrip, /dx_agent_bridge_snapshot_from_settings_for_roots/);
  assert.doesNotMatch(pluginStrip, /root_paths\(cx\)/);
  assert.doesNotMatch(pluginStrip, /\bread_(?:first_)?json\b|\blatest_receipts\b|\breceipt_root\b|std::fs|serde_json::from_/);
  assert.match(pluginStrip, /self\.configured_plugins/);
  assert.match(pluginStrip, /MAX_VISIBLE_CONFIGURED_PLUGIN_OPTIONS/);
  assert.match(pluginStrip, /\.take\(MAX_VISIBLE_CONFIGURED_PLUGIN_OPTIONS\)/);
  assert.match(pluginStrip, /\.skip\(MAX_VISIBLE_CONFIGURED_PLUGIN_OPTIONS\)/);
  assert.match(pluginStrip, /ButtonLike::new/);
  assert.match(pluginStrip, /configured_plugin_trigger_label\(&plugin\)/);
  assert.match(pluginStrip, /Tooltip::text\(configured_plugin_tooltip\(&plugin\)\)/);
  assert.match(pluginStrip, /render_configured_plugin_menu/);
  assert.match(pluginStrip, /render_configured_plugin_overflow_menu/);
  assert.match(pluginStrip, /configured_plugin_menu_action_label\(&plugin\)/);
  assert.match(pluginStrip, /configured_plugin_icon/);
  assert.match(source, /fn configured_plugin_tooltip\(plugin: &DxConfiguredPluginSummary\) -> String/);
  assert.match(source, /fn configured_plugin_trigger_label\(plugin: &DxConfiguredPluginSummary\) -> String/);
  assert.match(source, /fn configured_plugin_status_chips\(plugin: &DxConfiguredPluginSummary\)/);
  assert.match(source, /fn configured_plugin_menu_action_label\(_plugin: &DxConfiguredPluginSummary\) -> String/);
  assert.match(source, /Prompt-only\. Inserts a request into the chat input; it does not execute this plugin\./);
  assert.match(source, /it does not execute the plugin\./);
  assert.match(source, /Chip::new\(plugin\.status\.clone\(\)\)/);
  assert.doesNotMatch(pluginStrip, /format!\("\{\} \/ \{\}", plugin\.node_id, plugin\.credential_status\)/);
  assert.doesNotMatch(pluginStrip, /Use configured plugin/);
  assert.match(source, /workflow_node_icon_asset_for/);
  assert.match(workflowNodeIconSource, /WorkflowNodeIconAsset/);
  assert.match(workflowNodeIconSource, /Icon::from_external_svg_with_original_colors/);
  assert.match(workflowNodeIconSource, /dx_icon_data_dir/);
  assert.match(workflowNodeIconSource, /dx_icon\(DxUiIcon::Plugins\)/);
  assert.match(promptInsert, /message_editor\.update\(cx, \|editor, cx\|/);
  assert.match(promptInsert, /Request DX Agent to review the configured plugin/);
  assert.match(promptInsert, /prompt_only: true/);
  assert.match(promptInsert, /trusted DX receipts/);
  assert.match(promptInsert, /raw action metadata/);
  assert.doesNotMatch(promptInsert, /action_id/);
  assert.doesNotMatch(promptInsert, /receipt_id/);
  assert.doesNotMatch(promptInsert, /Use configured plugin/);
  assert.doesNotMatch(promptInsert, /run_command/);
  assert.doesNotMatch(pluginStrip, /list_agent_plugins|inspect_agent_plugin_runtime_status|prepare_agent_plugin_runtime/);
  assert.doesNotMatch(pluginStrip, /api_key|access_token|refresh_token|client_secret|password/i);
  assert.doesNotMatch(promptInsert, /api_key|access_token|refresh_token|client_secret|password/i);
  assert.doesNotMatch(pluginStrip, configuredPluginPromptLeakPattern);
  assert.doesNotMatch(promptInsert, configuredPluginPromptLeakPattern);
});

function sliceBetween(start: string, end: string): string {
  return sliceBetweenIn(source, start, end);
}

function sliceBetweenIn(haystack: string, start: string, end: string): string {
  const startIndex = haystack.indexOf(start);
  assert.notEqual(startIndex, -1, `expected ${start}`);

  const endIndex = haystack.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `expected ${end} after ${start}`);

  return haystack.slice(startIndex, endIndex);
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
