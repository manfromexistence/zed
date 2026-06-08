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
  assert.match(dxIcons, /pub fn dx_loading_icon\(size: IconSize, color: Color, duration_secs: u64\) -> AnyElement/);
  assert.match(dxIcons, /DxUiIcon::Loading => IconName::DxLoader/);
  assert.match(dxIcons, /DxUiIcon::Settings => IconName::DxCog/);
  assert.match(dxIcons, /DxUiIcon::Evidence => IconName::Public/);
  assert.match(dxIcons, /DxUiIcon::Source => IconName::FolderSearch/);
  assert.match(dxIcons, /DxUiIcon::Storage => IconName::DatabaseZap/);
  assert.match(dxIcons, /DxUiIcon::Copy => IconName::Copy/);
  assert.match(dxIcons, /DxUiIcon::Move => IconName::ArrowRightLeft/);
  assert.match(dxIcons, /DxUiIcon::Duplicate => IconName::BookCopy/);
  assert.match(dxIcons, /DxUiIcon::PasteInto => IconName::ReplyArrowRight/);
  assert.match(dxIcons, /with_rotate_animation\(duration_secs\)/);
  assert.doesNotMatch(dxIcons, /LoadCircle|Settings => IconName::Settings/);
});

test("DX shell chrome uses semantic icons instead of scattered literals", () => {
  const dxIcons = read("crates/ui/src/dx_icons.rs");
  const titleBar = read("crates/title_bar/src/title_bar.rs");
  const forgePanel = read("crates/agent_ui/src/dx_forge_panel/panel.rs");
  const forgePanelView = read("crates/agent_ui/src/dx_forge_panel/panel_view.rs");
  const forgeProviderView = read("crates/agent_ui/src/dx_forge_panel/providers/view.rs");
  const stylePanel = read("crates/agent_ui/src/dx_style_panel/panel.rs");
  const launchStylePanel = read("crates/agent_ui/src/dx_launch_workspace/style_panel.rs");
  const agentButton = functionBody(titleBar, "render_agent_screen_button");
  const screenKindIcon = functionBody(titleBar, "screen_kind_icon");
  const hiddenButtons = functionBody(titleBar, "render_hidden_feature_buttons");

  for (const icon of ["Agent", "Browser", "Icons", "Fonts", "Media", "Ui", "Check"]) {
    assert.ok(
      titleBar.includes(`dx_icon(DxUiIcon::${icon})`),
      `title bar should use semantic DX icon ${icon}`,
    );
  }

  assert.match(dxIcons, /DxUiIcon::Agent => IconName::ZedAgent/);
  assert.match(dxIcons, /DxUiIcon::Ai => IconName::ZedAssistant/);
  assert.match(forgePanel, /dx_icon\(DxUiIcon::Forge\)/);
  assert.match(forgePanelView, /icon: dx_icon\(DxUiIcon::Media\)/);
  assert.match(forgeProviderView, /ProviderGroup::Media => dx_icon\(DxUiIcon::Media\)/);
  assert.doesNotMatch(forgePanelView, /icon: IconName::Image/);
  assert.doesNotMatch(forgeProviderView, /ProviderGroup::Media => IconName::Image/);
  assert.match(stylePanel, /dx_icon\(DxUiIcon::Style\)/);
  assert.match(launchStylePanel, /dx_icon\(DxUiIcon::Style\)/);
  assert.doesNotMatch(launchStylePanel, /IconName::Sliders/);
  assert.doesNotMatch(agentButton, /IconName::ZedAssistant/);
  assert.match(agentButton, /dx_icon\(DxUiIcon::Agent\)/);
  assert.doesNotMatch(screenKindIcon, /IconName::ToolWeb/);
  assert.doesNotMatch(
    hiddenButtons,
    /IconName::(?:SquareDot|Font|Image|Blocks|Check)/,
  );
});

test("DX loading and tool surfaces use semantic icon helpers", () => {
  const prelude = read("crates/ui/src/prelude.rs");
  const button = read("crates/ui/src/components/button/button.rs");
  const threadItem = read("crates/ui/src/components/ai/thread_item.rs");
  const sidebar = read("crates/sidebar/src/sidebar.rs");
  const projectPanel = read("crates/project_panel/src/project_panel.rs");
  const agentDiff = read("crates/agent_ui/src/agent_diff.rs");
  const conversationView = read("crates/agent_ui/src/conversation_view.rs");
  const voiceControls = read("crates/agent_ui/src/conversation_view/voice_controls.rs");
  const agentConfiguration = read("crates/agent_ui/src/agent_configuration.rs");
  const manageProfilesModal = read(
    "crates/agent_ui/src/agent_configuration/manage_profiles_modal.rs",
  );
  const composerProfileOptions = read(
    "crates/agent_ui/src/conversation_view/composer_profile_options.rs",
  );
  const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
  const contextServerModal = read(
    "crates/agent_ui/src/agent_configuration/configure_context_server_modal.rs",
  );
  const aiSettingItem = read("crates/ui/src/components/ai/ai_setting_item.rs");
  const agentRegistryUi = read("crates/agent_ui/src/agent_registry_ui.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const checkPanelView = read("crates/agent_ui/src/dx_check_panel_view.rs");
  const launchPromptSources = read("crates/agent_ui/src/dx_launch_prompts/source.rs");
  const launchSourceKinds = read("crates/agent_ui/src/dx_launch_workspace/sources/kinds.rs");
  const screenCarousel = read("crates/workspace/src/screen_carousel.rs");

  assert.match(prelude, /pub use crate::\{DxUiIcon, dx_icon, dx_loading_icon\};/);

  for (const source of [
    button,
    threadItem,
    sidebar,
    agentDiff,
    conversationView,
    contextServerModal,
  ]) {
    assert.match(source, /dx_loading_icon\(/);
    assert.doesNotMatch(source, /IconName::LoadCircle/);
  }

  assert.match(voiceControls, /dx_icon\(DxUiIcon::Loading\)/);
  assert.doesNotMatch(voiceControls, /IconName::LoadCircle/);

  for (const icon of ["Plugins", "Extensions", "Automations", "Settings"]) {
    assert.ok(
      sidebar.includes(`dx_icon(DxUiIcon::${icon})`),
      `sidebar should use semantic DX icon ${icon}`,
    );
  }
  assert.match(sidebar, /dx_icon\(DxUiIcon::Search\)/);
  assert.match(sidebar, /dx_icon\(DxUiIcon::Agent\)/);
  assert.match(projectPanel, /then_some\(dx_icon\(DxUiIcon::Project\)\)/);
  assert.match(agentConfiguration, /IconButton::new\("context-server-config-menu", dx_icon\(DxUiIcon::Settings\)\)/);
  assert.doesNotMatch(agentConfiguration, /IconButton::new\("context-server-config-menu", IconName::Settings\)/);
  assert.match(agentConfiguration, /Icon::new\(dx_icon\(DxUiIcon::Gateway\)\)/);
  assert.doesNotMatch(agentConfiguration, /Icon::new\(IconName::Sliders\)/);
  assert.match(manageProfilesModal, /builtin_profiles::MEDIA => dx_icon\(DxUiIcon::Media\)/);
  assert.match(manageProfilesModal, /builtin_profiles::SEARCH => dx_icon\(DxUiIcon::Search\)/);
  assert.doesNotMatch(manageProfilesModal, /builtin_profiles::MEDIA => IconName::Image/);
  assert.doesNotMatch(manageProfilesModal, /builtin_profiles::SEARCH => IconName::ToolSearch/);
  assert.match(manageProfilesModal, /Icon::new\(dx_icon\(DxUiIcon::Settings\)\)/);
  assert.match(manageProfilesModal, /Some\(dx_icon\(DxUiIcon::Settings\)\)/);
  assert.match(aiSettingItem, /IconButton::new\("menu", dx_icon\(DxUiIcon::Settings\)\)/);
  assert.match(composerProfileOptions, /pub\(super\) enum ComposerOptionIcon/);
  assert.match(composerProfileOptions, /ComposerOptionIcon::Dx\(DxUiIcon::Media\)/);
  assert.match(composerProfileOptions, /ComposerOptionIcon::Dx\(DxUiIcon::Search\)/);
  assert.match(composerProfileOptions, /ComposerOptionIcon::Dx\(DxUiIcon::Gateway\)/);
  assert.doesNotMatch(composerProfileOptions, /IconName::Sliders/);
  assert.match(threadView, /slot\.icon\.icon_name\(\)/);
  assert.match(threadView, /option\.icon\.icon_name\(\)/);
  assert.match(agentRegistryUi, /Icon::new\(dx_icon\(DxUiIcon::Search\)\)/);
  assert.doesNotMatch(agentRegistryUi, /Icon::new\(IconName::MagnifyingGlass\)/);

  for (const icon of ["Search", "Plugins", "Automations", "Evidence", "Media", "Check"]) {
    assert.ok(
      agentPanel.includes(`dx_icon(DxUiIcon::${icon})`),
      `Agent launch surface should use semantic DX icon ${icon}`,
    );
  }

  for (const icon of ["Receipts", "Evidence", "Storage", "Settings", "Source"]) {
    assert.ok(
      launchWorkspace.includes(`dx_icon(DxUiIcon::${icon})`),
      `Launch diagnostics should use semantic DX icon ${icon}`,
    );
  }

  assert.match(agentDiff, /IconButton::new\("review", dx_icon\(DxUiIcon::Receipts\)\)/);
  assert.match(checkPanelView, /Self::Sections => dx_icon\(DxUiIcon::Check\)/);
  assert.match(checkPanelView, /Self::WebAudit => dx_icon\(DxUiIcon::Evidence\)/);
  assert.match(launchPromptSources, /DxSourceKind::MediaOutput => dx_icon\(DxUiIcon::Media\)/);
  assert.match(launchPromptSources, /DxSourceKind::DxToolchainConfig => dx_icon\(DxUiIcon::Settings\)/);
  assert.match(launchSourceKinds, /DxSourceKind::MediaOutput => dx_icon\(DxUiIcon::Media\)/);
  assert.match(launchSourceKinds, /DxSourceKind::DxToolchainConfig => dx_icon\(DxUiIcon::Settings\)/);
  assert.match(screenCarousel, /WorkspaceScreenKind::Browser => dx_icon\(DxUiIcon::Browser\)/);
});

test("legacy loader/settings names do not leak into the DX icon contract", () => {
  const guardedSources = [
    "crates/ui/src/dx_icons.rs",
    "crates/ui/src/components/button/button.rs",
    "crates/ui/src/components/ai/thread_item.rs",
    "crates/project_panel/src/project_panel.rs",
    "crates/title_bar/src/title_bar.rs",
    "crates/agent_ui/src/dx_forge_panel/panel.rs",
    "crates/agent_ui/src/dx_style_panel/panel.rs",
    "crates/agent_ui/src/agent_diff.rs",
    "crates/agent_ui/src/conversation_view.rs",
    "crates/agent_ui/src/conversation_view/voice_controls.rs",
    "crates/agent_ui/src/agent_configuration.rs",
    "crates/agent_ui/src/agent_configuration/manage_profiles_modal.rs",
    "crates/agent_ui/src/agent_configuration/configure_context_server_modal.rs",
    "crates/ui/src/components/ai/ai_setting_item.rs",
    "crates/sidebar/src/sidebar.rs",
    "crates/agent_ui/src/agent_panel.rs",
    "crates/agent_ui/src/dx_launch_workspace.rs",
    "crates/agent_ui/src/dx_check_panel_view.rs",
    "crates/agent_ui/src/dx_launch_prompts/source.rs",
    "crates/agent_ui/src/dx_launch_workspace/sources/kinds.rs",
    "crates/workspace/src/screen_carousel.rs",
    "crates/web_preview/src/web_preview_view.rs",
  ].map(read).join("\n");

  assert.doesNotMatch(guardedSources, /loader-circle|IconName::LoadCircle/);
  assert.doesNotMatch(guardedSources, /IconName::Settings/);
});
