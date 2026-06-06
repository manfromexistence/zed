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
const flowDictatePath = "../flow/src/bin/flow-dictate.rs";
const flowDictationHostReadmePath = "../flow/tools/flow-dictation-host/README.md";
const flowWhisperModelKey = "whisper-tiny-ggml";

const threadView = readFileSync(threadViewPath, "utf8");
const conversationModule = readFileSync(conversationModulePath, "utf8");
const messageEditor = readFileSync(messageEditorPath, "utf8");
const audioPipeline = readFileSync(audioPipelinePath, "utf8");
const audioModule = readFileSync(audioModulePath, "utf8");
const dxHandoff = readFileSync(dxHandoffPath, "utf8");
const agentUiCargo = readFileSync(agentUiCargoPath, "utf8");
const zedCargo = readFileSync(zedCargoPath, "utf8");
const flowDictate = readFileSync(flowDictatePath, "utf8");
const flowDictationHostReadme = readFileSync(flowDictationHostReadmePath, "utf8");

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
  const voiceIcon = sourceSlice(
    voiceButtons,
    "let voice_icon = match state.phase",
    "let voice_color = match state.phase",
  );
  const voiceDisabled = sourceSlice(
    voiceButtons,
    "let voice_disabled =",
    "let speak_icon = match state.phase",
  );
  const recordingPanel = sourceSlice(
    voiceControls,
    "pub(super) fn render_voice_recording_panel",
    "fn render_voice_level_meter",
  );

  assert.match(voiceControls, /agent-composer-voice-input/);
  assert.match(voiceControls, /agent-composer-text-to-speech/);
  assert.match(voiceControls, /struct ComposerVoiceAvailability/);
  assert.match(voiceControls, /has_composer_text: bool/);
  assert.match(voiceControls, /stt_ready: bool/);
  assert.match(voiceControls, /tts_ready: bool/);
  assert.match(voiceControls, /IconName::Mic/);
  assert.match(voiceControls, /IconName::AudioOn/);
  assert.match(voiceControls, /IconName::Stop/);
  assert.match(voiceIcon, /ComposerVoicePhase::Recording\s*\|\s*ComposerVoicePhase::Transcribing => IconName::Stop/);
  assert.match(voiceIcon, /ComposerVoicePhase::Speaking => IconName::Mic/);
  assert.doesNotMatch(voiceIcon, /ComposerVoicePhase::Speaking => IconName::Stop/);
  assert.match(voiceButtons, /let speak_icon = match state\.phase/);
  assert.match(voiceButtons, /ComposerVoicePhase::Speaking => IconName::Stop/);
  assert.match(voiceButtons, /!availability\.stt_ready/);
  assert.match(voiceButtons, /\.disabled\(voice_disabled\)/);
  assert.match(voiceDisabled, /ComposerVoicePhase::Speaking => true/);
  assert.match(
    voiceButtons,
    /!availability\.has_composer_text/,
  );
  assert.match(voiceButtons, /!availability\.tts_ready/);
  assert.match(
    voiceButtons,
    /state\.speak_tooltip\(availability\)/,
  );
  assert.match(voiceControls, /Type text in the composer before reading aloud/);
  assert.match(voiceControls, /Kokoro runtime is not ready/);
  assert.match(voiceControls, /Flow STT is not ready/);
  assert.match(voiceControls, /Kokoro read-aloud is active/);
  assert.match(voiceButtons, /agent-composer-voice-input[\s\S]+\.on_click\(on_voice_click\)/);
  assert.match(
    voiceButtons,
    /agent-composer-text-to-speech", speak_icon\)[\s\S]+\.on_click\(on_speak_click\)/,
  );
  assert.doesNotMatch(voiceButtons, /dummy|mock|placeholder|fake|demo/i);
  assert.doesNotMatch(recordingPanel, /dummy|mock|placeholder|fake|demo/i);
});

test("voice recording UI exposes real recording and transcription states", () => {
  const voiceControls = readFileSync(voiceControlsPath, "utf8");
  const recordingPanel = sourceSlice(
    voiceControls,
    "pub(super) fn render_voice_recording_panel",
    "fn render_voice_level_meter",
  );
  const toggleVoiceRecording = sourceSlice(
    threadView,
    "fn toggle_flow_voice_recording",
    "fn stop_flow_voice_action",
  );
  const stopVoiceAction = sourceSlice(
    threadView,
    "fn stop_flow_voice_action",
    "fn cancel_active_flow_voice_action",
  );
  const cancelActiveVoiceAction = sourceSlice(
    threadView,
    "fn cancel_active_flow_voice_action",
    "fn start_flow_voice_recording",
  );
  const recordingPanelWiring = sourceSlice(
    threadView,
    "render_voice_recording_panel(",
    "|this, panel| this.child(panel)",
  );

  assert.match(voiceControls, /enum ComposerVoicePhase/);
  assert.match(voiceControls, /Recording/);
  assert.match(voiceControls, /Transcribing/);
  assert.match(voiceControls, /Speaking/);
  assert.match(voiceControls, /render_voice_recording_panel/);
  assert.match(voiceControls, /Recording with Flow/);
  assert.match(voiceControls, /Transcribing with Flow STT/);
  assert.match(voiceControls, /Reading with Kokoro/);
  assert.match(voiceControls, /90s max/);
  assert.match(voiceControls, /agent-composer-stop-voice-recording/);
  assert.match(voiceControls, /agent-composer-cancel-flow-transcription/);
  assert.match(voiceControls, /agent-composer-voice-level-meter/);
  assert.match(voiceControls, /update_recording_telemetry/);
  assert.match(voiceControls, /captured_duration: Duration/);
  assert.match(voiceControls, /input_level: f32/);
  assert.match(voiceControls, /Duration::ZERO/);
  assert.match(voiceControls, /const VOICE_RECORDING_STOP_TRANSCRIBE_LABEL/);
  assert.match(voiceControls, /const VOICE_RECORDING_STOP_TRANSCRIBE_TOOLTIP/);
  assert.match(voiceControls, /const VOICE_RECORDING_DISCARD_LABEL/);
  assert.match(voiceControls, /const VOICE_RECORDING_DISCARD_TOOLTIP/);
  assert.match(voiceControls, /Stop recording and transcribe/);
  assert.match(voiceControls, /Cancel Flow transcription/);
  assert.match(voiceControls, /Stop Kokoro read-aloud/);
  assert.match(voiceControls, /agent-composer-discard-voice-recording/);
  assert.match(voiceControls, /Discard recording/);
  assert.match(
    recordingPanel,
    /ComposerVoicePhase::Recording => \([\s\S]+VOICE_RECORDING_STOP_TRANSCRIBE_LABEL/,
  );
  assert.match(recordingPanel, /Button::new\(stop_button_id, stop_button_label\)/);
  assert.match(recordingPanel, /Button::new\([\s\S]+VOICE_RECORDING_DISCARD_LABEL/);
  assert.match(recordingPanel, /IconName::Trash/);
  assert.doesNotMatch(
    recordingPanel,
    /agent-composer-discard-voice-recording",\s*IconName::Close/,
  );
  assert.doesNotMatch(recordingPanel, /agent-composer-stop-kokoro-read-aloud/);
  assert.match(
    recordingPanel,
    /ComposerVoicePhase::Recording \| ComposerVoicePhase::Transcribing/,
  );
  assert.doesNotMatch(
    recordingPanel,
    /ComposerVoicePhase::Recording\s*\|\s*ComposerVoicePhase::Transcribing\s*\|\s*ComposerVoicePhase::Speaking/,
  );
  assert.match(voiceControls, /agent-composer-retry-voice-input/);
  assert.match(voiceControls, /Retry Flow voice input/);
  assert.match(voiceControls, /agent-composer-dismiss-voice-error/);
  assert.match(voiceControls, /Dismiss Flow voice error/);
  assert.match(voiceControls, /Retry or dismiss to continue/);
  assert.match(threadView, /flow_recording_session[\s\S]+telemetry\(\)/);
  assert.match(threadView, /MAX_RECORDING_SECONDS/);
  assert.match(threadView, /captured_duration\s*>=\s*Duration::from_secs\(MAX_RECORDING_SECONDS as u64\)/);
  assert.match(threadView, /stop_flow_voice_recording\(window, cx\)/);
  assert.match(threadView, /fn cancel_flow_voice_recording/);
  assert.match(threadView, /Flow voice recording discarded/);
  assert.match(threadView, /fn dismiss_flow_voice_error/);
  assert.match(threadView, /Flow voice error dismissed/);
  assert.match(
    threadView,
    /ComposerVoicePhase::Speaking => self\.stop_flow_voice_playback\(cx\)/,
  );
  assert.match(toggleVoiceRecording, /ComposerVoicePhase::Speaking => \{\}/);
  assert.doesNotMatch(
    toggleVoiceRecording,
    /ComposerVoicePhase::Speaking => self\.stop_flow_voice_playback\(cx\)/,
  );
  assert.match(
    threadView,
    /ComposerVoicePhase::Transcribing => self\.cancel_flow_speech_operation\(cx\)/,
  );
  assert.match(
    stopVoiceAction,
    /ComposerVoicePhase::Recording => self\.stop_flow_voice_recording\(window, cx\)/,
  );
  assert.match(
    cancelActiveVoiceAction,
    /ComposerVoicePhase::Recording => self\.cancel_flow_voice_recording\(cx\)/,
  );
  assert.doesNotMatch(cancelActiveVoiceAction, /stop_flow_voice_recording/);
  assert.doesNotMatch(cancelActiveVoiceAction, /finish\(/);
  assert.doesNotMatch(cancelActiveVoiceAction, /transcribe_recording/);
  assert.match(threadView, /Flow STT canceled/);
  assert.match(
    threadView,
    /MessageEditorEvent::Cancel if self\.composer_voice_state\.is_busy\(\) => \{\s*self\.cancel_active_flow_voice_action\(cx\)\s*\}/,
  );
  assert.doesNotMatch(
    threadView,
    /MessageEditorEvent::Cancel if self\.composer_voice_state\.is_busy\(\) => \{\s*self\.stop_flow_voice_action\(window, cx\)\s*\}/,
  );
  assert.match(
    recordingPanelWiring,
    /cx\.listener\(\|this, _event, window, cx\| \{\s*this\.stop_flow_voice_action\(window, cx\);\s*\}\)/,
  );
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
  const missingSttModelMessage = sourceSlice(
    runtime,
    "fn missing_stt_model_message",
    "fn append_stt_backend_args",
  );
  const appendSttBackendArgs = sourceSlice(
    runtime,
    "fn append_stt_backend_args",
    "fn flow_host_env_file",
  );
  const flowHostEnvFile = sourceSlice(
    runtime,
    "fn flow_host_env_file",
    "fn find_whisper_model_file",
  );
  const requestedWhisperLanguage = sourceSlice(
    runtime,
    "fn requested_whisper_language",
    "impl KokoroTtsRuntime",
  );
  const parseTranscriptOutput = sourceSlice(
    runtime,
    "fn parse_transcript_output",
    "fn command_error",
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
    "fn synthesize(&self, text: &str, cancellation: &FlowSpeechCancellation) -> Result<PathBuf>",
    "impl FlowRecordingSession",
  );
  const timeoutHelper = sourceSlice(
    runtime,
    "fn run_command_with_timeout",
    "fn apply_tts_process_env",
  );
  const dataRootCandidates = sourceSlice(
    runtime,
    "fn candidate_flow_data_roots",
    "fn find_kokoro_python",
  );
  const defaultFlowRoot = sourceTail(runtime, "fn default_flow_root");

  assert.match(runtime, /FlowSpeechRuntime/);
  assert.match(runtime, /pub\(crate\) fn stt_available/);
  assert.match(runtime, /pub\(crate\) fn tts_available/);
  assert.match(runtime, /FlowSpeechCancellation/);
  assert.match(runtime, /AtomicBool/);
  assert.match(runtime, /Ordering/);
  assert.match(runtime, /G:\\\\Dx\\\\flow|DX_FLOW_ROOT|FLOW_ROOT/);
  assert.match(
    runtime,
    /FLOW_PARAKEET_EXECUTION_MODEL_KEY: &str = "parakeet-tdt-0\.6b-v3-int8"/,
  );
  assert.match(
    runtime,
    /FLOW_NEMOTRON_EXECUTION_MODEL_KEY: &str = "nemotron-speech-streaming-en-0\.6b-int8"/,
  );
  assert.match(
    runtime,
    /FLOW_WHISPER_EXECUTION_MODEL_KEY: &str = "whisper-tiny-ggml"/,
  );
  assert.match(
    runtime,
    /PARAKEET_MODEL_DIR: &str = "models\/stt\/parakeet-tdt-0\.6b-v3-int8"/,
  );
  assert.match(
    runtime,
    /NEMOTRON_MODEL_DIR: &str = "models\/stt\/nemotron-speech-streaming-en-0\.6b-int8"/,
  );
  assert.match(runtime, /WHISPER_MODEL_FILE: &str = "models\/stt\/ggml-tiny\.bin"/);
  assert.match(runtime, /enum FlowSttArtifactShape/);
  assert.match(runtime, /SherpaTransducer\s*\{\s*model_dir: &'static str\s*\}/);
  assert.match(runtime, /WhisperCpp\s*\{\s*model_file: &'static str\s*\}/);
  assert.match(runtime, /fn whisper\(\) -> Self/);
  assert.match(
    runtime,
    /FLOW_WHISPER_EXECUTION_MODEL_KEY => Some\(Self::whisper\(\)\)/,
  );
  assert.match(
    runtime,
    /FLOW_DEFAULT_STT_MODEL_KEY: &str = "parakeet-tdt-0\.6b-v3-int8"/,
    "Zed must name the same Parakeet default as the copied Friday Flow runtime",
  );
  assert.match(runtime, /DX_FLOW_STT_MODEL/);
  assert.match(runtime, /FLOW_STT_MODEL/);
  assert.match(runtime, /selected_stt_model/);
  assert.match(runtime, /Unsupported Flow STT model/);
  assert.match(runtime, /flow_host_env_file/);
  assert.match(runtime, /find_whisper_cpp_binary/);
  assert.match(runtime, /FLOW_WHISPER_CPP_BINARY/);
  assert.match(runtime, /DX_WHISPER_CPP_BINARY/);
  assert.match(runtime, /FLOW_WHISPER_CPP_EXE/);
  assert.match(runtime, /FLOW_WHISPER_CPP/);
  assert.match(runtime, /find_whisper_model_file/);
  assert.match(runtime, /FLOW_WHISPER_MODEL/);
  assert.match(runtime, /DX_FLOW_WHISPER_MODEL/);
  assert.match(runtime, /requested_whisper_language/);
  assert.match(runtime, /FLOW_WHISPER_LANGUAGE/);
  assert.match(runtime, /DX_FLOW_WHISPER_LANGUAGE/);
  assert.doesNotMatch(runtime, /parakeet_unified_en_int8/);
  assert.match(runtime, /flow-dictate/);
  assert.match(startRecording, /ensure_stt_ready\(\)\?/);
  assert.match(startRecording, /device\.default_input_config\(\)\?/);
  assert.match(startRecording, /build_input_stream/);
  assert.match(startRecording, /stream\.play\(\)\?/);
  assert.match(transcribeRecording, /cancellation: &FlowSpeechCancellation/);
  assert.match(transcribeRecording, /ensure_stt_ready\(\)\?/);
  assert.match(transcribeRecording, /TemporarySpeechFile::new/);
  assert.match(transcribeRecording, /write_recording_wav/);
  assert.match(transcribeRecording, /arg\("--file"\)/);
  assert.match(transcribeRecording, /arg\(audio_file\.path\(\)\)/);
  assert.match(transcribeRecording, /arg\("--model"\)/);
  assert.match(transcribeRecording, /arg\(stt_model\.key\)/);
  assert.match(transcribeRecording, /append_stt_backend_args\(&mut command, stt_model\)\?/);
  assert.match(appendSttBackendArgs, /FlowSttArtifactShape::WhisperCpp/);
  assert.match(appendSttBackendArgs, /find_whisper_cpp_binary\(\)/);
  assert.match(appendSttBackendArgs, /find_whisper_model_file\(model_file\)/);
  assert.match(appendSttBackendArgs, /arg\("--whisper-bin"\)/);
  assert.match(appendSttBackendArgs, /arg\("--whisper-model"\)/);
  assert.match(appendSttBackendArgs, /requested_whisper_language\(\)/);
  assert.match(appendSttBackendArgs, /arg\("--whisper-language"\)/);
  assert.match(flowHostEnvFile, /path\.is_absolute\(\)/);
  assert.match(flowHostEnvFile, /self\.flow_root\.join\(path\)/);
  assert.match(flowHostEnvFile, /file_is_nonempty\(&path\)/);
  assert.match(missingSttModelMessage, /FLOW_WHISPER_MODEL/);
  assert.match(missingSttModelMessage, /DX_FLOW_WHISPER_MODEL/);
  assert.match(missingSttModelMessage, /FLOW_WHISPER_CPP_BINARY/);
  assert.match(missingSttModelMessage, /DX_WHISPER_CPP_BINARY/);
  assert.match(missingSttModelMessage, /FLOW_WHISPER_CPP_EXE/);
  assert.match(missingSttModelMessage, /FLOW_WHISPER_CPP/);
  assert.match(requestedWhisperLanguage, /FLOW_WHISPER_LANGUAGE/);
  assert.match(requestedWhisperLanguage, /DX_FLOW_WHISPER_LANGUAGE/);
  assert.match(transcribeRecording, /STT_COMMAND_TIMEOUT/);
  assert.match(
    transcribeRecording,
    /run_command_with_timeout\([^,]+,[^,]+,[^,]+,\s*Some\(cancellation\)/,
  );
  assert.match(transcribeRecording, /Flow STT transcription/);
  assert.match(parseTranscriptOutput, /Flow STT transcription failed/);
  assert.match(parseTranscriptOutput, /extract_stt_transcript/);
  assert.match(parseTranscriptOutput, /Flow STT finished without a transcript/);
  assert.doesNotMatch(parseTranscriptOutput, /Flow Parakeet transcription failed/);
  assert.doesNotMatch(parseTranscriptOutput, /stdout\.lines\(\)\.rev\(\)/);
  assert.match(runtime, /fn extract_stt_transcript/);
  assert.match(runtime, /Flow STT transcript marker is missing/);
  assert.match(runtime, /Flow STT transcript marker is empty/);
  assert.match(runtime, /trim_stt_transcript_payload/);
  assert.match(runtime, /preserves_raw_backslash_sequences_from_flow_output/);
  assert.doesNotMatch(runtime, /replace\("\\\\r\\\\n",\s*"\\n"\)/);
  assert.doesNotMatch(runtime, /replace\("\\\\n",\s*"\\n"\)/);
  assert.doesNotMatch(runtime, /replace\("\\\\\\"",\s*"\\""\)/);
  assert.match(runtime, /extracts_multiline_stt_transcript_after_status_lines/);
  assert.match(runtime, /rejects_status_lines_without_transcript_marker/);
  assert.match(runtime, /rejects_empty_transcript_marker/);
  assert.match(runtime, /\\n/);
  assert.match(runtime, /ready/);
  assert.doesNotMatch(runtime, /contains\(" ready"\)/);
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
  assert.match(speakText, /cancellation: &FlowSpeechCancellation/);
  assert.match(speakText, /Friday Kokoro TTS runtime is not available/);
  assert.match(ensureSttReady, /Flow STT runtime is not built/);
  assert.match(ensureSttReady, /DX_FLOW_DICTATE_BINARY/);
  assert.match(ensureSttReady, /FLOW_DEFAULT_STT_MODEL_KEY/);
  assert.match(ensureSttReady, /FLOW_PARAKEET_EXECUTION_MODEL_KEY/);
  assert.match(ensureSttReady, /FLOW_NEMOTRON_EXECUTION_MODEL_KEY/);
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
  assert.match(synthesize, /fs::remove_file\(&output_path\)/);
  assert.match(runtime, /STT_COMMAND_TIMEOUT/);
  assert.match(runtime, /TTS_COMMAND_TIMEOUT/);
  assert.match(timeoutHelper, /stdin\(Stdio::null\(\)\)/);
  assert.match(timeoutHelper, /stdout\(Stdio::piped\(\)\)/);
  assert.match(timeoutHelper, /stderr\(Stdio::piped\(\)\)/);
  assert.match(timeoutHelper, /try_wait\(\)/);
  assert.match(timeoutHelper, /cancellation\.is_cancelled\(\)/);
  assert.match(timeoutHelper, /child\.kill\(\)/);
  assert.match(timeoutHelper, /child\.wait\(\)/);
  assert.match(timeoutHelper, /was canceled/);
  assert.match(timeoutHelper, /timed out after/);
  assert.match(runtime, /HUGGINGFACE_HUB_CACHE/);
  assert.match(runtime, /hf_home\.join\("hub"\)/);
  assert.match(runtime, /fn normalize_flow_data_root/);
  assert.match(runtime, /fn canonicalize_candidate_path/);
  assert.match(runtime, /fn candidate_paths_equal/);
  assert.match(runtime, /fn flow_root_from_dictate_binary/);
  assert.match(dataRootCandidates, /normalize_flow_data_root/);
  assert.match(dataRootCandidates, /FLOW_DATA_DIR/);
  assert.match(dataRootCandidates, /DX_SCAN_FLOW_DRIVES/);
  assert.match(dataRootCandidates, /path\.join\("data"\)/);
  assert.match(dataRootCandidates, /push_unique_path/);
  assert.match(defaultFlowRoot, /flow_root_ready/);
  assert.match(defaultFlowRoot, /join\("src"\)[\s\S]+join\("bin"\)[\s\S]+join\("flow-dictate\.rs"\)/);
  assert.doesNotMatch(defaultFlowRoot, /PARAKEET_MODEL_DIR/);
  assert.match(runtime, /RecordingTelemetry/);
  assert.match(runtime, /pub\(crate\) const MAX_RECORDING_SECONDS: usize = 90/);
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

test("voice runtime status summary reports precise readiness blockers", () => {
  const runtime = readFileSync(flowRuntimePath, "utf8");
  const statusSummary = sourceSlice(
    runtime,
    "pub(crate) fn status_summary",
    "pub(crate) fn stt_available",
  );
  const missingSttModelMessage = sourceSlice(
    runtime,
    "fn missing_stt_model_message",
    "fn append_stt_backend_args",
  );

  assert.match(statusSummary, /Friday Kokoro ready/);
  assert.match(statusSummary, /Friday Kokoro missing/);
  assert.match(statusSummary, /Flow STT command ready/);
  assert.match(statusSummary, /Flow STT command missing/);
  assert.match(missingSttModelMessage, /Flow \{\} model files are missing or empty under/);
  assert.match(missingSttModelMessage, /Flow \{\} requires a non-empty GGML model from FLOW_WHISPER_MODEL, DX_FLOW_WHISPER_MODEL/);
  assert.match(missingSttModelMessage, /plus a whisper\.cpp binary from FLOW_WHISPER_CPP_BINARY, DX_WHISPER_CPP_BINARY, FLOW_WHISPER_CPP_EXE, or FLOW_WHISPER_CPP/);
  assert.match(missingSttModelMessage, /Flow \{\} requires a whisper\.cpp binary from FLOW_WHISPER_CPP_BINARY, DX_WHISPER_CPP_BINARY, FLOW_WHISPER_CPP_EXE, or FLOW_WHISPER_CPP/);
});

test("Flow dictation host exposes focused STT model selection", () => {
  assert.match(flowDictate, /const DEFAULT_STT_MODEL_KEY: &str = "parakeet-tdt-0\.6b-v3-int8"/);
  assert.match(
    flowDictate,
    /const NEMOTRON_STT_MODEL_KEY: &str = "nemotron-speech-streaming-en-0\.6b-int8"/,
  );
  assert.match(
    flowDictate,
    /const WHISPER_STT_MODEL_KEY: &str = "whisper-tiny-ggml"/,
  );
  assert.match(
    flowDictate,
    /const WHISPER_STT_MODEL_FILE: &str = "models\/stt\/ggml-tiny\.bin"/,
  );
  assert.match(flowDictate, /struct DictationSttModel/);
  assert.match(flowDictate, /enum DictationSttRuntime/);
  assert.match(flowDictate, /SherpaTransducer\s*\{\s*root: &'static str\s*\}/);
  assert.match(flowDictate, /WhisperCpp\s*\{\s*model_file: &'static str\s*\}/);
  assert.match(flowDictate, /fn selected_stt_model/);
  assert.match(flowDictate, /fn option_value/);
  assert.match(flowDictate, /--model/);
  assert.match(flowDictate, /--whisper-bin/);
  assert.match(flowDictate, /--whisper-cpp/);
  assert.match(flowDictate, /--whisper-model/);
  assert.match(flowDictate, /--whisper-language/);
  assert.match(flowDictate, /models\/stt\/parakeet-tdt-0\.6b-v3-int8/);
  assert.match(flowDictate, /models\/stt\/nemotron-speech-streaming-en-0\.6b-int8/);
  assert.match(flowDictate, /models\/stt\/ggml-tiny\.bin/);
  assert.match(flowDictate, /Unsupported STT model/);
  assert.match(flowDictate, /supported_stt_models/);
  assert.match(flowDictate, /load_sherpa_transducer/);
  assert.match(flowDictate, /load_stt_backend/);
  assert.match(flowDictate, /transcribe_with_whisper_cpp/);
  assert.match(flowDictate, /resolve_whisper_cpp_binary/);
  assert.match(flowDictate, /FLOW_WHISPER_CPP_BINARY/);
  assert.match(flowDictate, /DX_WHISPER_CPP_BINARY/);
  assert.match(flowDictate, /FLOW_WHISPER_CPP_EXE/);
  assert.match(flowDictate, /FLOW_WHISPER_CPP/);
  assert.match(flowDictate, /FLOW_WHISPER_MODEL/);
  assert.match(flowDictate, /DX_FLOW_WHISPER_MODEL/);
  assert.match(flowDictate, /FLOW_WHISPER_LANGUAGE/);
  assert.match(flowDictate, /DX_FLOW_WHISPER_LANGUAGE/);
  assert.match(flowDictate, /DictationSttRuntime::WhisperCpp/);
  assert.doesNotMatch(flowDictate, /let mut recognizer = load_sherpa_transducer\(selected_model\)\?/);
  assert.match(flowDictate, /selected_model\.label/);
  assert.match(flowDictationHostReadme, /--model parakeet-tdt-0\.6b-v3-int8/);
  assert.match(flowDictationHostReadme, /--model nemotron-speech-streaming-en-0\.6b-int8/);
  assert.match(flowDictationHostReadme, new RegExp(`--model ${flowWhisperModelKey}`));
  assert.match(flowDictationHostReadme, /focused host supports Sherpa Parakeet, Sherpa Nemotron, and whisper\.cpp Whisper/);
  assert.match(flowDictationHostReadme, /--whisper-cpp/);
  assert.match(flowDictationHostReadme, /DX_WHISPER_CPP_BINARY/);
  assert.match(flowDictationHostReadme, /FLOW_WHISPER_CPP_EXE/);
  assert.match(flowDictationHostReadme, /FLOW_WHISPER_CPP/);
  assert.match(flowDictationHostReadme, /FLOW_WHISPER_CPP_BINARY/);
  assert.match(flowDictationHostReadme, /DX_FLOW_WHISPER_MODEL/);
  assert.match(flowDictationHostReadme, /FLOW_WHISPER_MODEL/);
  assert.match(flowDictationHostReadme, /--whisper-language/);
  assert.match(flowDictationHostReadme, /FLOW_WHISPER_LANGUAGE/);
  assert.match(flowDictationHostReadme, /DX_FLOW_WHISPER_LANGUAGE/);
});

test("voice text paths use the real message editor contents and insert APIs", () => {
  const interruptAndSend = sourceSlice(
    threadView,
    "pub fn interrupt_and_send",
    "fn stop_current_and_send_new_message",
  );
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
  assert.match(threadView, /if self\.composer_voice_state\.is_busy\(\)/);
  assert.match(threadView, /Finish Flow voice action before sending/);
  assert.match(interruptAndSend, /if self\.composer_voice_state\.is_busy\(\)/);
  assert.match(interruptAndSend, /Finish Flow voice action before sending/);
  assert.match(
    threadView,
    /this\.message_editor\.update\(cx,\s*\|editor, cx\|[\s\S]+insert_transcript_text\(&transcript/,
    "Flow STT transcripts from the composer mic must insert into the composer input",
  );
  assert.doesNotMatch(
    threadView,
    /let active_editor = this\.active_editor\(cx\)[\s\S]+insert_transcript_text\(&transcript/,
    "Composer mic transcripts must not follow an edited historical message",
  );
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
  assert.match(
    speakComposerText,
    /ComposerVoicePhase::Speaking => \{\s*self\.stop_flow_voice_playback\(cx\);\s*return;\s*\}/,
  );
  assert.match(
    speakComposerText,
    /ComposerVoicePhase::Recording \| ComposerVoicePhase::Transcribing/,
  );
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
    "- Verification:",
  );

  assert.match(voiceHandoff, /flow-dictate\.exe` now exists/);
  assert.match(voiceHandoff, /flow-dictate --file <wav> --model <key>/);
  assert.match(voiceHandoff, /Parakeet as the default/);
  assert.match(voiceHandoff, /Nemotron as an explicit opt-in/);
  assert.match(voiceHandoff, /Whisper Tiny GGML as an explicit opt-in/);
  assert.match(voiceHandoff, /DX_FLOW_STT_MODEL/);
  assert.match(voiceHandoff, /FLOW_STT_MODEL/);
  assert.match(voiceHandoff, /unsupported values fail closed/);
  assert.match(voiceHandoff, /Keyless Whisper source exists/);
  assert.match(voiceHandoff, /keyless-whisper/);
  assert.match(voiceHandoff, /focused `flow-dictate` host now exposes `whisper-tiny-ggml` through a whisper\.cpp subprocess boundary/);
  assert.match(voiceHandoff, /FLOW_WHISPER_CPP_BINARY/);
  assert.match(voiceHandoff, /FLOW_WHISPER_MODEL/);
  assert.match(voiceHandoff, /DX_FLOW_WHISPER_MODEL/);
  assert.match(voiceHandoff, /silent-WAV Parakeet smoke test passed/);
  assert.match(voiceHandoff, /Nemotron smoke proof/);
  assert.match(voiceHandoff, /Whisper smoke proof/);
  assert.match(voiceHandoff, /live Zed microphone proof still needs? the governed validation window/);
  assert.match(voiceHandoff, /G:\\Flow\\data\\models\\tts\\kokoro_82m/);
  assert.match(voiceHandoff, /config\.json/);
  assert.match(voiceHandoff, /kokoro-v1_0\.pth/);
  assert.match(voiceHandoff, /live Kokoro synthesis\/playback proof still remains deferred/);
  assert.match(voiceHandoff, /tracked\/cancelable WAV playback handle/);
  assert.match(voiceHandoff, /Live audible playback proof is still deferred/);
  assert.match(voiceHandoff, /read-aloud toolbar button exposes the visible Kokoro stop path/);
  assert.match(voiceHandoff, /inline panel stays status-only while Kokoro is speaking/);
  assert.match(voiceHandoff, /toolbar mic stays disabled as Mic while Kokoro is speaking/);
  assert.doesNotMatch(
    voiceHandoff,
    /runtime-green|production-ready|launch-ready|Nemotron live proof passed|Whisper live proof passed|Whisper smoke test passed|Kokoro live proof passed|live audible playback proof passed|live Zed microphone proof passed/i,
  );
});

function sourceSlice(source: string, startNeedle: string, endNeedle: string) {
  const start = source.indexOf(startNeedle);
  assert.notEqual(start, -1, `expected ${startNeedle}`);
  const end = source.indexOf(endNeedle, start + startNeedle.length);
  assert.notEqual(end, -1, `expected ${endNeedle} after ${startNeedle}`);
  return source.slice(start, end);
}

function sourceTail(source: string, startNeedle: string) {
  const start = source.indexOf(startNeedle);
  assert.notEqual(start, -1, `expected ${startNeedle}`);
  return source.slice(start);
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
