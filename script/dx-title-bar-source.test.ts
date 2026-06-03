import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const sourcePath = "crates/title_bar/src/application_menu.rs";
const source = readFileSync(sourcePath, "utf8").replace(/\r\n/g, "\n");
const titleBarPath = "crates/title_bar/src/title_bar.rs";
const titleBarSource = readFileSync(titleBarPath, "utf8").replace(/\r\n/g, "\n");
const popoverMenuPath = "crates/ui/src/components/popover_menu.rs";
const popoverMenuSource = readFileSync(popoverMenuPath, "utf8").replace(/\r\n/g, "\n");

test("application menu activation checks stale entry indexes before handle use", () => {
  const activation = functionBody("navigate_menus_in_direction");

  assert.doesNotMatch(
    activation,
    /self\s*\.\s*entries\s*\[\s*current_index\s*\]/,
    "application menu activation must not direct-index the current entry",
  );
  assert.doesNotMatch(
    activation,
    /self\s*\.\s*entries\s*\[\s*next_index\s*\]/,
    "application menu activation must not direct-index the next entry",
  );
  assert.match(
    activation,
    /let\s+Some\(current_entry\)\s*=\s*self\s*\.\s*entries\s*\.\s*get\(\s*current_index\s*\)\s*else\s*\{\s*return;\s*\};/s,
    "application menu activation must check the current entry before hiding it",
  );
  assert.match(
    activation,
    /let\s+Some\(next_entry\)\s*=\s*self\s*\.\s*entries\s*\.\s*get\(\s*next_index\s*\)\s*else\s*\{\s*return;\s*\};/s,
    "application menu activation must check the next entry before cloning its handle",
  );
  assert.match(activation, /current_entry\.handle\.hide\(cx\);/);
  assert.match(
    activation,
    /let\s+next_handle\s*=\s*next_entry\.handle\.clone\(\);/,
  );
});

test("application menu hover closes deployed popovers after leaving the menu area", () => {
  const standardMenu = functionBody("render_standard_menu");

  assert.match(standardMenu, /\.on_hover\(move \|hover_enter, window, cx\| \{/);
  assert.match(standardMenu, /if \*hover_enter && !current_handle\.is_deployed\(\)/);
  assert.match(standardMenu, /else if !\*hover_enter/);
  assert.match(standardMenu, /window\.on_next_frame\(move \|window, _cx\| \{/);
  assert.match(standardMenu, /window\.on_next_frame\(move \|window, cx\| \{/);
  assert.match(standardMenu, /handle\.is_deployed\(\) && !handle\.is_pointer_near\(window, px\(18\.0\)\)/);
  assert.doesNotMatch(standardMenu, /handle\.is_deployed\(\) && !handle\.is_focused\(window, cx\)/);
  assert.match(standardMenu, /handle\.hide\(cx\);/);
  assert.match(popoverMenuSource, /pub fn is_pointer_near\(&self, window: &Window, padding: Pixels\) -> bool/);
  assert.match(popoverMenuSource, /trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>/);
  assert.match(popoverMenuSource, /menu_bounds: Rc<Cell<Option<Bounds<Pixels>>>>/);
  assert.match(popoverMenuSource, /bounds\.dilate\(padding\)\.contains\(&position\)/);
  assert.match(popoverMenuSource, /element_state\.trigger_bounds\.set\(Some\(bounds\)\)/);
  assert.match(popoverMenuSource, /element_state\.menu_bounds\.set\(/);
});

test("title bar screen and right-tool buttons use domain-specific icons", () => {
  assert.match(
    titleBarSource,
    /WorkspaceScreenKind::Browser => IconName::ToolWeb/,
    "Browser screen dock button should use the preview/browser tool icon",
  );
  assert.match(
    titleBarSource,
    /"screen-dock-agent",\s*IconName::ZedAssistant/s,
    "screen dock should expose a real AI button",
  );
  assert.match(
    titleBarSource,
    /zed_actions::assistant::FocusAgentFullscreen\.boxed_clone\(\)/,
    "AI screen dock button should open the real Agent panel fullscreen action",
  );
  assert.match(
    titleBarSource,
    /"titlebar-shadcn-ui-panel",\s*IconName::Blocks,\s*"UI"/s,
    "UI panel titlebar button should use the component blocks icon",
  );
  assert.match(
    titleBarSource,
    /"titlebar-dx-style-panel",\s*IconName::Sliders,\s*"Style"/s,
    "Style panel titlebar button should use the controls/sliders icon",
  );
  assert.doesNotMatch(titleBarSource, /WorkspaceScreenKind::Browser => IconName::Public/);
});

test("title-bar source guard stays scoped to worker-owned files", () => {
  assert.equal(sourcePath, "crates/title_bar/src/application_menu.rs");
  assert.equal(titleBarPath, "crates/title_bar/src/title_bar.rs");
  assert.equal(popoverMenuPath, "crates/ui/src/components/popover_menu.rs");
  assert.doesNotMatch(sourcePath, /test/i);
  assert.doesNotMatch(titleBarPath, /test/i);
  assert.doesNotMatch(popoverMenuPath, /test/i);
});

function functionBody(name: string): string {
  const start = source.search(new RegExp(`fn\\s+${name}\\b`));
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
}
