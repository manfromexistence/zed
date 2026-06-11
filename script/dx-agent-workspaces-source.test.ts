import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;
const enumBody = (source: string, name: string) =>
  source.match(new RegExp(`enum ${name} \\{[\\s\\S]*?\\}`))?.[0] ?? "";

test("DX agent workspace taxonomy has first-class Zed screens", () => {
  const item = read("crates/workspace/src/item.rs");
  const actions = read("crates/zed_actions/src/lib.rs");
  const agentUi = read("crates/agent_ui/src/agent_ui.rs");
  const zed = read("crates/zed/src/zed.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const agentScreen = read("crates/agent_ui/src/agent_screen.rs");
  const workspace = read("crates/workspace/src/workspace.rs");
  const pane = read("crates/workspace/src/pane.rs");
  const titleBar = read("crates/title_bar/src/title_bar.rs");
  const sidebar = read("crates/sidebar/src/sidebar.rs");
  const carousel = read("crates/workspace/src/screen_carousel.rs");
  const dxWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const agentWorkspace = read(
    "crates/agent_ui/src/dx_launch_workspace/agent_workspace.rs",
  );
  const screenKinds = enumBody(item, "WorkspaceScreenKind");

  assert.ok(existsSync("crates/agent_ui/src/dx_launch_workspace/agent_workspace.rs"));
  assert.ok(lineCount("crates/agent_ui/src/dx_launch_workspace/agent_workspace.rs") < 260);
  for (const kind of ["Agent", "Automations", "Connections", "Tools", "Editor"]) {
    assert.match(screenKinds, new RegExp(`\\b${kind}\\b`));
  }

  for (const action of ["OpenAutomations", "OpenConnections", "OpenTools"]) {
    assert.match(actions, new RegExp(`\\b${action}\\b`));
  }

  assert.match(agentUi, /^mod connections_screen;$/m);
  assert.match(agentUi, /^mod tools_screen;$/m);
  assert.match(agentUi, /pub use crate::connections_screen::ConnectionsScreen;/);
  assert.match(agentUi, /pub use crate::tools_screen::ToolsScreen;/);
  assert.match(zed, /register_action\(agent_ui::ConnectionsScreen::open\)/);
  assert.match(zed, /register_action\(agent_ui::ToolsScreen::open\)/);

  assert.match(agentPanel, /AgentPanelHostKind::ConnectionsWorkspace/);
  assert.match(agentPanel, /AgentPanelHostKind::ToolsWorkspace/);
  assert.match(agentPanel, /new_connections_workspace/);
  assert.match(agentPanel, /new_tools_workspace/);
  assert.match(agentPanel, /render_connections_workspace_screen/);
  assert.match(agentPanel, /render_tools_workspace_screen/);
  assert.match(agentPanel, /render_connections_screen\(status\.as_ref\(\), cx\)/);
  assert.match(agentPanel, /render_tools_screen\(status\.as_ref\(\), cx\)/);

  assert.match(workspace, /WorkspaceScreenKind::Connections => \{[\s\S]*?zed_actions::assistant::OpenConnections\.boxed_clone\(\)/);
  assert.match(workspace, /WorkspaceScreenKind::Tools => \{[\s\S]*?zed_actions::assistant::OpenTools\.boxed_clone\(\)/);
  assert.match(pane, /WorkspaceScreenKind::Connections/);
  assert.match(pane, /WorkspaceScreenKind::Tools/);
  assert.match(titleBar, /WorkspaceScreenKind::Connections => "Connections"/);
  assert.match(titleBar, /WorkspaceScreenKind::Tools => "Plugins"/);
  assert.match(titleBar, /WorkspaceScreenKind::Connections => dx_icon\(DxUiIcon::Connections\)/);
  assert.match(titleBar, /WorkspaceScreenKind::Tools => dx_icon\(DxUiIcon::Plugins\)/);
  assert.match(titleBar, /WorkspaceScreenKind::Agent => IconName::Sparkle/);
  assert.match(carousel, /WorkspaceScreenKind::Connections => "Connections"/);
  assert.match(carousel, /WorkspaceScreenKind::Tools => "Plugins"/);
  assert.match(carousel, /WorkspaceScreenKind::Agent => IconName::Sparkle/);
  assert.match(agentScreen, /Icon::new\(dx_icon\(DxUiIcon::Agent\)\)/);
  assert.match(dxWorkspace, /"dx-agent-overview-section"[\s\S]*?dx_icon\(DxUiIcon::Agent\)/);
  assert.match(dxWorkspace, /"dx-agent-subagents-section"[\s\S]*?dx_icon\(DxUiIcon::Agent\)/);
  assert.match(agentWorkspace, /"dx-agent-overview-active-thread"[\s\S]*?dx_icon\(DxUiIcon::Agent\)/);
  assert.match(agentWorkspace, /"dx-agent-environment-worktrees"[\s\S]*?dx_icon\(DxUiIcon::Project\)/);
  assert.match(agentWorkspace, /"dx-agent-sources-total"[\s\S]*?dx_icon\(DxUiIcon::Source\)/);

  assert.match(
    sidebar,
    /fn activate_workspace_screen\([\s\S]*?workspace\.activate_screen_kind\(kind, window, cx\)/,
  );
  assert.match(sidebar, /"sidebar-toolbar-connections"[\s\S]*?activate_workspace_screen\(\s*WorkspaceScreenKind::Connections/);
  assert.match(sidebar, /"sidebar-activity-connections"[\s\S]*?activate_workspace_screen\(WorkspaceScreenKind::Connections/);
  assert.match(sidebar, /"sidebar-toolbar-plugins"[\s\S]*?activate_workspace_screen\(WorkspaceScreenKind::Tools/);
  assert.match(sidebar, /"sidebar-activity-plugins"[\s\S]*?activate_workspace_screen\(WorkspaceScreenKind::Tools/);

  assert.match(dxWorkspace, /^mod agent_workspace;$/m);
  assert.match(dxWorkspace, /pub background_thread_count: usize/);
  assert.match(agentPanel, /background_thread_count: background_task_count/);

  for (const [id, label, section] of [
    ["dx-agent-overview-section", "Progress", "AgentOverview"],
    ["dx-agent-threads-section", "Environment", "AgentThreads"],
    ["dx-agent-tasks-section", "Sources", "AgentTasks"],
    ["dx-agent-subagents-section", "Subagents", "AgentSubagents"],
    ["dx-agent-approvals-section", "Readiness", "AgentApprovals"],
  ] as const) {
    assert.match(dxWorkspace, new RegExp(`"${id}"`));
    assert.match(dxWorkspace, new RegExp(`"${label}"`));
    assert.match(dxWorkspace, new RegExp(`DxLaunchRailSection::${section}`));
  }

  for (const fnName of [
    "agent_overview_section",
    "agent_environment_section",
    "agent_sources_section",
    "agent_subagents_section",
    "agent_approvals_section",
  ]) {
    assert.match(agentWorkspace, new RegExp(`fn ${fnName}\\(`));
  }

  assert.match(agentWorkspace, /status\.active_status/);
  assert.match(agentWorkspace, /status\.background_thread_count/);
  assert.match(agentWorkspace, /status\.agent_bridge\.active_task_count/);
  assert.match(agentWorkspace, /status\.agent_bridge\.trusted_tool_bridge/);
  assert.match(agentWorkspace, /status\.source_sets\.attachment_summary\(\)/);
  assert.match(agentWorkspace, /subagent_summary\(status, cx\)/);
  assert.doesNotMatch(agentWorkspace, /No active source context/);
  assert.match(agentWorkspace, /Blocked trusted tool approval receipts need review/);
  assert.match(dxWorkspace, /compact_status_row\(\s*"dx-subagents-more"/);
  assert.ok(!dxWorkspace.includes('"+{} more"'));
  assert.match(agentWorkspace, /compact_status_row\([\s\S]*?"dx-agent-approvals-gate"/);
  assert.doesNotMatch(agentWorkspace, /metric_row\(\s*"Gate review"/);
});

test("Connections workspace is wired to provider, channel, social, gateway, and credential state", () => {
  const dxWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const screen = read("crates/agent_ui/src/dx_launch_workspace/connections_screen.rs");
  const bridge = read("crates/agent_ui/src/dx_agent_bridge.rs");
  const connectionsScreen = read("crates/agent_ui/src/connections_screen.rs");

  assert.ok(existsSync("crates/agent_ui/src/dx_launch_workspace/connections_screen.rs"));
  assert.match(dxWorkspace, /^mod connections_screen;$/m);
  assert.match(dxWorkspace, /pub\(crate\) use connections_screen::render_connections_screen;/);
  assert.match(connectionsScreen, /AgentPanel::new_connections_workspace\(workspace, window, cx\)/);
  assert.match(connectionsScreen, /WorkspaceScreenKind::Connections/);
  assert.match(connectionsScreen, /fn show_toolbar\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(connectionsScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);

  for (const title of ["Providers", "Channels", "Social", "Gateway", "Credentials"]) {
    assert.match(screen, new RegExp(`section_title\\("${title}"`));
  }
  assert.match(screen, /agents::dx_agent_provider_state\(snapshot, cx\)/);
  assert.match(screen, /agents::dx_agent_social_state\(snapshot, cx\)/);
  assert.match(screen, /connected_accounts_summary/);
  assert.match(screen, /credential_error/);
  assert.match(screen, /credential_expires_at/);
  assert.match(screen, /No DX Agents channel receipt\/schema is available yet\./);
  assert.match(screen, /No first-class provider gateway health receipt is available yet\./);
  assert.match(screen, /AiSettingItem::new/);
  assert.match(screen, /ListItem::new/);

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
    assert.match(bridge, new RegExp(`pub ${field}:`));
  }
});

test("Plugins workspace exposes first-party catalog cards without fake approvals", () => {
  const dxWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const screen = read("crates/agent_ui/src/dx_launch_workspace/tools_screen.rs");
  const catalog = read(
    "crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs",
  );
  const toolsScreen = read("crates/agent_ui/src/tools_screen.rs");
  const bridge = read("crates/agent_ui/src/dx_agent_bridge.rs");

  assert.ok(existsSync("crates/agent_ui/src/dx_launch_workspace/tools_screen.rs"));
  assert.ok(
    existsSync("crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs"),
  );
  assert.match(dxWorkspace, /^mod tools_screen;$/m);
  assert.match(dxWorkspace, /pub\(crate\) use tools_screen::render_tools_screen;/);
  assert.match(toolsScreen, /AgentPanel::new_tools_workspace\(workspace, window, cx\)/);
  assert.match(toolsScreen, /WorkspaceScreenKind::Tools/);
  assert.match(toolsScreen, /fn show_toolbar\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(toolsScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(toolsScreen, /"Plugins"\.into\(\)/);
  assert.match(toolsScreen, /Plugins Screen Opened/);

  assert.match(screen, /^mod catalog;$/m);
  assert.match(screen, /use catalog::\{PluginCatalogEntry, first_party_plugin_catalog\};/);
  assert.match(screen, /plugin_catalog_summary\(snapshot\)/);
  assert.match(screen, /plugin_catalog_cards\(snapshot, cx\)/);
  assert.match(screen, /plugin_catalog_card\(entry, snapshot, cx\)/);
  assert.match(screen, /"Plugins"/);
  assert.match(screen, /"dx-plugins-bridge-section"/);
  assert.match(screen, /"Bridge Status"/);
  assert.match(screen, /"dx-plugins-mcp-section"/);
  assert.match(screen, /"dx-plugins-receipts-section"/);
  assert.match(screen, /ListHeader::new\(title\)/);
  assert.match(screen, /ListItem::new\("dx-plugins-catalog-summary"\)/);
  assert.match(screen, /\.id\(SharedString::from\(format!\("dx-plugin-card-\{\}"/);
  assert.match(screen, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(screen, /\.tooltip\(Tooltip::text\(tooltip\)\)/);
  assert.doesNotMatch(screen, /use super::\{[^}]*metric_row|section_title|ListItemSpacing::ExtraDense|IconSize::XSmall|LabelSize::XSmall/);
  assert.match(screen, /trusted_tool_bridge/);
  assert.match(screen, /trusted_tool_ids/);
  assert.match(screen, /approved_plugin_tool_count/);
  assert.match(screen, /approved_automation_tool_count/);
  assert.match(screen, /blocked_tool_count/);
  assert.match(screen, /bridge_contract_id/);
  assert.match(screen, /agents::dx_agent_receipt_state\(snapshot, cx\)/);
  assert.doesNotMatch(screen, /No approved Browser tool receipt is available yet\./);
  assert.doesNotMatch(screen, /No approved Computer tool receipt is available yet\./);
  assert.match(screen, /MCP tool receipts are pending trusted bridge approval\./);
  assert.match(screen, /AiSettingItem::new/);
  assert.match(screen, /ListItem::new/);

  assert.match(catalog, /struct PluginCatalogEntry/);
  for (const id of ["dx.browser", "dx.computer", "dx.driven"]) {
    assert.match(catalog, new RegExp(`id: "${id}"`));
    assert.match(screen, new RegExp(`"${id}"`));
  }
  for (const field of [
    "permissions",
    "inputs",
    "outputs",
    "credentials",
    "receipts",
    "source_root",
    "runtime",
  ]) {
    assert.match(catalog, new RegExp(`pub ${field}:`));
  }
  assert.match(catalog, /source_root: "crates\/web_preview\/src"/);
  assert.match(catalog, /source_root: "G:\\\\Dx\\\\js"/);
  assert.match(catalog, /source_root: "crates\/agent_ui\/src\/dx_agent_bridge"/);
  assert.doesNotMatch(catalog, /n8n|Activepieces|OpenClaw|ZeroClaw/i);

  for (const field of [
    "present",
    "trust_policy",
    "approved_plugin_tool_count",
    "approved_automation_tool_count",
    "blocked_tool_count",
    "receipt_count",
    "bridge_contract_id",
    "trusted_tool_ids",
  ]) {
    assert.match(bridge, new RegExp(`pub ${field}:`));
  }
});
