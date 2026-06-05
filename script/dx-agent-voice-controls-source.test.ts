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
const audioModulePath = "crates/audio/src/audio.rs";
const dxHandoffPath = "DX.md";
const agentUiCargoPath = "crates/agent_ui/Cargo.toml";
const zedCargoPath = "crates/zed/Cargo.toml";

const threadView = readFileSync(threadViewPath, "utf8");
const conversationModule = readFileSync(conversationModulePath, "utf8");
const messageEditor = readFileSync(messageEditorPath, "utf8");
const audioPipeline = readFileSync(audioPipelinePath, "utf8");
const audioModule = readFileSync(audioModulePath, "utf8");
const dxHandoff = readFileSync(dxHandoffPath, "utf8");
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
  const voiceButtons = sourceSlice(
    voiceControls,
    "pub(super) fn render_voice_buttons",
    "pub(super) fn render_voice_recording_panel",
  );
  const recordingPanel = sourceSlice(
    voiceControls,
    "pub(super) fn render_voice_recording_panel",
    "fn render_voice_level_meter",
  );

  assert.match(voiceControls, /agent-composer-voice-input/);
  assert.match(voiceControls, /agent-composer-text-to-speech/);
  assert.match(voiceControls, /IconName::Mic/);
  assert.match(voiceControls, /IconName::AudioOn/);
  assert.match(voiceControls, /IconName::Stop/);
  assert.match(voiceButtons, /agent-composer-voice-input[\s\S]+\.on_click\(on_voice_click\)/);
  assert.match(
    voiceButtons,
    /agent-composer-text-to-speech[\s\S]+\.on_click\(on_speak_click\)/,
  );
  assert.doesNotMatch(voiceButtons, /dummy|mock|placeholder|fake|demo/i);
  assert.doesNotMatch(recordingPanel, /dummy|mock|placeholder|fake|demo/i);
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
  assert.match(voiceControls, /Stop Kokoro read-aloud/);
  assert.match(voiceControls, /agent-composer-discard-voice-recording/);
  assert.match(voiceControls, /Discard voice recording/);
  assert.match(voiceControls, /agent-composer-stop-kokoro-read-aloud/);
  assert.match(threadView, /flow_recording_session[\s\S]+telemetry\(\)/);
  assert.match(threadView, /fn cancel_flow_voice_recording/);
  assert.match(threadView, /Flow voice recording discarded/);
  assert.match(
    threadView,
    /ComposerVoicePhase::Speaking => self\.stop_flow_voice_playback\(cx\)/,
  );
  assert.match(threadView, /ComposerVoicePhase::Transcribing => \{\}/);
});

test("voice runtime uses Flow speech code instead of dummy text", () => {
  const runtime = readFileSync(flowRuntimePath, "utf8");
  const startRecording = sourceSlice(
    runtime,
    "pub(crate) fn start_recording",
    "pub(crate) fn transcribe_recording",
  );
  const transcribeRecording = sourceSlice(
    runtime,
    "pub(crate) fn transcribe_recording",
    "pub(crate) fn speak_text",
  );
  const finishRecording = sourceSlice(
    runtime,
    "pub(crate) fn finish",
    "fn build_input_stream",
  );
  const typedInputStream = sourceSlice(
    runtime,
    "fn build_input_stream_typed",
    "fn downmix_and_resample",
  );
  const speakText = sourceSlice(
    runtime,
    "pub(crate) fn speak_text",
    "pub(crate) fn status_summary",
  );
  const ensureSttReady = sourceSlice(
    runtime,
    "fn ensure_stt_ready",
    "fn parakeet_ready",
  );
  const synthesize = sourceSlice(
    runtime,
    "fn synthesize(&self, text: &str) -> Result<PathBuf>",
    "impl FlowRecordingSession",
  );
  const timeoutHelper = sourceSlice(
    runtime,
    "fn run_command_with_timeout",
    "fn apply_tts_process_env",
  );

  assert.match(runtime, /FlowSpeechRuntime/);
  assert.match(runtime, /G:\\\\Dx\\\\flow|DX_FLOW_ROOT|FLOW_ROOT/);
  assert.match(
    runtime,
    /FRIDAY_DEFAULT_STT_MODEL_KEY: &str = "parakeet_unified_en_int8"/,
  );
  assert.match(
    runtime,
    /FLOW_PARAKEET_EXECUTION_MODEL_KEY: &str = "parakeet-tdt-0\.6b-v3-int8"/,
  );
  assert.match(
    runtime,
    /PARAKEET_MODEL_DIR: &str = "models\/stt\/parakeet-tdt-0\.6b-v3-int8"/,
  );
  assert.match(runtime, /flow-dictate/);
  assert.match(startRecording, /ensure_stt_ready\(\)\?/);
  assert.match(startRecording, /device\.default_input_config\(\)\?/);
  assert.match(startRecording, /build_input_stream/);
  assert.match(startRecording, /stream\.play\(\)\?/);
  assert.match(transcribeRecording, /ensure_stt_ready\(\)\?/);
  assert.match(transcribeRecording, /TemporarySpeechFile::new/);
  assert.match(transcribeRecording, /write_recording_wav/);
  assert.match(transcribeRecording, /arg\("--file"\)/);
  assert.match(transcribeRecording, /arg\(audio_file\.path\(\)\)/);
  assert.match(transcribeRecording, /STT_COMMAND_TIMEOUT/);
  assert.match(transcribeRecording, /Flow Parakeet transcription/);
  assert.doesNotMatch(transcribeRecording, /remove_file/);
  assert.match(runtime, /TemporarySpeechFile/);
  assert.match(runtime, /impl Drop for TemporarySpeechFile/);
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
  assert.match(speakText, /Friday Kokoro TTS runtime is not available/);
  assert.match(ensureSttReady, /Flow Parakeet runtime is not built/);
  assert.match(ensureSttReady, /DX_FLOW_DICTATE_BINARY/);
  assert.match(ensureSttReady, /FRIDAY_DEFAULT_STT_MODEL_KEY/);
  assert.match(ensureSttReady, /FLOW_PARAKEET_EXECUTION_MODEL_KEY/);
  assert.match(synthesize, /Command::new\(&self\.python\)/);
  assert.match(synthesize, /arg\(&self\.runner\)/);
  assert.match(synthesize, /arg\("--model-kind"\)/);
  assert.match(synthesize, /arg\("kokoro"\)/);
  assert.match(synthesize, /zed-kokoro-tts/);
  assert.match(synthesize, /arg\("--language"\)/);
  assert.match(synthesize, /arg\("English"\)/);
  assert.match(synthesize, /arg\("--speaker"\)/);
  assert.match(synthesize, /DEFAULT_KOKORO_VOICE/);
  assert.match(synthesize, /arg\("--device"\)/);
  assert.match(synthesize, /arg\("cpu"\)/);
  assert.match(synthesize, /apply_tts_process_env/);
  assert.match(synthesize, /TTS_COMMAND_TIMEOUT/);
  assert.match(synthesize, /Friday Kokoro TTS/);
  assert.match(synthesize, /fs::metadata\(&output_path\)/);
  assert.match(runtime, /STT_COMMAND_TIMEOUT/);
  assert.match(runtime, /TTS_COMMAND_TIMEOUT/);
  assert.match(timeoutHelper, /stdin\(Stdio::null\(\)\)/);
  assert.match(timeoutHelper, /stdout\(Stdio::piped\(\)\)/);
  assert.match(timeoutHelper, /stderr\(Stdio::piped\(\)\)/);
  assert.match(timeoutHelper, /try_wait\(\)/);
  assert.match(timeoutHelper, /child\.kill\(\)/);
  assert.match(timeoutHelper, /child\.wait\(\)/);
  assert.match(timeoutHelper, /timed out after/);
  assert.match(runtime, /HUGGINGFACE_HUB_CACHE/);
  assert.match(runtime, /hf_home\.join\("hub"\)/);
  assert.match(runtime, /RecordingTelemetry/);
  assert.match(runtime, /recent_input_level/);
  assertBefore(
    finishRecording,
    "drop(_stream);",
    "if samples.len() < MIN_RECORDING_SAMPLES",
    "recording finish must close the microphone stream before short-recording errors",
  );
  assert.match(typedInputStream, /TARGET_SAMPLE_RATE as usize \* MAX_RECORDING_SECONDS/);
  assert.match(typedInputStream, /processed\.into_iter\(\)\.take\(remaining\)/);
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
  assert.match(audioPipeline, /play_wav_file_tracked/);
  assert.doesNotMatch(runtime, /mock|placeholder|dummy/i);
});

test("voice text paths use the real message editor contents and insert APIs", () => {
  const transcriptHelper = sourceSlice(
    messageEditor,
    "pub fn insert_transcript_text",
    "pub fn set_placeholder_text",
  );
  const transcriptSeparator = sourceSlice(
    messageEditor,
    "fn transcript_insertion_text",
    "impl Focusable for MessageEditor",
  );

  assert.match(messageEditor, /pub fn text\(&self, cx: &App\) -> String/);
  assert.match(messageEditor, /pub fn insert_text\(/);
  assert.match(messageEditor, /pub fn insert_transcript_text\(/);
  assert.match(messageEditor, /should_prefix_transcript_separator/);
  assert.match(transcriptHelper, /let transcript = transcript\.trim\(\)/);
  assert.match(transcriptHelper, /if transcript\.is_empty\(\)/);
  assert.match(transcriptHelper, /finalize_last_transaction\(cx\)/);
  assert.match(
    transcriptHelper,
    /selections\s*\.newest::<MultiBufferOffset>/,
  );
  assert.match(transcriptHelper, /selection\.range\(\)/);
  assert.match(transcriptHelper, /transcript_insertion_text/);
  assert.match(transcriptHelper, /editor\.insert\(&insertion/);
  assert.match(transcriptSeparator, /previous_transcript_neighbor/);
  assert.match(transcriptSeparator, /next_transcript_neighbor/);
  assert.match(transcriptSeparator, /replacement_range\.start/);
  assert.match(transcriptSeparator, /replacement_range\.end/);
  assert.match(transcriptSeparator, /is_whitespace/);
  assert.match(transcriptSeparator, /is_opening_transcript_punctuation/);
  assert.match(transcriptSeparator, /is_closing_transcript_punctuation/);
  assert.match(transcriptSeparator, /matches!\([\s\S]*character,[\s\S]*'\.'/);
  assert.match(threadView, /message_editor\.read\(cx\)\.text\(cx\)/);
  assert.match(threadView, /let active_editor = this\.active_editor\(cx\)/);
  assert.match(threadView, /insert_transcript_text\(&transcript/);
  assert.match(threadView, /Audio::play_wav_file_tracked/);
  assert.match(threadView, /std::fs::remove_file\(&audio_path\)/);
  assert.match(conversationModule, /AudioPlaybackHandle/);
  assert.match(threadView, /AudioSettings::get_global\(cx\)\.input_audio_device\.clone\(\)/);
  assert.match(threadView, /start_recording\(input_audio_device\.as_ref\(\)\)/);
  assert.doesNotMatch(threadView, /set_text\(&transcript/);
});

test("voice playback keeps audio feature wiring and fallback states", () => {
  const speakComposerText = sourceSlice(
    threadView,
    "fn speak_composer_text",
    "fn stop_flow_voice_playback",
  );
  const stopPlayback = sourceSlice(
    threadView,
    "fn stop_flow_voice_playback",
    "fn report_flow_voice_error",
  );

  assert.match(agentUiCargo, /audio = \["dep:audio"\]/);
  assert.match(zedCargo, /agent_ui = \{ workspace = true, features = \["audio"\] \}/);
  assert.match(threadView, /#\[cfg\(feature = "audio"\)\]/);
  assert.match(threadView, /#\[cfg\(not\(feature = "audio"\)\)\]/);
  assert.match(threadView, /Kokoro playback failed/);
  assert.match(threadView, /Zed audio playback is not available in this build/);
  assert.match(audioModule, /pub use audio_pipeline::\{Audio, AudioPlaybackHandle\}/);
  assert.match(audioPipeline, /pub struct AudioPlaybackHandle/);
  assert.match(audioPipeline, /struct TrackedAudioSource/);
  assert.match(audioPipeline, /AtomicBool/);
  assert.match(audioPipeline, /pub fn cancel\(&self\)/);
  assert.match(audioPipeline, /pub fn is_complete\(&self\) -> bool/);
  assert.match(audioPipeline, /impl<S> Drop for TrackedAudioSource<S>/);
  assert.match(audioPipeline, /completed\.store\(true, Ordering::Relaxed\)/);
  assert.match(audioPipeline, /play_wav_file_tracked/);
  assert.match(audioPipeline, /output_mixer\.add\(source\)/);
  assert.match(speakComposerText, /flow_playback_handle = Some\(playback_handle\.clone\(\)\)/);
  assert.match(speakComposerText, /std::fs::remove_file\(&audio_path\)/);
  assert.match(speakComposerText, /flow_playback_id/);
  assert.match(speakComposerText, /if this\.flow_playback_id != playback_id/);
  assert.match(speakComposerText, /playback_handle\.is_complete\(\)/);
  assert.match(speakComposerText, /Kokoro finished reading the composer/);
  assert.match(stopPlayback, /handle\.cancel\(\)/);
  assert.match(stopPlayback, /Kokoro read-aloud stopped/);
});

test("voice handoff keeps runtime readiness honest", () => {
  const voiceHandoff = sourceSlice(
    dxHandoff,
    "## Agent Composer Flow Speech",
    "- Source-only verification:",
  );

  assert.match(voiceHandoff, /flow-dictate\.exe` is still not present/);
  assert.match(voiceHandoff, /live STT proof still needs the governed artifact\/build step/);
  assert.match(voiceHandoff, /G:\\Flow\\data\\models\\tts\\kokoro_82m/);
  assert.match(voiceHandoff, /config\.json/);
  assert.match(voiceHandoff, /kokoro-v1_0\.pth/);
  assert.match(voiceHandoff, /live Kokoro synthesis\/playback proof still remains deferred/);
  assert.match(voiceHandoff, /tracked\/cancelable WAV playback handle/);
  assert.match(voiceHandoff, /Live audible playback proof is still deferred/);
  assert.doesNotMatch(voiceHandoff, /runtime-green|production-ready|launch-ready/i);
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
