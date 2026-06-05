import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const threadViewPath = "crates/agent_ui/src/conversation_view/thread_view.rs";
const conversationModulePath = "crates/agent_ui/src/conversation_view.rs";
const voiceControlsPath =
  "crates/agent_ui/src/conversation_view/voice_controls.rs";
const flowRuntimePath = "crates/agent_ui/src/flow_speech_runtime.rs";
const messageEditorPath = "crates/agent_ui/src/message_editor.rs";
const audioPipelinePath = "crates/audio/src/audio_pipeline.rs";
const agentUiCargoPath = "crates/agent_ui/Cargo.toml";
const zedCargoPath = "crates/zed/Cargo.toml";

const threadView = readFileSync(threadViewPath, "utf8");
const conversationModule = readFileSync(conversationModulePath, "utf8");
const messageEditor = readFileSync(messageEditorPath, "utf8");
const audioPipeline = readFileSync(audioPipelinePath, "utf8");
const agentUiCargo = readFileSync(agentUiCargoPath, "utf8");
const zedCargo = readFileSync(zedCargoPath, "utf8");

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
  assert.match(voiceControls, /agent-composer-stop-voice-recording/);
  assert.match(voiceControls, /agent-composer-voice-level-meter/);
  assert.match(voiceControls, /update_recording_telemetry/);
  assert.match(voiceControls, /captured_duration: Duration/);
  assert.match(voiceControls, /input_level: f32/);
  assert.match(voiceControls, /Duration::ZERO/);
  assert.match(voiceControls, /Stop recording and transcribe/);
  assert.match(threadView, /flow_recording_session[\s\S]+telemetry\(\)/);
  assert.match(
    threadView,
    /ComposerVoicePhase::Transcribing \| ComposerVoicePhase::Speaking => \{\}/,
  );
});

test("voice runtime uses Flow speech code instead of dummy text", () => {
  const runtime = readFileSync(flowRuntimePath, "utf8");

  assert.match(runtime, /FlowSpeechRuntime/);
  assert.match(runtime, /G:\\\\Dx\\\\flow|DX_FLOW_ROOT|FLOW_ROOT/);
  assert.match(runtime, /parakeet_unified_en_int8/);
  assert.match(runtime, /parakeet-tdt-0\.6b-v3-int8/);
  assert.match(runtime, /flow-dictate/);
  assert.match(runtime, /--file/);
  assert.match(runtime, /DeviceId/);
  assert.match(runtime, /resolve_input_device/);
  assert.match(runtime, /device_by_id/);
  assert.match(runtime, /qwen3_tts_runner\.py/);
  assert.match(runtime, /FLOW_TTS_PYTHON/);
  assert.match(runtime, /FLOW_TTS_RUNNER/);
  assert.match(runtime, /DX_KOKORO_TTS_RUNNER/);
  assert.match(runtime, /DX_KOKORO_MODEL_DIR/);
  assert.match(runtime, /DX_FLOW_DATA_ROOT/);
  assert.match(runtime, /FLOW_DATA_DIR/);
  assert.match(runtime, /KokoroTtsRuntime|kokoro_82m/);
  assert.match(runtime, /run_command_with_timeout/);
  assert.match(runtime, /STT_COMMAND_TIMEOUT/);
  assert.match(runtime, /TTS_COMMAND_TIMEOUT/);
  assert.match(runtime, /stdin\(Stdio::null\(\)\)/);
  assert.match(runtime, /stdout\(Stdio::piped\(\)\)/);
  assert.match(runtime, /stderr\(Stdio::piped\(\)\)/);
  assert.match(runtime, /try_wait\(\)/);
  assert.match(runtime, /child\.kill\(\)/);
  assert.match(runtime, /child\.wait\(\)/);
  assert.match(runtime, /timed out after/);
  assert.match(runtime, /RecordingTelemetry/);
  assert.match(runtime, /recent_input_level/);
  assert.match(runtime, /wrote an empty WAV file/);
  assert.match(runtime, /config\.json/);
  assert.match(runtime, /kokoro-v1_0\.pth/);
  assert.match(runtime, /af_heart\.pt/);
  assert.match(runtime, /af_bella\.pt/);
  assert.match(runtime, /file_is_nonempty/);
  assert.match(runtime, /is_file\(\)/);
  assert.doesNotMatch(runtime, /--transcribe/);
  assert.doesNotMatch(runtime, /arg\("-t"\)/);
  assert.doesNotMatch(runtime, /arg\("--speak"\)/);
  assert.doesNotMatch(runtime, /arg\("-s"\)/);
  assert.doesNotMatch(runtime, /Command::new\("flow"\)/);
  assert.doesNotMatch(runtime, /flow\.exe/);
  assert.match(audioPipeline, /play_wav_file/);
  assert.doesNotMatch(runtime, /mock|placeholder|dummy/i);
});

test("voice text paths use the real message editor contents and insert APIs", () => {
  assert.match(messageEditor, /pub fn text\(&self, cx: &App\) -> String/);
  assert.match(messageEditor, /pub fn insert_text\(/);
  assert.match(threadView, /message_editor\.read\(cx\)\.text\(cx\)/);
  assert.match(threadView, /insert_text\(&transcript/);
  assert.match(threadView, /Audio::play_wav_file/);
  assert.match(threadView, /std::fs::remove_file\(&audio_path\)/);
  assert.match(conversationModule, /use audio::\{Audio, AudioSettings, Sound\}/);
  assert.match(threadView, /AudioSettings::get_global\(cx\)\.input_audio_device\.clone\(\)/);
  assert.match(threadView, /start_recording\(input_audio_device\.as_ref\(\)\)/);
  assert.doesNotMatch(threadView, /set_text\(&transcript/);
});

test("voice playback keeps audio feature wiring and fallback states", () => {
  assert.match(agentUiCargo, /audio = \["dep:audio"\]/);
  assert.match(zedCargo, /agent_ui = \{ workspace = true, features = \["audio"\] \}/);
  assert.match(threadView, /#\[cfg\(feature = "audio"\)\]/);
  assert.match(threadView, /#\[cfg\(not\(feature = "audio"\)\)\]/);
  assert.match(threadView, /Kokoro playback failed/);
  assert.match(threadView, /Zed audio playback is not available in this build/);
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
