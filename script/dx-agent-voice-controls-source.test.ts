import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const threadViewPath = "crates/agent_ui/src/conversation_view/thread_view.rs";
const conversationModulePath = "crates/agent_ui/src/conversation_view.rs";
const voiceControlsPath =
  "crates/agent_ui/src/conversation_view/voice_controls.rs";
const flowRuntimePath = "crates/agent_ui/src/flow_speech_runtime.rs";
const messageEditorPath = "crates/agent_ui/src/message_editor.rs";

const threadView = readFileSync(threadViewPath, "utf8");
const conversationModule = readFileSync(conversationModulePath, "utf8");
const messageEditor = readFileSync(messageEditorPath, "utf8");

test("agent composer voice controls live in focused modules", () => {
  assert.ok(existsSync(voiceControlsPath), "expected composer voice controls module");
  assert.ok(existsSync(flowRuntimePath), "expected Flow speech runtime module");

  assert.match(conversationModule, /mod voice_controls;/);
  assert.match(threadView, /use super::voice_controls::/);
  assert.match(threadView, /use crate::flow_speech_runtime::/);
});

test("composer renders separate mic and read-aloud buttons before send", () => {
  const controls = sourceSlice(
    threadView,
    ".children(self.render_voice_controls(window, cx))",
    ".child(self.render_send_button(cx))",
  );

  assert.match(controls, /render_voice_controls\(window, cx\)/);
  assertBefore(
    threadView,
    ".children(self.render_voice_controls(window, cx))",
    ".child(self.render_send_button(cx))",
    "voice controls must stay beside the composer and before send",
  );

  const voiceControls = readFileSync(voiceControlsPath, "utf8");
  assert.match(voiceControls, /agent-composer-voice-input/);
  assert.match(voiceControls, /agent-composer-text-to-speech/);
  assert.match(voiceControls, /IconName::Mic/);
  assert.match(voiceControls, /IconName::AudioOn/);
  assert.match(voiceControls, /IconName::Stop/);
});

test("voice recording UI exposes real recording and transcription states", () => {
  const voiceControls = readFileSync(voiceControlsPath, "utf8");

  assert.match(voiceControls, /enum ComposerVoicePhase/);
  assert.match(voiceControls, /Recording/);
  assert.match(voiceControls, /Transcribing/);
  assert.match(voiceControls, /Speaking/);
  assert.match(voiceControls, /render_voice_recording_panel/);
  assert.match(voiceControls, /Recording with Flow/);
  assert.match(voiceControls, /Transcribing with Parakeet/);
  assert.match(voiceControls, /Reading with Kokoro/);
});

test("voice runtime uses Flow speech code instead of dummy text", () => {
  const runtime = readFileSync(flowRuntimePath, "utf8");

  assert.match(runtime, /FlowSpeechRuntime/);
  assert.match(runtime, /G:\\\\Dx\\\\flow|DX_FLOW_ROOT|FLOW_ROOT/);
  assert.match(runtime, /parakeet-tdt-0\.6b-v3-int8/);
  assert.match(runtime, /kokoro-v1\.0\.int8\.onnx/);
  assert.match(runtime, /flow-dictate/);
  assert.match(runtime, /--file/);
  assert.match(runtime, /--speak/);
  assert.doesNotMatch(runtime, /mock|placeholder|dummy/i);
});

test("voice text paths use the real message editor contents and insert APIs", () => {
  assert.match(messageEditor, /pub fn text\(&self, cx: &App\) -> String/);
  assert.match(messageEditor, /pub fn insert_text\(/);
  assert.match(threadView, /message_editor\.read\(cx\)\.text\(cx\)/);
  assert.match(threadView, /insert_text\(&transcript/);
  assert.doesNotMatch(threadView, /set_text\(&transcript/);
});

function sourceSlice(source: string, startNeedle: string, endNeedle: string) {
  const start = source.indexOf(startNeedle);
  assert.notEqual(start, -1, `expected ${startNeedle}`);
  const end = source.indexOf(endNeedle, start + startNeedle.length);
  assert.notEqual(end, -1, `expected ${endNeedle} after ${startNeedle}`);
  return source.slice(start, end);
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
