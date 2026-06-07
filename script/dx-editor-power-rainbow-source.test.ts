import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const defaultSettings = read("assets/settings/default.json");
const editor = read("crates/editor/src/editor.rs");
const editorElement = read("crates/editor/src/element.rs");
const editorInput = read("crates/editor/src/input.rs");
const editorSettings = read("crates/editor/src/editor_settings.rs");
const settingsContent = read("crates/settings_content/src/editor.rs");
const vscodeImport = read("crates/settings/src/vscode_import.rs");
const uiComponents = read("crates/ui/src/components.rs");
const rainbowGlow = read("crates/ui/src/components/dx_rainbow_glow.rs");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

test("Power Mode setting is typed, default-off, and import-safe", () => {
  assert.match(defaultSettings, /"power_mode": \{\s*"enabled": false,\s*\}/s);
  assert.match(settingsContent, /pub power_mode: Option<PowerModeContent>/);
  assert.match(settingsContent, /pub struct PowerModeContent \{\s*\/\/\/ Whether to show subtle caret particles and paint-only shake after typing\.\s*\/\/\/\s*\/\/\/ Default: false\s*pub enabled: Option<bool>,\s*\}/s);
  assert.match(editorSettings, /pub power_mode: PowerMode/);
  assert.match(editorSettings, /pub struct PowerMode \{\s*pub enabled: bool,\s*\}/s);
  assert.match(editorSettings, /let power_mode = editor\.power_mode\.unwrap\(\);/);
  assert.match(editorSettings, /enabled: power_mode\.enabled\.unwrap\(\)/);
  assert.match(vscodeImport, /power_mode: None/);
});

test("Power Mode only queues bounded committed typing effects", () => {
  assert.match(editor, /const POWER_MODE_MAX_PARTICLES: usize = 48;/);
  assert.match(editor, /const POWER_MODE_MAX_PENDING_BURSTS: usize = 4;/);
  assert.match(editor, /const POWER_MODE_PARTICLES_PER_BURST: usize = 5;/);
  assert.match(editor, /const POWER_MODE_SHAKE_MAX_PX: f32 = 1\.4;/);
  assert.match(editor, /pub\(crate\) fn queue_power_mode_insert_effect/);
  assert.match(editor, /!self\.mode\.is_full\(\) \|\| !EditorSettings::get_global\(cx\)\.power_mode\.enabled/);
  assert.match(editor, /\.min\(POWER_MODE_MAX_PENDING_BURSTS\)/);
  assert.match(editor, /if self\.power_mode_effects\.particles\.len\(\) > POWER_MODE_MAX_PARTICLES/);
  assert.match(editorInput, /let is_composing_input = self\.ime_transaction\.is_some\(\)/);
  assert.match(editorInput, /let should_queue_power_mode_effect = !is_bulk_input\s*&& !is_composing_input\s*&& !text\.is_empty\(\)\s*&& edits\s*\.iter\(\)\s*\.any\(\|\(_, inserted_text\)\| !inserted_text\.is_empty\(\)\);/s);
  assert.match(editorInput, /if should_queue_power_mode_effect \{\s*this\.queue_power_mode_insert_effect\(cx\);\s*\}/s);
  assert.doesNotMatch(editor + editorInput, /std::fs|Command::new|thread::sleep/);
});

test("Power Mode paints without permanently shifting editor layout", () => {
  assert.match(editorElement, /editor\.flush_power_mode_pending_bursts_at\(cursor_center, cx\);/);
  assert.match(editorElement, /editor\.power_mode_paint_state\(cx\)/);
  assert.match(editorElement, /let original_content_origin = layout\.content_origin;/);
  assert.match(editorElement, /layout\.content_origin = point\(\s*original_content_origin\.x \+ power_mode_state\.shake_offset\.x,\s*original_content_origin\.y \+ power_mode_state\.shake_offset\.y,\s*\);/s);
  assert.match(editorElement, /self\.paint_power_mode_particles\(\s*layout,\s*&power_mode_state\.particles,\s*power_mode_state\.shake_offset,\s*window,\s*\);/s);
  assert.match(editorElement, /layout\.content_origin = original_content_origin;/);
  assert.match(editorElement, /if power_mode_state\.alive \{\s*window\.request_animation_frame\(\);\s*\}/s);
});

test("DX rainbow glow helper is reusable and motion-aware", () => {
  assert.match(uiComponents, /mod dx_rainbow_glow;/);
  assert.match(uiComponents, /pub use dx_rainbow_glow::\*;/);
  assert.match(rainbowGlow, /pub enum DxRainbowMotion \{\s*Animated,\s*Reduced,\s*\}/s);
  assert.match(rainbowGlow, /DxRainbowMotion::Reduced => DX_RAINBOW_REDUCED_PHASE/);
  assert.match(rainbowGlow, /if motion\.is_animated\(\) \{\s*window\.request_animation_frame\(\);\s*\}/s);
  assert.match(rainbowGlow, /pub fn dx_rainbow_caret_color\(motion: DxRainbowMotion\) -> Hsla/);
  assert.match(rainbowGlow, /pub fn paint_dx_rainbow_caret_glow/);
  assert.match(editorElement, /let rainbow_motion = if EditorSettings::get_global\(cx\)\.cursor_blink \{\s*DxRainbowMotion::Animated\s*\} else \{\s*DxRainbowMotion::Reduced\s*\};/s);
  assert.match(editorElement, /\(selection\.is_local && use_rainbow_caret\)\.then_some\(rainbow_motion\)/);
  assert.match(editorElement, /refresh_rainbow_caret \|= cursor\s*\.rainbow_motion\s*\.is_some_and\(DxRainbowMotion::is_animated\);/s);
  assert.match(editorElement, /paint_dx_rainbow_caret_glow\(bounds, self\.color, window\);/);
});

test("editor visual effects guard is discoverable", () => {
  assert.match(registry, /"script\/dx-editor-power-rainbow-source\.test\.ts"/);
});
