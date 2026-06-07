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
