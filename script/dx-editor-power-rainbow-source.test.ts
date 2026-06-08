import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const defaultSettings = read("assets/settings/default.json");
const editor = read("crates/editor/src/editor.rs");
const editorElement = read("crates/editor/src/element.rs");
const editorInput = read("crates/editor/src/input.rs");
const rainbowCaret = read("crates/editor/src/rainbow_caret.rs");
const editorSettings = read("crates/editor/src/editor_settings.rs");
const settingsContent = read("crates/settings_content/src/editor.rs");
const settingsPageData = read("crates/settings_ui/src/page_data.rs");
const vscodeImport = read("crates/settings/src/vscode_import.rs");
const uiComponents = read("crates/ui/src/components.rs");
const rainbowGlow = read("crates/ui/src/components/dx_rainbow_glow.rs");
const dxLaunchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");
const allSettings = read("docs/src/reference/all-settings.md");
const vimDocs = read("docs/src/vim.md");
const visualCustomizationDocs = read("docs/src/visual-customization.md");
const vscodeMigrationDocs = read("docs/src/migrate/vs-code.md");

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
  assert.equal(
    ((editor + editorInput).match(/queue_power_mode_insert_effect/g) ?? []).length,
    2,
    "expected one method definition and one guarded input call site",
  );
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
  assert.match(defaultSettings, /"cursor_blink": false/);
  assert.match(defaultSettings, /"rainbow_caret_animation": true/);
  assert.match(settingsContent, /\/\/\/ Default: false\s*pub cursor_blink: Option<bool>,/);
  assert.match(
    settingsContent,
    /\/\/\/ Default: true\s*pub rainbow_caret_animation: Option<bool>,/s,
  );
  assert.match(editorSettings, /pub rainbow_caret_animation: bool/);
  assert.match(
    editorSettings,
    /rainbow_caret_animation: editor\.rainbow_caret_animation\.unwrap\(\)/,
  );
  assert.match(vscodeImport, /let cursor_blink = self\.read_enum\("editor\.cursorBlinking"/);
  assert.match(vscodeImport, /"blink" \| "phase" \| "expand" \| "smooth" => Some\(true\)/);
  assert.match(vscodeImport, /"solid" => Some\(false\)/);
  assert.match(
    vscodeImport,
    /let rainbow_caret_animation = match cursor_blink \{\s*Some\(false\) => Some\(false\),\s*_ => None,\s*\};/s,
  );
  assert.match(vscodeImport, /cursor_blink,\s*rainbow_caret_animation,/s);
  assert.match(
    settingsPageData,
    /title: "Rainbow Caret Animation"[\s\S]*json_path: Some\("rainbow_caret_animation"\)[\s\S]*settings_content\.editor\.rainbow_caret_animation/s,
  );
  assert.match(
    allSettings,
    /## Cursor Blink[\s\S]*- Setting: `cursor_blink`[\s\S]*- Default: `false`[\s\S]*## Rainbow Caret Animation[\s\S]*- Setting: `rainbow_caret_animation`[\s\S]*- Default: `true`/s,
  );
  assert.match(vimDocs, /\| rainbow_caret_animation \|[\s\S]*\| `true`\s*\|/);
  assert.match(visualCustomizationDocs, /"rainbow_caret_animation": true/);
  assert.match(
    vscodeMigrationDocs,
    /\| `editor\.cursorBlinking`\s*\|\s*`cursor_blink`, disables `rainbow_caret_animation` for `"solid"`\s*\|/,
  );
  assert.match(uiComponents, /mod dx_rainbow_glow;/);
  assert.match(
    uiComponents,
    /pub use dx_rainbow_glow::\{\s*DxRainbowGlow, DxRainbowMotion, DxRainbowPaintSample, dx_rainbow_paint_sample,\s*paint_dx_rainbow_caret_glow,\s*\};/s,
  );
  assert.match(rainbowGlow, /const DX_RAINBOW_STRIPE_COUNT: usize = 25;/);
  assert.match(rainbowGlow, /const DX_RAINBOW_STOP_COUNT: usize = 17;/);
  assert.match(rainbowGlow, /const DX_RAINBOW_STOPS: \[Hsla; DX_RAINBOW_STOP_COUNT\] = \[/);
  assert.match(rainbowGlow, /h: 0\.966503268[\s\S]*h: 0\.025773196[\s\S]*h: 0\.512562814[\s\S]*h: 0\.940074906/s);
  assert.match(rainbowGlow, /linear_color_stop, linear_gradient/);
  assert.match(rainbowGlow, /pub enum DxRainbowMotion \{\s*Animated,\s*Reduced,\s*\}/s);
  assert.match(rainbowGlow, /enum DxRainbowDirection \{\s*Forward,\s*Reverse,\s*\}/s);
  assert.match(rainbowGlow, /motion: DxRainbowMotion::Reduced/);
  assert.match(rainbowGlow, /pub fn animated\(\) -> Self \{\s*Self::new\(\)\.motion\(DxRainbowMotion::Animated\)\s*\}/s);
  assert.match(rainbowGlow, /self\.height = height\.max\(Pixels::ZERO\);/);
  assert.match(rainbowGlow, /self\.radius = radius\.max\(Pixels::ZERO\);/);
  assert.match(rainbowGlow, /self\.phase_offset = normalize_phase\(offset\);/);
  assert.doesNotMatch(rainbowGlow, /SystemTime|UNIX_EPOCH/);
  assert.doesNotMatch(rainbowGlow, /as_secs_f64|DX_RAINBOW_CYCLE_SECONDS/);
  assert.match(rainbowGlow, /const DX_RAINBOW_CARET_CYCLE_NANOS: u128 = 2_400_000_000;/);
  assert.match(rainbowGlow, /const DX_RAINBOW_GLOW_STRIPE_CYCLE_NANOS: u128 = 6_000_000_000;/);
  assert.match(rainbowGlow, /const DX_RAINBOW_GLOW_NEAR_WASH_CYCLE_NANOS: u128 = 7_200_000_000;/);
  assert.match(rainbowGlow, /const DX_RAINBOW_GLOW_OUTER_WASH_CYCLE_NANOS: u128 = 8_400_000_000;/);
  assert.match(rainbowGlow, /static DX_RAINBOW_STARTED_AT: OnceLock<Instant> = OnceLock::new\(\);/);
  assert.match(
    rainbowGlow,
    /fn dx_rainbow_elapsed_nanos\(\) -> u128 \{[\s\S]*DX_RAINBOW_STARTED_AT[\s\S]*get_or_init\(Instant::now\)[\s\S]*elapsed\(\)[\s\S]*as_nanos\(\)[\s\S]*\}/s,
  );
  assert.match(
    rainbowGlow,
    /fn dx_rainbow_cycle_phase\(elapsed_nanos: u128, cycle_nanos: u128\) -> f32 \{[\s\S]*let cycle_position = elapsed_nanos % cycle_nanos;[\s\S]*cycle_position as f32 \/ cycle_nanos as f32/s,
  );
  assert.match(rainbowGlow, /DxRainbowMotion::Animated => \{[\s\S]*let phase = dx_rainbow_cycle_phase\(elapsed_nanos, cycle_nanos\);[\s\S]*DxRainbowDirection::Forward => phase,[\s\S]*DxRainbowDirection::Reverse => -phase,/s);
  assert.match(rainbowGlow, /DxRainbowMotion::Reduced => DX_RAINBOW_REDUCED_PHASE/);
  assert.doesNotMatch(rainbowGlow, /pub fn is_animated/);
  assert.match(
    rainbowGlow,
    /pub struct DxRainbowPaintSample \{\s*phase: f32,\s*color: Hsla,\s*should_request_animation_frame: bool,\s*\}/s,
  );
  assert.match(rainbowGlow, /pub fn color\(self\) -> Hsla \{\s*self\.color\s*\}/s);
  assert.match(
    rainbowGlow,
    /pub fn should_request_animation_frame\(self\) -> bool \{\s*self\.should_request_animation_frame\s*\}/s,
  );
  assert.doesNotMatch(
    rainbowGlow,
    /pub phase: f32|pub color: Hsla|pub should_request_animation_frame: bool/,
  );
  assert.match(
    rainbowGlow,
    /pub fn dx_rainbow_paint_sample\(\s*motion: DxRainbowMotion,\s*phase_offset: f32,\s*alpha: f32,\s*\) -> DxRainbowPaintSample/s,
  );
  assert.match(
    rainbowGlow,
    /dx_rainbow_phase_now\(\s*motion,\s*phase_offset,\s*DX_RAINBOW_CARET_CYCLE_NANOS,\s*DxRainbowDirection::Forward,\s*\)/s,
  );
  assert.match(
    rainbowGlow,
    /fn dx_rainbow_paint_sample_for_cycle\(\s*motion: DxRainbowMotion,\s*phase_offset: f32,\s*alpha: f32,\s*cycle_nanos: u128,\s*direction: DxRainbowDirection,\s*elapsed_nanos: u128,\s*\) -> DxRainbowPaintSample/s,
  );
  assert.match(
    rainbowGlow,
    /fn dx_rainbow_hsla\(phase: f32, alpha: f32\) -> Hsla \{[\s\S]*let start = DX_RAINBOW_STOPS\[stop_ix\];[\s\S]*let end = DX_RAINBOW_STOPS\[stop_ix \+ 1\];[\s\S]*h: lerp_hue\(start\.h, end\.h, mix\),[\s\S]*s: lerp_unit\(start\.s, end\.s, mix\),[\s\S]*l: lerp_unit\(start\.l, end\.l, mix\),[\s\S]*a: clamp_unit\(alpha\),/s,
  );
  assert.doesNotMatch(rainbowGlow, /DX_RAINBOW_SATURATION|DX_RAINBOW_LIGHTNESS/);
  assert.match(rainbowGlow, /should_request_animation_frame: motion\.is_animated\(\)/);
  assert.doesNotMatch(rainbowGlow, /pub fn request_animation_frame/);
  assert.match(rainbowGlow, /if stripe_sample\.should_request_animation_frame \{\s*window\.request_animation_frame\(\);\s*\}/s);
  assert.match(rainbowGlow, /fn normalize_phase\(phase: f32\) -> f32 \{[\s\S]*phase\.is_finite\(\)[\s\S]*phase\.rem_euclid\(1\.\)[\s\S]*0\./s);
  assert.match(rainbowGlow, /fn clamp_unit\(value: f32\) -> f32 \{[\s\S]*value\.is_finite\(\)[\s\S]*value\.clamp\(0\., 1\.\)[\s\S]*0\./s);
  assert.match(rainbowGlow, /fn paint_dx_rainbow_wash/);
  assert.match(
    rainbowGlow,
    /fn paint_dx_rainbow_wash[\s\S]*let bounds = window\.pixel_snap_bounds\(bounds\);\s*if bounds\.size\.width <= px\(0\.\) \|\| bounds\.size\.height <= px\(0\.\) \{\s*return;\s*\}/s,
  );
  assert.match(rainbowGlow, /let elapsed_nanos = dx_rainbow_elapsed_nanos\(\);/);
  assert.match(rainbowGlow, /DX_RAINBOW_GLOW_STRIPE_CYCLE_NANOS,[\s\S]*DxRainbowDirection::Forward,[\s\S]*elapsed_nanos,/s);
  assert.match(rainbowGlow, /DX_RAINBOW_GLOW_NEAR_WASH_CYCLE_NANOS,[\s\S]*DxRainbowDirection::Forward,[\s\S]*elapsed_nanos,/s);
  assert.match(rainbowGlow, /DX_RAINBOW_GLOW_OUTER_WASH_CYCLE_NANOS,[\s\S]*DxRainbowDirection::Reverse,[\s\S]*elapsed_nanos,/s);
  assert.match(rainbowGlow, /let phase_step = 1\. \/ DX_RAINBOW_STRIPE_COUNT as f32;/);
  assert.match(rainbowGlow, /let next_stripe_phase = stripe_phase \+ phase_step;/);
  assert.match(
    rainbowGlow,
    /linear_gradient\(\s*90\.,\s*linear_color_stop\(dx_rainbow_hsla\(stripe_phase, alpha\), 0\.\),\s*linear_color_stop\(dx_rainbow_hsla\(next_stripe_phase, alpha\), 1\.\),\s*\)/s,
  );
  assert.match(rainbowGlow, /paint_dx_rainbow_stripes\(bounds, radius, stripe_sample\.phase, 1\., window\);/);
  assert.match(rainbowGlow, /if bounds\.size\.width <= px\(0\.\) \|\| bounds\.size\.height <= px\(0\.\) \{/);
  const caretGlowStart = rainbowGlow.indexOf("pub fn paint_dx_rainbow_caret_glow");
  assert.ok(caretGlowStart >= 0, "expected caret glow helper");
  const caretGlowEnd = rainbowGlow.indexOf("fn paint_dx_rainbow_glow", caretGlowStart);
  assert.ok(caretGlowEnd > caretGlowStart, "expected caret glow slice sentinel");
  const caretGlow = rainbowGlow.slice(caretGlowStart, caretGlowEnd);
  assert.match(
    caretGlow,
    /if bounds\.size\.width <= px\(0\.\) \|\| bounds\.size\.height <= px\(0\.\) \{\s*return;\s*\}[\s\S]*let color = clamp_hsla_channels\(color\);[\s\S]*let outer =/,
  );
  assert.match(
    rainbowGlow,
    /fn clamp_hsla_channels\(color: Hsla\) -> Hsla \{\s*hsla\(\s*normalize_phase\(color\.h\),\s*clamp_unit\(color\.s\),\s*clamp_unit\(color\.l\),\s*clamp_unit\(color\.a\),\s*\)\s*\}/s,
  );
  assert.doesNotMatch(rainbowGlow, /pub fn dx_rainbow_caret_color/);
  assert.doesNotMatch(rainbowGlow, /pub fn dx_rainbow_hsla/);
  assert.doesNotMatch(rainbowGlow, /pub fn dx_rainbow_phase_now/);
  assert.doesNotMatch(rainbowGlow, /pub fn paint_dx_rainbow_glow/);
  assert.match(rainbowGlow, /pub fn paint_dx_rainbow_caret_glow/);
  assert.match(
    editorElement,
    /let animate_rainbow_caret = EditorSettings::get_global\(cx\)\.rainbow_caret_animation\s*&& editor\.focus_handle\.contains_focused\(window, cx\);[\s\S]*let rainbow_motion = if animate_rainbow_caret \{\s*DxRainbowMotion::Animated\s*\} else \{\s*DxRainbowMotion::Reduced\s*\};/s,
  );
  assert.doesNotMatch(
    editorElement,
    /EditorSettings::get_global\(cx\)\.cursor_blink \{\s*DxRainbowMotion::Animated/s,
  );
  const rainbowMotionStart = editorElement.indexOf("let show_local_cursors");
  assert.ok(rainbowMotionStart >= 0, "expected rainbow motion setup slice");
  const rainbowMotionEnd = editorElement.indexOf("for (player_color, selections)", rainbowMotionStart);
  assert.ok(rainbowMotionEnd > rainbowMotionStart, "expected rainbow motion setup sentinel");
  assert.doesNotMatch(editorElement.slice(rainbowMotionStart, rainbowMotionEnd), /cursor_blink/);
  assert.match(
    editorElement,
    /let supports_rainbow_caret = matches!\(\s*selection\.cursor_shape,\s*CursorShape::Bar \| CursorShape::Underline\s*\);/s,
  );
  assert.match(
    editorElement,
    /selection\.is_local\s*&& use_rainbow_caret\s*&& supports_rainbow_caret/s,
  );
  assert.match(editorElement, /rainbow_glow: rainbow_motion\.is_some\(\) && selection\.is_newest/);
  assert.match(editorElement, /rainbow_cursor_motion: Option<DxRainbowMotion>/);
  assert.match(editorElement, /WindowBackgroundAppearance/);
  assert.match(editor, /mod rainbow_caret;/);
  assert.match(editor, /use rainbow_caret::RainbowCaretContrastCache;/);
  assert.match(editor, /rainbow_caret_contrast_cache: RainbowCaretContrastCache,/);
  assert.match(
    editor,
    /rainbow_caret_contrast_cache: RainbowCaretContrastCache::default\(\),/,
  );
  assert.match(rainbowCaret, /const RAINBOW_CARET_CONTRAST_CACHE_BUCKETS: usize = 240;/);
  assert.match(rainbowCaret, /pub\(crate\) struct RainbowCaretContrastCache/);
  assert.match(
    rainbowCaret,
    /entries: \[Option<RainbowCaretContrastCacheEntry>; RAINBOW_CARET_CONTRAST_CACHE_BUCKETS\]/,
  );
  assert.match(rainbowCaret, /fn quantized_hue_bucket\(hue: f32\) -> usize/);
  assert.match(rainbowCaret, /let bucket = quantized_hue_bucket\(foreground\.h\);[\s\S]*let foreground = canonicalize_rainbow_foreground\(foreground, bucket\);/s);
  assert.match(rainbowCaret, /fn canonicalize_rainbow_foreground\(color: Hsla, bucket: usize\) -> Hsla/);
  assert.match(rainbowCaret, /h: \(bucket as f32 \+ 0\.5\) \/ RAINBOW_CARET_CONTRAST_CACHE_BUCKETS as f32/);
  assert.match(rainbowCaret, /fn clamp_unit\(value: f32\) -> f32 \{[\s\S]*value\.is_finite\(\)[\s\S]*value\.clamp\(0\., 1\.\)[\s\S]*0\./s);
  assert.match(rainbowCaret, /foreground: QuantizedRainbowForeground::new\(foreground\)/);
  assert.match(rainbowCaret, /h: quantize_phase\(color\.h\)/);
  assert.match(
    rainbowCaret,
    /ensure_minimum_contrast\(foreground, background, minimum_apca_contrast\)/,
  );
  assert.doesNotMatch(rainbowCaret, /Vec<|HashMap|VecDeque|Box<|Rc<|Arc</);
  assert.match(
    editorElement,
    /let \(cursor_layouts, rainbow_cursor_motion\) = self\.editor\.update[\s\S]*\(cursors, rainbow_cursor_motion\)[\s\S]*\(cursor_layouts, rainbow_cursor_motion\)/s,
  );
  assert.match(
    editorElement,
    /const RAINBOW_CARET_MIN_APCA_CONTRAST: f32 = 45\.0;[\s\S]*let \(rainbow_color, should_request_rainbow_frame\) = layout\s*\.rainbow_cursor_motion\s*\.map\(\|motion\| \{\s*let editor_background = cx\.theme\(\)\.colors\(\)\.editor_background;[\s\S]*let contrast_background = if editor_background\.a < 1\.0 \{[\s\S]*let appearance_base = match cx\.theme\(\)\.appearance \{[\s\S]*Appearance::Dark => Hsla::black\(\),[\s\S]*Appearance::Light => Hsla::white\(\),[\s\S]*let background_base = match cx\.theme\(\)\.window_background_appearance\(\) \{[\s\S]*WindowBackgroundAppearance::Opaque => \{[\s\S]*let background = cx\.theme\(\)\.colors\(\)\.background;[\s\S]*appearance_base\.blend\(background\)[\s\S]*_ => appearance_base,[\s\S]*background_base\.blend\(editor_background\)[\s\S]*\} else \{\s*editor_background\s*\};[\s\S]*let sample = dx_rainbow_paint_sample\(motion, 0\., 1\.\);[\s\S]*editor\.rainbow_caret_contrast_cache\.adjusted_color\(\s*sample\.color\(\),\s*contrast_background,\s*RAINBOW_CARET_MIN_APCA_CONTRAST,\s*\)[\s\S]*\(Some\(color\), sample\.should_request_animation_frame\(\)\)\s*\}\)\s*\.unwrap_or\(\(None, false\)\);[\s\S]*let cursor_rainbow_color = if cursor\.rainbow_motion\.is_some\(\) \{\s*rainbow_color\s*\} else \{\s*None\s*\};[\s\S]*cursor\.paint\(layout\.content_origin, window, cx, cursor_rainbow_color\);/s,
  );
  const paintCursorsStart = editorElement.indexOf("fn paint_cursors(");
  assert.ok(paintCursorsStart >= 0, "expected paint_cursors");
  const rainbowMotionMapStart = editorElement.indexOf(".rainbow_cursor_motion", paintCursorsStart);
  assert.ok(rainbowMotionMapStart > paintCursorsStart, "expected rainbow motion map in paint_cursors");
  const beforeRainbowMotionMap = editorElement.slice(paintCursorsStart, rainbowMotionMapStart);
  assert.doesNotMatch(
    beforeRainbowMotionMap,
    /let (?:editor_background|contrast_background) =/,
    "rainbow contrast-background work must stay lazy until rainbow cursor motion is active",
  );
  assert.match(
    editorElement,
    /if should_request_rainbow_frame \{\s*window\.request_animation_frame\(\);\s*\}/,
  );
  assert.match(
    editorElement,
    /let color = rainbow_color\.unwrap_or\(self\.color\);[\s\S]*(fill|outline)\(bounds, color/s,
  );
  assert.match(
    editorElement,
    /if rainbow_color\.is_some\(\) && self\.rainbow_glow \{\s*paint_dx_rainbow_caret_glow\(bounds, color, window\);\s*\}/s,
  );
  assert.match(
    editorElement,
    /window\.paint_quad\(cursor\);[\s\S]*if let Some\(block_text\) = &self\.block_text[\s\S]*if let Some\(name\) = &mut self\.cursor_name/s,
  );
});

test("DX launch rail chrome mounts the shared rainbow glow", () => {
  const sourcesRailStart = dxLaunchWorkspace.indexOf("fn render_sources_rail");
  const progressRailStart = dxLaunchWorkspace.indexOf("fn render_right_rail");
  const railGlowStart = dxLaunchWorkspace.indexOf("fn rail_rainbow_glow");
  const railPinStart = dxLaunchWorkspace.indexOf("fn rail_pin_header");
  assert.ok(sourcesRailStart >= 0, "expected sources rail renderer");
  assert.ok(progressRailStart > sourcesRailStart, "expected progress rail renderer");
  assert.ok(railGlowStart > progressRailStart, "expected rail rainbow glow helper");
  assert.ok(railPinStart > railGlowStart, "expected rail pin helper after glow helper");

  const sourcesRail = dxLaunchWorkspace.slice(sourcesRailStart, progressRailStart);
  const progressRail = dxLaunchWorkspace.slice(progressRailStart, railGlowStart);
  const railGlow = dxLaunchWorkspace.slice(railGlowStart, railPinStart);

  assert.match(dxLaunchWorkspace, /use ui::\{[\s\S]*DxRainbowGlow/);
  assert.match(
    sourcesRail,
    /rail_pin_header\([\s\S]*"dx-sources-rail-pin"[\s\S]*\)\)\s*\.child\(rail_rainbow_glow\("dx-sources-rail-rainbow-glow", 0\.\)\)/s,
  );
  assert.match(
    progressRail,
    /rail_pin_header\([\s\S]*"dx-progress-rail-pin"[\s\S]*\)\)\s*\.child\(rail_rainbow_glow\("dx-progress-rail-rainbow-glow", 0\.18\)\)/s,
  );
  assert.match(
    railGlow,
    /div\(\)[\s\S]*\.h\(px\(8\.0\)\)[\s\S]*\.flex_none\(\)[\s\S]*\.overflow_hidden\(\)[\s\S]*DxRainbowGlow::new\(\)[\s\S]*\.id\(id\)[\s\S]*\.height\(px\(3\.0\)\)[\s\S]*\.radius\(px\(2\.0\)\)[\s\S]*\.phase_offset\(phase_offset\)/s,
  );
  assert.doesNotMatch(dxLaunchWorkspace, /DxRainbowGlow::animated|DxRainbowMotion::Animated/);
});

test("editor visual effects guard is discoverable", () => {
  assert.match(registry, /"script\/dx-editor-power-rainbow-source\.test\.ts"/);
});
