import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const bridgeBlock = (source: string) => {
  const start = source.indexOf('const DX_WWW_TURBO_BRIDGE_SCHEMA = "zed.web_preview.dx_www_turbo_bridge.v1"');
  const end = source.indexOf("  const domTreeNode =", start);
  assert.ok(start >= 0, "expected DX WWW turbo bridge schema");
  assert.ok(end > start, "expected bridge block before DOM snapshot code");
  return source.slice(start, end);
};

test("Web Preview exposes a scoped DX-WWW turbo bridge without starting runtime work", () => {
  const source = read("crates/web_preview/src/web_preview_view.rs");
  const block = bridgeBlock(source);

  assert.match(block, /"zed\.web_preview\.dx_www_turbo_bridge\.v1"/);
  assert.match(block, /"\/_dx\/hot-reload\/version"/);
  assert.match(block, /"\/_dx\/hot-reload\/events"/);
  assert.match(block, /"\/_dx\/devtools\/session"/);
  assert.match(block, /searchParams\.set\("resource", resource\)/);
  assert.doesNotMatch(block, /searchParams\.set\("target"/);
  assert.match(block, /dxWwwLoopbackHost/);
  assert.match(block, /host === "localhost"/);
  assert.match(block, /host === "127\.0\.0\.1"/);
  assert.match(block, /cache: "no-store"/);
  assert.match(block, /DX_WWW_TURBO_BRIDGE_CACHE_MS = 2000/);
  assert.doesNotMatch(block, /setInterval/);
  assert.match(block, /token_present: Boolean\(hotReloadJson\.token\)/);
  assert.doesNotMatch(block, /token:\s*hotReloadJson\.token/);
  assert.match(block, /starts_dev_server: false/);
  assert.match(block, /mutates_source: false/);
  assert.match(block, /no_node_modules_required: true/);
});

test("DX-WWW turbo bridge state is stored in the native session snapshot", () => {
  const source = read("crates/web_preview/src/web_preview_view.rs");

  assert.match(source, /latest_dx_www_turbo_bridge: Option<Value>/);
  assert.match(source, /latest_dx_www_turbo_bridge: None/);
  assert.match(source, /"dx_www_turbo_bridge": self\.latest_dx_www_turbo_bridge_summary\(\)/);
  assert.match(source, /fn latest_dx_www_turbo_bridge_summary\(&self\) -> Option<Value>/);
  assert.match(source, /fn dx_www_turbo_bridge_snapshot\(&self, payload: &Value\) -> Value/);
  assert.match(source, /"dx-www-turbo-bridge" =>/);
  assert.match(source, /self\.latest_dx_www_turbo_bridge = Some\(snapshot\)/);
  assert.match(source, /self\.latest_dx_www_turbo_bridge = None;\s*self\.page_title = None;/);
});

test("DX Studio bridge reports when its API aliases are attached", () => {
  const source = read("crates/web_preview/src/dx_studio_bridge/api.ts");

  assert.match(source, /collectBaseBridgeReadiness/);
  assert.match(source, /collectDxWwwTurboBridge\?\.\(reason\)/);
  assert.match(source, /"dx-studio-api-attached"/);
  assert.match(source, /"dx-studio-dom-ready"/);
  assert.match(source, /"dx-studio-ready"/);
});
