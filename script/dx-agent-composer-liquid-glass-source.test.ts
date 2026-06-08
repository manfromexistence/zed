import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const cargo = read("crates/agent_ui/Cargo.toml");
const composerGlass = read("crates/agent_ui/src/conversation_view/composer_liquid_glass.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const messageEditor = read("crates/agent_ui/src/message_editor.rs");
const liquidGlass = read("crates/liquid_glass/src/lib.rs");
const liquidGlassElement = read("crates/liquid_glass/src/element.rs");
const workspaceItem = read("crates/workspace/src/item.rs");
const workspaceSource = read("crates/workspace/src/workspace.rs");
const paneSource = read("crates/workspace/src/pane.rs");
const titleBarSource = read("crates/title_bar/src/title_bar.rs");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

const sourceWindow = (source: string, needle: string, before = 600, after = 600) => {
  const index = source.indexOf(needle);
  assert.ok(index >= 0, `missing ${needle}`);
  return source.slice(Math.max(0, index - before), index + needle.length + after);
};

const functionBody = (source: string, name: string): string => {
  const signature = new RegExp(
    `\\n    (?:pub\\(crate\\)\\s+|pub\\s+)?fn ${name}\\(`,
  );
  const match = signature.exec(source);
  assert.ok(match?.index !== undefined, `expected function ${name}`);

  const start = match.index + 1;
  const openBrace = source.indexOf("{", start);
  assert.ok(openBrace > start, `expected ${name} to have a body`);

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

  assert.fail(`expected ${name} body to close`);
};

test("Agent composer uses shared liquid glass primitives", () => {
  assert.match(cargo, /liquid_glass\.workspace = true/);
  assert.match(threadView, /use liquid_glass::load_glass_surface;/);
  assert.match(
    composerGlass,
    /use gpui::\{AnyElement, App, Hsla, RenderImage\};/,
  );
  assert.match(
    composerGlass,
    /use liquid_glass::\{bounded_liquid_glass_layer, control_surface_liquid_glass_style\};/,
  );
  assert.match(composerGlass, /theme_is_transparent/);
  assert.match(
    liquidGlass,
    /pub use backgrounds::\{BackgroundAsset, load_backgrounds, load_glass_surface\};/,
  );
  assert.match(liquidGlass, /pub fn default_liquid_glass_style\(\) -> LiquidGlassStyle/);
  assert.match(liquidGlass, /pub fn control_surface_liquid_glass_style\(\) -> LiquidGlassStyle/);
  assert.match(liquidGlass, /bounded_liquid_glass_layer/);
  assert.match(
    liquidGlassElement,
    /pub fn bounded_liquid_glass_layer\(\s*source_image: Arc<RenderImage>,\s*style: LiquidGlassStyle,\s*\) -> AnyElement/,
  );
  assert.match(
    liquidGlassElement,
    /style\.paint\(window, bounds, bounds, source_image\.clone\(\)\)/,
  );
  assert.match(liquidGlass, /let state = ui_state::UiState::default\(\);/);
  assert.doesNotMatch(liquidGlass, /liquid_glass_view|LiquidGlassView|register_serializable_item|observe_new|NewLiquidGlass/);
  assert.doesNotMatch(workspaceItem, /LiquidGlass/);
  assert.doesNotMatch(workspaceSource + paneSource + titleBarSource, /NewLiquidGlass|WorkspaceScreenKind::LiquidGlass|New Liquid Glass/);
  assert.doesNotMatch(
    threadView,
    /default_liquid_glass_style|paint_liquid_glass_layer|control_surface_liquid_glass_style|canvas\(/,
  );
});

test("control surface liquid glass material values are guarded", () => {
  const controlStyle = sourceWindow(
    liquidGlass,
    "pub fn control_surface_liquid_glass_style",
    0,
    1200,
  );

  for (const value of [
    "power_factor: 3.0",
    "a: 0.7",
    "b: 2.3",
    "c: 5.2",
    "d: 6.9",
    "f_power: 1.0",
    "noise: 0.06",
    "glow_weight: 0.25",
    "glow_edge0: 0.5",
    "glow_edge1: -0.5",
    "glow_bias: 0.0",
    "chromatic_aberration: 0.008",
    "aberration_samples: 5",
    "blur_radius: 2.0",
    "blur_iterations: 1",
    "blur_downscale: 0.5",
  ]) {
    assert.ok(controlStyle.includes(value), `missing ${value}`);
  }
});

test("composer glass layer is bounded to the composer shell", () => {
  const glassLayer = sourceWindow(
    composerGlass,
    "pub(super) fn render_composer_liquid_glass_layer",
    0,
    900,
  );
  const composerShell = sourceWindow(threadView, '"agent-composer-liquid-glass-source"', 900, 4000);

  assert.match(
    composerGlass,
    /pub\(super\) fn render_composer_liquid_glass_layer\(source_image: Arc<RenderImage>\) -> AnyElement/,
  );
  assert.match(
    composerGlass,
    /bounded_liquid_glass_layer\(source_image, control_surface_liquid_glass_style\(\)\)/,
  );
  assert.doesNotMatch(composerGlass, /paint_liquid_glass_layer|canvas\(/);
  assert.match(liquidGlassElement, /\.absolute\(\)\s*\.inset_0\(\)\s*\.size_full\(\)/);
  assert.match(composerGlass, /ComposerGlassSurfaceStyle/);
  assert.match(composerGlass, /composer_glass_surface_style/);
  assert.match(threadView, /COMPOSER_EMPTY_STATE_MAX_LINES: usize = 8/);
  assert.match(threadView, /COMPOSER_COLLAPSED_MAX_HEIGHT_REMS: f32 = 18\./);
  assert.match(threadView, /COMPOSER_COLLAPSED_EDITOR_MAX_HEIGHT_REMS: f32 = 9\./);
  assert.match(composerGlass, /MIN_TEXT_READABILITY_CONTRAST: f32 = 45\.0/);
  assert.match(composerGlass, /MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST: f32 = 30\.0/);
  assert.match(composerGlass, /READABILITY_PLATE_ALPHA: f32 = 0\.62/);
  assert.match(composerGlass, /TRANSPARENT_READABILITY_VEIL_ALPHA: f32 = 0\.18/);
  assert.match(composerGlass, /background: transparent/);
  assert.match(composerGlass, /text_placeholder/);
  assert.match(composerGlass, /icon_muted/);
  assert.doesNotMatch(
    composerGlass,
    /FALLBACK_BACKGROUND_EDITOR_ALPHA|READABILITY_FALLBACK_BACKGROUND_ALPHA|READABILITY_OVERLAY_ALPHA/,
  );
  assert.match(
    composerGlass,
    /needs_readability_plate\(\s*text,\s*text_muted,\s*text_placeholder,\s*icon,\s*icon_muted,\s*readability_base,\s*\)/,
  );
  assert.match(
    composerGlass,
    /apca_contrast\(icon_muted, readability_base\)\.abs\(\)\s*< MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST/,
  );
  assert.doesNotMatch(
    threadView,
    /composer_glass_readability_background|composer_glass_fallback_background|composer_glass_border_color/,
  );
  assert.match(
    composerShell,
    /\.child\(render_composer_liquid_glass_layer\(glass_source\)\)[\s\S]*\.when_some\(\s*glass_surface_style\.readability_overlay[\s\S]*\.child\(\s*v_flex\(\)\s*\.relative\(\)/,
  );
  assert.doesNotMatch(glassLayer + composerShell, /std::fs|spawn|background_executor|thread::sleep/);
});

test("composer preserves the real editor and controls", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const messageEditorRender = functionBody(messageEditor, "render");
  const syncEmptyStateMode = functionBody(threadView, "sync_editor_mode_for_empty_state");
  const collapsedEditorScroll = sourceWindow(
    renderMessageEditor,
    '"agent-composer-collapsed-editor-scroll"',
    0,
    900,
  );

  assert.match(
    threadView,
    /use_keyed_state\(\s*\(\s*"agent-composer-liquid-glass-source",\s*cx\.entity_id\(\)\.as_u64\(\),\s*\),\s*cx,\s*\|_, _\| load_glass_surface\(\),\s*\)/s,
  );
  assert.match(threadView, /\.relative\(\)\s*\.overflow_hidden\(\)\s*\.rounded_md\(\)/);
  assert.match(
    threadView,
    /\.border_color\(glass_surface_style\.border\)\s*\.bg\(glass_surface_style\.background\)/,
  );
  assert.match(threadView, /render_composer_liquid_glass_layer\(glass_source\)/);
  assert.match(threadView, /self\.message_editor\.clone\(\)/);
  assert.match(threadView, /self\.render_add_context_button\(cx\)/);
  assert.match(threadView, /self\.render_profile_option_slots\(cx\)/);
  assert.match(threadView, /self\.render_voice_controls\(window, cx\)/);
  assert.match(threadView, /self\.render_send_button\(cx\)/);
  assert.match(messageEditorRender, /background: cx\.theme\(\)\.system\(\)\.transparent/);
  assert.match(renderMessageEditor, /let expands_editor_area = editor_expanded && has_messages;/);
  assert.match(renderMessageEditor, /else \{\s*this\.flex_1\(\)\.w_full\(\)\s*\}/);
  assert.doesNotMatch(renderMessageEditor, /else \{\s*this\.flex_1\(\)\.size_full\(\)\s*\}/);
  assert.equal(
    renderMessageEditor.match(/render_composer_liquid_glass_layer\(glass_source\)/g)?.length ?? 0,
    1,
  );
  assert.match(renderMessageEditor, /has_messages && !expands_editor_area[\s\S]*max_h\(rems\(COMPOSER_COLLAPSED_MAX_HEIGHT_REMS\)\)/);
  assert.match(collapsedEditorScroll, /\.overflow_y_scroll\(\)/);
  assert.match(collapsedEditorScroll, /\.max_h\(rems\(COMPOSER_COLLAPSED_EDITOR_MAX_HEIGHT_REMS\)\)/);
  assert.match(syncEmptyStateMode, /let max_lines = if has_messages \{[\s\S]*COMPOSER_COLLAPSED_MAX_LINES[\s\S]*\} else \{[\s\S]*COMPOSER_EMPTY_STATE_MAX_LINES/);
  assert.match(syncEmptyStateMode, /EditorMode::AutoHeight \{[\s\S]*min_lines: COMPOSER_MIN_LINES,[\s\S]*max_lines: Some\(max_lines\)/);
  assert.doesNotMatch(syncEmptyStateMode, /EditorMode::Full/);
});

test("composer liquid glass guard is discoverable", () => {
  assert.match(registry, /"script\/dx-agent-composer-liquid-glass-source\.test\.ts"/);
});
