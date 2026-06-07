import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const functionBody = (source: string, name: string) => {
  const start = source.search(new RegExp(`fn\\s+${name}\\s*\\(`));
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = source.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
};

test("DX semantic icon layer owns rebrand-specific aliases", () => {
  const icons = read("crates/icons/src/icons.rs");
  const uiRoot = read("crates/ui/src/ui.rs");
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const loaderAsset = read("assets/icons/dx_loader.svg");
  const cogAsset = read("assets/icons/dx_cog.svg");

  assert.match(icons, /\bDxLoader,/);
  assert.match(icons, /\bDxCog,/);
  assert.match(loaderAsset, /viewBox="0 0 16 16"/);
  assert.match(loaderAsset, /M8 1\.75v2\.5/);
  assert.match(cogAsset, /viewBox="0 0 16 16"/);
  assert.match(cogAsset, /M8\.13 2h-\.26/);

  assert.match(uiRoot, /mod dx_icons;/);
  assert.match(uiRoot, /pub use dx_icons::\*/);
  assert.match(dxIcons, /pub enum DxUiIcon/);
  assert.match(dxIcons, /pub fn dx_icon\(icon: DxUiIcon\) -> IconName/);
  assert.match(dxIcons, /DxUiIcon::Loading => IconName::DxLoader/);
  assert.match(dxIcons, /DxUiIcon::Settings => IconName::DxCog/);
  assert.doesNotMatch(dxIcons, /LoadCircle|Settings => IconName::Settings/);
});

test("DX shell chrome uses semantic icons instead of scattered literals", () => {
  const titleBar = read("crates/title_bar/src/title_bar.rs");
  const forgePanel = read("crates/agent_ui/src/dx_forge_panel/panel.rs");
  const stylePanel = read("crates/agent_ui/src/dx_style_panel/panel.rs");
  const agentButton = functionBody(titleBar, "render_agent_screen_button");
  const screenKindIcon = functionBody(titleBar, "screen_kind_icon");
  const hiddenButtons = functionBody(titleBar, "render_hidden_feature_buttons");

  for (const icon of ["Ai", "Browser", "Icons", "Fonts", "Media", "Ui", "Check"]) {
    assert.ok(
      titleBar.includes(`dx_icon(DxUiIcon::${icon})`),
      `title bar should use semantic DX icon ${icon}`,
    );
  }

  assert.match(forgePanel, /dx_icon\(DxUiIcon::Forge\)/);
  assert.match(stylePanel, /dx_icon\(DxUiIcon::Style\)/);
  assert.doesNotMatch(agentButton, /IconName::ZedAssistant/);
  assert.doesNotMatch(screenKindIcon, /IconName::ToolWeb/);
  assert.doesNotMatch(
    hiddenButtons,
    /IconName::(?:SquareDot|Font|Image|Blocks|Check)/,
  );
});

test("legacy loader/settings names do not leak into the DX icon contract", () => {
  const guardedSources = [
    "crates/ui/src/dx_icons.rs",
    "crates/title_bar/src/title_bar.rs",
    "crates/agent_ui/src/dx_forge_panel/panel.rs",
    "crates/agent_ui/src/dx_style_panel/panel.rs",
  ].map(read).join("\n");

  assert.doesNotMatch(guardedSources, /loader-circle|IconName::LoadCircle/);
  assert.doesNotMatch(guardedSources, /IconName::Settings/);
});
