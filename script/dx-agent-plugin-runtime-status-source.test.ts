import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const runtimeStatusPath = "crates/agent/src/tools/agent_plugin_runtime_status_tool.rs";
const runtimeAliasesPath = "crates/agent/src/tools/dx_plugin_runtime_aliases.rs";
const catalogPath = "crates/agent/src/tools/agent_plugin_catalog_tool.rs";
const manifestPath = "crates/agent/src/tools/dx_plugin_manifest.rs";
const toolsPath = "crates/agent/src/tools.rs";

const forbiddenUpstream =
  /\b(?:n8n|OpenClaw|ZeroClaw|claude-plugins-official|external[_-]plugins|CatalogSourceKind::ZeroclawProviders|zeroclaw|inspirations[\\/]|G:\\\\Dx\\\\inspirations)\b/i;

test("runtime status exposes DX plugin aliases without inventing readiness", () => {
  const status = read(runtimeStatusPath);
  const aliases = read(runtimeAliasesPath);
  const tools = read(toolsPath);

  assert.match(tools, /^mod dx_plugin_runtime_aliases;$/m);
  assert.match(status, /dx_plugin_runtime_aliases::dx_plugin_runtime_aliases/);
  assert.match(status, /"dx_plugin_runtime_aliases": dx_plugin_runtime_aliases/);
  assert.match(aliases, /DX_PLUGIN_RUNTIME_STATUS_ALIASES_SCHEMA/);
  assert.match(aliases, /DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA/);
  assert.match(aliases, /pub\(super\) fn dx_plugin_runtime_aliases\(/);
  assert.match(aliases, /"claim_policy": "aliases_are_display_and_lookup_metadata_not_authorization"/);
  assert.match(aliases, /runtime_scorecard_lane_ready\(runtime_green_readiness_scorecard, "browser_webpreview"\)/);
  assert.match(aliases, /runtime_scorecard_lane_ready\(runtime_green_readiness_scorecard, "managed_chrome"\)/);
  assert.match(aliases, /runtime_scorecard_lane_ready\(runtime_green_readiness_scorecard, "pc_use"\)/);
  assert.doesNotMatch(aliases, /"runtime_green_claim_ready": true/);
});

test("DX Browser, Computer, and Driven aliases map to the existing guarded lanes", () => {
  const status = read(runtimeAliasesPath);

  assert.match(status, /"dx\.browser"[\s\S]*?"maps_to_runtime_plugin_ids": \["zed\.browser"\]/);
  assert.match(status, /"dx\.browser"[\s\S]*?"maps_to_lane_ids": \["browser_webpreview"\]/);
  assert.match(status, /"dx\.computer"[\s\S]*?"maps_to_runtime_plugin_ids": \["zed\.chrome", "zed\.pc_use"\]/);
  assert.match(status, /"dx\.computer"[\s\S]*?"maps_to_lane_ids": \["managed_chrome", "pc_use"\]/);
  assert.match(status, /"dx\.computer"[\s\S]*?"os_wide_control": false/);
  assert.match(status, /"dx\.computer"[\s\S]*?"pc_use_future_executor_only": true/);
  assert.match(status, /"dx\.driven"[\s\S]*?"runtime_status_kind": "guarded_workflow_surface"/);
  assert.match(status, /"dx\.driven"[\s\S]*?"does_not_claim_executor_runtime": true/);
  assert.match(status, /"dx\.driven"[\s\S]*?"ready": false/);
});

test("catalog advertises the runtime alias schema and field", () => {
  const catalog = read(catalogPath);
  const manifest = read(manifestPath);

  assert.match(manifest, /zed\.dx_plugins\.runtime_status_alias\.v1/);
  assert.match(manifest, /zed\.dx_plugins\.runtime_status_aliases\.v1/);
  assert.match(catalog, /"dx_plugin_runtime_status_alias_schema": DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA/);
  assert.match(catalog, /"dx_plugin_runtime_status_aliases_schema": DX_PLUGIN_RUNTIME_STATUS_ALIASES_SCHEMA/);
  assert.match(catalog, /"dx_plugin_runtime_status_aliases_field": "dx_plugin_runtime_aliases"/);
  assert.match(catalog, /"dx_default_enabled_plugins": dx_default_enabled_plugins/);
});

test("runtime alias source stays DX-native", () => {
  for (const path of [runtimeStatusPath, runtimeAliasesPath, catalogPath, manifestPath]) {
    assert.doesNotMatch(read(path), forbiddenUpstream, `${path} must not reference forbidden upstream plugin sources`);
  }
});
