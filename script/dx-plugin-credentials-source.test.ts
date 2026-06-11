import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

const agentPanelPath = "crates/agent_ui/src/agent_panel.rs";
const modalPath = "crates/agent_ui/src/dx_plugin_credentials.rs";
const fieldsPath = "crates/agent_ui/src/dx_plugin_credentials/fields.rs";
const keychainPath = "crates/agent_ui/src/dx_plugin_credentials/keychain.rs";
const credentialsPath = "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/credentials.rs";
const workflowNodesPath = "crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs";
const workflowNodeScreenPath =
  "crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs";

test("DX plugin credential metadata maps input fields before UI capture", () => {
  const workflowNodes = read(workflowNodesPath);
  const credentials = read(credentialsPath);

  assert.match(workflowNodes, /DxWorkflowNodeCredentialInputSummary/);
  assert.match(workflowNodes, /fn credential_type_values/);
  assert.match(credentials, /pub\(crate\) struct DxWorkflowNodeCredentialInputSummary/);
  assert.match(credentials, /pub inputs: Vec<DxWorkflowNodeCredentialInputSummary>/);
  assert.match(credentials, /fn credential_input_rows/);
  assert.match(credentials, /display_string_field\(value, &\["name"\]\)/);
  assert.match(credentials, /display_string_field\(value, &\["kind"\]\)/);
  assert.match(credentials, /display_string_field\(value, &\["credential_status"\]\)/);
  assert.match(credentials, /fn credential_input_values/);
  assert.match(credentials, /collect_array_field\(value, &\["inputs"\]/);
  assert.match(credentials, /collect_array_field\(value, &\["fields"\]/);
  assert.match(credentials, /collect_object_values\(value, &\["properties"\]/);
  assert.match(credentials, /collect_object_values\(value, &\["schema", "properties"\]/);
  assert.match(credentials, /fn fallback_credential_input/);
  assert.match(credentials, /if field_values\.is_empty\(\)/);
  assert.match(credentials, /secret: bool_field\(value, &\["secret"\]\)\.unwrap_or\(true\)/);
});

test("DX plugin credential GPUI modal stores values only in the platform keychain", () => {
  for (const path of [modalPath, fieldsPath, keychainPath]) {
    assert.ok(existsSync(path), `expected focused credential module ${path}`);
  }

  const agentUi = read("crates/agent_ui/src/agent_ui.rs");
  const agentPanel = read(agentPanelPath);
  const modal = read(modalPath);
  const fields = read(fieldsPath);
  const keychain = read(keychainPath);
  const workflowNodeScreen = read(workflowNodeScreenPath);
  const testModuleIndex = lastMatchIndex(agentPanel, /\r?\n#\[cfg\(test\)\]\r?\nmod tests/g);
  assert.ok(testModuleIndex > 0, "expected agent_panel.rs to contain the test module marker");
  const beforeTests = agentPanel.slice(0, testModuleIndex);
  const tests = agentPanel.slice(testModuleIndex);

  assert.match(agentUi, /^mod dx_plugin_credentials;$/m);
  assert.match(modal, /^mod fields;$/m);
  assert.match(modal, /^mod keychain;$/m);
  assert.match(beforeTests, /pub\(crate\) fn open_dx_workflow_node_credentials_modal/);
  assert.match(beforeTests, /pub\(crate\) fn draft_dx_plugin_credentials_saved_prompt/);
  assert.doesNotMatch(tests, /open_dx_workflow_node_credentials_modal/);
  assert.match(workflowNodeScreen, /node\.configured[\s\S]*?draft_dx_workflow_node_configuration_prompt/);
  assert.match(workflowNodeScreen, /credential_status == "not_required"[\s\S]*?draft_dx_workflow_node_configuration_prompt/);
  assert.match(workflowNodeScreen, /node\.credentials\.is_empty\(\)[\s\S]*?draft_dx_workflow_node_configuration_prompt/);
  assert.match(workflowNodeScreen, /open_dx_workflow_node_credentials_modal/);
  assert.match(workflowNodeScreen, /"Configure credentials"/);

  assert.match(modal, /pub\(crate\) struct DxPluginCredentialModal/);
  assert.match(modal, /impl ModalView for DxPluginCredentialModal/);
  assert.match(modal, /save_plugin_credentials/);
  assert.match(modal, /credentials_saved_prompt/);
  assert.match(modal, /clear_secret_inputs/);
  assert.match(modal, /menu::SelectNext/);
  assert.match(modal, /menu::SelectPrevious/);
  assert.match(fields, /InputField::new/);
  assert.match(fields, /\.masked\(true\)/);
  assert.match(fields, /MAX_PLUGIN_CREDENTIAL_INPUT_BYTES/);
  assert.match(fields, /input_text_within_limit/);
  assert.match(fields, /credential_input_placeholder/);
  assert.match(fields, /safe_credential_label/);
  assert.match(fields, /looks_like_secret_value/);
  assert.doesNotMatch(fields, /input\.placeholder|placeholder: display_string_field/);
  assert.match(keychain, /dx_plugin_credential_keychain_url/);
  assert.match(keychain, /credential_key_segment/);
  assert.match(keychain, /credential_key_hash/);
  assert.match(keychain, /cx\.write_credentials/);
  assert.match(keychain, /credential_storage_unavailable_reason/);
  assert.match(keychain, /approved_by_trusted_bridge/);
  assert.match(keychain, /source_owned/);
  assert.match(keychain, /first_party/);

  const promptStart = keychain.indexOf("pub(super) fn credentials_saved_prompt");
  const promptEnd = keychain.indexOf("\nfn credential_writes", promptStart);
  const promptSource = keychain.slice(promptStart, promptEnd);
  assert.match(promptSource, /local_save_reference/);
  assert.match(promptSource, /saved_field_count/);
  assert.doesNotMatch(promptSource, /keychain_handle|credential_id:|input_id:/);
  assert.doesNotMatch(modal, /update_settings_file|write_json_receipt|serde_json::to_(?:string|vec)|log::|println!/);
  assert.doesNotMatch(keychain, /update_settings_file|write_json_receipt|serde_json::to_(?:string|vec)|log::|println!/);
});

function lastMatchIndex(source: string, pattern: RegExp): number {
  let lastIndex = -1;
  for (const match of source.matchAll(pattern)) {
    lastIndex = match.index ?? lastIndex;
  }
  return lastIndex;
}

test("DX plugin credential modules stay focused", () => {
  assert.ok(lineCount(modalPath) < 280);
  assert.ok(lineCount(fieldsPath) < 205);
  assert.ok(lineCount(keychainPath) < 220);
  assert.ok(lineCount(credentialsPath) < 180);
});
