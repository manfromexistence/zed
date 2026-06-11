import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const agentCargo = read("crates/agent_ui/Cargo.toml");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const messageEditor = read("crates/agent_ui/src/message_editor.rs");
const agentSettings = read("crates/agent_settings/src/agent_settings.rs");
const settingsContentAgent = read("crates/settings_content/src/agent.rs");
const settingsPageData = read("crates/settings_ui/src/page_data.rs");
const liquidGlassElement = read("crates/liquid_glass/src/element.rs");
const liquidGlassBackgrounds = read("crates/liquid_glass/src/backgrounds.rs");
const liquidGlassLib = read("crates/liquid_glass/src/lib.rs");
const liquidGlassState = read("crates/liquid_glass/src/ui_state.rs");
const composerLiquidGlass = read(
  "crates/agent_ui/src/conversation_view/liquid_glass_composer.rs",
);
const macosRenderer = read("crates/gpui_macos/src/metal_renderer.rs");
const macosShader = read("crates/gpui_macos/src/shaders.metal");
const wgpuShader = read("crates/gpui_wgpu/src/shaders.wgsl");
const windowsShader = read("crates/gpui_windows/src/shaders.hlsl");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

const escapeRegExp = (text: string) =>
  text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const functionBody = (source: string, name: string): string => {
  const signature = new RegExp(
    `\\n\\s*(?:pub\\(crate\\)\\s+|pub\\(super\\)\\s+|pub\\s+)?fn ${name}\\(`,
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

test("Agent composer mounts the shared Liquid Glass renderer primitive", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const renderGlassSurface = functionBody(
    threadView,
    "render_liquid_glass_message_editor_surface",
  );

  assert.match(agentCargo, /liquid_glass\.workspace = true/);
  assert.match(composerLiquidGlass, /use liquid_glass::\{/);
  assert.match(threadView, /render_agent_liquid_glass_message_editor_surface/);
  assert.match(composerLiquidGlass, /load_liquid_glass_backdrop_carrier/);
  assert.match(composerLiquidGlass, /agent_liquid_glass_geometry_from_settings/);
  assert.match(composerLiquidGlass, /agent_liquid_glass_style_from_settings/);
  assert.match(renderMessageEditor, /render_liquid_glass_message_editor_surface\(/);
  assert.match(renderGlassSurface, /AgentSettings::get_global\(cx\)\.liquid_glass\.clone\(\)/);
  assert.match(renderGlassSurface, /if !settings\.enabled\s*\{\s*return None;\s*\}/);
  assert.match(renderGlassSurface, /render_agent_liquid_glass_message_editor_surface\(/);
  assert.match(renderMessageEditor, /let uses_liquid_glass = glass_surface\.is_some\(\)/);
  assert.match(renderMessageEditor, /if uses_liquid_glass\s*\{\s*colors\.panel_background\.opacity\(0\.08\)/);
  assert.match(renderMessageEditor, /self\.message_editor\.clone\(\)/);
  assert.match(renderMessageEditor, /self\.render_add_context_button\(cx\)/);
  assert.match(renderMessageEditor, /self\.render_profile_option_slots\(cx\)/);
  assert.match(renderMessageEditor, /self\.render_voice_controls\(window, cx\)/);
  assert.match(renderMessageEditor, /self\.render_send_button\(cx\)/);
  assert.match(
    messageEditor,
    /background: cx\.theme\(\)\.system\(\)\.transparent/,
  );
  assert.doesNotMatch(threadView, /composer_liquid_glass|paint_liquid_glass\(/);
});

test("Agent composer uses recovered Liquid Glass geometry instead of full-rectangle glass", () => {
  const geometryLayer = functionBody(
    liquidGlassElement,
    "liquid_glass_layer_with_geometry",
  );

  assert.match(liquidGlassElement, /pub struct LiquidGlassGeometry/);
  assert.match(liquidGlassElement, /pub fn glass_bounds\(/);
  assert.match(liquidGlassElement, /width \* pixel_scale/);
  assert.match(liquidGlassElement, /height \* pixel_scale/);
  assert.match(liquidGlassElement, /Bounds::centered_at/);
  assert.match(liquidGlassElement, /window\.mouse_position\(\)/);
  assert.match(liquidGlassElement, /source_bounds\.contains\(&mouse_position\)/);
  assert.match(
    geometryLayer,
    /style\.paint\(\s*window,\s*bounds,\s*geometry\.glass_bounds/,
  );
  assert.doesNotMatch(
    geometryLayer,
    /style\.paint\(window, bounds, bounds, source_image\.clone\(\)\)/,
  );

  assert.match(composerLiquidGlass, /agent_liquid_glass_geometry_from_settings/);
  assert.match(composerLiquidGlass, /settings\.width/);
  assert.match(composerLiquidGlass, /settings\.height/);
  assert.match(composerLiquidGlass, /settings\.pixel_scale/);
  assert.match(composerLiquidGlass, /settings\.position/);
  assert.match(composerLiquidGlass, /settings\.mouse_control/);
  assert.match(liquidGlassElement, /log::debug!\("failed to paint Liquid Glass layer: \{error\}"\)/);
  assert.doesNotMatch(liquidGlassElement, /let _ = window\.paint_liquid_glass/);
  assert.match(liquidGlassBackgrounds, /static BACKDROP_CARRIER: OnceLock<Arc<RenderImage>>/);
  assert.match(liquidGlassBackgrounds, /\.get_or_init\(\|\|/);
});

test("Agent Liquid Glass settings preserve the tuned recovered Rust effect values", () => {
  assert.match(
    settingsContentAgent,
    /pub struct AgentLiquidGlassSettingsContent/,
  );
  assert.match(
    settingsContentAgent,
    /pub liquid_glass: Option<AgentLiquidGlassSettingsContent>/,
  );
  assert.match(agentSettings, /pub struct AgentLiquidGlassSettings/);
  assert.match(agentSettings, /pub liquid_glass: AgentLiquidGlassSettings/);
  assert.match(
    agentSettings,
    /impl From<settings::AgentLiquidGlassSettingsContent> for AgentLiquidGlassSettings/,
  );
  assert.doesNotMatch(
    liquidGlassLib,
    /agent_settings::AgentLiquidGlassSettings/,
    "shared Liquid Glass crate must not depend on Agent settings types",
  );
  assert.doesNotMatch(
    read("crates/liquid_glass/Cargo.toml"),
    /agent_settings/,
    "shared Liquid Glass crate must stay settings-neutral",
  );
  assert.match(composerLiquidGlass, /pub\(super\) fn agent_liquid_glass_style_from_settings/);
  assert.match(agentSettings, /const LIQUID_GLASS_BACKGROUND_COUNT: usize = 11/);
  assert.match(agentSettings, /const LIQUID_GLASS_VARIANT_SIZES: &\[\(f32, f32\)\]/);
  assert.match(agentSettings, /sanitize_f32\(content\.width, variant_width, 0\.01, 10\.0\)/);
  assert.match(agentSettings, /sanitize_f32\(content\.height, variant_height, 0\.01, 10\.0\)/);
  assert.match(agentSettings, /sanitize_pair\(/);

  for (const [field, value] of [
    ["power_factor", "3.0"],
    ["width", "3.5"],
    ["height", "3.5"],
    ["a", "0.7"],
    ["b", "2.3"],
    ["c", "5.2"],
    ["d", "6.9"],
    ["f_power", "1.0"],
    ["noise", "0.0"],
    ["glow_weight", "0.054"],
    ["glow_edge0", "1.0"],
    ["glow_edge1", "-1.0"],
    ["glow_bias", "0.353"],
    ["chromatic_aberration", "0.0"],
    ["aberration_samples", "1"],
    ["blur_radius", "0.0"],
    ["blur_iterations", "0"],
    ["blur_downscale", "0.1"],
    ["mouse_control", "false"],
    ["position", "[512.0, 384.0]"],
    ["pixel_scale", "100.0"],
    ["camera_position", "[0.0, 0.0]"],
    ["velocity", "2.0"],
    ["camera_velocity", "2.0"],
    ["current_bg", "0"],
    ["glass_variant", "0"],
  ]) {
    assert.match(
      liquidGlassState,
      new RegExp(`${field}: ${escapeRegExp(value)}`),
      `expected tuned default ${field} = ${value}`,
    );
  }
});

test("platform Liquid Glass shaders preserve recovered glow and backdrop behavior", () => {
  for (const [name, source] of [
    ["wgpu", wgpuShader],
    ["windows", windowsShader],
    ["macos", macosShader],
  ] as const) {
    assert.doesNotMatch(
      source,
      /max\(edge1 - edge0, 0\.00001\)/,
      `${name} shader must preserve reversed glow smoothstep edges`,
    );
    assert.match(
      source,
      /denominator = edge1 - edge0/,
      `${name} shader must keep the signed smoothstep denominator`,
    );
    assert.doesNotMatch(
      source,
      /(?:vec3<f32>|float3)\(0\.93,\s*0\.95,\s*0\.99\)/,
      `${name} shader must not force a milky tint over the recovered effect`,
    );
    assert.doesNotMatch(
      source,
      /(?:mix|lerp)\(color\.a,\s*0\.18,\s*0\.82\)/,
      `${name} shader must not override recovered alpha behavior`,
    );
  }

  const drawLiquidGlass = functionBody(macosRenderer, "draw_liquid_glass");
  assert.match(drawLiquidGlass, /set_vertex_bytes\(\s*SpriteInputIndex::ViewportSize/);
  assert.match(drawLiquidGlass, /set_fragment_bytes\(\s*SpriteInputIndex::ViewportSize/);
});

test("Settings UI exposes permanent Agent Liquid Glass controls", () => {
  assert.match(settingsPageData, /SettingsPageItem::SectionHeader\("Agent Liquid Glass"\)/);
  assert.match(settingsPageData, /macro_rules! liquid_glass_setting_item/);
  assert.match(settingsPageData, /macro_rules! liquid_glass_vector_setting_item/);
  assert.match(settingsPageData, /json_path: Some\(\$path\)/);
  assert.match(settingsPageData, /\.liquid_glass\s*\.get_or_insert_default\(\)/);
  for (const path of [
    "agent.liquid_glass.enabled",
    "agent.liquid_glass.power_factor",
    "agent.liquid_glass.width",
    "agent.liquid_glass.height",
    "agent.liquid_glass.a",
    "agent.liquid_glass.b",
    "agent.liquid_glass.c",
    "agent.liquid_glass.d",
    "agent.liquid_glass.f_power",
    "agent.liquid_glass.noise",
    "agent.liquid_glass.glow_weight",
    "agent.liquid_glass.glow_edge0",
    "agent.liquid_glass.glow_edge1",
    "agent.liquid_glass.glow_bias",
    "agent.liquid_glass.chromatic_aberration",
    "agent.liquid_glass.aberration_samples",
    "agent.liquid_glass.blur_radius",
    "agent.liquid_glass.blur_iterations",
    "agent.liquid_glass.blur_downscale",
    "agent.liquid_glass.mouse_control",
    "agent.liquid_glass.position.0",
    "agent.liquid_glass.position.1",
    "agent.liquid_glass.pixel_scale",
    "agent.liquid_glass.camera_position.0",
    "agent.liquid_glass.camera_position.1",
    "agent.liquid_glass.velocity",
    "agent.liquid_glass.camera_velocity",
    "agent.liquid_glass.current_bg",
    "agent.liquid_glass.glass_variant",
  ]) {
    assert.match(
      settingsPageData,
      new RegExp(`"${path.replace(/\./g, "\\.")}"`),
      `expected settings UI control for ${path}`,
    );
  }
});

test("composer Liquid Glass guard remains discoverable", () => {
  assert.match(registry, /"script\/dx-agent-composer-liquid-glass-source\.test\.ts"/);
});
