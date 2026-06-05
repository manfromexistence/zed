import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const agentUi = readFileSync("crates/agent_ui/src/agent_ui.rs", "utf8");
const moduleRoot = readFileSync("crates/agent_ui/src/dx_forge_panel.rs", "utf8");
const panel = readFileSync("crates/agent_ui/src/dx_forge_panel/panel.rs", "utf8");
const snapshot = readFileSync("crates/agent_ui/src/dx_forge_panel/snapshot.rs", "utf8");
const panelView = readFileSync("crates/agent_ui/src/dx_forge_panel/panel_view.rs", "utf8");
const rows = readFileSync("crates/agent_ui/src/dx_forge_panel/rows.rs", "utf8");
const forgeSources = [moduleRoot, panel, snapshot, panelView, rows].join("\n");

test("Forge panel is wired through agent UI without touching Git panel ownership", () => {
  assert.match(agentUi, /\nmod dx_forge_panel;\n/);
  assert.match(agentUi, /dx_forge_panel::panel::init\(cx\);/);
  assert.ok(
    agentUi.indexOf("agent_panel::init(cx);") <
      agentUi.indexOf("dx_forge_panel::panel::init(cx);"),
    "Forge panel should be registered beside the existing agent panels",
  );
  assert.doesNotMatch(forgeSources, /git_panel|GitPanel|git_ui::/);
});

test("Forge panel owns a stable local action and dock identity", () => {
  assert.match(moduleRoot, /actions!\(\s*dx_forge,/);
  assert.match(moduleRoot, /TogglePanel/);
  assert.match(panel, /workspace\.register_action\(\|workspace, _:\s*&TogglePanel/);
  assert.match(panel, /const DX_FORGE_PANEL_KEY: &str = "dx_forge_panel";/);
  assert.match(panel, /fn persistent_name\(\) -> &'static str \{\s*"Forge"/);
  assert.match(panel, /DockPosition::Left/);
  assert.match(panel, /Some\(IconName::Archive\)/);
  assert.match(panel, /fn activation_priority\(&self\) -> u32 \{\s*4\s*\}/);
  assert.match(panel, /fn starts_open\(&self, _:\s*&Window, _:\s*&App\) -> bool \{\s*false/);
  assert.doesNotMatch(panel, /zed_actions::dx_forge/);
});

test("Forge snapshot reuses existing bounded DX readers", () => {
  assert.match(snapshot, /tool_history_snapshot\(workspace_roots\)/);
  assert.match(snapshot, /source_set_snapshot\(workspace_roots\)/);
  assert.match(snapshot, /"Forge History"/);
  assert.match(snapshot, /"Restore Previews"/);
  assert.match(snapshot, /"Media Outputs"/);
  assert.doesNotMatch(snapshot, /std::process|Command::new|powershell|cmd\.exe/);
});

test("Forge panel renders real receipt, restore, and media states", () => {
  assert.match(panelView, /fn receipt_section/);
  assert.match(panelView, /fn restore_section/);
  assert.match(panelView, /fn media_section/);
  assert.match(panelView, /No Forge receipts found/);
  assert.match(panelView, /No restore previews found/);
  assert.match(panelView, /No media outputs found/);
  assert.match(panelView, /side_panel_header_controls/);
});

test("Forge panel files stay small and professionally named", () => {
  const lineCounts = new Map([
    ["dx_forge_panel.rs", moduleRoot],
    ["panel.rs", panel],
    ["snapshot.rs", snapshot],
    ["panel_view.rs", panelView],
    ["rows.rs", rows],
  ]);

  for (const [name, source] of lineCounts) {
    assert.ok(
      source.split("\n").length <= 360,
      `${name} should stay small enough to review quickly`,
    );
  }

  for (const term of [
    "prototype",
    "skeleton",
    "mock",
    "fake",
    "sample",
    "placeholder",
  ]) {
    assert.doesNotMatch(forgeSources.toLowerCase(), new RegExp(`\\b${term}\\b`));
  }
});
