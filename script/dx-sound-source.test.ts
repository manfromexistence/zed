import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const audio = readFileSync("crates/audio/src/audio.rs", "utf8");
const audioPipeline = readFileSync("crates/audio/src/audio_pipeline.rs", "utf8");
const dxSounds = existsSync("crates/audio/src/dx_sounds.rs")
  ? readFileSync("crates/audio/src/dx_sounds.rs", "utf8")
  : "";
const sidebar = readFileSync("crates/sidebar/src/sidebar.rs", "utf8");
const workspace = readFileSync("crates/workspace/src/workspace.rs", "utf8");
const dock = readFileSync("crates/workspace/src/dock.rs", "utf8");
const editor = readFileSync("crates/editor/src/editor.rs", "utf8");
const editorInput = readFileSync("crates/editor/src/input.rs", "utf8");
const agentPanel = readFileSync("crates/agent_ui/src/agent_panel.rs", "utf8");
const conversationView = readFileSync(
  "crates/agent_ui/src/conversation_view.rs",
  "utf8",
);
const sidebarCargo = readFileSync("crates/sidebar/Cargo.toml", "utf8");
const workspaceCargo = readFileSync("crates/workspace/Cargo.toml", "utf8");
const editorCargo = readFileSync("crates/editor/Cargo.toml", "utf8");

const requiredAssets = [
  "dx_action_confirm",
  "dx_attention_beep",
  "dx_chat_drop_magic",
  "dx_delete_soft",
  "dx_drag_watch_tick",
  "dx_hover_soft",
  "dx_magic_fire",
  "dx_magic_heal",
  "dx_magic_wand",
  "dx_menu_snap",
  "dx_panel_close",
  "dx_panel_open",
  "dx_pop_confirm",
  "dx_screen_launch",
  "dx_success_chime",
  "dx_typing_key",
];

const requiredSoundVariants = [
  "DxActionConfirm",
  "DxAttentionBeep",
  "DxChatDropMagic",
  "DxDeleteSoft",
  "DxDragWatchTick",
  "DxHoverSoft",
  "DxMagicFire",
  "DxMagicHeal",
  "DxMagicWand",
  "DxMenuSnap",
  "DxPanelClose",
  "DxPanelOpen",
  "DxPopConfirm",
  "DxScreenLaunch",
  "DxSuccessChime",
  "DxTypingKey",
];

test("DX sound assets and enum variants stay in sync", () => {
  for (const [index, asset] of requiredAssets.entries()) {
    assert.ok(
      existsSync(`assets/sounds/${asset}.wav`),
      `expected assets/sounds/${asset}.wav`,
    );
    assert.match(audio, new RegExp(`\\b${requiredSoundVariants[index]}\\b`));
  }
});

test("DX sound playback uses semantic events with throttling", () => {
  assert.match(audio, /mod dx_sounds;/);
  assert.match(audio, /pub use dx_sounds::\{DxSoundEvent, DxSoundPolicy\};/);
  assert.match(dxSounds, /pub enum DxSoundEvent/);
  assert.match(dxSounds, /pub enum DxSoundPolicy/);
  assert.doesNotMatch(dxSounds, /Self::TypingKey \| Self::HoverSoft => DxSoundPolicy::ExplicitOptIn/);
  assert.match(dxSounds, /HoverSoft[\s\S]+DxSoundPolicy::ExplicitOptIn/);
  assert.match(dxSounds, /pub\(crate\) fn gain\(self\) -> f32 \{\s*0\.10\s*\}/);
  assert.match(audioPipeline, /dx_sound_last_played: HashMap<DxSoundEvent, Instant>/);
  assert.match(audioPipeline, /pub fn play_dx_sound\(event: DxSoundEvent, cx: &mut App\)/);
  assert.match(audioPipeline, /fn should_play_dx_sound/);
  assert.match(audioPipeline, /source\.amplify\(event\.gain\(\)\)/);
  assert.match(audioPipeline, /event\.cooldown\(\)/);
});

test("safe UI surfaces use semantic DX sound events", () => {
  assert.match(sidebarCargo, /^audio\.workspace = true$/m);
  assert.match(workspaceCargo, /^audio\.workspace = true$/m);
  assert.match(editorCargo, /^audio\.workspace = true$/m);
  assert.match(editorInput, /DxSoundEvent::TypingKey/);
  assert.match(editorInput, /should_queue_power_mode_effect[\s\S]+Audio::play_dx_sound\(DxSoundEvent::TypingKey, cx\)/);
  assert.match(editor, /DxSoundEvent::DeleteSoft/);
  assert.match(sidebar, /DxSoundEvent::DragWatchTick/);
  assert.match(sidebar, /DxSoundEvent::ChatDropMagic/);
  assert.match(sidebar, /DxSoundEvent::MagicHeal/);
  assert.match(sidebar, /DxSoundEvent::DeleteSoft/);
  assert.match(workspace, /DxSoundEvent::ScreenLaunch/);
  assert.match(workspace, /DxSoundEvent::ActionConfirm/);
  assert.match(dock, /DxSoundEvent::ActionConfirm/);
  assert.match(workspace, /DxSoundEvent::PanelOpen/);
  assert.match(workspace, /DxSoundEvent::PanelClose/);
  assert.match(agentPanel, /DxSoundEvent::SuccessChime/);
  assert.match(conversationView, /DxSoundEvent::SuccessChime/);
});
