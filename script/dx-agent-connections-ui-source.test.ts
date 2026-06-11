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

test("DX connection UI uses semantic icons and real Zed sidebar routes", () => {
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const sidebar = read("crates/sidebar/src/sidebar.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");

  for (const icon of [
    "Connections",
    "Channels",
    "Gateway",
    "Credentials",
    "Computer",
    "Mcp",
    "Permissions",
  ]) {
    assert.match(dxIcons, new RegExp(`DxUiIcon::${icon}`));
  }

  assert.match(dxIcons, /DxUiIcon::Connections => IconName::UserGroup/);
  assert.match(dxIcons, /DxUiIcon::Channels => IconName::QueueMessage/);
  assert.match(dxIcons, /DxUiIcon::Gateway => IconName::Server/);
  assert.match(dxIcons, /DxUiIcon::Credentials => IconName::LockOutlined/);
  assert.match(dxIcons, /DxUiIcon::Permissions => IconName::UserCheck/);

  assert.match(sidebar, /"sidebar-toolbar-connections"[\s\S]*?dx_icon\(DxUiIcon::Connections\)[\s\S]*?"Connections"[\s\S]*?activate_workspace_screen\(\s*WorkspaceScreenKind::Connections/);
  assert.match(sidebar, /"sidebar-activity-connections"[\s\S]*?dx_icon\(DxUiIcon::Connections\)[\s\S]*?"Connections"[\s\S]*?activate_workspace_screen\(WorkspaceScreenKind::Connections/);
  assert.match(sidebar, sidebarWorkspaceActionArm("Connections", "OpenConnections"));
  assert.match(agentPanel, /OpenConnections/);
  assert.match(agentPanel, /ConnectionsScreen::open_or_focus\(workspace, window, cx\)/);
});

test("DX connection entities render with Zed AI/list components, not ad hoc badges", () => {
  const agents = read("crates/agent_ui/src/dx_launch_workspace/agents.rs");
  const connectionRows = read(
    "crates/agent_ui/src/dx_launch_workspace/agents/connection_rows.rs",
  );
  const socialRows = read("crates/agent_ui/src/dx_launch_workspace/agents/social/rows.rs");
  const providerRows = read(
    "crates/agent_ui/src/dx_launch_workspace/agents/providers/rows.rs",
  );
  const automationRows = read(
    "crates/agent_ui/src/dx_launch_workspace/agents/automations/rows.rs",
  );
  const socialActions = read(
    "crates/agent_ui/src/dx_launch_workspace/agents/social_actions.rs",
  );

  assert.match(agents, /^mod connection_rows;$/m);
  assert.ok(
    existsSync("crates/agent_ui/src/dx_launch_workspace/agents/connection_rows.rs"),
  );
  assert.match(connectionRows, /AiSettingItem/);
  assert.match(connectionRows, /ListItem/);
  assert.match(connectionRows, /AiSettingItemStatus::Stopped/);
  assert.match(connectionRows, /unavailable_capability_row/);

  for (const source of [socialRows, providerRows, automationRows, socialActions]) {
    assert.match(source, /AiSettingItem::new/);
    assert.match(source, /connection_detail_row/);
    assert.doesNotMatch(source, /super::super::super::metric_row/);
  }

  assert.match(providerRows, /ListItem::new/);
  assert.match(socialRows, /social_qr_detail/);
  assert.match(socialRows, /credential_error/);
  assert.match(providerRows, /provider_status/);
  assert.match(automationRows, /automation_status/);
});

test("DX connection UI keeps missing channels and gateways explicit", () => {
  const social = read("crates/agent_ui/src/dx_launch_workspace/agents/social.rs");
  const connectionRows = read(
    "crates/agent_ui/src/dx_launch_workspace/agents/connection_rows.rs",
  );
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const connectionsScreen = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen.rs",
  );

  assert.match(social, /connection_unavailable_rows\(\)/);
  assert.match(connectionRows, /unavailable_capability_row/);
  assert.match(connectionRows, /"dx-agent-channels-unavailable"/);
  assert.match(connectionRows, /dx_icon\(DxUiIcon::Channels\)/);
  assert.match(connectionRows, /No DX Agents channel receipt\/schema is available yet\./);
  assert.match(connectionRows, /"dx-agent-gateway-unavailable"/);
  assert.match(connectionRows, /dx_icon\(DxUiIcon::Gateway\)/);
  assert.match(connectionRows, /No first-class provider gateway health receipt is available yet\./);

  assert.match(launchWorkspace, /section_title\(\s*"Agent Connections",\s*dx_icon\(DxUiIcon::Connections\),\s*\)/);
  assert.match(launchWorkspace, /section_title\("Agent Providers", dx_icon\(DxUiIcon::Gateway\)\)/);
  assert.match(connectionsScreen, /section_title\("Channels", dx_icon\(DxUiIcon::Channels\)\)/);
  assert.match(connectionsScreen, /section_title\("Gateway", dx_icon\(DxUiIcon::Gateway\)\)/);
  assert.match(connectionsScreen, /No DX Agents channel receipt\/schema is available yet\./);
  assert.match(connectionsScreen, /No first-class provider gateway health receipt is available yet\./);
});
