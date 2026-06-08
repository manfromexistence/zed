import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const modelSelectorPath = "crates/agent_ui/src/model_selector.rs";
const languageModelSelectorPath =
  "crates/agent_ui/src/language_model_selector.rs";
const modelSelectorComponentsPath =
  "crates/agent_ui/src/ui/model_selector_components.rs";
const agentModelSelectorPath = "crates/agent_ui/src/agent_model_selector.rs";
const modelSelectorPopoverPath = "crates/agent_ui/src/model_selector_popover.rs";
const profileSelectorPath = "crates/agent_ui/src/profile_selector.rs";
const modeSelectorPath = "crates/agent_ui/src/mode_selector.rs";

const productionSource = (source: string) =>
  source.split(/\r?\n#\[cfg\(test\)\]\r?\nmod tests\s*\{/)[0] ?? source;

const modelSelector = productionSource(readFileSync(modelSelectorPath, "utf8"));
const languageModelSelector = productionSource(
  readFileSync(languageModelSelectorPath, "utf8"),
);
const modelSelectorComponents = productionSource(
  readFileSync(modelSelectorComponentsPath, "utf8"),
);
const agentModelSelector = productionSource(
  readFileSync(agentModelSelectorPath, "utf8"),
);
const modelSelectorPopover = productionSource(
  readFileSync(modelSelectorPopoverPath, "utf8"),
);
const profileSelector = productionSource(readFileSync(profileSelectorPath, "utf8"));
const modeSelector = productionSource(readFileSync(modeSelectorPath, "utf8"));

function sourceBlock(source: string, needle: string): string {
  const start = source.indexOf(needle);
  assert.ok(start >= 0, `expected source block for ${needle}`);

  const openBrace = source.indexOf("{", start);
  assert.ok(openBrace >= start, `expected ${needle} to open a block`);

  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
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

  assert.fail(`expected ${needle} block to close`);
}

test("agent model selector skips stale fuzzy candidate ids before model lookup", () => {
  const fuzzySearch = sliceBetween(
    modelSelector,
    "async fn fuzzy_search(",
    "match model_list {",
  );

  assert.doesNotMatch(fuzzySearch, /candidates\s*\[\s*mat\.candidate_id\s*\]/);
  assert.doesNotMatch(fuzzySearch, /model_list\s*\[\s*mat\.candidate_id\s*\]/);
  assert.match(fuzzySearch, /candidates\.get\(mat\.candidate_id\)\?/);
  assert.match(fuzzySearch, /model_list\.get\(mat\.candidate_id\)\?/);
  assert.match(fuzzySearch, /\.filter_map\(\|mat\|/);
});

test("language model selector skips stale fuzzy candidate ids before model lookup", () => {
  const fuzzySearch = sliceBetween(
    languageModelSelector,
    "pub fn fuzzy_search(&self, query: &str) -> Vec<ModelInfo> {",
    "pub fn exact_search",
  );

  assert.doesNotMatch(
    fuzzySearch,
    /self\.candidates\s*\[\s*mat\.candidate_id\s*\]/,
  );
  assert.doesNotMatch(fuzzySearch, /self\.models\s*\[\s*mat\.candidate_id\s*\]/);
  assert.match(fuzzySearch, /self\.candidates\.get\(mat\.candidate_id\)\?/);
  assert.match(fuzzySearch, /self\.models\.get\(mat\.candidate_id\)\?/);
  assert.match(fuzzySearch, /\.filter_map\(\|mat\|/);
  assertBefore(
    fuzzySearch,
    "matched_models.sort_unstable_by_key",
    "matched_models\n            .into_iter()\n            .take(MAX_SELECTOR_FUZZY_MATCHES)",
    "language model fuzzy matches must preserve sorted ranking before applying the result cap",
  );
});

test("agent UI selector source guard is focused on production selector code", () => {
  assert.equal(modelSelectorPath, "crates/agent_ui/src/model_selector.rs");
  assert.equal(
    languageModelSelectorPath,
    "crates/agent_ui/src/language_model_selector.rs",
  );
  assert.doesNotMatch(modelSelector, /#\[cfg\(test\)\]/);
  assert.doesNotMatch(languageModelSelector, /#\[cfg\(test\)\]/);
});

test("model selector GPUI chrome preserves bounded labels and keyboard-reachable header controls", () => {
  const agentTrigger = sliceBetween(
    agentModelSelector,
    'Button::new("active-model", model_name)',
    "tooltip,",
  );
  const popoverTrigger = sliceBetween(
    modelSelectorPopover,
    'Button::new("active-model", model_name)',
    "tooltip,",
  );
  const profileTrigger = sliceBetween(
    profileSelector,
    'Button::new("profile-selector", selected_profile)',
    "let tooltip",
  );
  const modeTrigger = sliceBetween(
    modeSelector,
    'Button::new("mode-selector-trigger", current_mode_name)',
    "PopoverMenu::new",
  );
  const header = sliceBetween(
    modelSelectorComponents,
    "impl RenderOnce for ModelSelectorHeader",
    "#[derive(IntoElement)]\npub struct ModelSelectorListItem",
  );
  const listItem = sliceBetween(
    modelSelectorComponents,
    "impl RenderOnce for ModelSelectorListItem",
    "#[derive(IntoElement)]\npub struct ModelSelectorFooter",
  );
  const pathIconBranch = sourceBlock(listItem, "ModelIcon::Path(icon_path) => {");

  for (const trigger of [agentTrigger, popoverTrigger, profileTrigger, modeTrigger]) {
    assert.match(trigger, /\.truncate\(true\)/);
  }

  assert.match(
    header,
    /let on_toggle: Option<Rc<dyn Fn\(&ClickEvent, &mut Window, &mut App\) \+ 'static>> =\s*self\.on_toggle\.map\(Rc::from\);/
  );
  assert.match(header, /\.when_some\(on_toggle\.clone\(\),[\s\S]*\.cursor_pointer\(\)[\s\S]*\.on_click\(/);
  assert.match(
    header,
    /div\(\)[\s\S]*\.min_w_0\(\)[\s\S]*\.flex_1\(\)[\s\S]*Label::new\(title\.clone\(\)\)[\s\S]*\.truncate\(\)[\s\S]*\.tooltip\(Tooltip::text\(title\.clone\(\)\)/,
  );
  assert.match(header, /IconButton::new\(format!\("model-provider-toggle-\{title_key\}"\), icon\)[\s\S]*\.tab_index\(0\)/);
  assert.match(listItem, /\.min_w_0\(\)[\s\S]*\.flex_1\(\)[\s\S]*Label::new\(self\.title\.clone\(\)\)\.truncate\(\)[\s\S]*\.tooltip\(Tooltip::text\(self\.title\)\)/);
  assert.match(
    listItem,
    /ModelIcon::Name\(icon_name\) => Icon::new\(icon_name\)[\s\S]*\.color\(model_icon_color\)[\s\S]*\.size\(IconSize::Small\)/,
  );
  assert.match(
    pathIconBranch,
    /Icon::from_external_svg_with_original_colors\(icon_path\)[\s\S]*\.size\(IconSize::Small\)/,
  );
  for (const source of [agentModelSelector, modelSelectorPopover, modelSelectorComponents]) {
    assert.match(
      source,
      /Icon::from_external_svg_with_original_colors/,
      "provider/model brand SVGs should use the explicit original-color path",
    );
  }
  assert.doesNotMatch(
    pathIconBranch,
    /\.color\(model_icon_color\)/,
    "brand SVG logos must not inherit selected/muted theme tints",
  );
});

function sliceBetween(haystack: string, start: string, end: string): string {
  const startIndex = haystack.indexOf(start);
  assert.notEqual(startIndex, -1, `expected ${start}`);
  const endIndex = haystack.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `expected ${end} after ${start}`);
  return haystack.slice(startIndex, endIndex);
}

function assertBefore(
  haystack: string,
  before: string,
  after: string,
  message: string,
) {
  const beforeIndex = haystack.indexOf(before);
  const afterIndex = haystack.indexOf(after);

  assert.notEqual(beforeIndex, -1, `expected ${before}`);
  assert.notEqual(afterIndex, -1, `expected ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
}
