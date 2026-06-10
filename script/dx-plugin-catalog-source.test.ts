import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const manifestPath = "crates/agent/src/tools/dx_plugin_manifest.rs";
const manifestEntriesPath = "crates/agent/src/tools/dx_plugin_manifest/entries.rs";
const catalogPath = "crates/agent/src/tools/agent_plugin_catalog_tool.rs";
const toolsPath = "crates/agent/src/tools.rs";
const pluginMetadataSurfacePaths = [
  manifestPath,
  manifestEntriesPath,
  catalogPath,
  "crates/agent_ui/src/dx_agent_bridge.rs",
  "crates/agent_ui/src/dx_agent_bridge/workflow_nodes.rs",
  "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/contract.rs",
  "crates/agent_ui/src/dx_agent_bridge/workflow_nodes/configured.rs",
  "crates/agent_ui/src/dx_launch_workspace/tools_screen.rs",
  "crates/agent_ui/src/dx_launch_workspace/tools_screen/catalog.rs",
  "crates/agent_ui/src/dx_launch_workspace/tools_screen/details.rs",
  "crates/agent_ui/src/dx_launch_workspace/tools_screen/workflow_nodes.rs",
  "crates/agent_ui/src/conversation_view/thread_view.rs",
];

const forbiddenUpstreamSource = /\b(?:n8n|OpenClaw|ZeroClaw|claude-plugins-official|external_plugins|inspirations[\\/])/i;
const forbiddenPluginSourceHooks =
  /CatalogSourceKind::ZeroclawProviders|zeroclaw_providers_input|ZEROCLAW_HOME|\.zeroclaw|G:\\\\Dx\\\\inspirations/i;
const rawSecretJsonKey =
  /"(?:(?:api|auth)[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret|password|private[_-]?key|secret(?:_value)?)"\s*:/i;

test("DX first-party plugin manifest model is typed and complete", () => {
  assert.ok(existsSync(manifestPath), "expected focused DX plugin manifest module");
  assert.ok(existsSync(manifestEntriesPath), "expected focused DX first-party plugin entries module");

  const manifest = [read(manifestPath), read(manifestEntriesPath)].join("\n");
  const requiredTypes = [
    "DxPluginManifest",
    "DxPluginCatalog",
    "DxPluginCatalogDiscovery",
    "DxPluginPermission",
    "DxPluginRuntime",
    "DxPluginPort",
    "DxPluginCredential",
    "DxPluginTrustStatus",
    "DxPluginReceipt",
  ];
  const requiredFields = [
    "id",
    "name",
    "category",
    "description",
    "permissions",
    "runtime",
    "inputs",
    "outputs",
    "credentials",
    "trust_status",
    "receipts",
  ];

  for (const typeName of requiredTypes) {
    assert.match(manifest, new RegExp(`struct\\s+${typeName}\\b`));
  }

  for (const field of requiredFields) {
    assert.match(manifest, new RegExp(`pub\\(crate\\)\\s+${field}:`));
  }

  assert.match(manifest, /DX_PLUGIN_MANIFEST_SCHEMA/);
  assert.match(manifest, /zed\.dx_plugins\.manifest\.v1/);
  assert.match(manifest, /DX_PLUGIN_CATALOG_DISCOVERY_SCHEMA/);
  assert.match(manifest, /zed\.dx_plugins\.catalog_discovery\.v1/);
});

test("DX catalog exposes Browser, Computer, and Driven as first-party manifests", () => {
  const manifest = [read(manifestPath), read(manifestEntriesPath)].join("\n");
  const catalog = read(catalogPath);
  const tools = read(toolsPath);

  assert.match(tools, /^mod dx_plugin_manifest;$/m);
  assert.match(catalog, /dx_first_party_plugin_catalog/);
  assert.match(catalog, /"dx_plugin_catalog": dx_plugin_catalog/);
  assert.match(catalog, /"dx_plugin_catalog_summary": dx_plugin_catalog_summary/);

  for (const [id, name] of [
    ["dx.browser", "Browser"],
    ["dx.computer", "Computer"],
    ["dx.driven", "Driven"],
  ]) {
    assert.match(manifest, new RegExp(`"${id}"`));
    assert.match(manifest, new RegExp(`"${name}"`));
  }

  assert.match(manifest, /category:\s*"browser_automation"/);
  assert.match(manifest, /category:\s*"computer_control"/);
  assert.match(manifest, /category:\s*"workflow_nodes"/);
  assert.match(manifest, /action_recording/);
  assert.match(manifest, /"agent_screen_recording_thumbnail"/);
  assert.match(manifest, /"dx_lanes"/);
  assert.match(manifest, /"worker_prompts"/);
  assert.match(manifest, /"checkpoint_receipts"/);
});

test("DX plugin discovery is allowlisted to DX-owned source roots only", () => {
  const manifest = read(manifestPath);

  assert.match(manifest, /source_root_policy: "dx_owned_source_roots_only"/);
  assert.match(manifest, /"crates\/agent\/src\/tools"/);
  assert.match(manifest, /"crates\/agent_ui\/src\/dx_agent_bridge"/);
  assert.match(manifest, /"crates\/web_preview\/src"/);
  assert.match(manifest, /"tools\/agent-plugins"/);
  assert.match(manifest, /"G:\\\\Dx\\\\js"/);
  assert.match(manifest, /allowlisted_source_roots/);
  assert.doesNotMatch(manifest, /read_dir|WalkDir|glob|canonicalize/);
});

test("DX plugin catalog source does not reference forbidden upstream plugin sources", () => {
  const scanned = [manifestPath, manifestEntriesPath, catalogPath].map((path) => [path, read(path)] as const);

  for (const [path, source] of scanned) {
    assert.doesNotMatch(source, forbiddenUpstreamSource, `${path} must stay DX-native`);
    assert.doesNotMatch(source, forbiddenPluginSourceHooks, `${path} must not hook plugin sources to provider/inspiration history`);
  }
});

test("DX plugin metadata/detail surfaces avoid upstream sources and raw secret keys", () => {
  for (const path of pluginMetadataSurfacePaths) {
    const source = read(path);
    assert.doesNotMatch(source, forbiddenUpstreamSource, `${path} must stay DX-native`);
    assert.doesNotMatch(source, forbiddenPluginSourceHooks, `${path} must not hook metadata to provider/inspiration history`);
    assert.doesNotMatch(source, rawSecretJsonKey, `${path} must not expose raw secret fields`);
  }
});

test("DX plugin manifests are prepared for the Plugins panel and DX Agents bridge", () => {
  const manifest = [read(manifestPath), read(manifestEntriesPath)].join("\n");

  assert.match(manifest, /"zed_plugins_panel"/);
  assert.match(manifest, /"dx_agents_bridge"/);
  assert.match(manifest, /"agent_panel"/);
  assert.match(manifest, /trust_status/);
  assert.match(manifest, /"first_party_trusted"/);
  assert.match(manifest, /"credential_status"/);
  assert.match(manifest, /receipt_root/);
  assert.match(manifest, /runtime_status_tool/);
  assert.match(manifest, /catalog_tool/);
});
