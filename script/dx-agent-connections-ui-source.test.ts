import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;
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
  const social = read("crates/agent_ui/src/dx_launch_workspace/agents/social.rs");
  const providers = read("crates/agent_ui/src/dx_launch_workspace/agents/providers.rs");
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
  assert.match(social, /screen_detail_row\(/);
  assert.match(providers, /screen_detail_row\(/);
  assert.doesNotMatch(`${social}\n${providers}`, /\bmetric_row\(/);
  assert.doesNotMatch(`${social}\n${providers}`, /\bmuted_card\(/);
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
  assert.match(connectionsScreen, /workspace_page_header\(\s*dx_icon\(DxUiIcon::Connections\)/);
  assert.match(connectionsScreen, /screen_section\(\s*"dx-connections-channels"/);
  assert.match(connectionsScreen, /screen_section\(\s*"dx-connections-gateway"/);
  assert.match(connectionsScreen, /No DX Agents channel receipt\/schema is available yet\./);
  assert.match(connectionsScreen, /No first-class provider gateway health receipt is available yet\./);
});

test("DX Connections workspace follows the Extensions-style GPUI page pattern", () => {
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const chrome = read("crates/agent_ui/src/dx_launch_workspace/screen_chrome.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const connectionsScreen = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen.rs",
  );
  const catalogChrome = read("crates/agent_ui/src/dx_launch_workspace/catalog_chrome.rs");
  const connectionsCatalog = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog.rs",
  );
  const connectionsCatalogEntry = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/entry.rs",
  );
  const connectionsCatalogDetails = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/details.rs",
  );
  const connectionsCatalogStatus = read(
    "crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/status.rs",
  );
  const connectionsCatalogSources = `${connectionsCatalog}\n${connectionsCatalogEntry}\n${connectionsCatalogDetails}\n${connectionsCatalogStatus}`;

  assert.match(launchWorkspace, /^mod screen_chrome;$/m);
  assert.match(launchWorkspace, /DxConnectionsCatalogState/);
  assert.match(chrome, /Headline::new\(title\)\.size\(HeadlineSize::Large\)/);
  assert.match(chrome, /ListHeader::new\(title\)/);
  assert.match(chrome, /elevated_surface_background\.opacity\(0\.5\)/);
  assert.match(chrome, /border_color\(cx\.theme\(\)\.colors\(\)\.border_variant\)/);
  assert.match(catalogChrome, /pub\(super\) fn render_catalog_row_labels/);
  assert.match(catalogChrome, /pub\(super\) fn render_catalog_status_chip/);
  assert.match(catalogChrome, /Chip::new\(label\)/);

  assert.match(agentPanel, /connections_catalog_state: DxConnectionsCatalogState/);
  assert.match(agentPanel, /_connections_catalog_query_subscription: Subscription/);
  assert.match(agentPanel, /render_connections_catalog_rows/);
  assert.match(agentPanel, /set_connections_catalog_filter/);
  assert.match(agentPanel, /set_connections_catalog_selected_entry/);

  assert.match(connectionsScreen, /^mod catalog;$/m);
  assert.match(connectionsScreen, /use super::screen_chrome::\{/);
  assert.match(connectionsScreen, /workspace_page_header\(/);
  assert.match(connectionsScreen, /render_connections_catalog\(/);
  assert.match(connectionsScreen, /workspace_stat\(/);
  assert.doesNotMatch(connectionsScreen, /section_title\(/);
  assert.doesNotMatch(connectionsScreen, /ListItemSpacing::ExtraDense/);

  assert.ok(
    existsSync("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/entry.rs"),
  );
  assert.ok(
    existsSync("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/details.rs"),
  );
  assert.ok(
    existsSync("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/status.rs"),
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog.rs") < 430,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/entry.rs") <
      340,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/details.rs") <
      460,
  );
  assert.ok(
    lineCount("crates/agent_ui/src/dx_launch_workspace/connections_screen/catalog/status.rs") <
      180,
  );
  assert.match(connectionsCatalogSources, /Editor::single_line/);
  assert.match(connectionsCatalogSources, /ToggleButtonGroup::single_row/);
  assert.match(connectionsCatalogSources, /UniformListScrollHandle/);
  assert.match(connectionsCatalogSources, /uniform_list\(/);
  assert.match(connectionsCatalog, /"dx-connections-catalog-list"/);
  assert.match(connectionsCatalog, /"Connection Catalog"/);
  assert.match(connectionsCatalog, /render_catalog_row_labels\(/);
  assert.match(connectionsCatalog, /\.start_slot\(\s*Icon::new\(entry\.icon\(\)\)/);
  assert.match(connectionsCatalog, /\.end_slot\(render_catalog_status_chip\(entry\.status\(snapshot\)\)\)/);
  assert.match(connectionsCatalog, /\.tooltip\(Tooltip::text\(tooltip\)\)/);
  assert.match(connectionsCatalogSources, /render_selected_connection_detail/);
  assert.match(connectionsCatalogSources, /Trusted tool bridge/);
  assert.match(connectionsCatalogSources, /Receipt authority/);
  assert.match(connectionsCatalogSources, /screen_detail_row\(/);
  assert.doesNotMatch(connectionsCatalog, /AiSettingItem::new/);
  assert.doesNotMatch(connectionsCatalog, /AiSettingItemSource/);
  assert.doesNotMatch(connectionsCatalogSources, /\b(?:Button::new|IconButton::new|run_dx_agent_public_command|run_dx_agents_public_action|DxAgentPublicCommand::Run)\b/);
});
