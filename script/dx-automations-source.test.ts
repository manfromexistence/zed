import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const sidebarWorkspaceActionArm = (kind: string, action: string) =>
  new RegExp(
    `WorkspaceScreenKind::${kind} => \\{\\s*` +
      `let action = zed_actions::assistant::${action}\\.boxed_clone\\(\\);\\s*` +
      `self\\.dispatch_workspace_action\\(action\\.as_ref\\(\\), window, cx\\);\\s*` +
      `return;\\s*\\}`,
  );
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("DX Automations expose typed bridge schema and pending composer contract", () => {
  const bridge = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const contract = read("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs");
  const actionTests = read(
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_tests.rs",
  );
  const actionSafetyTests = read(
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_safety_tests.rs",
  );
  const contractTests = read(
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
  );
  const contractSafetyTests = read(
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_safety_tests.rs",
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
  assert.match(contract, /pub\(crate\) fn has_successful_execution_proof/);
  assert.match(contract, /pub\(crate\) fn has_failed_execution_proof/);
  assert.match(contract, /waiting_for_automation_composer_contract/);
  assert.match(contract, /pending_backend_contract/);
  assert.doesNotMatch(contract, /HashMap<String,\s*Value>/);
  assert.match(actionTests, /automation_run_action_requires_typed_backend_target/);
  assert.match(actionTests, /automation_composer_actions_parse_backend_contract/);
  assert.match(actionSafetyTests, /automation_row_actions_reject_mismatched_row_targets/);
  assert.match(actionSafetyTests, /automation_row_actions_reject_generic_or_mismatched_public_commands/);
  assert.match(contractTests, /automation_rows_parse_composer_ready_contract_fields/);
  assert.match(contractTests, /automation_composer_falls_back_to_pending_backend_contract/);
  assert.match(contractSafetyTests, /automation_execution_proof_requires_successful_run_receipt/);
  assert.match(contractSafetyTests, /automation_execution_proof_flags_failed_run_receipt/);
  assert.match(contractSafetyTests, /automation_text_fields_are_redacted_cleaned_and_bounded/);
});

test("DX Automations remain receipt-backed and do not fake scheduled execution", () => {
  const contract = read("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs");
  const actions = read("crates/agent_ui/src/dx_agent_bridge/automation_actions.rs");
  const runtime = read("crates/agent_ui/src/dx_agent_bridge/runtime.rs");
  const commands = read("crates/agent_ui/src/dx_agent_bridge/commands.rs");
  const configuration = read("crates/agent_ui/src/agent_configuration.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const automationScreenView = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen.rs",
  );
  const automationScreenSections = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections.rs",
  );
  const automationScreenDrafts = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/drafts.rs",
  );
  const automationScreenRows = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/rows.rs",
  );
  const rail = read("crates/agent_ui/src/dx_launch_workspace/agents/automations.rs");
  const composer = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs");
  const labels = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs");
  const rows = read("crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs");

  assert.match(contract, /pub\(super\) fn automation_composer/);
  assert.match(contract, /pub\(super\) fn automations\(value: &Value\)/);
  assert.match(contract, /pub receipt_present: bool/);
  assert.match(contract, /pub fields_receipt_backed: bool/);
  assert.match(contract, /fn field_summary\(&self, limit: usize\)/);
  assert.match(contract, /fn field_summary_label\(&self\)/);
  assert.match(contract, /Field template pending receipt/);
  assert.match(contract, /pub actions: Vec<DxAgentRowAction>/);
  assert.match(contract, /array_field\(value, &\["automations"\]\)[\s\S]*\.take\(12\)/);
  assert.match(actions, /pub\(super\) fn automation_composer_actions/);
  assert.match(actions, /pub\(super\) fn automation_row_actions/);
  assert.match(actions, /pub\(crate\) fn automation_public_command_for_action/);
  assert.match(actions, /is_safe_automation_id_arg/);
  assert.match(actions, /"automate run --id/);
  assert.match(actions, /"automate enable --id/);
  assert.match(actions, /"automate save-draft --json"/);
  assert.match(actions, /receipt_filename == "automate-run-latest\.json"/);
  assert.match(actions, /receipt_filename == "automate-list-latest\.json"/);
  assert.match(actions, /receipt_filename == "automate-draft-latest\.json"/);
  assert.match(actions, /writes_receipt/);
  assert.match(actions, /secrets_exposed/);
  assert.match(actions, /is_secret_like_arg/);
  assert.doesNotMatch(actions, /is_dx_agents_command\(command, "run --json"\)/);
  assert.doesNotMatch(runtime, /pub\(super\) fn automations/);
  assert.doesNotMatch(runtime, /fn automation_row_actions/);
  assert.match(commands, /AutomationSaveDraft/);
  assert.match(commands, /AutomationEnable \{ automation_id: String \}/);
  assert.match(commands, /AutomationRun \{ automation_id: String \}/);
  assert.match(commands, /dx_agents_automation_args\("run", automation_id\)/);

  assert.match(rail, /dx_agent_automation_composer_contract/);
  assert.match(rail, /"dx agents automate list --json"/);
  assert.match(rail, /muted_card\("Run automation list receipt"/);
  assert.match(agentPanel, /new_automation_workspace/);
  assert.match(agentPanel, /render_automation_workspace_screen/);
  assert.match(agentPanel, /register_action\(\|workspace, _: &OpenAutomations[\s\S]*?AutomationScreen::open_or_focus/);
  assert.doesNotMatch(
    agentPanel,
    /AgentPanelHostKind::AutomationWorkspace[\s\S]{0,1200}OpenProjectDebugTasks/,
    "Automations launch rail action must not route to debugger tasks",
  );
  assert.match(launchWorkspace, /pub\(crate\) use automation_screen::\{[\s\S]*render_automation_screen/);
  assert.doesNotMatch(launchWorkspace, /pub\(crate\) fn render_automation_screen/);
  for (const section of ["Drafts", "Schedules", "Runs", "History", "Failures"]) {
    assert.match(automationScreenView, new RegExp(`screen_section\\(\\s*"dx-automation-${section.toLowerCase()}"`));
  }
  assert.match(automationScreenView, /Composer receipt state, schedule contracts, history, and handoff evidence/);
  assert.doesNotMatch(automationScreenView, /Receipt-backed composer/);
  for (const state of [
    "drafts_state",
    "schedules_state",
    "runs_state",
    "history_state",
    "failures_state",
  ]) {
    assert.match(automationScreenView, new RegExp(`sections::${state}`));
  }
  assert.doesNotMatch(automationScreenView, /agents::dx_agent_automation_state/);
  assert.doesNotMatch(automationScreenView, /agents::dx_agent_receipt_state/);
  assert.match(automationScreenSections, /"pending DX Agents runtime"/);
  assert.match(automationScreenSections, /"dx agents automate list --json"/);
  assert.match(automationScreenSections, /No automation schedule rows from DX Agents/);
  assert.match(automationScreenSections, /No automation history rows in the current DX Agents receipt list/);
  assert.match(automationScreenSections, /No failed automation run receipts in the current list/);
  assert.match(automationScreenSections, /has_successful_execution_proof\(\)/);
  assert.match(automationScreenSections, /has_failed_execution_proof\(\)/);
  assert.match(automationScreenDrafts, /AiSettingItem::new/);
  assert.match(automationScreenRows, /has_successful_execution_proof\(\)/);
  assert.match(automationScreenRows, /has_failed_execution_proof\(\)/);
  assert.match(automationScreenRows, /AiSettingItem::new/);
  assert.match(automationScreenRows, /ListItem::new/);
  assert.match(composer, /"pending runtime"/);
  assert.match(composer, /composer\.receipt_filename/);
  assert.match(composer, /composer\.unavailable_reason/);
  assert.match(composer, /composer\.field_summary\(4\)/);
  assert.match(composer, /composer\.field_summary_label\(\)/);
  assert.match(automationScreenDrafts, /composer\.field_summary\(5\)/);
  assert.match(automationScreenDrafts, /composer\.field_summary_label\(\)/);
  assert.match(automationScreenDrafts, /composer\.empty_field_summary_label\(\)/);
  assert.match(configuration, /composer\.field_summary\(4\)/);
  assert.match(configuration, /composer\.field_summary_label\(\)/);
  assert.doesNotMatch(composer, /Fields: \{field_summary\}/);
  assert.doesNotMatch(configuration, /Fields: \{composer_fields\}/);
  assert.match(rows, /automation\.name/);
  assert.match(rows, /automation\.prompt/);
  assert.match(labels, /automation\.destination/);
  assert.match(rows, /automation\.last_run/);
  assert.match(rows, /automation\.next_run/);
  assert.match(rows, /requested next/);
  assert.match(labels, /Execution proof pending/);
  assert.match(labels, /automation\.receipts/);
  assert.match(labels, /automation\.history/);
  assert.match(configuration, /automation\.has_successful_execution_proof\(\)/);
  assert.match(configuration, /automation\.has_failed_execution_proof\(\)/);
  assert.match(configuration, /Execution proof failed/);
  assert.match(rows, /automation\.has_successful_execution_proof\(\)/);
  assert.match(rows, /automation\.has_failed_execution_proof\(\)/);
  assert.doesNotMatch(
    configuration,
    /!\s*automation\.receipts\.is_empty\(\)\s*\|\|\s*!\s*automation\.history\.is_empty\(\)/,
  );
  assert.doesNotMatch(
    rows,
    /!\s*automation\.receipts\.is_empty\(\)\s*\|\|\s*!\s*automation\.history\.is_empty\(\)/,
  );
  assert.match(configuration, /Button::new\("dx-agent-automation-save-draft", "Save Draft"\)/);
  assert.match(configuration, /Button::new\("dx-agent-automation-enable", "Enable"\)/);
  assert.match(configuration, /let save_draft_action = dx_agent_row_action\(&composer\.actions, "save_draft"\)/);
  assert.match(configuration, /let enable_action = dx_agent_row_action\(&composer\.actions, "enable"\)/);
  assert.match(configuration, /automation_public_command_for_action/);
  assert.match(actions, /DxAgentPublicCommand::AutomationSaveDraft/);
  assert.match(actions, /DxAgentPublicCommand::AutomationEnable \{ automation_id \}/);
  assert.match(actions, /DxAgentPublicCommand::AutomationRun \{ automation_id \}/);
  assert.match(configuration, /automation\.status\.enabled && automation\.status\.runtime_available/);
  assert.match(configuration, /Execution proof pending/);
  assert.match(
    configuration,
    /run_action\.map_or\(false, \|action\| action\.enabled\)\s*&& automation\.status\.runtime_available/,
  );
  assert.match(configuration, /let run_command = run_action\.and_then\(automation_public_command_for_action\)/);
  assert.doesNotMatch(
    configuration,
    /dx-agent-automation-run-[\s\S]{0,700}DxAgentPublicCommand::Run/,
    "Automation row run must never dispatch the generic runtime run command",
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
  const workspaceScreenKind = item.match(/pub enum WorkspaceScreenKind \{[\s\S]*?\}/)?.[0] ?? "";

  assert.match(agentUi, /^mod automation_screen;$/m);
  assert.match(agentUi, /pub use crate::automation_screen::AutomationScreen;/);
  assert.match(workspaceScreenKind, /Agent/);
  assert.match(workspaceScreenKind, /Automations/);
  assert.match(workspaceScreenKind, /Editor/);
  assert.match(zedActions, /OpenAutomations/);
  assert.match(zed, /register_action\(agent_ui::AutomationScreen::open\)/);

  assert.match(automationScreen, /pub struct AutomationScreen \{\s*panel: Entity<AgentPanel>,\s*\}/);
  assert.match(automationScreen, /AgentPanel::new_automation_workspace\(workspace, window, cx\)/);
  assert.match(automationScreen, /pub\(crate\) fn open_or_focus\(/);
  assert.match(automationScreen, /workspace\.dismiss_zoomed_agent_panel\(window, cx\);/);
  assert.match(automationScreen, /\.pane_for_screen_kind\(WorkspaceScreenKind::Automations, cx\)/);
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
  assert.match(agentPanel, /let status_workspace =\s*matches!\(\s*self\.host_kind,[\s\S]*?AgentPanelHostKind::AutomationWorkspace[\s\S]*?AgentPanelHostKind::ConnectionsWorkspace[\s\S]*?AgentPanelHostKind::ToolsWorkspace[\s\S]*?\);/);
  assert.match(agentPanel, /status_workspace\s*\|\|\s*\(self\.should_render_dx_launch_chrome\(cx\)/);
  assert.match(workspace, /WorkspaceScreenKind::Automations => \{\s*window\.dispatch_action\(\s*zed_actions::assistant::OpenAutomations\.boxed_clone\(\),\s*cx,\s*\);\s*\}/s);
  assert.match(
    pane,
    /WorkspaceScreenKind::Agent\s*\|\s*WorkspaceScreenKind::Automations\s*\|\s*WorkspaceScreenKind::Connections\s*\|\s*WorkspaceScreenKind::Tools\s*\|\s*WorkspaceScreenKind::Onboarding/s,
  );
  assert.match(sidebar, /"sidebar-toolbar-automations"[\s\S]*?activate_workspace_screen\(\s*WorkspaceScreenKind::Automations/);
  assert.match(sidebar, /"sidebar-activity-automations"[\s\S]*?activate_workspace_screen\(WorkspaceScreenKind::Automations/);
  assert.match(sidebar, sidebarWorkspaceActionArm("Automations", "OpenAutomations"));
  assert.match(titleBar, /WorkspaceScreenKind::Automations/);
  assert.match(titleBar, /zed_actions::assistant::OpenAutomations\.boxed_clone\(\)/);
  assert.doesNotMatch(automationScreen, /dummy|fake|OpenProjectDebugTasks/);
});

test("DX Automation workspace follows the Extensions-style GPUI page pattern", () => {
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const chrome = read("crates/agent_ui/src/dx_launch_workspace/screen_chrome.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const catalogChrome = read("crates/agent_ui/src/dx_launch_workspace/catalog_chrome.rs");
  const automationScreen = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen.rs",
  );
  const automationCatalog = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog.rs",
  );
  const automationCatalogEntry = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/entry.rs",
  );
  const automationCatalogDetails = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/details.rs",
  );
  const automationCatalogSources = `${automationCatalog}\n${automationCatalogEntry}\n${automationCatalogDetails}`;
  const sections = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections.rs",
  );
  const drafts = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/drafts.rs",
  );
  const rows = read(
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/rows.rs",
  );

  assert.match(launchWorkspace, /^mod screen_chrome;$/m);
  assert.match(launchWorkspace, /DxAutomationCatalogState/);
  assert.match(chrome, /Headline::new\(title\)\.size\(HeadlineSize::Large\)/);
  assert.match(chrome, /ListHeader::new\(title\)/);
  assert.match(chrome, /elevated_surface_background\.opacity\(0\.5\)/);
  assert.match(catalogChrome, /pub\(super\) fn render_catalog_row_labels/);
  assert.match(catalogChrome, /pub\(super\) fn render_catalog_status_chip/);
  assert.match(catalogChrome, /Chip::new\(label\)/);

  assert.match(agentPanel, /automation_catalog_state: DxAutomationCatalogState/);
  assert.match(agentPanel, /_automation_catalog_query_subscription: Subscription/);
  assert.match(agentPanel, /render_automation_catalog_rows/);
  assert.match(agentPanel, /set_automation_catalog_filter/);
  assert.match(agentPanel, /set_automation_catalog_selected_entry/);

  assert.match(automationScreen, /^mod catalog;$/m);
  assert.match(automationScreen, /workspace_page_header\(\s*dx_icon\(DxUiIcon::Automations\)/);
  assert.match(automationScreen, /render_automation_catalog\(/);
  assert.match(automationScreen, /workspace_stat\(/);
  assert.doesNotMatch(automationScreen, /section_title\(/);
  assert.match(automationCatalogSources, /Editor::single_line/);
  assert.match(automationCatalogSources, /ToggleButtonGroup::single_row/);
  assert.match(automationCatalogSources, /UniformListScrollHandle/);
  assert.match(automationCatalogSources, /uniform_list\(/);
  assert.match(automationCatalog, /"dx-automation-catalog-list"/);
  assert.match(automationCatalog, /"Automation Catalog"/);
  assert.match(automationCatalog, /render_catalog_row_labels\(/);
  assert.match(automationCatalog, /\.start_slot\(\s*Icon::new\(entry\.icon\(\)\)/);
  assert.match(automationCatalog, /\.end_slot\(render_catalog_status_chip\(entry\.status\(snapshot\)\)\)/);
  assert.match(automationCatalog, /\.tooltip\(Tooltip::text\(tooltip\)\)/);
  assert.match(automationCatalogSources, /render_selected_automation_detail/);
  assert.match(automationCatalogSources, /Composer contract/);
  assert.match(automationCatalogSources, /Execution proof/);
  assert.match(automationCatalogSources, /screen_detail_row\(/);
  assert.doesNotMatch(automationCatalog, /AiSettingItem::new/);
  assert.doesNotMatch(automationCatalog, /AiSettingItemSource/);
  assert.doesNotMatch(automationCatalogSources, /\b(?:Button::new|IconButton::new|run_dx_agent_public_command|run_dx_agents_public_action|DxAgentPublicCommand::Run)\b/);
  assert.match(sections, /screen_detail_row\(/);
  assert.match(sections, /screen_empty_state\(/);
  assert.match(drafts, /screen_detail_stack\(/);
  assert.match(rows, /screen_detail_row\(/);
  assert.doesNotMatch(rows, /ListItemSpacing::ExtraDense/);
});

test("DX Automation source stays split into focused files", () => {
  for (const file of [
    "crates/agent_ui/src/automation_screen.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_actions.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_actions_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_safety_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs",
    "crates/agent_ui/src/dx_agent_bridge/command_args_tests.rs",
    "crates/agent_ui/src/dx_launch_workspace/screen_chrome.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/entry.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/details.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/drafts.rs",
    "crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/rows.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs",
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs",
  ]) {
    assert.ok(existsSync(file), `expected automation source file ${file}`);
  }

  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge.rs") < 880);
  assert.ok(lineCount("crates/agent_ui/src/automation_screen.rs") < 115);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen.rs") < 150);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog.rs") < 430);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/entry.rs") < 260,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/catalog/details.rs") <
      360,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/screen_chrome.rs") < 180);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/sections.rs") < 230);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/drafts.rs") < 90,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/automation_screen/sections/rows.rs") < 260,
  );
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions.rs") < 230);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions_safety_tests.rs") < 80);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_actions_tests.rs") < 110);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract.rs") < 520);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract_safety_tests.rs") < 130);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/automation_contract_tests.rs") < 180);
  assert.ok(lineCount("crates/agent_ui/src/dx_agent_bridge/command_args_tests.rs") < 50);
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations.rs") < 70);
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/composer.rs") < 65,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/labels.rs") < 70,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs") < 170,
  );
});
