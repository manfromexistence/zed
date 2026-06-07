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
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const automationScreenView = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen.rs",
  );
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
  assert.match(agentPanel, /"dx-launch-automations"/);
  assert.match(agentPanel, /zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.doesNotMatch(
    agentPanel,
    /"dx-launch-automations"[\s\S]*?OpenProjectDebugTasks/,
    "Automations launch rail action must not route to debugger tasks",
  );
  assert.match(launchWorkspace, /pub\(crate\) use automation_screen::render_automation_screen;/);
  assert.doesNotMatch(launchWorkspace, /pub\(crate\) fn render_automation_screen/);
  assert.match(automationScreenView, /"pending DX Agents runtime"/);
  assert.match(automationScreenView, /agents::dx_agent_automation_state/);
  assert.match(automationScreenView, /agents::dx_agent_receipt_state/);
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

test("DX Automations have a first-class workspace tab contract", () => {
  const agentUi = read("crates/agent_ui/src/agent_ui.rs");
  const automationScreen = read("crates/agent_ui/src/automation_screen.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const item = read("crates/workspace/src/item.rs");
  const workspace = read("crates/workspace/src/workspace.rs");
  const pane = read("crates/workspace/src/pane.rs");
  const sidebar = read("crates/sidebar/src/sidebar.rs");
  const titleBar = read("crates/title_bar/src/title_bar.rs");
  const zedActions = read("crates/zed_actions/src/lib.rs");
  const zed = read("crates/zed/src/zed.rs");

  assert.match(agentUi, /^mod automation_screen;$/m);
  assert.match(agentUi, /pub use crate::automation_screen::AutomationScreen;/);
  assert.match(item, /pub enum WorkspaceScreenKind \{[\s\S]*Agent,\s*Automations,\s*Editor,/);
  assert.match(zedActions, /OpenAutomations/);
  assert.match(zed, /register_action\(agent_ui::AutomationScreen::open\)/);

  assert.match(automationScreen, /pub struct AutomationScreen \{\s*panel: Entity<AgentPanel>,\s*\}/);
  assert.match(automationScreen, /AgentPanel::new_automation_workspace\(workspace, window, cx\)/);
  assert.match(automationScreen, /pub\(crate\) fn open_or_focus\(/);
  assert.match(automationScreen, /workspace\.dismiss_zoomed_agent_panel\(window, cx\);/);
  assert.match(automationScreen, /workspace\.pane_for_screen_kind\(WorkspaceScreenKind::Automations, cx\)/);
  assert.match(automationScreen, /item\.screen_kind\(cx\) == WorkspaceScreenKind::Automations/);
  assert.match(automationScreen, /workspace\.add_item\(target_pane, Box::new\(item\), None, true, true, window, cx\);/);
  assert.match(automationScreen, /fn tab_content_text\(&self,[\s\S]*"Automations"\.into\(\)/);
  assert.match(automationScreen, /fn screen_kind\(&self\) -> WorkspaceScreenKind \{\s*WorkspaceScreenKind::Automations\s*\}/);
  assert.match(automationScreen, /dx_icon\(DxUiIcon::Automations\)/);
  assert.match(automationScreen, /fn show_toolbar\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(automationScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(automationScreen, /pub fn open\([\s\S]*_: &OpenAutomations/);

  assert.match(agentPanel, /AgentPanelHostKind::AutomationWorkspace/);
  assert.match(agentPanel, /render_automation_workspace_screen/);
  assert.match(agentPanel, /let automation_workspace =\s*matches!\(self\.host_kind, AgentPanelHostKind::AutomationWorkspace\);/);
  assert.match(agentPanel, /automation_workspace\s*\|\|\s*\(self\.should_render_dx_launch_chrome\(cx\)/);
  assert.match(workspace, /WorkspaceScreenKind::Automations => \{\s*window\.dispatch_action\(\s*zed_actions::assistant::OpenAutomations\.boxed_clone\(\),\s*cx,\s*\);\s*\}/s);
  assert.match(
    pane,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Onboarding/s,
  );
  assert.match(sidebar, /"sidebar-toolbar-automations"[\s\S]*?zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.match(sidebar, /"sidebar-activity-automations"[\s\S]*?zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.match(titleBar, /WorkspaceScreenKind::Automations/);
  assert.match(titleBar, /zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.doesNotMatch(automationScreen, /dummy|fake|OpenProjectDebugTasks/);
});

test("DX Automation source stays split into focused files", () => {
  for (const file of [
    "crates/agent_ui/src/automation_screen.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs",
  ]) {
    assert.ok(existsSync(file), `expected automation source file ${file}`);
  }

  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge.rs") < 880);
  assert.ok(lineCount("crates/agent_ui/src/automation_screen.rs") < 115);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen.rs") < 120);
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
