import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

test("DX Icon is the default project-panel icon theme and vendors the latest Material icon extension", () => {
  const defaultSettings = read("assets/settings/default.json");
  const iconTheme = read("crates/theme/src/icon_theme.rs");

  assert.match(defaultSettings, /"icon_theme": "DX Icon"/);
  assert.match(iconTheme, /pub const DEFAULT_ICON_THEME_NAME: &str = "DX Icon";/);
  assert.match(iconTheme, /id: "dx-icon"\.into\(\)/);
  assert.match(
    iconTheme,
    /const DX_ICON_THEME_SOURCE_EXTENSION: &str = "material-icon-theme@v1\.3\.1";/,
  );

  for (const path of [
    "extensions/dx-icon/extension.toml",
    "extensions/dx-icon/README.md",
    "extensions/dx-icon/LICENSE",
    "extensions/dx-icon/icon_themes/dx-icon.json",
    "extensions/dx-icon/icons/file.svg",
    "extensions/dx-icon/icons/folder.svg",
    "extensions/dx-icon/icons/folder-open.svg",
  ]) {
    assert.ok(existsSync(path), `missing ${path}`);
  }

  const extensionToml = read("extensions/dx-icon/extension.toml");
  const extensionReadme = read("extensions/dx-icon/README.md");
  const packageJson = read("extensions/dx-icon/package.json");
  const packageLock = read("extensions/dx-icon/package-lock.json");
  const buildScript = read("extensions/dx-icon/src/build.ts");
  const themeScript = read("extensions/dx-icon/src/theme.ts");
  const dxIconTheme = read("extensions/dx-icon/icon_themes/dx-icon.json");

  assert.match(extensionToml, /^id = "dx-icon"$/m);
  assert.match(extensionToml, /^name = "DX Icon"$/m);
  assert.match(extensionToml, /^version = "1\.3\.1-dx\.1"$/m);
  assert.match(extensionToml, /icon_themes = \["icon_themes\/dx-icon\.json"\]/);
  assert.match(extensionReadme, /Derived from zed-extensions\/material-icon-theme v1\.3\.1/);
  assert.match(packageJson, /"name": "dx-icon"/);
  assert.match(packageJson, /"version": "1\.3\.1-dx\.1"/);
  assert.match(packageLock, /"name": "dx-icon"/);
  assert.match(packageLock, /"version": "1\.3\.1-dx\.1"/);
  assert.doesNotMatch(buildScript, /material-icon-theme\.json/);
  assert.match(buildScript, /"dx-icon\.json"/);
  assert.match(buildScript, /themes: \[darkTheme, lightTheme\]/);
  assert.match(themeScript, /name: options\.name/);
  assert.match(themeScript, /appearance: options\.appearance/);
  assert.match(dxIconTheme, /"name": "DX Icon"/);
  assert.match(dxIconTheme, /"author": "DX, based on Zed Material Icon Theme"/);
  assert.match(dxIconTheme, /"name": "DX Icon Light"/);
  assert.match(dxIconTheme, /"directory_icons"/);
  assert.match(dxIconTheme, /"file_icons"/);
  assert.match(dxIconTheme, /"collapsed": "\.\/icons\/folder\.svg"/);
  assert.match(dxIconTheme, /"expanded": "\.\/icons\/folder-open\.svg"/);
  assert.match(dxIconTheme, /"typescript"/);
  assert.match(dxIconTheme, /"rust"/);
});
