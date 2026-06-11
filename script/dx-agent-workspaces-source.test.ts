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
  assert.match(
    agentPanel,
    /render_tools_screen\(status\.as_ref\(\), &mut self\.tools_catalog_state, window, cx\)/,
  );

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
  const sidebarTopActions = sidebar.slice(
    sidebar.indexOf('"sidebar-toolbar-connections"'),
    sidebar.indexOf("fn dispatch_workspace_action"),
  );
  assert.doesNotMatch(
    sidebarTopActions,
    /WorkspaceScreenKind::(?:Connections|Tools|Automations) => \{[\s\S]*?dispatch_workspace_action/,
    "sidebar top actions should use the workspace screen router instead of split first-class action paths",
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

test("Tools workspace exposes trusted bridge contracts without fake approvals", () => {
  const dxWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const screen = read("crates/agent_ui/src/dx_launch_workspace/tools_screen.rs");
  const catalogScreen = read("crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs");
  const detailScreen = read("crates/agent_ui/src/dx_launch_workspace/tools_screen/details.rs");
  const workflowNodeScreen = read(
    "crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs",
  );
  const workflowNodeIcons = read("crates/agent_ui/src/workflow_node_icons.rs");
  const pluginScreenSources = `${screen}\n${catalogScreen}\n${detailScreen}\n${workflowNodeScreen}`;
  const forbiddenPluginSource =
    /\b(?:n8n|OpenClaw|ZeroClaw|claude-plugins-official|external_plugins|inspirations[\\/]|G:\\\\Dx\\\\inspirations)\b/i;
  const rawSecretUiPattern =
    /\b(?:api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret|password|private[_-]?key|authorization|bearer)\b/i;
  const toolsScreen = read("crates/agent_ui/src/tools_screen.rs");
  const agentUi = read("crates/agent_ui/src/agent_ui.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const bridge = read("crates/agent_ui/src/dx_agent_bridge.rs");

  assert.ok(existsSync("crates/agent_ui/src/dx_launch_workspace/tools_screen.rs"));
  assert.match(dxWorkspace, /^mod tools_screen;$/m);
  assert.match(dxWorkspace, /pub\(crate\) use tools_screen::\{/);
  assert.match(screen, /^mod details;$/m);
  assert.match(dxWorkspace, /DxPluginsCatalogState/);
  assert.match(dxWorkspace, /render_workflow_node_catalog_rows/);
  assert.match(toolsScreen, /AgentPanel::new_tools_workspace\(workspace, window, cx\)/);
  assert.match(toolsScreen, /WorkspaceScreenKind::Tools/);
  assert.match(toolsScreen, /fn show_toolbar\(&self\) -> bool \{\s*false\s*\}/);
  assert.match(toolsScreen, /fn can_split\(&self\) -> bool \{\s*false\s*\}/);

  assert.match(toolsScreen, /"Plugins"\.into\(\)/);
  assert.match(toolsScreen, /"Plugins Screen Opened"/);
  assert.match(agentPanel, /tools_catalog_state: DxPluginsCatalogState/);
  assert.match(agentPanel, /DxPluginsCatalogState::new\(window, cx\)/);
  assert.match(
    agentPanel,
    /render_tools_screen\(status\.as_ref\(\), &mut self\.tools_catalog_state, window, cx\)/,
  );

  for (const title of ["Workflow Nodes", "Browser", "Computer", "MCP", "Receipts", "Permissions"]) {
    assert.match(screen, new RegExp(`section_title\\("${title}"`));
  }
  assert.match(screen, /catalog::render_workflow_node_catalog\(/);
  assert.match(catalogScreen, /pub\(crate\) struct DxPluginsCatalogState/);
  assert.match(catalogScreen, /UniformListScrollHandle/);
  assert.match(catalogScreen, /query_editor: Entity<Editor>/);
  assert.match(catalogScreen, /selected_node_id: Option<String>/);
  assert.match(catalogScreen, /enum PluginCatalogFilter/);
  assert.match(catalogScreen, /category_filter: Option<String>/);
  assert.match(catalogScreen, /set_selected_node/);
  assert.match(catalogScreen, /selected_node\(/);
  assert.match(catalogScreen, /filtered_node_indices/);
  assert.match(catalogScreen, /EditorElement::new/);
  assert.match(catalogScreen, /BufferSearchBar/);
  assert.match(catalogScreen, /ToggleButtonGroup::single_row/);
  assert.match(catalogScreen, /ToggleButtonGroupStyle::Outlined/);
  assert.match(catalogScreen, /filter-all-categories/);
  assert.match(catalogScreen, /uniform_list\(\s*"dx-workflow-node-plugins"/);
  assert.match(catalogScreen, /track_scroll\(&state\.list\)/);
  assert.match(catalogScreen, /vertical_scrollbar_for\(&state\.list, window, cx\)/);
  assert.match(catalogScreen, /MAX_FILTERED_WORKFLOW_NODE_RESULTS/);
  assert.match(catalogScreen, /render_selected_workflow_node_detail/);
  assert.match(detailScreen, /render_workflow_node_configuration/);
  assert.match(detailScreen, /render_workflow_node_contract/);
  assert.match(detailScreen, /render_workflow_node_permissions/);
  assert.match(detailScreen, /render_workflow_node_ports/);
  assert.match(detailScreen, /render_workflow_node_dynamic_options/);
  assert.match(detailScreen, /render_workflow_node_receipts/);
  assert.match(detailScreen, /render_workflow_node_actions/);
  assert.match(detailScreen, /render_workflow_node_trust/);
  assert.match(detailScreen, /Credential Setup/);
  assert.match(detailScreen, /Permissions/);
  assert.match(detailScreen, /Inputs/);
  assert.match(detailScreen, /Outputs/);
  assert.match(detailScreen, /Dynamic Options/);
  assert.match(detailScreen, /Receipts/);
  assert.match(detailScreen, /Actions/);
  assert.match(detailScreen, /Trust/);
  assert.match(detailScreen, /DX Agents credential bridge/);
  assert.match(detailScreen, /node\.permissions/);
  assert.match(detailScreen, /node\.inputs/);
  assert.match(detailScreen, /node\.outputs/);
  assert.match(detailScreen, /node\.dynamic_options/);
  assert.match(detailScreen, /node\.receipts/);
  assert.match(detailScreen, /node\.actions/);
  assert.match(detailScreen, /node\.trust/);
  assert.match(workflowNodeScreen, /selected: bool/);
  assert.match(workflowNodeScreen, /on_select: impl Fn/);
  assert.match(workflowNodeScreen, /hover\(\|this\| this\.bg/);
  assert.match(pluginScreenSources, /render_plugin_config_menu/);
  assert.match(detailScreen, /catalog\.serializer_format\.clone\(\)/);
  assert.match(detailScreen, /catalog\.schema_version\.clone\(\)/);
  assert.doesNotMatch(pluginScreenSources, /"dx\.serializer\.machine"/);
  assert.match(workflowNodeScreen, /WeakEntity<AgentPanel>/);
  assert.match(workflowNodeScreen, /Configure credentials/);
  assert.match(workflowNodeScreen, /Review configuration/);
  assert.match(workflowNodeScreen, /Review plugin contract/);
  assert.match(workflowNodeScreen, /draft_dx_workflow_node_configuration_prompt/);
  assert.match(agentPanel, /pub\(crate\) fn draft_dx_workflow_node_configuration_prompt/);
  assert.match(agentPanel, /fn workflow_node_configuration_prompt/);
  assert.match(agentPanel, /insert_dx_launch_prompt/);
  assert.match(agentPanel, /credential bridge and receipt contracts only/);
  assert.match(agentUi, /^mod workflow_node_icons;$/m);
  assert.match(pluginScreenSources, /workflow_node_icon_asset_for/);
  assert.match(workflowNodeIcons, /pub\(crate\) enum WorkflowNodeIconAsset/);
  assert.match(workflowNodeIcons, /pub\(crate\) fn workflow_node_icon_asset_for/);
  assert.match(workflowNodeIcons, /Icon::from_external_svg_with_original_colors/);
  assert.match(workflowNodeIcons, /dx_icon_data_dir/);
  assert.match(workflowNodeIcons, /svgl\.json/);
  assert.match(workflowNodeIcons, /WORKFLOW_NODE_ICON_PREVIEW_CACHE/);
  assert.match(workflowNodeIcons, /WORKFLOW_NODE_SVGL_ICON_PACK/);
  assert.match(workflowNodeIcons, /write_workflow_node_icon_preview_for_candidate/);
  assert.match(workflowNodeIcons, /svgl_candidate_aliases/);
  assert.match(workflowNodeIcons, /"github_dark"/);
  assert.match(workflowNodeIcons, /"drive"/);
  assert.match(workflowNodeIcons, /dx_icon\(DxUiIcon::Plugins\)/);
  assert.doesNotMatch(pluginScreenSources, forbiddenPluginSource);
  assert.doesNotMatch(pluginScreenSources, rawSecretUiPattern);
  assert.doesNotMatch(pluginScreenSources, /\b(?:node|plugin)\.run_command\b/);
  assert.doesNotMatch(workflowNodeScreen, /run_dx_agent_public_command|run_command/);
  assert.doesNotMatch(pluginScreenSources, /\bBadge|Pill|TagList|badge_/i);
  assert.doesNotMatch(pluginScreenSources, /<iframe|iframe|WebView|webview|embed_url|external_workflow_url/i);
  assert.match(screen, /trusted_tool_bridge/);
  assert.match(screen, /trusted_tool_ids/);
  assert.match(pluginScreenSources, /workflow_node_catalog/);
  assert.match(pluginScreenSources, /configured_plugins/);
  assert.match(pluginScreenSources, /approved_plugin_tool_count/);
  assert.match(pluginScreenSources, /approved_automation_tool_count/);
  assert.match(screen, /blocked_tool_count/);
  assert.match(screen, /bridge_contract_id/);
  assert.match(screen, /agents::dx_agent_receipt_state\(snapshot, cx\)/);
  assert.match(screen, /No approved Browser tool receipt is available yet\./);
  assert.match(screen, /No approved Computer tool receipt is available yet\./);
  assert.match(screen, /MCP tool receipts are pending trusted bridge approval\./);
  assert.match(pluginScreenSources, /ListItem::new/);
  for (const [name, source] of [
    ["tools_screen.rs", screen],
    ["tools_screen/catalog.rs", catalogScreen],
    ["tools_screen/details.rs", detailScreen],
    ["tools_screen/workflow_nodes.rs", workflowNodeScreen],
  ] as const) {
    assert.doesNotMatch(
      source,
      /\b(?:read_json|read_first_json|latest_receipts)\b|std::fs|serde_json::from_(?:slice|str)|read_to_end/,
      `${name} must render from bridge snapshots instead of direct receipt IO`,
    );
  }

  for (const field of [
    "workflow_node_catalog",
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

  assert.ok(
    existsSync("crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs"),
    "expected focused workflow-node bridge parser module",
  );
  const workflowNodes = read("crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs");
  const workflowNodeConfigured = read(
    "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs",
  );
  const workflowNodeBridgeSources = `${workflowNodes}\n${workflowNodeConfigured}`;
  assert.match(workflowNodes, /DxWorkflowNodeCatalogSummary/);
  assert.match(workflowNodes, /DxWorkflowNodeSummary/);
  assert.match(workflowNodeBridgeSources, /DxConfiguredPluginSummary/);
  assert.match(workflowNodes, /MAX_WORKFLOW_NODE_ROWS/);
  assert.match(workflowNodeBridgeSources, /MAX_CONFIGURED_PLUGIN_ROWS/);
  assert.match(workflowNodes, /missing_serializer_format/);
  assert.doesNotMatch(workflowNodes, /"dx\.serializer\.machine"/);
  assert.match(workflowNodes, /credential_status/);
  assert.match(workflowNodes, /credential_types/);
  assert.match(workflowNodes, /redact_action_scalar/);
  assert.doesNotMatch(workflowNodes, /<iframe|iframe|WebView|webview|embed_url|external_workflow_url/i);
});

test("Plugins workspace follows the Extensions-style GPUI catalog pattern", () => {
  const catalog = read("crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs");
  const workflowNodes = read("crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs");

  assert.match(catalog, /Headline::new\("Plugins"\)\.size\(HeadlineSize::Large\)/);
  assert.match(catalog, /ToggleButtonGroup::single_row\(\s*"dx-plugin-filter-buttons"/);
  assert.match(catalog, /uniform_list\(\s*"dx-workflow-node-plugins"/);
  assert.match(catalog, /border_b_1\(\)/);
  assert.match(catalog, /overflow_x_scroll\(\)/);
  assert.match(catalog, /KeyContext/);
  assert.match(catalog, /TextStyle/);
  assert.match(catalog, /WithScrollbar/);
  assert.doesNotMatch(catalog, /\.take\(24\)/);

  assert.match(workflowNodes, /\.h\(rems_from_px\(110\.\)\)/);
  assert.match(workflowNodes, /Chip::new\(node\.runtime\.clone\(\)\)/);
  assert.match(workflowNodes, /fn plugin_source_row\(node: &DxWorkflowNodeSummary\)/);
  assert.match(workflowNodes, /dx_icon\(DxUiIcon::Source\)/);
  assert.match(workflowNodes, /node\.source_package/);
  assert.match(workflowNodes, /node\.source_path/);
  assert.match(workflowNodes, /fn plugin_status_chips\(node: &DxWorkflowNodeSummary\)/);
  assert.match(workflowNodes, /Chip::new\(plugin_configured_state_label\(node\)\)/);
  assert.match(workflowNodes, /fn plugin_configured_state_label\(node: &DxWorkflowNodeSummary\) -> &'static str/);
  assert.match(workflowNodes, /node\.credential_status/);
  assert.match(workflowNodes, /node\.dynamic_option_count/);
  assert.match(workflowNodes, /bounded_plugin_card_text/);
  assert.match(workflowNodes, /"Configured"/);
  assert.match(workflowNodes, /"Ready"/);
  assert.match(workflowNodes, /"Needs Setup"/);
  assert.match(workflowNodes, /render_plugin_config_menu\(node, panel\)/);
  assert.doesNotMatch(workflowNodes, /Badge|badge/);
});
