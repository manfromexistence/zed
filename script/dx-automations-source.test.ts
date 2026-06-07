import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("DX Automations expose typed bridge schema and pending composer contract", () => {
  const bridge = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const contract = read("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs");
  const contractTests = read(
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
  );

  assert.match(bridge, /^mod automation_contract;$/m);
  assert.match(bridge, /automation_contract::\{automation_composer, automations\}/);
  assert.match(bridge, /pub automation_composer: DxAgentAutomationComposer/);
  assert.match(bridge, /"automate-composer-latest\.json"/);
  assert.match(bridge, /"automation-composer-latest\.json"/);
  assert.match(contract, /struct DxAgentAutomation \{/);
  for (const field of [
    "name",
    "prompt",
    "schedule",
    "status",
    "destination",
    "last_run",
    "next_run",
    "receipts",
    "history",
  ]) {
    assert.match(contract, new RegExp(`pub ${field}:`));
  }

  assert.match(contract, /struct DxAgentAutomationComposer \{/);
  assert.match(contract, /struct DxAgentAutomationComposerField \{/);
  assert.match(contract, /struct DxAgentAutomationSchedule \{/);
  assert.match(contract, /struct DxAgentAutomationStatus \{/);
  assert.match(contract, /struct DxAgentAutomationDestination \{/);
  assert.match(contract, /struct DxAgentAutomationReceiptRef \{/);
  assert.match(contract, /struct DxAgentAutomationHistoryEntry \{/);
  assert.match(contract, /waiting_for_automation_composer_contract/);
  assert.match(contract, /pending_backend_contract/);
  assert.doesNotMatch(contract, /HashMap<String,\s*Value>/);
  assert.match(contractTests, /automation_rows_parse_composer_ready_contract_fields/);
  assert.match(contractTests, /automation_composer_falls_back_to_pending_backend_contract/);
});

test("DX Automations remain receipt-backed and do not fake scheduled execution", () => {
  const contract = read("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs");
  const runtime = read("crates/agent_ui/src/dx_agent_bridge/runtime.rs");
  const configuration = read("crates/agent_ui/src/agent_configuration.rs");
  const rail = read("crates/agent_ui/src/dx_launch_workspace/agents/automations.rs");
  const composer = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs");
  const labels = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs");
  const rows = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs");

  assert.match(contract, /pub\(super\) fn automation_composer/);
  assert.match(contract, /pub\(super\) fn automations\(value: &Value\)/);
  assert.match(contract, /array_field\(value, &\["automations"\]\)[\s\S]*\.take\(12\)/);
  assert.match(contract, /fn automation_row_actions/);
  assert.match(contract, /is_dx_agents_command\(refresh_command, "automate list --json"\)/);
  assert.match(contract, /receipt_filename == "run-latest\.json"/);
  assert.match(contract, /receipt_filename == "automate-list-latest\.json"/);
  assert.match(contract, /writes_receipt/);
  assert.match(contract, /secrets_exposed/);
  assert.match(contract, /is_secret_like_arg/);
  assert.doesNotMatch(runtime, /pub\(super\) fn automations/);
  assert.doesNotMatch(runtime, /fn automation_row_actions/);

  assert.match(rail, /dx_agent_automation_composer_contract/);
  assert.match(rail, /"dx agents automate list --json"/);
  assert.match(rail, /muted_card\("Run automation list receipt"/);
  assert.match(composer, /"pending runtime"/);
  assert.match(composer, /composer\.receipt_filename/);
  assert.match(composer, /composer\.unavailable_reason/);
  assert.match(rows, /automation\.name/);
  assert.match(rows, /automation\.prompt/);
  assert.match(labels, /automation\.destination/);
  assert.match(rows, /automation\.last_run/);
  assert.match(rows, /automation\.next_run/);
  assert.match(labels, /automation\.receipts/);
  assert.match(labels, /automation\.history/);
  assert.match(configuration, /Button::new\("dx-agent-automation-save-draft", "Save Draft"\)/);
  assert.match(configuration, /Button::new\("dx-agent-automation-enable", "Enable"\)/);
  assert.match(configuration, /\.disabled\(!composer\.save_draft_available\)/);
  assert.match(configuration, /\.disabled\(!composer\.enable_available\)/);
  assert.match(
    configuration,
    /run_action\.map_or\(false, \|action\| action\.enabled\)\s*&& automation\.status\.runtime_available/,
  );
});

test("DX Automation source stays split into focused files", () => {
  for (const file of [
    "crates/agent_ui/src/dx_agent_bridge/automation_contract.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs",
  ]) {
    assert.ok(existsSync(file), `expected automation source file ${file}`);
  }

  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge.rs") < 880);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs") < 520);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs") < 150);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations.rs") < 70);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs") < 65,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs") < 70,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs") < 145,
  );
});
