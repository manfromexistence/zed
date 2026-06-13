import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const agentCargo = read("crates/agent_ui/Cargo.toml");
const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
const agentScreen = read("crates/agent_ui/src/agent_screen.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const messageEditor = read("crates/agent_ui/src/message_editor.rs");
const workspace = read("crates/workspace/src/workspace.rs");
const agentSettings = read("crates/agent_settings/src/agent_settings.rs");
const defaultSettings = read("assets/settings/default.json");
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
const wgpuRenderer = read("crates/gpui_wgpu/src/wgpu_renderer.rs");
const macosShader = read("crates/gpui_macos/src/shaders.metal");
const wgpuShader = read("crates/gpui_wgpu/src/shaders.wgsl");
const windowsShader = read("crates/gpui_windows/src/shaders.hlsl");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

const escapeRegExp = (text: string) =>
  text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const jsonDefaultPattern = (field: string, value: string) =>
  new RegExp(`"${field}":\\s*${escapeRegExp(value).replace(/\s+/g, "\\s*")}`);

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

const functionBodyAfter = (
  source: string,
  anchor: string,
  name: string,
): string => {
  const anchorIndex = source.indexOf(anchor);
  assert.notEqual(anchorIndex, -1, `expected anchor ${anchor}`);
  return functionBody(source.slice(anchorIndex), name);
};

test("Agent composer mounts the shared Liquid Glass renderer primitive", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const renderGlassSurface = functionBody(
    threadView,
    "render_liquid_glass_chat_input_surface",
  );

  assert.match(agentCargo, /liquid_glass\.workspace = true/);
  assert.match(composerLiquidGlass, /use liquid_glass::\{/);
  assert.match(threadView, /render_agent_liquid_glass_chat_input_surface/);
  assert.match(composerLiquidGlass, /load_liquid_glass_backdrop_carrier/);
  assert.match(composerLiquidGlass, /bounded_liquid_glass_layer/);
  assert.match(composerLiquidGlass, /agent_liquid_glass_style_from_settings/);
  assert.match(renderMessageEditor, /render_liquid_glass_chat_input_surface\(/);
  assert.match(renderGlassSurface, /AgentSettings::get_global\(cx\)\.liquid_glass\.clone\(\)/);
  assert.match(renderGlassSurface, /if !settings\.enabled\s*\{\s*return None;\s*\}/);
  assert.match(renderGlassSurface, /render_agent_liquid_glass_chat_input_surface\(/);
  assert.match(threadView, /chat_input_full_width: bool/);
  assert.match(threadView, /set_chat_input_full_width/);
  assert.match(agentPanel, /set_chat_input_full_width\(/);
  assert.match(agentPanel, /AgentPanelHostKind::BuilderWorkspace/);
  assert.match(renderMessageEditor, /let uses_liquid_glass = glass_surface\.is_some\(\)/);
  assert.match(renderMessageEditor, /let chat_input_full_width = self\.chat_input_full_width/);
  assert.match(renderMessageEditor, /let chat_input_border = if focus_handle\.is_focused\(window\)/);
  assert.match(renderMessageEditor, /colors\.border_focused/);
  assert.match(renderMessageEditor, /\.id\("agent-liquid-glass-chat-input-container"\)/);
  assert.match(renderMessageEditor, /if chat_input_full_width\s*\{\s*this\.w_full\(\)\.flex_1\(\)/);
  assert.match(renderMessageEditor, /\.border_color\(chat_input_border\)/);
  assert.match(renderMessageEditor, /if uses_liquid_glass\s*\{\s*panel_background\.opacity\(0\.08\)/);
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

test("Agent chat input uses the full container bounds for Liquid Glass", () => {
  const boundedLayer = functionBody(liquidGlassElement, "bounded_liquid_glass_layer");
  const geometryLayer = functionBody(
    liquidGlassElement,
    "liquid_glass_layer_with_geometry",
  );

  assert.match(
    boundedLayer,
    /style\.paint\(window, bounds, bounds, source_image\.clone\(\)\)/,
    "Bounded Liquid Glass must fill the actual chat-input container",
  );
  assert.match(composerLiquidGlass, /render_agent_liquid_glass_chat_input_surface/);
  assert.match(composerLiquidGlass, /\.id\("agent-liquid-glass-chat-input-surface"\)/);
  assert.match(composerLiquidGlass, /bounded_liquid_glass_layer\(/);
  assert.doesNotMatch(
    composerLiquidGlass,
    /liquid_glass_layer_with_geometry\(/,
    "Agent chat input must not use standalone demo geometry",
  );
  assert.doesNotMatch(
    composerLiquidGlass,
    /agent_liquid_glass_geometry_from_settings/,
    "Agent chat input must not center a standalone glass quad inside the container",
  );

  assert.match(liquidGlassElement, /pub struct LiquidGlassGeometry/);
  assert.match(liquidGlassElement, /pub fn glass_bounds\(/);
  assert.match(liquidGlassElement, /let width = finite_or_default\(self\.width, 3\.5\)/);
  assert.match(liquidGlassElement, /let height = finite_or_default\(self\.height, 3\.5\)/);
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
  assert.match(liquidGlassElement, /fn has_drawable_area\(bounds: Bounds<Pixels>\) -> bool/);
  assert.match(liquidGlassElement, /width\.is_finite\(\) && height\.is_finite\(\)/);
  assert.match(liquidGlassElement, /if !has_drawable_area\(source_bounds\) \|\| !has_drawable_area\(glass_bounds\)\s*\{\s*return;\s*\}/);
  assert.match(liquidGlassElement, /log::debug!\("failed to paint Liquid Glass layer: \{error\}"\)/);
  assert.doesNotMatch(liquidGlassElement, /let _ = window\.paint_liquid_glass/);
  assert.match(liquidGlassBackgrounds, /static BACKDROP_CARRIER: OnceLock<Arc<RenderImage>>/);
  assert.match(liquidGlassBackgrounds, /\.get_or_init\(\|\|/);
});

test("Agent panel and fullscreen AI screen share the Liquid Glass chat input container", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const renderGlassSurface = functionBody(
    threadView,
    "render_liquid_glass_chat_input_surface",
  );
  const threadRender = functionBodyAfter(
    threadView,
    "impl Render for ThreadView",
    "render",
  );
  const panelRender = functionBodyAfter(
    agentPanel,
    "impl Render for AgentPanel",
    "render",
  );
  const agentScreenNew = functionBody(agentScreen, "new");
  const agentScreenRender = functionBodyAfter(
    agentScreen,
    "impl Render for AgentScreen",
    "render",
  );
  const focusFullscreen = functionBody(agentPanel, "focus_fullscreen");
  const newBuilderWorkspace = functionBody(agentPanel, "new_builder_workspace");
  const renderDxLaunchWorkspace = functionBody(agentPanel, "render_dx_launch_workspace");
  const fullscreenAgentCenter = functionBody(agentPanel, "render_fullscreen_agent_center");
  const centerScreen = functionBody(workspace, "render_center_screen");

  assert.match(
    renderGlassSurface,
    /Some\(render_agent_liquid_glass_chat_input_surface\(&settings\)\)/,
    "Enabled Liquid Glass settings must return the shared chat input surface",
  );
  assert.match(
    renderMessageEditor,
    /\.id\("agent-liquid-glass-chat-input-container"\)[\s\S]*\.when_some\(glass_surface,\s*\|this,\s*surface\|\s*this\.child\(surface\)\)/,
    "Chat input container must mount the real Liquid Glass surface child",
  );
  assert.match(
    renderMessageEditor,
    /\.when\(chat_input_full_width,\s*\|this\|\s*this\.flex_grow_1\(\)\)/,
    "Explicit full-width chat input mode must still be available for hosts that opt into it",
  );
  assert.match(
    renderMessageEditor,
    /\.bg\(if uses_liquid_glass\s*\{\s*panel_background\.opacity\(0\.08\)\s*\}\s*else\s*\{\s*panel_background\.opacity\(0\.72\)\s*\}\)/,
    "Liquid Glass chat input must use transparent chrome while the glass surface is mounted",
  );
  assert.doesNotMatch(
    renderMessageEditor,
    /\.bg\(\s*colors\.panel_background\.opacity\(0\.72\)\s*\)/,
    "Chat input must not unconditionally fall back to flat panel chrome",
  );
  assert.match(
    threadRender,
    /\.child\(self\.render_message_editor\(window, cx\)\)/,
    "ThreadView must own the chat input so every host gets the same Liquid Glass container",
  );
  assert.match(
    panelRender,
    /AgentPanelHostKind::Sidechat \| AgentPanelHostKind::BuilderWorkspace => \{\}/,
    "Agent side panel and fullscreen builder host must share the same AgentPanel render branch",
  );
  assert.match(
    agentScreenNew,
    /AgentPanel::new_builder_workspace\(workspace, window, cx\)/,
    "Fullscreen AI screen item must construct the same builder AgentPanel",
  );
  assert.match(
    agentScreenRender,
    /div\(\)\.size_full\(\)\.child\(self\.panel\.clone\(\)\)/,
    "Fullscreen AI screen item must render the shared AgentPanel directly",
  );
  assert.match(
    focusFullscreen,
    /AgentScreen::open_or_focus\(workspace, window, cx\)/,
    "Fullscreen Agent action must open the AgentScreen item route",
  );
  assert.match(
    newBuilderWorkspace,
    /panel\.host_kind = AgentPanelHostKind::BuilderWorkspace/,
    "Fullscreen AgentPanel constructor must use the builder workspace host kind",
  );
  assert.match(
    newBuilderWorkspace,
    /panel\.ensure_thread_initialized\(window, cx\)/,
    "Fullscreen AgentPanel constructor must initialize the shared thread path",
  );
  assert.match(
    panelRender,
    /VisibleSurface::AgentThread\(conversation_view\) => \{[\s\S]*conversation_view\.set_chat_input_full_width\(false, cx\);[\s\S]*parent[\s\S]*\.child\(self\.render_dx_launch_workspace\(\s*conversation_view\.clone\(\)\.into_any_element\(\),\s*window,\s*cx,\s*\)\)/,
    "AgentPanel must render the shared ConversationView path with the bounded chat input",
  );
  assert.match(
    centerScreen,
    /workspace-agent-screen-center/,
    "Workspace center must host the fullscreen AI Agent panel surface",
  );
  assert.match(
    centerScreen,
    /\.child\(zoomed_view\)/,
    "Fullscreen AI screen must reuse the zoomed AgentPanel rather than a separate flat chat input",
  );
  assert.match(
    centerScreen,
    /if self\.zoomed_is_agent_panel\s*&& let Some\(zoomed_view\) = self\.zoomed\.as_ref\(\)\.and_then\(\|view\| view\.upgrade\(\)\)\s*\{[\s\S]*\.id\("workspace-agent-screen-center"\)[\s\S]*\.child\(zoomed_view\)/,
    "Fullscreen AI screen must be the zoomed AgentPanel branch",
  );
  assert.match(
    renderDxLaunchWorkspace,
    /if !self\.should_render_dx_launch_chrome\(cx\)\s*\{\s*return center;\s*\}/,
    "Agent side panel must pass the ThreadView chat input through without a fullscreen wrapper",
  );
  assert.match(
    renderDxLaunchWorkspace,
    /let center = self\.render_fullscreen_agent_center\(center, cx\);/,
    "Fullscreen AI screen must wrap the same ConversationView center instead of replacing the chat input",
  );
  assert.match(
    fullscreenAgentCenter,
    /\.child\(div\(\)\.size_full\(\)\.min_w_0\(\)\.overflow_hidden\(\)\.child\(center\)\)/,
    "Fullscreen AI center must retain the AgentPanel ConversationView child that contains the Liquid Glass chat input",
  );
  assert.doesNotMatch(
    panelRender,
    /render_message_editor|render_liquid_glass_chat_input_surface|render_agent_liquid_glass_chat_input_surface|message_editor\.clone\(\)|agent-liquid-glass-chat-input-(?:container|surface)/,
    "AgentPanel should not carry a duplicate chat input implementation outside ThreadView",
  );
  assert.doesNotMatch(
    centerScreen,
    /render_message_editor|render_liquid_glass_chat_input_surface|render_agent_liquid_glass_chat_input_surface|message_editor\.clone\(\)|agent-liquid-glass-chat-input-(?:container|surface)/,
    "Workspace fullscreen AI host must not define a separate flat chat input; it must mount the zoomed AgentPanel",
  );
});

test("Agent chat input add-context trigger carries DX web tool transparent logos", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const addContextButton = functionBody(threadView, "render_add_context_button");
  const logoStrip = functionBody(threadView, "render_dx_web_tool_logo_strip");
  const logoTable =
    threadView.match(/const DX_WEB_TOOL_LOGOS: &\[DxWebToolLogo\] = &\[[\s\S]*?\n\];/)?.[0] ??
    "";

  for (const tool of [
    "design",
    "graphics",
    "presentations",
    "spreadsheets",
    "video",
    "music",
    "whiteboard",
    "3d",
    "shader",
    "www",
  ]) {
    for (const appearance of ["light", "dark"]) {
      assert.ok(
        existsSync(`assets/icons/dx_web_tools/${tool}-${appearance}-transparent.svg`),
        `missing ${tool} ${appearance} transparent logo asset`,
      );
    }

    assert.match(
      threadView,
      new RegExp(`icons/dx_web_tools/${tool}-(?:light|dark)-transparent\\.svg`),
      `missing ${tool} transparent logo assets`,
    );
  }

  assert.match(threadView, /struct DxWebToolLogo/);
  assert.match(threadView, /const DX_WEB_TOOL_LOGOS: &\[DxWebToolLogo\]/);
  assert.equal(
    (logoTable.match(/DxWebToolLogo \{/g) ?? []).length,
    10,
    "expected the chat input strip to carry DX web tool logos (incl. shader light/dark transparent variant next to the others)",
  );
  assert.match(logoStrip, /cx\.theme\(\)\.appearance\.is_light\(\)/);
  assert.match(logoStrip, /Icon::from_path\(logo\.path_for_theme\(is_light\)\)/);
  assert.match(logoStrip, /\.id\("dx-web-tool-logo-strip"\)/);
  assert.match(logoStrip, /is_agent_www_tool_preview_enabled\(tool_id\)/);
  assert.match(logoStrip, /open_agent_www_tool_preview\(tool_id, window, cx\)/);
  assert.match(renderMessageEditor, /render_dx_web_tool_logo_strip\(cx\)/);
  assert.match(renderMessageEditor, /render_agent_thread_messages_back_button/);
  assert.match(renderMessageEditor, /showing_www_preview/);
  assert.match(
    renderMessageEditor,
    /self\.render_add_context_button\(cx\)[\s\S]*self\.render_dx_web_tool_logo_strip\(cx\)[\s\S]*self\.render_send_button\(cx\)/,
  );
  assert.doesNotMatch(addContextButton, /render_dx_web_tool_logo_strip/);
  assert.match(addContextButton, /IconButton::new\("add-context", IconName::Plus\)/);
});

test("Agent thread embeds DX WWW preview for whiteboard and shader without leaving chat input", () => {
  const agentThreadPreview = read("crates/agent_ui/src/agent_thread_www_preview.rs");
  const webPreviewInit = read("crates/web_preview/src/web_preview.rs");
  const render = functionBodyAfter(threadView, "impl Render for ThreadView", "render");
  const openPreview = functionBody(threadView, "open_agent_www_tool_preview");
  const closePreview = functionBody(threadView, "close_agent_www_tool_preview");
  const previewSurface = functionBody(threadView, "render_agent_www_preview_surface");

  assert.match(agentThreadPreview, /AGENT_WWW_TOOL_WHITEBOARD/);
  assert.match(agentThreadPreview, /AGENT_WWW_TOOL_SHADER/);
  assert.match(agentThreadPreview, /register_agent_thread_www_preview_hooks/);
  assert.match(webPreviewInit, /agent_thread_www_preview::register_hooks\(\)/);
  assert.match(render, /render_agent_www_preview_surface/);
  assert.match(render, /AgentThreadCenterSurface::WwwPreview/);
  assert.match(openPreview, /preview_url_for_agent_www_tool/);
  assert.match(openPreview, /agent_thread_www_preview_hooks\(\)/);
  assert.match(closePreview, /AgentThreadCenterSurface::Messages/);
  assert.match(previewSurface, /id\("agent-thread-www-preview"\)/);
  assert.match(previewSurface, /FLOATING_MESSAGE_EDITOR_SAFE_PADDING_PX/);
  assert.match(threadView, /IconButton::new\("agent-thread-messages-back", IconName::ArrowLeft\)/);
});

test("Agent Liquid Glass settings preserve the tuned recovered Rust effect values", () => {
  const uiStateDefault = functionBodyAfter(
    liquidGlassState,
    "impl Default for UiState",
    "default",
  );
  const agentLiquidGlassDefault = functionBodyAfter(
    agentSettings,
    "impl Default for AgentLiquidGlassSettings",
    "default",
  );

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
      uiStateDefault,
      new RegExp(`${field}: ${escapeRegExp(value)}`),
      `expected tuned default ${field} = ${value}`,
    );
    assert.match(
      agentLiquidGlassDefault,
      new RegExp(`${field}: ${escapeRegExp(value)}`),
      `expected Agent runtime default ${field} = ${value}`,
    );
    assert.match(
      defaultSettings,
      jsonDefaultPattern(field, value),
      `expected persisted Agent setting default ${field} = ${value}`,
    );
  }

  for (const [field, value] of [
    ["blur_radius", "0.0"],
    ["blur_iterations", "0"],
    ["blur_downscale", "0.1"],
  ]) {
    assert.match(
      uiStateDefault,
      new RegExp(`${field}: ${escapeRegExp(value)}`),
      `expected recovered reference default ${field} = ${value}`,
    );
  }

  for (const [field, value] of [
    ["blur_radius", "2.0"],
    ["blur_iterations", "1"],
    ["blur_downscale", "0.5"],
  ]) {
    assert.match(
      agentLiquidGlassDefault,
      new RegExp(`${field}: ${escapeRegExp(value)}`),
      `expected Agent chat input default ${field} = ${value}`,
    );
    assert.match(
      defaultSettings,
      jsonDefaultPattern(field, value),
      `expected persisted Agent chat input default ${field} = ${value}`,
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
  const refreshLiquidGlassBackdrop = functionBody(
    macosRenderer,
    "refresh_liquid_glass_backdrop",
  );
  const drawWgpuLiquidGlass = functionBody(wgpuRenderer, "draw_liquid_glass");
  assert.match(drawLiquidGlass, /let Some\(backdrop_texture\) = self\.liquid_glass_backdrop_texture\.as_ref\(\) else \{\s*return true;\s*\}/);
  assert.match(refreshLiquidGlassBackdrop, /let Some\(backdrop_texture\) = self\.liquid_glass_backdrop_texture\.as_ref\(\) else \{\s*return true;\s*\}/);
  assert.match(drawWgpuLiquidGlass, /if primitives\.is_empty\(\)\s*\{\s*return true;\s*\}/);
  assert.match(drawWgpuLiquidGlass, /let Some\(backdrop_view\) = self\.resources\(\)\.liquid_glass_backdrop_view\.as_ref\(\) else \{\s*return true;\s*\}/);
  assert.match(drawLiquidGlass, /set_vertex_bytes\(\s*SpriteInputIndex::ViewportSize/);
  assert.match(drawLiquidGlass, /set_fragment_bytes\(\s*SpriteInputIndex::ViewportSize/);
});

test("Settings UI exposes permanent Agent Liquid Glass controls", () => {
  assert.match(settingsPageData, /SettingsPageItem::SectionHeader\("Agent Liquid Glass"\)/);
  assert.match(settingsPageData, /macro_rules! liquid_glass_setting_item/);
  assert.match(settingsPageData, /macro_rules! liquid_glass_vector_setting_item/);
  assert.match(settingsPageData, /json_path: Some\(\$path\)/);
  assert.match(settingsPageData, /\.liquid_glass\s*\.get_or_insert_default\(\)/);
  assert.doesNotMatch(settingsPageData, /Agent composer|composer glass size/);
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
