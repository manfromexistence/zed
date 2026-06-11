import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;
const assertBefore = (body: string, before: string, after: string, message: string) => {
  const beforeIndex = body.indexOf(before);
  const afterIndex = body.indexOf(after);
  assert.notEqual(beforeIndex, -1, `${message}: missing ${before}`);
  assert.notEqual(afterIndex, -1, `${message}: missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
};

test("DX Agent bridge stays split by command, runtime, and receipt ownership", () => {
  const parent = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const expectedModules = [
    "crates/agent_ui/src/dx_agent_bridge/automation_actions.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_args.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_args_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_safety.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_receipts.rs",
    "crates/agent_ui/src/dx_agent_bridge/commands.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs",
    "crates/agent_ui/src/dx_agent_bridge/local_files.rs",
    "crates/agent_ui/src/dx_agent_bridge/paths.rs",
    "crates/agent_ui/src/dx_agent_bridge/receipts.rs",
    "crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs",
    "crates/agent_ui/src/dx_agent_bridge/receipts/trusted_tool_bridge.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_connection_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog_fields.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_display.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs",
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/contract.rs",
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs",
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured_authorization.rs",
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/credentials.rs",
  ];

  for (const module of expectedModules) {
    assert.ok(existsSync(module), `expected focused DX Agent bridge module ${module}`);
  }

  assert.match(parent, /^mod command_receipts;$/m);
  assert.match(parent, /^mod command_safety;$/m);
  assert.match(parent, /^mod automation_actions;$/m);
  assert.match(parent, /^mod automation_contract;$/m);
  assert.match(parent, /^mod command_args;$/m);
  assert.match(parent, /^mod commands;$/m);
  assert.match(parent, /^mod local_file_labels;$/m);
  assert.match(parent, /^mod local_files;$/m);
  assert.match(parent, /^mod paths;$/m);
  assert.match(parent, /^mod receipts;$/m);
  assert.match(parent, /^mod runtime;$/m);
  assert.match(parent, /^mod workflow_nodes;$/m);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_agent_bridge.rs") < 880,
    "dx_agent_bridge.rs should stay a coordinator and type boundary",
  );
});

test("DX Agent bridge delegates bridge commands and receipt parsing", () => {
  const parent = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const agentConfiguration = read("crates/agent_ui/src/agent_configuration.rs");
  const safety = read("crates/agent_ui/src/dx_agent_bridge/command_safety.rs");
  const commandArgs = read("crates/agent_ui/src/dx_agent_bridge/command_args.rs");
  const commandArgsTests = read("crates/agent_ui/src/dx_agent_bridge/command_args_tests.rs");
  const safetyTests = read("crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs");
  const commandReceipts = read("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs");
  const commands = read("crates/agent_ui/src/dx_agent_bridge/commands.rs");
  const catalogActiveProviderLabel = read(
    "crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label.rs",
  );
  const catalogLabels = read("crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs");
  const catalogLabelsTests = read("crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs");
  const catalogActiveProviderLabelTests = read(
    "crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label_tests.rs",
  );
  const localFileLabels = read("crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs");
  const localFiles = read("crates/agent_ui/src/dx_agent_bridge/local_files.rs");
  const paths = read("crates/agent_ui/src/dx_agent_bridge/paths.rs");
  const receipts = read("crates/agent_ui/src/dx_agent_bridge/receipts.rs");
  const receiptStrings = read("crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs");
  const trustedToolBridge = read(
    "crates/agent_ui/src/dx_agent_bridge/receipts/trusted_tool_bridge.rs",
  );
  const runtimeCatalog = read("crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs");
  const runtimeConnectionTests = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_connection_tests.rs",
  );
  const runtimeCatalogTests = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog_tests.rs",
  );
  const runtimeCatalogFields = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog_fields.rs",
  );
  const runtimeDisplay = read("crates/agent_ui/src/dx_agent_bridge/runtime_display.rs");
  const runtimeProviderModels = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs",
  );
  const runtimeProviderModelsTests = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models_tests.rs",
  );
  const workflowNodes = read("crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs");
  const workflowNodeContract = read(
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/contract.rs",
  );
  const workflowNodeConfigured = read(
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs",
  );
  const workflowNodeConfiguredAuthorization = read(
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured_authorization.rs",
  );
  const workflowNodeCredentials = read(
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/credentials.rs",
  );
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const workflowNodeSources = `${workflowNodes}\n${workflowNodeContract}\n${workflowNodeConfigured}\n${workflowNodeConfiguredAuthorization}\n${workflowNodeCredentials}`;
  const runtime = read("crates/agent_ui/src/dx_agent_bridge/runtime.rs");
  const runtimeTests = read("crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs");
  const forbiddenWorkflowNodeSources =
    /\b(?:n8n|OpenClaw|ZeroClaw|claude-plugins-official|external_plugins|inspirations[\\/]|G:\\\\Dx\\\\inspirations)\b/i;
  const rawSecretDisplayPattern =
    /\b(?:api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret|password|private[_-]?key|authorization|bearer)\b/i;

  assert.doesNotMatch(parent, /fn run_bridge_command/);
  assert.doesNotMatch(parent, /fn contract_summary/);
  assert.doesNotMatch(parent, /fn social_accounts/);
  assert.doesNotMatch(parent, /fn is_secret_like_arg/);
  assert.doesNotMatch(parent, /fn public_command_for_runtime/);
  assert.match(parent, /use self::command_safety::\{/);
  assert.match(parent, /use self::paths::\{/);
  assert.match(parent, /pub\(crate\) fn dx_agent_bridge_snapshot_for_roots/);
  assert.match(parent, /dx_agent_bridge_snapshot_from_settings_for_roots/);
  assert.match(parent, /let settings = settings\.with_workspace_roots\(workspace_roots\);/);
  assert.match(parent, /pub\(crate\) fn dx_agent_dx_home_for_roots/);
  assert.match(parent, /pub\(crate\) fn dx_agent_receipt_root_for_roots/);
  assert.match(parent, /receipt_root_configured: bool/);
  assert.match(parent, /provider_catalog_path_configured: bool/);
  assert.match(parent, /active_agent_receipt_root\(workspace_roots\)/);
  assert.match(parent, /active_provider_catalog_path\(workspace_roots\)/);
  assert.doesNotMatch(parent, /DEFAULT_AGENT_RECEIPT_ROOT|DEFAULT_PROVIDER_CATALOG_PATH/);
  assert.match(parent, /^mod catalog_active_provider_label;$/m);
  assert.match(parent, /^mod catalog_labels;$/m);
  assert.match(parent, /^mod workflow_nodes;$/m);
  assert.match(parent, /read_first_json_with_default_path/);
  assert.match(parent, /"plugins\/workflow-node-catalog-latest\.json"/);
  assert.match(parent, /pub\(crate\) use self::workflow_nodes::\{/);
  assert.match(parent, /workflow_node_catalog: workflow_node_catalog_summary/);
  assert.match(workflowNodes, /^mod contract;$/m);
  assert.match(workflowNodes, /^mod configured;$/m);
  assert.match(workflowNodes, /^mod configured_authorization;$/m);
  assert.match(workflowNodes, /^mod credentials;$/m);
  assert.match(workflowNodes, /pub\(crate\) use self::contract::\{/);
  assert.match(workflowNodes, /pub\(crate\) use self::credentials::\{/);
  assert.match(workflowNodes, /pub\(crate\) use self::configured::DxConfiguredPluginSummary/);
  assert.match(workflowNodes, /use self::contract::\{/);
  assert.match(workflowNodes, /use self::configured::\{/);
  assert.match(workflowNodes, /use self::credentials::workflow_node_credential_rows/);
  assert.match(workflowNodes, /ConfiguredPluginIndex/);
  assert.match(workflowNodes, /configured_plugin_index/);
  assert.match(workflowNodes, /configured_plugin_rows/);
  assert.match(
    parent,
    /pub\(crate\) use self::catalog_active_provider_label::\{\s*catalog_active_provider_label, catalog_active_provider_value_label,\s*\};/s,
  );
  assert.match(parent, /pub\(crate\) use self::catalog_labels::\{/);
  assert.match(
    parent,
    /pub\(crate\) use self::catalog_labels::\{\s*catalog_cache_state_label, catalog_detail_label, catalog_receipt_status_label,\s*\};/s,
  );
  assert.match(safety, /pub\(crate\) fn is_secret_like_arg/);
  assert.match(safety, /pub\(crate\) fn redact_action_scalar/);
  assert.match(safety, /pub\(crate\) fn public_command_for_runtime/);
  assert.match(safety, /pub\(crate\) fn is_safe_automation_id_arg/);
  assert.match(safety, /pub\(crate\) fn is_safe_platform_arg/);
  assert.match(safety, /pub\(crate\) fn bridge_command_label/);
  assert.match(safety, /#\[path = "command_safety_tests\.rs"\]/);
  assert.match(safety, /normalized\.contains\(marker\)/);
  assert.match(safety, /let mut redact_next = false/);
  assert.match(safety, /redact_next = is_secret_flag_arg\(arg\)/);
  assert.match(safety, /fn is_secret_flag_arg/);
  assert.match(safetyTests, /dx_agent_secret_marker_guard_covers_bridge_receipt_scalars/);
  assert.match(safetyTests, /public_command_for_runtime_maps_legacy_dx_agents_commands/);
  assert.match(safetyTests, /safe_automation_ids_refuse_secrets_and_shell_shapes/);
  assert.match(safetyTests, /bridge_command_label_redacts_secret_like_args/);
  assert.match(commandArgs, /pub\(super\) fn dx_agents_args/);
  assert.match(commandArgs, /pub\(super\) fn dx_agents_automation_args/);
  assert.match(commandArgs, /pub\(super\) fn dx_agents_platform_args/);
  assert.match(commandArgs, /#\[path = "command_args_tests\.rs"\]/);
  assert.match(commandArgsTests, /automation_run_args_match_dx_agents_contract/);
  assert.match(commandArgsTests, /automation_enable_args_match_dx_agents_contract/);
  assert.match(safetyTests, /bridge_command_label_redacts_secret_key_value_args/);
  assert.match(commandReceipts, /pub\(super\) fn write_json_receipt/);
  assert.match(commandReceipts, /pub\(super\) fn write_action_error_receipt/);
  assert.match(commandReceipts, /pub\(super\) fn clear_action_error_receipt/);
  assert.match(commands, /pub\(crate\) fn run_dx_agent_public_command/);
  assert.match(commands, /pub\(crate\) enum DxAgentPublicCommand/);
  assert.match(commands, /AutomationSaveDraft/);
  assert.match(commands, /AutomationEnable \{ automation_id: String \}/);
  assert.match(commands, /AutomationRun \{ automation_id: String \}/);
  assert.match(localFiles, /pub\(super\) fn read_json/);
  assert.match(localFiles, /pub\(super\) fn read_first_json/);
  assert.match(localFiles, /pub\(super\) fn read_first_json_with_path/);
  assert.match(localFiles, /pub\(super\) fn read_first_json_with_default_path/);
  assert.match(localFiles, /pub\(super\) fn latest_receipts/);
  assert.match(localFiles, /pub\(super\) fn dx_home_from_receipt_root/);
  assert.match(localFiles, /receipt_file_label/);
  assert.match(localFiles, /MAX_RECEIPT_BYTES/);
  assert.match(paths, /pub\(super\) fn active_agent_receipt_root/);
  assert.match(paths, /DxProjectContext::receipt_root_candidates/);
  assert.match(paths, /pub\(super\) fn active_provider_catalog_path/);
  assert.match(paths, /provider_catalog_path_candidates\(workspace_roots\)/);
  assert.match(paths, /DEFAULT_PROVIDER_CATALOG_PATH/);
  assert.match(paths, /project_root_key/);
  assert.match(paths, /\.find\(\|root\| root\.is_dir\(\)\)/);
  assert.match(paths, /\.find\(\|path\| path\.is_file\(\)\)/);
  assert.match(agentConfiguration, /fn dx_agent_workspace_roots\(&self, cx: &App\) -> Vec<String>/);
  assert.match(agentConfiguration, /root_paths\(cx\)/);
  assert.match(agentConfiguration, /dx_agent_bridge_snapshot_for_roots\(cx, &workspace_roots\)/);
  assert.match(agentConfiguration, /dx_agent_dx_home_for_roots\(cx, &workspace_roots\)/);
  assert.match(agentConfiguration, /dx_agent_receipt_root_for_roots\(cx, &workspace_roots\)/);
  assert.doesNotMatch(agentConfiguration, /dx_agent_bridge_snapshot\(cx\)/);
  assert.doesNotMatch(agentConfiguration, /dx_agent_dx_home\(cx\)|dx_agent_receipt_root\(cx\)/);
  assert.match(localFileLabels, /pub\(crate\) fn receipt_file_label/);
  assert.match(localFileLabels, /eq_ignore_ascii_case\("json"\)/);
  assert.match(localFileLabels, /receipt_file_label_accepts_uppercase_json_extension/);
  assert.match(receipts, /pub\(super\) fn contract_summary/);
  assert.match(receipts, /pub\(super\) fn receipt_index_summary/);
  assert.match(receipts, /^mod receipt_strings;$/m);
  assert.match(receipts, /^mod trusted_tool_bridge;$/m);
  assert.match(receipts, /pub\(super\) use self::trusted_tool_bridge::trusted_tool_bridge_summary/);
  assert.match(receipts, /use self::receipt_strings::\{/);
  assert.match(receiptStrings, /pub\(in super::super\) fn receipt_string_field/);
  assert.match(receiptStrings, /pub\(in super::super\) fn receipt_string_array_field/);
  assert.match(receiptStrings, /pub\(in super::super\) fn receipt_string_values_field/);
  assert.match(trustedToolBridge, /pub\(super\) fn trusted_tool_bridge_summary/);
  assert.match(trustedToolBridge, /trusted_tool_bridge_value/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_cache_state_label/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_detail_label/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_receipt_status_label/);
  assert.deepEqual(catalogLabels.match(/^pub\(crate\) fn \w+/gm), [
    "pub(crate) fn catalog_cache_state_label",
    "pub(crate) fn catalog_detail_label",
    "pub(crate) fn catalog_receipt_status_label",
  ]);
  assert.match(catalogActiveProviderLabel, /pub\(crate\) fn catalog_active_provider_label/);
  assert.match(catalogActiveProviderLabel, /pub\(crate\) fn catalog_active_provider_value_label/);
  assert.deepEqual(catalogActiveProviderLabel.match(/^pub\(crate\) fn \w+/gm), [
    "pub(crate) fn catalog_active_provider_label",
    "pub(crate) fn catalog_active_provider_value_label",
  ]);
  assert.match(catalogLabels, /#\[path = "catalog_labels_tests\.rs"\]/);
  assert.match(catalogActiveProviderLabel, /#\[path = "catalog_active_provider_label_tests\.rs"\]/);
  assert.match(catalogLabelsTests, /catalog_detail_label_separates_catalog_from_readiness/);
  assert.match(catalogLabelsTests, /catalog_receipt_status_label_humanizes_waiting_states/);
  assert.doesNotMatch(catalogLabelsTests, /catalog_active_provider_label_/);
  assert.match(catalogActiveProviderLabelTests, /catalog_active_provider_label_prefers_display_name/);
  assert.match(
    catalogActiveProviderLabelTests,
    /catalog_active_provider_label_waits_for_receipt_before_claiming_none/,
  );
  assert.match(catalogActiveProviderLabelTests, /catalog_active_provider_label_ignores_blank_display_name/);
  assert.match(runtime, /pub\(super\) fn social_accounts/);
  assert.match(runtime, /runtime_catalog::catalog_summary/);
  assert.match(runtime, /#\[path = "runtime_catalog_fields\.rs"\]\s*mod runtime_catalog_fields;/);
  assert.match(runtime, /runtime_provider_models::\{models, providers\}/);
  assert.match(runtime, /#\[path = "runtime_display\.rs"\]/);
  assert.match(runtime, /#\[path = "runtime_catalog_tests\.rs"\]\s*mod runtime_catalog_tests;/);
  assert.match(runtimeCatalog, /pub\(in super::super\) fn catalog_summary/);
  assert.deepEqual(runtimeCatalog.match(/^pub\(in super::super\) fn \w+/gm), [
    "pub(in super::super) fn catalog_summary",
  ]);
  assert.match(runtimeCatalog, /root_exists: bool/);
  assert.match(runtimeCatalog, /catalog_honesty_fields/);
  assert.match(runtimeCatalog, /binary_cache_path/);
  assert.match(runtimeCatalog, /bool_field\(catalog, &\["loaded"\]\)/);
  assert.match(runtimeCatalogFields, /pub\(super\) struct CatalogHonestyFields/);
  assert.match(runtimeCatalogFields, /configured_provider_count/);
  assert.match(runtimeCatalogFields, /enabled_provider_count/);
  assert.match(runtimeCatalogFields, /active_provider_id/);
  assert.match(runtimeCatalogFields, /missing_receipt_root/);
  assert.match(runtimeCatalogFields, /waiting_for_provider_receipt/);
  assert.match(runtimeDisplay, /pub\(super\) fn display_string_field/);
  assert.match(runtimeDisplay, /pub\(super\) fn display_string_array_field/);
  assert.match(runtimeDisplay, /redact_action_scalar\(&value\)/);
  assert.match(runtimeDisplay, /const MAX_RUNTIME_DISPLAY_CHARS: usize = 180;/);
  assert.match(runtimeProviderModels, /pub\(in super::super\) fn providers/);
  assert.match(runtimeProviderModels, /pub\(in super::super\) fn models/);
  assert.deepEqual(runtimeProviderModels.match(/^pub\(in super::super\) fn \w+/gm), [
    "pub(in super::super) fn providers",
    "pub(in super::super) fn models",
  ]);
  for (const [name, source] of [
    ["runtime_catalog.rs", runtimeCatalog],
    ["runtime_catalog_fields.rs", runtimeCatalogFields],
    ["runtime_display.rs", runtimeDisplay],
    ["runtime_provider_models.rs", runtimeProviderModels],
  ]) {
    assert.doesNotMatch(source, /^pub\s+(?:fn|struct|enum|mod|use)\b/m, `${name} must not expose public APIs`);
    assert.doesNotMatch(
      source,
      /^pub\(crate\)\s+(?:fn|struct|enum|mod|use)\b/m,
      `${name} must not expose crate APIs`,
    );
  }
  assert.match(runtimeProviderModels, /fn grouped_model_rows/);
  assert.match(runtime, /#\[path = "runtime_connection_tests\.rs"\]/);
  assert.match(runtimeProviderModels, /#\[path = "runtime_provider_models_tests\.rs"\]/);
  assert.match(runtimeProviderModelsTests, /grouped_model_rows_skip_blank_string_entries/);
  assert.match(
    runtimeProviderModelsTests,
    /provider_and_model_rows_bound_and_redact_display_values/,
  );
  assert.match(workflowNodes, /pub\(super\) fn workflow_node_catalog_summary/);
  assert.match(workflowNodes, /DxWorkflowNodeCatalogSummary/);
  assert.match(workflowNodes, /DxWorkflowNodeSummary/);
  assert.match(workflowNodes, /DxConfiguredPluginSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodePortSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodePermissionSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodeCredentialSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodeDynamicOptionSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodeReceiptSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodeActionSummary/);
  assert.match(workflowNodeSources, /DxWorkflowNodeTrustSummary/);
  assert.match(workflowNodes, /MAX_WORKFLOW_NODE_ROWS/);
  assert.match(workflowNodeSources, /MAX_CONFIGURED_PLUGIN_ROWS/);
  assert.match(workflowNodeSources, /MAX_WORKFLOW_NODE_PORT_ROWS/);
  assert.match(workflowNodeSources, /MAX_WORKFLOW_NODE_PERMISSION_ROWS/);
  assert.match(workflowNodeSources, /MAX_WORKFLOW_NODE_DYNAMIC_OPTION_ROWS/);
  assert.match(workflowNodeSources, /MAX_WORKFLOW_NODE_ACTION_ROWS/);
  assert.match(workflowNodes, /redact_action_scalar/);
  assert.match(workflowNodes, /credential_status/);
  assert.match(workflowNodes, /credential_types/);
  assert.match(workflowNodes, /missing_schema_version/);
  assert.match(workflowNodes, /missing_serializer_format/);
  assert.match(
    workflowNodes,
    /configured_plugin_count: nodes\.iter\(\)\.filter\(\|node\| node\.configured\)\.count\(\)/,
  );
  assert.match(workflowNodes, /configured_index\.has_configured_plugin_data\(\)/);
  assert.match(workflowNodes, /configured_index\.contains_node\(&id, &source_root_id, &source_path\)/);
  assert.match(workflowNodeConfigured, /struct ConfiguredPluginIndex/);
  assert.match(workflowNodeConfigured, /node_keys: HashSet<\(String, String, String\)>/);
  assert.match(workflowNodeConfigured, /MAX_CONFIGURED_PLUGIN_INDEX_ROWS/);
  assert.match(workflowNodeConfiguredAuthorization, /MAX_CONFIGURED_PLUGIN_ID_CHARS/);
  assert.match(workflowNodeConfiguredAuthorization, /MAX_CONFIGURED_PLUGIN_SOURCE_CHARS/);
  assert.match(workflowNodeConfiguredAuthorization, /DX_CONFIGURED_PLUGIN_SOURCE_ROOTS/);
  assert.match(workflowNodeConfiguredAuthorization, /"repo_agent_tools"/);
  assert.match(workflowNodeConfiguredAuthorization, /"repo_agent_ui_bridge"/);
  assert.match(workflowNodeConfiguredAuthorization, /"repo_agent_ui_bridge_module"/);
  assert.match(workflowNodeConfiguredAuthorization, /"repo_web_preview"/);
  assert.match(workflowNodeConfiguredAuthorization, /"workspace_agent_plugins"/);
  assert.match(workflowNodeConfiguredAuthorization, /"workspace_playwright_runner"/);
  assert.match(workflowNodeConfiguredAuthorization, /"dxjs_runtime"/);
  assert.match(workflowNodeConfiguredAuthorization, /const TRUSTED_TOOL_POLICY: &str = "receipt_authorized_only"/);
  assert.match(workflowNodeConfigured, /fn configured_plugin_values/);
  assert.match(workflowNodeConfigured, /fn configured_plugin_key/);
  assert.match(workflowNodeConfiguredAuthorization, /fn configured_plugin_row_is_authorized/);
  assert.match(workflowNodeConfiguredAuthorization, /fn configured_plugin_identity_is_safe/);
  assert.match(workflowNodeConfiguredAuthorization, /fn configured_plugin_source_root_is_allowed/);
  assert.match(workflowNodeConfiguredAuthorization, /fn configured_plugin_source_path_is_safe/);
  assert.match(workflowNodeConfiguredAuthorization, /fn configured_plugin_state_is_usable/);
  assert.match(
    workflowNodeConfigured,
    /Some\(\(plugin\.node_id, plugin\.source_root_id, plugin\.source_path\)\)/,
  );
  assert.match(
    workflowNodeConfigured,
    /configured_plugin_row_is_authorized\(&row\)\.then_some\(row\)/,
  );
  assert.match(workflowNodeConfigured, /bool_field\(value, &\["writes_receipts"\]\)/);
  assert.match(workflowNodeConfiguredAuthorization, /configured_plugin_source_root_is_allowed\(&row\.source_root_id\)/);
  assert.match(workflowNodeConfiguredAuthorization, /configured_plugin_source_path_is_safe\(&row\.source_path\)/);
  assert.match(agentPanel, /fn valid_configured_plugin_source_root\(value: &str\) -> bool/);
  assert.match(agentPanel, /fn valid_configured_plugin_source_path\(value: &str\) -> bool/);
  assert.match(agentPanel, /valid_configured_plugin_source_root\(&plugin\.source_root_id\)/);
  assert.match(agentPanel, /valid_configured_plugin_source_path\(&plugin\.source_path\)/);
  assert.match(agentPanel, /let configured_node_keys = snapshot[\s\S]*?workflow_node_catalog[\s\S]*?nodes/);
  assert.match(agentPanel, /configured_node_keys\.contains\(&\(/);
  assert.match(agentPanel, /plugin\.node_id\.clone\(\)/);
  assert.match(agentPanel, /plugin\.source_root_id\.clone\(\)/);
  assert.match(agentPanel, /plugin\.source_path\.clone\(\)/);
  assert.match(workflowNodeConfiguredAuthorization, /row\.trust_policy == TRUSTED_TOOL_POLICY/);
  assert.match(workflowNodeConfiguredAuthorization, /row\.approved_by_trusted_bridge/);
  assert.match(workflowNodeConfiguredAuthorization, /row\.writes_receipt/);
  assert.match(workflowNodeConfiguredAuthorization, /!row\.secrets_exposed/);
  assert.match(workflowNodeConfiguredAuthorization, /configured_plugin_state_is_usable\(&row\.status\)/);
  assert.match(workflowNodeConfiguredAuthorization, /configured_plugin_state_is_usable\(&row\.credential_status\)/);
  assert.match(workflowNodeConfiguredAuthorization, /!value\.contains\('\\0'\)/);
  assert.ok(workflowNodeConfiguredAuthorization.includes("!value.starts_with('/')"));
  assert.ok(workflowNodeConfiguredAuthorization.includes("!value.starts_with('\\\\')"));
  assert.ok(workflowNodeConfiguredAuthorization.includes("!value.contains('\\\\')"));
  assert.match(workflowNodeConfiguredAuthorization, /!has_windows_drive_prefix\(value\)/);
  assert.match(workflowNodeConfiguredAuthorization, /matches!\(segment, "" \| "\." \| "\.\."\)/);
  assert.doesNotMatch(workflowNodeSources, forbiddenWorkflowNodeSources);
  assert.doesNotMatch(workflowNodeSources, rawSecretDisplayPattern);
  assert.doesNotMatch(workflowNodes, /(?<!display_)string_field\(value, &\["(?:description|source_package|source_package_version|source_root_id|source_path|credential_status|configure_action)"\]\)/);
  assert.doesNotMatch(workflowNodes, /(?<!display_)string_array_field\(value, &\["credential_types"\]\)/);
  for (const field of [
    "permissions",
    "inputs",
    "outputs",
    "credentials",
    "dynamic_options",
    "receipts",
    "actions",
    "trust",
    "source_package_version",
    "source_root_id",
    "source_path",
  ]) {
    assert.match(workflowNodes, new RegExp(`pub ${field}:`));
  }
  for (const field of [
    "source_package",
    "source_package_version",
    "credential_status",
  ]) {
    assert.match(workflowNodes, new RegExp(`${field}:\\s*display_string_field\\(value, &\\["${field}"\\]`));
  }
  assert.match(
    workflowNodes,
    /let source_root_id = display_string_field\(value, &\["source_root_id"\]\)[\s\S]*?missing_source_root_id/,
  );
  assert.match(
    workflowNodes,
    /let source_path = display_string_field\(value, &\["source_path"\]\)[\s\S]*?missing_source_path/,
  );
  assertBefore(
    workflowNodes,
    "let source_root_id = display_string_field(value, &[\"source_root_id\"])",
    "let configured = if configured_index.has_configured_plugin_data()",
    "source provenance must be parsed before configured-node matching",
  );
  for (const field of [
    "source_root_id",
    "source_path",
    "trust_policy",
    "approved_by_trusted_bridge",
    "writes_receipt",
    "secrets_exposed",
  ]) {
    assert.match(workflowNodeSources, new RegExp(`pub ${field}:`));
  }
  assert.match(workflowNodes, /let credential_types = credential_type_values\(value, &credentials\)/);
  assert.match(workflowNodes, /credential_types,/);
  assert.match(workflowNodes, /fn credential_type_values/);
  assert.match(
    workflowNodes,
    /let declared = display_string_array_field\(value, &\["credential_types"\], MAX_DETAIL_ITEMS\)/,
  );
  for (const helper of [
    "workflow_node_permission_rows",
    "workflow_node_port_rows",
    "workflow_node_credential_rows",
    "workflow_node_dynamic_option_rows",
    "workflow_node_receipt_rows",
    "workflow_node_action_rows",
    "workflow_node_trust_summary",
  ]) {
    assert.match(workflowNodeSources, new RegExp(`\\b${helper}\\(`));
  }
  assert.match(workflowNodeSources, /missing_credential_metadata/);
  assert.match(workflowNodeSources, /missing_permissions/);
  assert.match(workflowNodeSources, /missing_trust_policy/);
  assert.match(workflowNodes, /missing_source_package_version/);
  assert.match(workflowNodes, /missing_source_root_id/);
  assert.match(workflowNodes, /missing_source_path/);
  assert.match(workflowNodes, /unknown_source_package/);
  assert.match(workflowNodeSources, /missing_action_id/);
  assert.match(workflowNodeSources, /missing_receipt_id/);
  assert.match(workflowNodeContract, /requires_user_enablement_for_input[\s\S]*?unwrap_or\(true\)/);
  assert.match(workflowNodeContract, /approved_by_trusted_bridge[\s\S]*?unwrap_or\(false\)/);
  assert.match(workflowNodeContract, /writes_receipts[\s\S]*?unwrap_or\(false\)/);
  assert.doesNotMatch(workflowNodes, /credential_types\.is_empty\(\)\s*\|\|\s*configured/);
  assert.doesNotMatch(
    workflowNodes,
    /action_id:[\s\S]{0,120}unwrap_or_else\(\|\| id\.clone\(\)\)/,
  );
  assert.doesNotMatch(workflowNodes, /api_key|access_token|refresh_token|client_secret|password/i);
  assert.match(runtime, /#\[path = "runtime_tests\.rs"\]/);
  assert.match(runtimeConnectionTests, /social_connection_cards_parse_auth_health_and_receipt_history/);
  assert.match(runtimeConnectionTests, /trusted_tool_bridge_summary_requires_receipt_authority/);
  assert.match(runtimeTests, /provider_rows_read_agent_cli_provider_receipts/);
  assert.match(runtimeTests, /model_rows_flatten_agent_cli_provider_model_groups/);
  assert.match(runtimeTests, /legacy_flat_model_rows_still_parse/);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions.rs") < 230);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions_safety_tests.rs") < 105);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions_tests.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs") < 520);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract_safety_tests.rs") < 130);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs") < 180);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_args.rs") < 45);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_args_tests.rs") < 50);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_safety.rs") < 130);
  assert.match(runtimeCatalogTests, /catalog_summary_reads_agent_cli_catalog_diagnostics/);
  assert.match(runtimeCatalogTests, /catalog_summary_derives_provider_honesty_from_provider_rows/);
  assert.match(runtimeCatalogTests, /catalog_summary_reports_missing_receipt_root/);
  assert.match(runtimeCatalogTests, /catalog_summary_reports_waiting_for_provider_receipt/);
  assert.match(
    runtimeCatalogTests,
    /catalog_summary_waits_for_provider_receipt_when_only_model_receipt_exists/,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs") < 130);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs") < 210);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/commands.rs") < 240);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs") < 110);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_active_provider_label_tests.rs") < 110,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/local_files.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/paths.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/receipts.rs") < 560);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs") < 75);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/receipts/trusted_tool_bridge.rs") < 95);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/workflow_nodes/contract.rs") < 280);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs") < 140);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured_authorization.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/workflow_nodes/credentials.rs") < 180);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_connection_tests.rs") < 120);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_catalog_tests.rs") < 130);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_catalog_fields.rs") < 125);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_display.rs") < 80);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs") < 215);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs") < 260);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_provider_models_tests.rs") < 120,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime.rs") < 420);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs") < 130);
});

test("DX Agent provider and model public commands persist JSON receipts", () => {
  const commands = read("crates/agent_ui/src/dx_agent_bridge/commands.rs");
  const publicRunnerStart = commands.indexOf("pub(crate) fn run_dx_agent_public_command");
  const metadataRunnerStart = commands.indexOf("pub(crate) fn run_dx_agent_metadata_command");
  const captureStart = commands.indexOf("fn receipt_capture(&self)");

  assert.ok(publicRunnerStart >= 0, "expected public command runner");
  assert.ok(metadataRunnerStart > publicRunnerStart, "expected metadata runner after public runner");
  assert.ok(captureStart >= 0, "expected public receipt capture mapping");

  const publicRunner = commands.slice(publicRunnerStart, metadataRunnerStart);
  const captureMapping = commands.slice(captureStart, publicRunnerStart);

  assert.match(commands, /struct DxAgentPublicReceiptCapture/);
  assert.match(commands, /use super::command_receipts::\{/);
  assert.match(captureMapping, /Self::ProvidersList => Some\(DxAgentPublicReceiptCapture \{/);
  assert.match(captureMapping, /receipt_filename: "providers-list-latest\.json"/);
  assert.match(captureMapping, /expected_schema: "dx\.agents\.zed\.providers_list\.v1"/);
  assert.match(captureMapping, /Self::ModelsList => Some\(DxAgentPublicReceiptCapture \{/);
  assert.match(captureMapping, /receipt_filename: "models-list-latest\.json"/);
  assert.match(captureMapping, /expected_schema: "dx\.agents\.zed\.models_list\.v1"/);
  assert.match(captureMapping, /Self::ProviderCatalogRegenerate => None/);
  assert.match(publicRunner, /let output = match run_bridge_command/);
  assert.match(publicRunner, /if let Some\(capture\) = command\.receipt_capture\(\) \{/);
  assert.match(
    publicRunner,
    /write_json_receipt\(\s*&receipt_root\.join\(capture\.receipt_filename\),\s*&output\.stdout,\s*capture\.expected_schema,\s*\)/,
  );
  assert.match(
    publicRunner,
    /write_action_error_receipt\(&receipt_root, &command_label, &error\)/,
  );
});

test("DX Agent bridge exposes receipt-backed connection cards and trusted tool bridge", () => {
  const parent = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const runtime = read("crates/agent_ui/src/dx_agent_bridge/runtime.rs");
  const runtimeProviderModels = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs",
  );
  const receipts = read("crates/agent_ui/src/dx_agent_bridge/receipts.rs");
  const trustedToolBridge = read(
    "crates/agent_ui/src/dx_agent_bridge/receipts/trusted_tool_bridge.rs",
  );
  const runtimeConnectionTests = read(
    "crates/agent_ui/src/dx_agent_bridge/runtime_connection_tests.rs",
  );

  for (const field of [
    "provider_id",
    "account_state",
    "auth_method",
    "qr_capability",
    "credential_health",
    "credential_expires_at",
    "credential_error",
    "receipt_history",
  ]) {
    assert.match(parent, new RegExp(`pub ${field}:`));
    assert.match(runtime, new RegExp(`\\["${field}"\\]`));
  }

  assert.match(parent, /pub trusted_tool_bridge: DxAgentTrustedToolBridgeSummary/);
  assert.match(parent, /struct DxAgentTrustedToolBridgeSummary/);
  assert.match(parent, /pub approved_plugin_tool_count: usize/);
  assert.match(parent, /pub approved_automation_tool_count: usize/);
  assert.match(parent, /pub blocked_tool_count: usize/);
  assert.match(parent, /pub bridge_contract_id: String/);
  assert.match(receipts, /tool_bridge/);
  assert.match(trustedToolBridge, /pub\(super\) fn trusted_tool_bridge_summary/);
  assert.match(trustedToolBridge, /tool_bridge/);
  assert.match(trustedToolBridge, /trusted_tool_bridge/);
  assert.match(trustedToolBridge, /approved_plugin_tool_count/);
  assert.match(trustedToolBridge, /approved_automation_tool_count/);
  assert.match(trustedToolBridge, /blocked_tool_count/);
  assert.match(trustedToolBridge, /trusted_tool_ids/);
  assert.match(trustedToolBridge, /receipt_string_array_field\(value, &\["trusted_tool_ids"\]\)/);
  assert.doesNotMatch(receipts, /n8n|OpenClaw|ZeroClaw/i);
  assert.doesNotMatch(trustedToolBridge, /n8n|OpenClaw|ZeroClaw/i);
  assert.match(
    runtimeProviderModels,
    /auth_method: display_string_field\(provider, &\["auth_method"\]\)/,
  );
  assert.match(
    runtimeProviderModels,
    /credential_health: display_string_field\(provider, &\["credential_health"\]\)/,
  );
  assert.doesNotMatch(runtime, /credential_health:[\s\S]{0,220}"present"/);
  assert.doesNotMatch(runtimeProviderModels, /credential_health:[\s\S]{0,220}"present"/);
  for (const field of [
    "provider_id",
    "provider",
    "platform",
    "label",
    "status",
    "account_state",
    "auth_method",
    "connect_method",
    "qr_capability",
    "credential_health",
    "credential_expires_at",
    "credential_error",
    "last_error",
    "next_action",
  ]) {
    assert.match(
      runtime,
      new RegExp(`display_string_field\\([\\s\\S]{0,96}&\\["${field}"\\]`),
      `visible social field ${field} should use display_string_field`,
    );
  }
  assert.match(runtime, /receipt_history: display_string_array_field\(account, &\["receipt_history"\], 8\)/);
  const socialAccountsStart = runtime.indexOf("pub(super) fn social_accounts");
  const socialAccountsEnd = runtime.indexOf("#[derive(Clone, Copy)]", socialAccountsStart);
  assert.ok(socialAccountsStart >= 0 && socialAccountsEnd > socialAccountsStart);
  const socialAccounts = runtime.slice(socialAccountsStart, socialAccountsEnd);
  assert.doesNotMatch(
    socialAccounts,
    /(?:provider_id|platform|label|status|account_state|auth_method|qr_capability|credential_health|credential_expires_at|credential_error|receipt_history|next_action):\s*string_(?:array_)?field\(/,
  );
  assert.match(runtimeConnectionTests, /social_connection_cards_parse_auth_health_and_receipt_history/);
  assert.match(runtimeConnectionTests, /social_connection_cards_redact_visible_account_fields/);
  assert.match(runtimeConnectionTests, /social_action_summary_redacts_visible_receipt_fields/);
  assert.match(runtimeConnectionTests, /trusted_tool_bridge_summary_requires_receipt_authority/);
});

test("DX Agent bridge local receipt reads reject post-metadata growth before parsing", () => {
  const localFiles = read("crates/agent_ui/src/dx_agent_bridge/local_files.rs");
  const readJsonStart = localFiles.indexOf("pub(super) fn read_json");
  const readJsonEnd = localFiles.indexOf("\npub(super) fn read_first_json");

  assert.ok(readJsonStart >= 0, "expected local read_json helper");
  assert.ok(readJsonEnd > readJsonStart, "expected read_json to stay focused");

  const readJson = localFiles.slice(readJsonStart, readJsonEnd);
  const growthLimitCheck =
    "u64::try_from(source.len()).unwrap_or(u64::MAX) > MAX_RECEIPT_BYTES";

  assert.match(readJson, /take\(MAX_RECEIPT_BYTES \+ 1\)/);
  assert.match(readJson, /read_to_end\(&mut source\)/);
  assert.match(readJson, new RegExp(growthLimitCheck.replace(/[().+]/g, "\\$&")));
  assert.match(readJson, /serde_json::from_slice\(&source\)/);
  assert.doesNotMatch(readJson, /read_to_string/);
  assert.ok(
    readJson.indexOf(growthLimitCheck) < readJson.indexOf("serde_json::from_slice"),
    "receipt buffers must be rejected over MAX_RECEIPT_BYTES before parsing",
  );
});

test("DX Agent bridge failed command stderr is compacted before error display", () => {
  const commands = read("crates/agent_ui/src/dx_agent_bridge/commands.rs");
  const commandReceipts = read("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs");
  const runStart = commands.indexOf("fn run_bridge_command");
  const runEnd = commands.length;
  const helperStart = commandReceipts.indexOf("fn failed_command_stderr_display");
  const helperEnd = commandReceipts.indexOf("\npub(super) fn write_json_receipt");

  assert.ok(runStart >= 0, "expected run_bridge_command helper");
  assert.ok(runEnd > runStart, "expected run_bridge_command helper");
  assert.ok(helperStart >= 0, "expected focused failed-command stderr display helper");
  assert.ok(helperEnd > helperStart, "expected helper before receipt writer");

  const runBridgeCommand = commands.slice(runStart, runEnd);
  const stderrHelper = commandReceipts.slice(helperStart, helperEnd);

  assert.match(commandReceipts, /const MAX_FAILED_COMMAND_STDERR_BYTES: usize = 2048;/);
  assert.match(commandReceipts, /const MAX_FAILED_COMMAND_STDERR_CHARS: usize = 500;/);
  assert.match(runBridgeCommand, /is_secret_like_arg\(arg\)/);
  assert.match(
    runBridgeCommand,
    /let stderr = failed_command_stderr_display\(&output\.stderr\);/,
  );
  assert.match(runBridgeCommand, /anyhow!\(\s*"`\{\}` failed: \{\}"/);
  const stderrDisplayCall = runBridgeCommand.indexOf(
    "failed_command_stderr_display(&output.stderr)",
  );
  const failedCommandAnyhow = runBridgeCommand.indexOf("`{}` failed: {}");
  assert.ok(failedCommandAnyhow > stderrDisplayCall, "expected failed-command anyhow");
  assert.ok(
    stderrDisplayCall < failedCommandAnyhow,
    "stderr must be compacted before inclusion in anyhow",
  );
  assert.doesNotMatch(commands, /String::from_utf8_lossy\(&output\.stderr\)/);
  assert.match(stderrHelper, /stderr\.len\(\) > MAX_FAILED_COMMAND_STDERR_BYTES/);
  assert.match(stderrHelper, /&stderr\[..visible_len\]/);
  assert.match(stderrHelper, /String::from_utf8_lossy\(&stderr\[..visible_len\]\)/);
  assert.match(stderrHelper, /split_whitespace\(\)\.collect::<Vec<_>>\(\)\.join\(" "\)/);
  assert.match(stderrHelper, /take\(MAX_FAILED_COMMAND_STDERR_CHARS\.saturating_sub\(3\)\)/);
  assert.match(stderrHelper, /display\.push_str\("\.\.\."\)/);
});

test("DX Agent bridge checks serialized receipt bytes before writing", () => {
  const commandReceipts = read("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs");
  const writeJsonStart = commandReceipts.indexOf("fn write_json_receipt");
  const writeActionErrorStart = commandReceipts.indexOf("fn write_action_error_receipt");
  const clearActionErrorStart = commandReceipts.indexOf("fn clear_action_error_receipt");
  const actionErrorDisplayStart = commandReceipts.indexOf("fn action_error_display_field");
  const serializerStart = commandReceipts.indexOf("fn serialized_pretty_receipt");
  const writeBytesStart = commandReceipts.indexOf("fn write_receipt_bytes");
  const tempPathStart = commandReceipts.indexOf("fn temp_receipt_path");
  const limitStart = commandReceipts.indexOf("fn ensure_serialized_receipt_bytes");

  assert.ok(writeJsonStart >= 0, "expected metadata receipt writer");
  assert.ok(writeActionErrorStart > writeJsonStart, "expected action-error receipt writer");
  assert.ok(clearActionErrorStart > writeActionErrorStart, "expected clear helper after writes");
  assert.ok(actionErrorDisplayStart > clearActionErrorStart, "expected action-error display helper");
  assert.ok(serializerStart > actionErrorDisplayStart, "expected shared serializer helper");
  assert.ok(writeBytesStart > serializerStart, "expected staged receipt writer helper");
  assert.ok(tempPathStart > writeBytesStart, "expected temporary receipt path helper");
  assert.ok(limitStart > tempPathStart, "expected serialized-byte limit helper");

  const writeJson = commandReceipts.slice(writeJsonStart, writeActionErrorStart);
  const writeActionError = commandReceipts.slice(writeActionErrorStart, clearActionErrorStart);
  const serializer = commandReceipts.slice(serializerStart, writeBytesStart);
  const writeBytes = commandReceipts.slice(writeBytesStart, tempPathStart);
  const tempPath = commandReceipts.slice(tempPathStart, limitStart);
  const limit = commandReceipts.slice(limitStart);

  assert.match(commandReceipts, /const MAX_ACTION_ERROR_DISPLAY_CHARS: usize = 500;/);
  assert.match(writeJson, /let bytes = serialized_pretty_receipt\(&value, "metadata"\)\?;/);
  assert.match(
    writeActionError,
    /"command": action_error_display_field\(command\)/,
    "action-error command display must be bounded before serialization",
  );
  assert.match(
    writeActionError,
    /"error": action_error_display_field\(&error\.to_string\(\)\)/,
    "action-error error display must be bounded before serialization",
  );
  assert.match(
    writeActionError,
    /let bytes = serialized_pretty_receipt\(&value, "action error"\)\?;/,
  );
  assert.match(writeJson, /write_receipt_bytes\(path, bytes, "metadata"\)\?;/);
  assert.match(writeActionError, /write_receipt_bytes\(&path, bytes, "action error"\)\?;/);
  assert.match(serializer, /serde_json::to_vec_pretty\(value\)/);
  assert.match(serializer, /bytes\.push\(b'\\n'\);/);
  assert.match(serializer, /ensure_serialized_receipt_bytes\(receipt_kind, &bytes\)\?;/);
  assert.match(writeBytes, /let temp_path = temp_receipt_path\(path\)\?;/);
  assert.match(writeBytes, /fs::write\(&temp_path, bytes\)/);
  assert.match(writeBytes, /fs::rename\(&temp_path, path\)/);
  assert.match(writeBytes, /fs::remove_file\(path\)/);
  assert.match(tempPath, /path\.with_file_name\(format!\(/);
  assert.ok(
    serializer.indexOf("bytes.push(b'\\n');") <
      serializer.indexOf("ensure_serialized_receipt_bytes(receipt_kind, &bytes)?"),
    "serialized receipt size check must include trailing newline",
  );
  assert.match(
    limit,
    /u64::try_from\(bytes\.len\(\)\)\.unwrap_or\(u64::MAX\) > MAX_RECEIPT_BYTES/,
  );
  assert.ok(
    writeJson.indexOf("serialized_pretty_receipt") < writeJson.indexOf("write_receipt_bytes"),
    "metadata receipts must be serialized and bounded before staged write",
  );
  assert.ok(
    writeActionError.indexOf("serialized_pretty_receipt") <
      writeActionError.indexOf("write_receipt_bytes"),
    "action-error receipts must be serialized and bounded before staged write",
  );
});

test("DX Agent receipt display strings are redacted and bounded at parser boundaries", () => {
  const receipts = read("crates/agent_ui/src/dx_agent_bridge/receipts.rs");
  const receiptStrings = read("crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs");

  assert.match(receiptStrings, /const MAX_RECEIPT_DISPLAY_CHARS: usize = 180;/);
  assert.match(receiptStrings, /fn receipt_string_field/);
  assert.match(receiptStrings, /safe_string_field\(value, path\)\.and_then\(bound_receipt_string\)/);
  assert.match(receiptStrings, /fn receipt_string_array_field/);
  assert.match(receiptStrings, /fn receipt_string_values_field/);
  assert.match(receiptStrings, /take\(MAX_RECEIPT_STRING_VALUES\)/);
  assert.match(receiptStrings, /split_whitespace\(\)\.collect::<Vec<_>>\(\)\.join\(" "\)/);
  assert.match(receiptStrings, /take\(MAX_RECEIPT_DISPLAY_CHARS\.saturating_sub\(3\)\)/);
  assert.match(receiptStrings, /bounded\.push_str\("\.\.\."\)/);
  assert.doesNotMatch(receipts, /(?<!receipt_|safe_)string_field\(/);
  assert.doesNotMatch(receipts, /(?<!receipt_)string_array_field\(/);
  assert.doesNotMatch(receipts, /(?<!receipt_)string_values_field\(/);

  const criticalBoundaries = [
    "safe_regeneration_command",
    "next_action",
    "operator_summary",
    "warning_reasons",
    "blocking_reasons",
    "recovery_commands",
    "last_error",
    "command",
    "status",
  ];

  for (const boundary of criticalBoundaries) {
    assert.match(
      receipts,
      new RegExp(`receipt_string_(field|array_field|values_field)\\(value, &\\["${boundary}"\\]`),
      `expected ${boundary} to use the redacted bounded receipt string helper`,
    );
  }

  assert.match(
    receipts,
    /let label = receipt_string_field\(row, &\["label"\]\)\?/,
    "release-gate acceptance row labels must be redacted and bounded",
  );
  assert.match(
    receipts,
    /let status =\s*receipt_string_field\(row, &\["status"\]\)/,
    "release-gate acceptance row statuses must be redacted and bounded",
  );
});
