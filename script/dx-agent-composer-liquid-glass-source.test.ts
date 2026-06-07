import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const cargo = read("crates/agent_ui/Cargo.toml");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const messageEditor = read("crates/agent_ui/src/message_editor.rs");
const liquidGlass = read("crates/liquid_glass/src/lib.rs");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

const sourceWindow = (source: string, needle: string, before = 600, after = 600) => {
  const index = source.indexOf(needle);
  assert.ok(index >= 0, `missing ${needle}`);
  return source.slice(Math.max(0, index - before), index + needle.length + after);
};

test("Agent composer uses shared liquid glass primitives", () => {
  assert.match(cargo, /liquid_glass\.workspace = true/);
  assert.match(threadView, /use gpui::\{List, RenderImage, TaskExt, canvas\};/);
  assert.match(
    threadView,
    /use liquid_glass::\{default_liquid_glass_style, load_glass_surface, paint_liquid_glass_layer\};/,
  );
  assert.match(threadView, /theme_is_transparent/);
  assert.match(liquidGlass, /pub use backgrounds::load_glass_surface;/);
  assert.match(liquidGlass, /pub fn default_liquid_glass_style\(\) -> LiquidGlassStyle/);
  assert.match(liquidGlass, /let state = ui_state::UiState::default\(\);/);
});

test("composer glass layer is bounded to the composer shell", () => {
  const glassLayer = sourceWindow(threadView, "fn render_composer_liquid_glass_layer", 0, 900);
  const composerShell = sourceWindow(threadView, '"agent-composer-liquid-glass-source"', 900, 4000);

  assert.match(threadView, /fn render_composer_liquid_glass_layer\(source_image: Arc<RenderImage>\) -> AnyElement/);
  assert.match(threadView, /paint_liquid_glass_layer\(window, bounds, bounds, source_image\.clone\(\), &style\)/);
  assert.match(threadView, /\.absolute\(\)\s*\.inset_0\(\)\s*\.size_full\(\)/);
  assert.match(threadView, /composer_glass_readability_background/);
  assert.match(threadView, /composer_glass_fallback_background/);
  assert.match(threadView, /composer_glass_border_color/);
  assert.doesNotMatch(glassLayer + composerShell, /std::fs|spawn|background_executor|thread::sleep/);
});

test("composer preserves the real editor and controls", () => {
  assert.match(
    threadView,
    /use_keyed_state\(\s*\(\s*"agent-composer-liquid-glass-source",\s*cx\.entity_id\(\)\.as_u64\(\),\s*\),\s*cx,\s*\|_, _\| load_glass_surface\(\),\s*\)/s,
  );
  assert.match(threadView, /\.relative\(\)\s*\.overflow_hidden\(\)\s*\.rounded_md\(\)/);
  assert.match(threadView, /\.border_color\(border_color\)\s*\.bg\(fallback_background\)/);
  assert.match(threadView, /render_composer_liquid_glass_layer\(glass_source\)/);
  assert.match(threadView, /self\.message_editor\.clone\(\)/);
  assert.match(threadView, /self\.render_add_context_button\(cx\)/);
  assert.match(threadView, /self\.render_profile_option_slots\(cx\)/);
  assert.match(threadView, /self\.render_voice_controls\(window, cx\)/);
  assert.match(threadView, /self\.render_send_button\(cx\)/);
  assert.match(messageEditor, /background: cx\.theme\(\)\.system\(\)\.transparent/);
});

test("composer liquid glass guard is discoverable", () => {
  assert.match(registry, /"script\/dx-agent-composer-liquid-glass-source\.test\.ts"/);
});
