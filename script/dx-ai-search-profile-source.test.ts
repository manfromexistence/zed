import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const defaultSettings = read("assets/settings/default.json");
const agentProfileSettings = read("crates/agent_settings/src/agent_profile.rs");
const composerProfileOptions = read(
  "crates/agent_ui/src/conversation_view/composer_profile_options.rs",
);
const thread = read("crates/agent/src/thread.rs");
const metasearchTool = read("crates/agent/src/tools/dx_metasearch_tool.rs");
const metasearchStatusTool = read("crates/agent/src/tools/dx_metasearch_status_tool.rs");
const metasearchExtractTool = read(
  "crates/agent/src/tools/dx_metasearch_source_extract_tool.rs",
);
const metasearchContextTool = read(
  "crates/agent/src/tools/dx_metasearch_context_adapter_tool.rs",
);
const metasearchBridge = read("crates/agent/src/dx_metasearch_agent_bridge.rs");
const sourceAttachmentTool = read("crates/agent/src/tools/dx_source_attachment_tool.rs");

const objectBlock = (source: string, key: string, fromIndex = 0) => {
  const keyIndex = source.indexOf(`"${key}": {`, fromIndex);
  assert.ok(keyIndex >= 0, `expected object key ${key}`);

  const bodyStart = source.indexOf("{", keyIndex);
  assert.ok(bodyStart > keyIndex, `expected object body for ${key}`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(keyIndex, index + 1);
      }
    }
  }

  assert.fail(`expected object ${key} to close`);
};

test("Search profile exposes only evidence-first DX metasearch tools", () => {
  assert.match(agentProfileSettings, /pub const SEARCH: &str = "search"/);
  assert.match(agentProfileSettings, /WRITE\s*\|\s*ASK\s*\|\s*MEDIA\s*\|\s*SEARCH\s*\|\s*STUDY/);
  assert.match(composerProfileOptions, /static SEARCH_COMPOSER_SLOTS: \[ComposerOptionSlot; 3\]/);

  const profiles = objectBlock(defaultSettings, "profiles");
  const searchProfile = objectBlock(profiles, "search");
  const searchTools = objectBlock(searchProfile, "tools");

  assert.match(searchProfile, /"enable_all_context_servers": false/);

  const enabledSearchTools = [...searchTools.matchAll(/"([^"]+)": true/g)]
    .map(([, tool]) => tool)
    .sort();
  assert.deepEqual(
    enabledSearchTools,
    [
      "diagnostics",
      "extract_dx_metasearch_source",
      "fetch",
      "find_path",
      "find_references",
      "get_code_actions",
      "go_to_definition",
      "grep",
      "inspect_dx_metasearch",
      "inspect_agent_plugin_runtime_status",
      "list_agent_plugins",
      "list_directory",
      "prepare_dx_metasearch_context",
      "prepare_dx_source_attachment",
      "read_file",
      "search_dx_metasearch",
      "search_web",
      "skill",
      "update_plan",
      "update_title",
    ].sort(),
  );

  for (const tool of [
    "inspect_dx_metasearch",
    "search_dx_metasearch",
    "extract_dx_metasearch_source",
    "prepare_dx_source_attachment",
    "prepare_dx_metasearch_context",
  ]) {
    assert.match(searchTools, new RegExp(`"${tool}": true`), `${tool} should be enabled`);
  }

  for (const tool of [
    "execute_dx_media_tool",
    "plan_dx_serializer_rlm_execution",
    "gate_dx_serializer_rlm_runner",
    "execute_dx_serializer_rlm_reducer",
    "restore_dx_forge_target",
  ]) {
    assert.doesNotMatch(searchTools, new RegExp(`"${tool}": true`), `${tool} stays out of Search`);
  }
});

test("AI profile metadata declares backend proof lane ids", () => {
  assert.match(agentProfileSettings, /pub runtime_proof_backend_lane_id: Option<&'static str>/);
  assert.match(
    agentProfileSettings,
    /runtime_proof_backend_lane_id: Some\("dx-metasearch-live-proof"\)/,
  );
  assert.match(
    agentProfileSettings,
    /runtime_proof_backend_lane_id: Some\("study-source-workspace-execution"\)/,
  );
  assert.match(
    agentProfileSettings,
    /runtime_proof_backend_lane_id: Some\("media-provider-readiness-proof"\)/,
  );
});

test("Search profile tools are backed by registered native Agent tools", () => {
  assert.match(metasearchStatusTool, /const NAME: &'static str = "inspect_dx_metasearch"/);
  assert.match(metasearchTool, /const NAME: &'static str = "search_dx_metasearch"/);
  assert.match(
    metasearchExtractTool,
    /const NAME: &'static str = "extract_dx_metasearch_source"/,
  );
  assert.match(sourceAttachmentTool, /const NAME: &'static str = "prepare_dx_source_attachment"/);
  assert.match(
    metasearchContextTool,
    /const NAME: &'static str = "prepare_dx_metasearch_context"/,
  );

  for (const toolType of [
    "DxMetasearchStatusTool",
    "DxMetasearchTool",
    "DxMetasearchSourceExtractTool",
    "DxSourceAttachmentTool",
    "DxMetasearchContextAdapterTool",
  ]) {
    assert.match(thread, new RegExp(`self\\.add_tool\\(${toolType}::new`));
  }
});

test("Study and Media profiles stay evidence-first while backends are pending", () => {
  const profiles = objectBlock(defaultSettings, "profiles");
  const studyProfile = objectBlock(profiles, "study");
  const studyTools = objectBlock(studyProfile, "tools");
  const mediaProfile = objectBlock(profiles, "media");
  const mediaTools = objectBlock(mediaProfile, "tools");

  assert.match(studyTools, /"extract_dx_metasearch_source": true/);
  assert.match(studyTools, /"prepare_dx_source_attachment": true/);
  assert.match(studyTools, /"prepare_dx_metasearch_context": true/);
  assert.match(studyTools, /"plan_dx_serializer_rlm_execution": true/);
  assert.match(studyTools, /"gate_dx_serializer_rlm_runner": true/);
  assert.match(studyTools, /"write_dx_serializer_rlm_reduced_context": true/);
  assert.match(studyTools, /"preview_dx_serializer_rlm_reducer_execution": true/);
  assert.doesNotMatch(
    studyTools,
    /"spawn_agent": true|"prepare_agent_plugin_runtime": true|"execute_dx_media_tool": true|"execute_dx_serializer_rlm_reducer": true|"plan_dx_runtime_proof": true/,
  );

  assert.deepEqual(
    [...mediaTools.matchAll(/"([^"]+)": true/g)].map(([, tool]) => tool).sort(),
    [
      "diagnostics",
      "fetch",
      "find_path",
      "gate_dx_media_tool_runner",
      "grep",
      "list_agent_plugins",
      "list_directory",
      "plan_dx_media_tool",
      "prepare_dx_source_attachment",
      "read_file",
      "search_web",
      "skill",
      "update_title",
    ].sort(),
  );
  assert.doesNotMatch(
    mediaTools,
    /"execute_dx_media_tool": true|"prepare_agent_plugin_runtime": true|"list_dx_launch_demo_recipes": true|"plan_dx_runtime_proof": true/,
  );
});

test("DX metasearch bridge keeps live proof reads bounded", () => {
  assert.match(metasearchBridge, /const MAX_METASEARCH_RESPONSE_BYTES: usize = 1_500_000;/);
  assert.match(metasearchBridge, /read_bounded_http_response_body\(/);
  assert.match(metasearchBridge, /\.take\(\(MAX_METASEARCH_RESPONSE_BYTES \+ 1\) as u64\)/);
  assert.match(metasearchBridge, /buffer\.len\(\) > MAX_METASEARCH_RESPONSE_BYTES/);
  assert.doesNotMatch(metasearchBridge, /\.read_to_end\(&mut body\)\s*\.await/);
});

test("DX metasearch live proof can persist honest status receipts", () => {
  const receiptHistory = read("crates/agent_ui/src/dx_receipt_history/buckets.rs");

  assert.match(metasearchBridge, /DX_METASEARCH_STATUS_RECEIPT_SCHEMA/);
  assert.match(metasearchBridge, /zed\.dx\.metasearch\.status_receipt\.v1/);
  assert.match(metasearchBridge, /pub\(crate\) struct DxMetasearchStatusReceipt/);
  assert.match(metasearchBridge, /pub\(crate\) fn unavailable_metasearch_status/);
  assert.match(metasearchBridge, /status: "unavailable"\.to_string\(\)/);

  assert.match(metasearchStatusTool, /pub write_status_receipt: bool/);
  assert.match(metasearchStatusTool, /DxMetasearchStatusReceiptTarget/);
  assert.match(metasearchStatusTool, /unavailable_metasearch_status/);
  assert.match(metasearchStatusTool, /\.join\("dx-metasearch"\)\.join\("status"\)/);
  assert.match(metasearchStatusTool, /DX_METASEARCH_STATUS_LATEST_FILE_NAME/);
  assert.match(metasearchStatusTool, /starts_metasearch_server": false/);
  assert.match(metasearchStatusTool, /dispatches_browser_input": false/);
  assert.match(metasearchStatusTool, /status_receipt\s*=\s*Some/);

  assert.match(receiptHistory, /Metasearch Status/);
  assert.match(
    receiptHistory,
    /Path::new\("tools"\)[\s\S]*?\.join\("dx-metasearch"\)[\s\S]*?\.join\("status"\)/,
  );
});
