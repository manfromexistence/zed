import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("DX Agent bridge stays split by command, runtime, and receipt ownership", () => {
  const parent = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const expectedModules = [
    "crates/agent_ui/src/dx_agent_bridge/command_safety.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_receipts.rs",
    "crates/agent_ui/src/dx_agent_bridge/commands.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs",
    "crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs",
    "crates/agent_ui/src/dx_agent_bridge/local_files.rs",
    "crates/agent_ui/src/dx_agent_bridge/receipts.rs",
    "crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_catalog_fields.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_display.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_provider_models_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime.rs",
    "crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs",
  ];

  for (const module of expectedModules) {
    assert.ok(existsSync(module), `expected focused DX Agent bridge module ${module}`);
  }

  assert.match(parent, /^mod command_receipts;$/m);
  assert.match(parent, /^mod command_safety;$/m);
  assert.match(parent, /^mod commands;$/m);
  assert.match(parent, /^mod local_file_labels;$/m);
  assert.match(parent, /^mod local_files;$/m);
  assert.match(parent, /^mod receipts;$/m);
  assert.match(parent, /^mod runtime;$/m);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_agent_bridge.rs") < 820,
    "dx_agent_bridge.rs should stay a coordinator and type boundary",
  );
});

test("DX Agent bridge delegates bridge commands and receipt parsing", () => {
  const parent = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const safety = read("crates/agent_ui/src/dx_agent_bridge/command_safety.rs");
  const safetyTests = read("crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs");
  const commandReceipts = read("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs");
  const commands = read("crates/agent_ui/src/dx_agent_bridge/commands.rs");
  const catalogLabels = read("crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs");
  const catalogLabelsTests = read("crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs");
  const localFileLabels = read("crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs");
  const localFiles = read("crates/agent_ui/src/dx_agent_bridge/local_files.rs");
  const receipts = read("crates/agent_ui/src/dx_agent_bridge/receipts.rs");
  const receiptStrings = read("crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs");
  const runtimeCatalog = read("crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs");
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
  const runtime = read("crates/agent_ui/src/dx_agent_bridge/runtime.rs");
  const runtimeTests = read("crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs");

  assert.doesNotMatch(parent, /fn run_bridge_command/);
  assert.doesNotMatch(parent, /fn contract_summary/);
  assert.doesNotMatch(parent, /fn social_accounts/);
  assert.doesNotMatch(parent, /fn is_secret_like_arg/);
  assert.doesNotMatch(parent, /fn public_command_for_runtime/);
  assert.match(parent, /use self::command_safety::\{/);
  assert.match(parent, /^mod catalog_labels;$/m);
  assert.match(parent, /pub\(crate\) use self::catalog_labels::\{/);
  assert.match(safety, /pub\(crate\) fn is_secret_like_arg/);
  assert.match(safety, /pub\(crate\) fn redact_action_scalar/);
  assert.match(safety, /pub\(crate\) fn public_command_for_runtime/);
  assert.match(safety, /pub\(crate\) fn is_safe_platform_arg/);
  assert.match(safety, /pub\(crate\) fn bridge_command_label/);
  assert.match(safety, /#\[path = "command_safety_tests\.rs"\]/);
  assert.match(safety, /normalized\.contains\(marker\)/);
  assert.match(safety, /let mut redact_next = false/);
  assert.match(safety, /redact_next = is_secret_flag_arg\(arg\)/);
  assert.match(safety, /fn is_secret_flag_arg/);
  assert.match(safetyTests, /dx_agent_secret_marker_guard_covers_bridge_receipt_scalars/);
  assert.match(safetyTests, /public_command_for_runtime_maps_legacy_dx_agents_commands/);
  assert.match(safetyTests, /bridge_command_label_redacts_secret_like_args/);
  assert.match(safetyTests, /bridge_command_label_redacts_secret_key_value_args/);
  assert.match(commandReceipts, /pub\(super\) fn write_json_receipt/);
  assert.match(commandReceipts, /pub\(super\) fn write_action_error_receipt/);
  assert.match(commandReceipts, /pub\(super\) fn clear_action_error_receipt/);
  assert.match(commands, /pub\(crate\) fn run_dx_agent_public_command/);
  assert.match(commands, /pub\(crate\) enum DxAgentPublicCommand/);
  assert.match(localFiles, /pub\(super\) fn read_json/);
  assert.match(localFiles, /pub\(super\) fn read_first_json/);
  assert.match(localFiles, /pub\(super\) fn latest_receipts/);
  assert.match(localFiles, /pub\(super\) fn dx_home_from_receipt_root/);
  assert.match(localFiles, /receipt_file_label/);
  assert.match(localFiles, /MAX_RECEIPT_BYTES/);
  assert.match(localFileLabels, /pub\(crate\) fn receipt_file_label/);
  assert.match(localFileLabels, /eq_ignore_ascii_case\("json"\)/);
  assert.match(localFileLabels, /receipt_file_label_accepts_uppercase_json_extension/);
  assert.match(receipts, /pub\(super\) fn contract_summary/);
  assert.match(receipts, /pub\(super\) fn receipt_index_summary/);
  assert.match(receipts, /^mod receipt_strings;$/m);
  assert.match(receipts, /use self::receipt_strings::\{/);
  assert.match(receiptStrings, /pub\(super\) fn receipt_string_field/);
  assert.match(receiptStrings, /pub\(super\) fn receipt_string_array_field/);
  assert.match(receiptStrings, /pub\(super\) fn receipt_string_values_field/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_cache_state_label/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_detail_label/);
  assert.match(catalogLabels, /pub\(crate\) fn catalog_active_provider_label/);
  assert.match(catalogLabels, /#\[path = "catalog_labels_tests\.rs"\]/);
  assert.match(catalogLabelsTests, /catalog_detail_label_separates_catalog_from_readiness/);
  assert.match(catalogLabelsTests, /catalog_active_provider_label_prefers_display_name/);
  assert.match(runtime, /pub\(super\) fn social_accounts/);
  assert.match(runtime, /runtime_catalog::catalog_summary/);
  assert.match(runtime, /#\[path = "runtime_catalog_fields\.rs"\]\s*mod runtime_catalog_fields;/);
  assert.match(runtime, /runtime_provider_models::\{models, providers\}/);
  assert.match(runtime, /#\[path = "runtime_display\.rs"\]/);
  assert.match(runtimeCatalog, /pub\(in super::super\) fn catalog_summary/);
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
  assert.match(runtimeProviderModels, /fn grouped_model_rows/);
  assert.match(runtimeProviderModels, /#\[path = "runtime_provider_models_tests\.rs"\]/);
  assert.match(runtimeProviderModelsTests, /grouped_model_rows_skip_blank_string_entries/);
  assert.match(
    runtimeProviderModelsTests,
    /provider_and_model_rows_bound_and_redact_display_values/,
  );
  assert.match(runtime, /#\[path = "runtime_tests\.rs"\]/);
  assert.match(runtimeTests, /provider_rows_read_agent_cli_provider_receipts/);
  assert.match(runtimeTests, /model_rows_flatten_agent_cli_provider_model_groups/);
  assert.match(runtimeTests, /legacy_flat_model_rows_still_parse/);
  assert.match(runtimeTests, /catalog_summary_reads_agent_cli_catalog_diagnostics/);
  assert.match(runtimeTests, /catalog_summary_derives_provider_honesty_from_provider_rows/);
  assert.match(runtimeTests, /catalog_summary_reports_missing_receipt_root/);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_safety.rs") < 120);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_safety_tests.rs") < 130);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_receipts.rs") < 210);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/commands.rs") < 240);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_labels.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/catalog_labels_tests.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/local_file_labels.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/local_files.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/receipts.rs") < 560);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/receipts/receipt_strings.rs") < 75);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_catalog.rs") < 90);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_catalog_fields.rs") < 120);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_display.rs") < 80);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_provider_models.rs") < 190);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_provider_models_tests.rs") < 120,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime.rs") < 420);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/runtime_tests.rs") < 210);
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
