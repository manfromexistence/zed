use crate::dx_project_context::DxProjectContext;
use anyhow::{Context as _, Result, anyhow};
use cpal::{
    DeviceId, FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use uuid::Uuid;

const TARGET_SAMPLE_RATE: u32 = 16_000;
pub(crate) const MAX_RECORDING_SECONDS: usize = 90;
const MIN_RECORDING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize / 4;
const FLOW_DEFAULT_STT_MODEL_KEY: &str = "parakeet-tdt-0.6b-v3-int8";
const FLOW_PARAKEET_EXECUTION_MODEL_KEY: &str = "parakeet-tdt-0.6b-v3-int8";
const FLOW_NEMOTRON_EXECUTION_MODEL_KEY: &str = "nemotron-speech-streaming-en-0.6b-int8";
const FLOW_WHISPER_EXECUTION_MODEL_KEY: &str = "whisper-tiny-ggml";
const PARAKEET_MODEL_DIR: &str = "models/stt/parakeet-tdt-0.6b-v3-int8";
const NEMOTRON_MODEL_DIR: &str = "models/stt/nemotron-speech-streaming-en-0.6b-int8";
const WHISPER_MODEL_FILE: &str = "models/stt/ggml-tiny.bin";
const KOKORO_MODEL_KEY: &str = "kokoro_82m";
const KOKORO_RUNNER_SCRIPT: &str = "tools/qwen3_tts_runner.py";
const DEFAULT_KOKORO_VOICE: &str = "af_bella";
const STT_COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const TTS_COMMAND_TIMEOUT: Duration = Duration::from_secs(180);
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(50);
const DX_FLOW_INPUT_DEVICE_ENV: &str = "DX_FLOW_INPUT_DEVICE";
const FLOW_INPUT_DEVICE_ENV: &str = "FLOW_INPUT_DEVICE";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;

#[derive(Clone, Debug)]
pub(crate) struct FlowSpeechRuntime {
    flow_root: PathBuf,
    flow_dictate_binary: Option<PathBuf>,
    selected_stt_model: Result<FlowSttModel, String>,
    kokoro_tts_runtime: Result<KokoroTtsRuntime, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct FlowSpeechReadinessSnapshot {
    pub(crate) flow_root: String,
    pub(crate) dictate_binary: Option<String>,
    pub(crate) stt_model: String,
    pub(crate) stt_ready: bool,
    pub(crate) stt_detail: String,
    pub(crate) kokoro_ready: bool,
    pub(crate) kokoro_detail: String,
    pub(crate) input_device_detail: String,
}

#[derive(Clone, Copy, Debug)]
struct FlowSttModel {
    key: &'static str,
    label: &'static str,
    artifact_shape: FlowSttArtifactShape,
}

#[derive(Clone, Copy, Debug)]
enum FlowSttArtifactShape {
    SherpaTransducer { model_dir: &'static str },
    WhisperCpp { model_file: &'static str },
}

#[derive(Clone)]
struct KokoroTtsRuntime {
    data_root: PathBuf,
    python: PathBuf,
    runner: PathBuf,
    model_dir: PathBuf,
    server: Arc<Mutex<Option<KokoroTtsServer>>>,
}

struct KokoroTtsServer {
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Receiver<String>,
    process_tree: FlowSpeechProcessTreeGuard,
}

impl std::fmt::Debug for KokoroTtsRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KokoroTtsRuntime")
            .field("data_root", &self.data_root)
            .field("python", &self.python)
            .field("runner", &self.runner)
            .field("model_dir", &self.model_dir)
            .finish_non_exhaustive()
    }
}

impl Drop for KokoroTtsServer {
    fn drop(&mut self) {
        self.terminate();
    }
}

impl KokoroTtsServer {
    fn terminate(&mut self) {
        self.process_tree.terminate();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
pub(crate) struct RecordedSpeech {
    samples: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FlowSpeechCancellation {
    canceled: Arc<AtomicBool>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RecordingTelemetry {
    captured_duration: Duration,
    input_level: f32,
}

pub(crate) struct FlowRecordingSession {
    runtime: FlowSpeechRuntime,
    samples: Arc<Mutex<Vec<f32>>>,
    telemetry: Arc<RecordingTelemetryState>,
    _stream: cpal::Stream,
    started_at: Instant,
    input_device_name: Option<String>,
}

#[derive(Debug, Default)]
struct RecordingTelemetryState {
    sample_count: AtomicUsize,
    input_level_bits: AtomicU32,
}

struct InputDeviceSelection {
    device: cpal::Device,
    name: Option<String>,
}

struct TemporarySpeechFile {
    path: PathBuf,
}

impl RecordingTelemetry {
    pub(crate) fn captured_duration(&self) -> Duration {
        self.captured_duration
    }

    pub(crate) fn input_level(&self) -> f32 {
        self.input_level
    }
}

impl RecordingTelemetryState {
    fn update(&self, sample_count: usize, input_level: f32) {
        self.sample_count.store(sample_count, Ordering::Relaxed);
        self.input_level_bits
            .store(input_level.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> RecordingTelemetry {
        let sample_count = self.sample_count.load(Ordering::Relaxed);
        let input_level = f32::from_bits(self.input_level_bits.load(Ordering::Relaxed));
        RecordingTelemetry {
            captured_duration: Duration::from_secs_f32(
                sample_count as f32 / TARGET_SAMPLE_RATE as f32,
            ),
            input_level,
        }
    }
}

impl TemporarySpeechFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl FlowSpeechCancellation {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn cancel(&self) {
        self.canceled.store(true, Ordering::Release);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.canceled.load(Ordering::Acquire)
    }
}

impl Drop for TemporarySpeechFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl FlowSttModel {
    fn parakeet() -> Self {
        Self {
            key: FLOW_PARAKEET_EXECUTION_MODEL_KEY,
            label: "Parakeet",
            artifact_shape: FlowSttArtifactShape::SherpaTransducer {
                model_dir: PARAKEET_MODEL_DIR,
            },
        }
    }

    fn nemotron() -> Self {
        Self {
            key: FLOW_NEMOTRON_EXECUTION_MODEL_KEY,
            label: "Nemotron",
            artifact_shape: FlowSttArtifactShape::SherpaTransducer {
                model_dir: NEMOTRON_MODEL_DIR,
            },
        }
    }

    fn whisper() -> Self {
        Self {
            key: FLOW_WHISPER_EXECUTION_MODEL_KEY,
            label: "Whisper Tiny GGML",
            artifact_shape: FlowSttArtifactShape::WhisperCpp {
                model_file: WHISPER_MODEL_FILE,
            },
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            FLOW_PARAKEET_EXECUTION_MODEL_KEY => Some(Self::parakeet()),
            FLOW_NEMOTRON_EXECUTION_MODEL_KEY => Some(Self::nemotron()),
            FLOW_WHISPER_EXECUTION_MODEL_KEY => Some(Self::whisper()),
            _ => None,
        }
    }
}

fn selected_stt_model() -> Result<FlowSttModel, String> {
    let requested = env::var("DX_FLOW_STT_MODEL")
        .or_else(|_| env::var("FLOW_STT_MODEL"))
        .unwrap_or_else(|_| FLOW_DEFAULT_STT_MODEL_KEY.to_string());
    let requested = requested.trim();
    let requested = if requested.is_empty() {
        FLOW_DEFAULT_STT_MODEL_KEY
    } else {
        requested
    };

    FlowSttModel::from_key(requested).ok_or_else(|| {
        format!(
            "Unsupported Flow STT model '{}'. Supported: {}, {}, {}.",
            requested,
            FLOW_PARAKEET_EXECUTION_MODEL_KEY,
            FLOW_NEMOTRON_EXECUTION_MODEL_KEY,
            FLOW_WHISPER_EXECUTION_MODEL_KEY
        )
    })
}

impl FlowSpeechRuntime {
    pub(crate) fn detect() -> Self {
        let flow_dictate_binary = env::var_os("DX_FLOW_DICTATE_BINARY")
            .map(PathBuf::from)
            .filter(|path| file_is_nonempty(path));
        let env_flow_root = env_flow_root();
        let binary_flow_root = flow_dictate_binary
            .as_ref()
            .and_then(|path| flow_root_from_dictate_binary(path));
        let flow_root =
            resolve_flow_root(env_flow_root, binary_flow_root).unwrap_or_else(default_flow_root);

        let flow_dictate_binary =
            flow_dictate_binary.or_else(|| find_binary(&flow_root, "flow-dictate"));
        let selected_stt_model = selected_stt_model();
        let kokoro_tts_runtime = KokoroTtsRuntime::detect(&flow_root);

        Self {
            flow_root,
            flow_dictate_binary,
            selected_stt_model,
            kokoro_tts_runtime,
        }
    }

    pub(crate) fn readiness_snapshot(&self) -> FlowSpeechReadinessSnapshot {
        FlowSpeechReadinessSnapshot {
            flow_root: self.flow_root.display().to_string(),
            dictate_binary: self
                .flow_dictate_binary
                .as_ref()
                .map(|path| path.display().to_string()),
            stt_model: self.stt_model_label(),
            stt_ready: self.stt_available(),
            stt_detail: self.stt_readiness_summary(),
            kokoro_ready: self.tts_available(),
            kokoro_detail: self.tts_readiness_summary().to_string(),
            input_device_detail: input_device_readiness_detail(),
        }
    }

    pub(crate) fn start_recording(
        &self,
        input_device_id: Option<&DeviceId>,
    ) -> Result<FlowRecordingSession> {
        self.ensure_stt_ready()?;

        let selection = resolve_input_device(input_device_id)?;
        let config = selection.device.default_input_config()?;
        let channels = config.channels() as usize;
        let input_sample_rate = config.sample_rate();
        let samples = Arc::new(Mutex::new(Vec::new()));
        let telemetry = Arc::new(RecordingTelemetryState::default());
        let stream = build_input_stream(
            &selection.device,
            &config,
            channels,
            input_sample_rate,
            Arc::clone(&samples),
            Arc::clone(&telemetry),
        )?;
        stream.play()?;

        Ok(FlowRecordingSession {
            runtime: self.clone(),
            samples,
            telemetry,
            _stream: stream,
            started_at: Instant::now(),
            input_device_name: selection.name,
        })
    }

    pub(crate) fn transcribe_recording(
        &self,
        recording: RecordedSpeech,
        cancellation: &FlowSpeechCancellation,
    ) -> Result<String> {
        let stt_model = self.ensure_stt_ready()?;
        let audio_file = TemporarySpeechFile::new(self.write_recording_wav(&recording)?);
        let binary = self
            .flow_dictate_binary
            .as_ref()
            .context("Flow STT dictation command is not available")?;
        let mut command = Command::new(binary);
        command
            .current_dir(&self.flow_root)
            .arg("--file")
            .arg(audio_file.path())
            .arg("--model")
            .arg(stt_model.key);
        self.append_stt_backend_args(&mut command, stt_model)?;
        apply_windows_process_flags(&mut command);
        let output = run_command_with_timeout(
            command,
            STT_COMMAND_TIMEOUT,
            "Flow STT transcription",
            Some(cancellation),
        )?;

        parse_transcript_output(output)
    }

    pub(crate) fn speak_text(
        &self,
        text: &str,
        cancellation: &FlowSpeechCancellation,
    ) -> Result<PathBuf> {
        let tts_runtime = self.tts_runtime()?;
        tts_runtime.synthesize(text, cancellation)
    }

    pub(crate) fn warm_tts_server(&self) -> Result<()> {
        self.tts_runtime()?.warm_server()
    }

    pub(crate) fn stt_readiness_summary(&self) -> String {
        match self.ensure_stt_ready() {
            Ok(stt_model) => format!("{} ready", stt_model.label),
            Err(error) => error.to_string(),
        }
    }

    fn stt_model_label(&self) -> String {
        self.selected_stt_model
            .as_ref()
            .map(|model| model.label.to_string())
            .unwrap_or_else(|error| error.clone())
    }

    pub(crate) fn tts_readiness_summary(&self) -> &str {
        match &self.kokoro_tts_runtime {
            Ok(_) => "Friday Kokoro ready",
            Err(message) => message,
        }
    }

    pub(crate) fn stt_available(&self) -> bool {
        self.ensure_stt_ready().is_ok()
    }

    pub(crate) fn tts_available(&self) -> bool {
        self.ensure_tts_ready().is_ok()
    }

    pub(crate) fn ensure_tts_ready(&self) -> Result<()> {
        self.tts_runtime().map(|_| ())
    }

    fn tts_runtime(&self) -> Result<&KokoroTtsRuntime> {
        self.kokoro_tts_runtime
            .as_ref()
            .map_err(|message| anyhow!("{}", message))
    }

    fn write_recording_wav(&self, recording: &RecordedSpeech) -> Result<PathBuf> {
        let tmp_dir = env::temp_dir().join("zed-flow-stt");
        fs::create_dir_all(&tmp_dir)?;
        let path = tmp_dir.join(format!(
            "zed-composer-recording-{}.wav",
            Uuid::new_v4().as_simple()
        ));
        write_wav_i16(&path, TARGET_SAMPLE_RATE, &recording.samples)?;
        Ok(path)
    }

    fn ensure_stt_model_ready(&self) -> Result<FlowSttModel> {
        let stt_model = self.stt_model()?;
        if self.stt_model_ready(stt_model) {
            Ok(stt_model)
        } else {
            Err(anyhow!("{}", self.missing_stt_model_message(stt_model)))
        }
    }

    fn ensure_stt_ready(&self) -> Result<FlowSttModel> {
        let stt_model = self.ensure_stt_model_ready()?;
        if self.flow_dictate_binary.is_some() {
            Ok(stt_model)
        } else {
            Err(anyhow!(
                "Flow STT runtime is not built. Build the focused flow-dictate host in {} or set DX_FLOW_DICTATE_BINARY. This maps Flow's default STT model {} to {} and supports {} and {} when DX_FLOW_STT_MODEL or FLOW_STT_MODEL selects them.",
                self.flow_root.display(),
                FLOW_DEFAULT_STT_MODEL_KEY,
                FLOW_PARAKEET_EXECUTION_MODEL_KEY,
                FLOW_NEMOTRON_EXECUTION_MODEL_KEY,
                FLOW_WHISPER_EXECUTION_MODEL_KEY
            ))
        }
    }

    #[allow(dead_code)]
    fn parakeet_ready(&self) -> bool {
        self.stt_model_ready(FlowSttModel::parakeet())
    }

    fn stt_model(&self) -> Result<FlowSttModel> {
        self.selected_stt_model
            .as_ref()
            .copied()
            .map_err(|error| anyhow!(error.clone()))
    }

    fn stt_model_ready(&self, stt_model: FlowSttModel) -> bool {
        match stt_model.artifact_shape {
            FlowSttArtifactShape::SherpaTransducer { model_dir } => {
                let root = self.flow_root.join(model_dir);
                [
                    "encoder.int8.onnx",
                    "decoder.int8.onnx",
                    "joiner.int8.onnx",
                    "tokens.txt",
                ]
                .iter()
                .all(|file| file_is_nonempty(&root.join(file)))
            }
            FlowSttArtifactShape::WhisperCpp { model_file } => {
                self.find_whisper_model_file(model_file).is_some()
                    && self.find_whisper_cpp_binary().is_some()
            }
        }
    }

    fn missing_stt_model_message(&self, stt_model: FlowSttModel) -> String {
        match stt_model.artifact_shape {
            FlowSttArtifactShape::SherpaTransducer { model_dir } => format!(
                "Flow {} model files are missing or empty under {}",
                stt_model.label,
                self.flow_root.join(model_dir).display()
            ),
            FlowSttArtifactShape::WhisperCpp { model_file } => {
                let model_ready = self.find_whisper_model_file(model_file).is_some();
                let binary_ready = self.find_whisper_cpp_binary().is_some();
                match (model_ready, binary_ready) {
                    (false, false) => format!(
                        "Flow {} requires a non-empty GGML model from FLOW_WHISPER_MODEL, DX_FLOW_WHISPER_MODEL, or {}, plus a whisper.cpp binary from FLOW_WHISPER_CPP_BINARY, DX_WHISPER_CPP_BINARY, FLOW_WHISPER_CPP_EXE, or FLOW_WHISPER_CPP",
                        stt_model.label,
                        self.flow_root.join(model_file).display()
                    ),
                    (false, true) => format!(
                        "Flow {} requires a non-empty GGML model from FLOW_WHISPER_MODEL, DX_FLOW_WHISPER_MODEL, or {}",
                        stt_model.label,
                        self.flow_root.join(model_file).display()
                    ),
                    (true, false) => format!(
                        "Flow {} requires a whisper.cpp binary from FLOW_WHISPER_CPP_BINARY, DX_WHISPER_CPP_BINARY, FLOW_WHISPER_CPP_EXE, or FLOW_WHISPER_CPP",
                        stt_model.label
                    ),
                    (true, true) => format!("Flow {} is ready", stt_model.label),
                }
            }
        }
    }

    fn append_stt_backend_args(
        &self,
        command: &mut Command,
        stt_model: FlowSttModel,
    ) -> Result<()> {
        if let FlowSttArtifactShape::WhisperCpp { model_file } = stt_model.artifact_shape {
            let binary = self.find_whisper_cpp_binary().context(
                "Flow Whisper Tiny GGML whisper.cpp binary is missing during transcription",
            )?;
            let model = self
                .find_whisper_model_file(model_file)
                .context("Flow Whisper Tiny GGML model is missing during transcription")?;

            command
                .arg("--whisper-bin")
                .arg(binary)
                .arg("--whisper-model")
                .arg(model);
            if let Some(language) = requested_whisper_language() {
                command.arg("--whisper-language").arg(language);
            }
        }

        Ok(())
    }

    fn flow_host_env_file(&self, name: &str) -> Option<PathBuf> {
        let path = env::var_os(name).map(PathBuf::from)?;
        let path = if path.is_absolute() {
            path
        } else {
            self.flow_root.join(path)
        };
        file_is_nonempty(&path).then_some(path)
    }

    fn find_whisper_model_file(&self, model_file: &'static str) -> Option<PathBuf> {
        ["FLOW_WHISPER_MODEL", "DX_FLOW_WHISPER_MODEL"]
            .into_iter()
            .find_map(|name| self.flow_host_env_file(name))
            .or_else(|| {
                let path = self.flow_root.join(model_file);
                file_is_nonempty(&path).then_some(path)
            })
    }

    fn find_whisper_cpp_binary(&self) -> Option<PathBuf> {
        [
            "FLOW_WHISPER_CPP_BINARY",
            "DX_WHISPER_CPP_BINARY",
            "FLOW_WHISPER_CPP_EXE",
            "FLOW_WHISPER_CPP",
        ]
        .into_iter()
        .find_map(|name| self.flow_host_env_file(name))
        .or_else(|| {
            let executable = if cfg!(windows) {
                "whisper-cli.exe"
            } else {
                "whisper-cli"
            };
            [
                self.flow_root
                    .join("tools")
                    .join("whisper.cpp")
                    .join("build")
                    .join("bin")
                    .join("Release")
                    .join(executable),
                self.flow_root
                    .join("tools")
                    .join("whisper.cpp")
                    .join("build")
                    .join("bin")
                    .join(executable),
                self.flow_root
                    .join("runtime")
                    .join("whisper.cpp")
                    .join("build")
                    .join("bin")
                    .join("Release")
                    .join(executable),
            ]
            .into_iter()
            .find(|path| file_is_nonempty(path))
        })
    }
}

fn requested_whisper_language() -> Option<String> {
    env::var("FLOW_WHISPER_LANGUAGE")
        .or_else(|_| env::var("DX_FLOW_WHISPER_LANGUAGE"))
        .ok()
        .map(|language| language.trim().to_string())
        .filter(|language| !language.is_empty())
}

fn missing_tts_readiness_message(flow_root: &Path, blockers: &[String]) -> String {
    let mut message = format!(
        "Friday Kokoro missing: no ready data root was found for {}. Checked data roots from DX_FLOW_DATA_ROOT, FLOW_DATA_DIR, copied Flow data, same-drive Flow data, LOCALAPPDATA, and DX_SCAN_FLOW_DRIVES; Python from FLOW_TTS_PYTHON or DX_KOKORO_TTS_PYTHON; runner from FLOW_TTS_RUNNER or DX_KOKORO_TTS_RUNNER; model files from DX_KOKORO_MODEL_DIR or models/tts/{KOKORO_MODEL_KEY} including config.json, kokoro-v1_0.pth, voices/af_heart.pt, and voices/af_bella.pt.",
        flow_root.display()
    );
    if !blockers.is_empty() {
        message.push_str(" Blockers: ");
        message.push_str(&blockers.join("; "));
    }
    message
}

fn env_path_if_nonempty_file(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| file_is_nonempty(path))
}

fn expected_kokoro_python_path(data_root: &Path) -> PathBuf {
    let flow_home = data_root.parent().unwrap_or(data_root);
    flow_home
        .join("runtime")
        .join("kokoro-tts")
        .join(".venv")
        .join(if cfg!(target_os = "windows") {
            "Scripts/python.exe"
        } else {
            "bin/python"
        })
}

fn missing_kokoro_model_artifacts(model_dir: &Path) -> Vec<String> {
    kokoro_model_required_artifacts(model_dir)
        .into_iter()
        .filter(|(_, path)| !file_is_nonempty(path))
        .map(|(label, path)| format!("{label} at {}", path.display()))
        .collect()
}

fn kokoro_model_required_artifacts(model_dir: &Path) -> [(&'static str, PathBuf); 4] {
    [
        ("config.json", model_dir.join("config.json")),
        ("kokoro-v1_0.pth", model_dir.join("kokoro-v1_0.pth")),
        (
            "voices/af_heart.pt",
            model_dir.join("voices").join("af_heart.pt"),
        ),
        (
            "voices/af_bella.pt",
            model_dir.join("voices").join("af_bella.pt"),
        ),
    ]
}

impl KokoroTtsRuntime {
    fn detect(flow_root: &Path) -> Result<Self, String> {
        let mut blockers = Vec::new();
        for data_root in candidate_flow_data_roots(flow_root) {
            if !data_root.exists() {
                blockers.push(format!("{} is missing", data_root.display()));
                continue;
            }

            match Self::from_data_root(data_root.clone()) {
                Ok(runtime) => return Ok(runtime),
                Err(error) => blockers.push(format!("{}: {}", data_root.display(), error)),
            }
        }

        Err(missing_tts_readiness_message(flow_root, &blockers))
    }

    fn from_data_root(data_root: PathBuf) -> Result<Self, String> {
        let python = env_path_if_nonempty_file("FLOW_TTS_PYTHON")
            .or_else(|| env_path_if_nonempty_file("DX_KOKORO_TTS_PYTHON"))
            .or_else(|| find_kokoro_python(&data_root))
            .ok_or_else(|| {
                format!(
                    "Python is missing. Set FLOW_TTS_PYTHON or DX_KOKORO_TTS_PYTHON, or install {}",
                    expected_kokoro_python_path(&data_root).display()
                )
            })?;
        let runner = env_path_if_nonempty_file("FLOW_TTS_RUNNER")
            .or_else(|| env_path_if_nonempty_file("DX_KOKORO_TTS_RUNNER"))
            .or_else(|| {
                let path = data_root.join(KOKORO_RUNNER_SCRIPT);
                file_is_nonempty(&path).then_some(path)
            })
            .ok_or_else(|| {
                format!(
                    "runner is missing. Set FLOW_TTS_RUNNER or DX_KOKORO_TTS_RUNNER, or provide {}",
                    data_root.join(KOKORO_RUNNER_SCRIPT).display()
                )
            })?;
        let model_dir = env::var_os("DX_KOKORO_MODEL_DIR")
            .map(PathBuf::from)
            .filter(|path| kokoro_model_dir_ready(path))
            .or_else(|| find_kokoro_model_dir(&data_root))
            .ok_or_else(|| {
                let model_dir = data_root.join("models").join("tts").join(KOKORO_MODEL_KEY);
                let missing = missing_kokoro_model_artifacts(&model_dir).join(", ");
                format!(
                    "model directory is missing required Kokoro files from DX_KOKORO_MODEL_DIR or {}: {}",
                    model_dir.display(),
                    missing
                )
            })?;

        Ok(Self {
            data_root,
            python,
            runner,
            model_dir,
            server: Arc::new(Mutex::new(None)),
        })
    }

    fn synthesize(&self, text: &str, cancellation: &FlowSpeechCancellation) -> Result<PathBuf> {
        match self.synthesize_with_cached_server(text, cancellation) {
            Ok(output_path) => Ok(output_path),
            Err(error) if cancellation.is_cancelled() => Err(error),
            Err(error) => {
                log::debug!(
                    "Cached Friday Kokoro TTS server failed; falling back to one-shot synthesis: {error:#}"
                );
                self.clear_cached_server();
                self.synthesize_once(text, cancellation)
            }
        }
    }

    fn warm_server(&self) -> Result<()> {
        let mut server_guard = self
            .server
            .lock()
            .map_err(|_| anyhow!("Friday Kokoro TTS server lock was poisoned"))?;
        if server_guard.is_none() {
            *server_guard = Some(self.start_server()?);
        }
        Ok(())
    }

    fn synthesize_with_cached_server(
        &self,
        text: &str,
        cancellation: &FlowSpeechCancellation,
    ) -> Result<PathBuf> {
        let output_path = self.prepare_tts_output_path()?;
        let result = self.synthesize_to_output_with_cached_server(text, &output_path, cancellation);
        if let Err(error) = result {
            let _ = fs::remove_file(&output_path);
            return Err(error);
        }
        self.validate_tts_output(&output_path)?;
        Ok(output_path)
    }

    fn synthesize_to_output_with_cached_server(
        &self,
        text: &str,
        output_path: &Path,
        cancellation: &FlowSpeechCancellation,
    ) -> Result<()> {
        let mut server_guard = self
            .server
            .lock()
            .map_err(|_| anyhow!("Friday Kokoro TTS server lock was poisoned"))?;
        if server_guard.is_none() {
            *server_guard = Some(self.start_server()?);
        }
        let server = server_guard
            .as_mut()
            .context("Friday Kokoro TTS server is not available")?;
        let request = serde_json::json!({
            "text": text,
            "output": output_path.display().to_string(),
            "speaker": DEFAULT_KOKORO_VOICE,
        });
        server.write_request(&request)?;
        let response =
            server.read_response(TTS_COMMAND_TIMEOUT, Some(cancellation), "Friday Kokoro TTS")?;
        ensure_kokoro_server_success(&response, "Friday Kokoro TTS")?;
        Ok(())
    }

    fn synthesize_once(
        &self,
        text: &str,
        cancellation: &FlowSpeechCancellation,
    ) -> Result<PathBuf> {
        let output_path = self.prepare_tts_output_path()?;
        let mut command = self.kokoro_command();
        command
            .arg("--text")
            .arg(text)
            .arg("--output")
            .arg(&output_path);

        let output = match run_command_with_timeout(
            command,
            TTS_COMMAND_TIMEOUT,
            "Friday Kokoro TTS",
            Some(cancellation),
        ) {
            Ok(output) => output,
            Err(error) => {
                let _ = fs::remove_file(&output_path);
                return Err(error);
            }
        };
        if !output.status.success() {
            let _ = fs::remove_file(&output_path);
            return Err(command_error("Friday Kokoro TTS failed", output));
        }
        self.validate_tts_output(&output_path)?;
        Ok(output_path)
    }

    fn start_server(&self) -> Result<KokoroTtsServer> {
        self.prepare_tts_dirs()?;
        let mut command = self.kokoro_command();
        command
            .arg("--server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .context("Failed to start Friday Kokoro TTS server")?;
        let process_tree = match create_flow_speech_process_tree_guard(&child) {
            Ok(process_tree) => process_tree,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error)
                    .context("Failed to protect Friday Kokoro TTS server process tree");
            }
        };
        let stdin = child
            .stdin
            .take()
            .context("Friday Kokoro TTS server stdin is unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("Friday Kokoro TTS server stdout is unavailable")?;
        let stdout_lines = spawn_kokoro_server_stdout_reader(stdout);
        let mut server = KokoroTtsServer {
            child,
            stdin,
            stdout_lines,
            process_tree,
        };
        let response = server.read_response(
            TTS_COMMAND_TIMEOUT,
            None,
            "Friday Kokoro TTS server startup",
        )?;
        ensure_kokoro_server_success(&response, "Friday Kokoro TTS server startup")?;
        Ok(server)
    }

    fn clear_cached_server(&self) {
        if let Ok(mut server) = self.server.lock() {
            server.take();
        }
    }

    fn kokoro_command(&self) -> Command {
        let mut command = Command::new(&self.python);
        command
            .arg(&self.runner)
            .arg("--model-kind")
            .arg("kokoro")
            .arg("--model-dir")
            .arg(&self.model_dir)
            .arg("--language")
            .arg("English")
            .arg("--speaker")
            .arg(DEFAULT_KOKORO_VOICE)
            .arg("--device")
            .arg("cpu");
        apply_tts_process_env(&mut command, &self.data_root);
        apply_windows_process_flags(&mut command);
        command
    }

    fn prepare_tts_output_path(&self) -> Result<PathBuf> {
        self.prepare_tts_dirs()?;
        let output_dir = env::temp_dir().join("zed-kokoro-tts");
        fs::create_dir_all(&output_dir)?;
        Ok(output_dir.join(format!(
            "zed-composer-kokoro-{}.wav",
            Uuid::new_v4().as_simple()
        )))
    }

    fn prepare_tts_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.data_root.join("huggingface"))?;
        fs::create_dir_all(self.data_root.join("torch"))?;
        Ok(())
    }

    fn validate_tts_output(&self, output_path: &Path) -> Result<()> {
        if !output_path.exists() {
            return Err(anyhow!(
                "Friday Kokoro TTS finished without writing {}",
                output_path.display()
            ));
        }
        let audio_size = match fs::metadata(output_path) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                let _ = fs::remove_file(output_path);
                return Err(anyhow!(
                    "Could not inspect {}: {}",
                    output_path.display(),
                    error
                ));
            }
        };
        if audio_size <= 44 {
            let _ = fs::remove_file(output_path);
            return Err(anyhow!(
                "Friday Kokoro TTS wrote an empty WAV file at {}",
                output_path.display()
            ));
        }
        Ok(())
    }
}

impl KokoroTtsServer {
    fn write_request(&mut self, request: &serde_json::Value) -> Result<()> {
        serde_json::to_writer(&mut self.stdin, request)
            .context("Failed to write Friday Kokoro TTS server request")?;
        self.stdin
            .write_all(b"\n")
            .context("Failed to finish Friday Kokoro TTS server request")?;
        self.stdin
            .flush()
            .context("Failed to flush Friday Kokoro TTS server request")?;
        Ok(())
    }

    fn read_response(
        &mut self,
        timeout: Duration,
        cancellation: Option<&FlowSpeechCancellation>,
        label: &str,
    ) -> Result<serde_json::Value> {
        let started_at = Instant::now();
        loop {
            if let Some(cancellation) = cancellation
                && cancellation.is_cancelled()
            {
                self.terminate();
                return Err(anyhow!("{label} was canceled"));
            }

            if let Some(status) = self
                .child
                .try_wait()
                .with_context(|| format!("Failed to inspect {label} process"))?
            {
                return Err(anyhow!("{label} exited early with {status}"));
            }

            let elapsed = started_at.elapsed();
            if elapsed >= timeout {
                self.terminate();
                return Err(anyhow!("{label} timed out after {}s", timeout.as_secs()));
            }

            let remaining = timeout.saturating_sub(elapsed);
            let poll_interval = remaining.min(COMMAND_POLL_INTERVAL);
            match self.stdout_lines.recv_timeout(poll_interval) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    return serde_json::from_str(line)
                        .with_context(|| format!("Failed to parse {label} JSON response"));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(anyhow!("{label} stdout closed before a response"));
                }
            }
        }
    }
}

fn spawn_kokoro_server_stdout_reader(stdout: ChildStdout) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if sender.send(line).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(
                        serde_json::json!({
                            "ok": false,
                            "error": format!("stdout read failed: {error}"),
                        })
                        .to_string(),
                    );
                    break;
                }
            }
        }
    });
    receiver
}

fn ensure_kokoro_server_success(response: &serde_json::Value, label: &str) -> Result<()> {
    if response.get("ok").and_then(|value| value.as_bool()) == Some(true) {
        return Ok(());
    }

    let detail = response
        .get("error")
        .and_then(|value| value.as_str())
        .filter(|error| !error.trim().is_empty())
        .unwrap_or("unknown Kokoro server error");
    Err(anyhow!("{label} failed: {detail}"))
}

impl FlowRecordingSession {
    pub(crate) fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub(crate) fn input_device_label(&self) -> &str {
        self.input_device_name
            .as_deref()
            .unwrap_or("selected microphone")
    }

    pub(crate) fn telemetry(&self) -> Result<RecordingTelemetry> {
        Ok(self.telemetry.snapshot())
    }

    pub(crate) fn finish(self) -> Result<(FlowSpeechRuntime, RecordedSpeech)> {
        let Self {
            runtime,
            samples,
            telemetry: _,
            _stream,
            started_at: _,
            input_device_name: _,
        } = self;

        drop(_stream);
        let samples = samples
            .lock()
            .map_err(|_| anyhow!("Microphone recording buffer was poisoned"))?
            .clone();

        if samples.len() < MIN_RECORDING_SAMPLES {
            return Err(anyhow!("Recording was too short for Flow STT"));
        }

        Ok((runtime, RecordedSpeech { samples }))
    }
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    channels: usize,
    input_sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
    telemetry: Arc<RecordingTelemetryState>,
) -> Result<cpal::Stream> {
    let stream_config = config.clone().into();
    match config.sample_format() {
        SampleFormat::F32 => build_input_stream_typed::<f32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::F64 => build_input_stream_typed::<f64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::I8 => build_input_stream_typed::<i8>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::I16 => build_input_stream_typed::<i16>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::I24 => build_input_stream_typed::<cpal::I24>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::I32 => build_input_stream_typed::<i32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::I64 => build_input_stream_typed::<i64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::U8 => build_input_stream_typed::<u8>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::U16 => build_input_stream_typed::<u16>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::U32 => build_input_stream_typed::<u32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        SampleFormat::U64 => build_input_stream_typed::<u64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
            Arc::clone(&telemetry),
        ),
        other => Err(anyhow!("Unsupported microphone sample format: {other:?}")),
    }
}

fn resolve_input_device(input_device_id: Option<&DeviceId>) -> Result<InputDeviceSelection> {
    let host = cpal::default_host();
    if let Some(device_id) = input_device_id {
        if let Some(device) = host.device_by_id(device_id) {
            return Ok(InputDeviceSelection {
                name: input_device_name(&device),
                device,
            });
        }
        log::warn!("Selected Flow microphone device was not found; falling back to default input");
    }

    let devices = host
        .input_devices()
        .context("Failed to enumerate microphone input devices")?
        .collect::<Vec<_>>();
    if devices.is_empty() {
        return Err(anyhow!(
            "No microphone input devices were found. Check Windows microphone permission and input device settings."
        ));
    }

    if let Some(requested) = requested_input_device_name() {
        let requested_lower = requested.to_ascii_lowercase();
        if let Some(device) = devices.iter().find(|device| {
            input_device_name(device)
                .map(|name| name.to_ascii_lowercase().contains(&requested_lower))
                .unwrap_or(false)
        }) {
            return Ok(InputDeviceSelection {
                name: input_device_name(device),
                device: device.clone(),
            });
        }
        log::warn!(
            "Requested Flow input device '{requested}' did not match any microphone; using auto-selection"
        );
    }

    let default = host.default_input_device();
    let default_score = default
        .as_ref()
        .and_then(input_device_name)
        .map(|name| input_device_score(&name))
        .unwrap_or(i32::MIN);

    let best = devices
        .iter()
        .filter_map(|device| {
            let name = input_device_name(device)?;
            Some((input_device_score(&name), name, device))
        })
        .max_by_key(|(score, _, _)| *score);

    if let Some((best_score, name, device)) = best {
        if best_score > default_score || default.is_none() {
            log::info!(
                "selected microphone-like Flow input '{name}' over default input (score {best_score} > {default_score})"
            );
            return Ok(InputDeviceSelection {
                name: Some(name),
                device: device.clone(),
            });
        }
    }

    default
        .or_else(|| devices.first().cloned())
        .map(|device| InputDeviceSelection {
            name: input_device_name(&device),
            device,
        })
        .context("No usable microphone input device found")
}

fn requested_input_device_name() -> Option<String> {
    env::var(DX_FLOW_INPUT_DEVICE_ENV)
        .or_else(|_| env::var(FLOW_INPUT_DEVICE_ENV))
        .ok()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn input_device_readiness_detail() -> String {
    requested_input_device_name()
        .map(|name| format!("Configured input device: {name}"))
        .unwrap_or_else(|| "Input device not checked; live capture proof deferred".to_string())
}

fn input_device_name(device: &cpal::Device) -> Option<String> {
    device
        .description()
        .map(|description| description.name().to_string())
        .ok()
}

fn input_device_score(name: &str) -> i32 {
    let lower = name.to_ascii_lowercase();
    let mut score = 0;

    for keyword in ["microphone", "mic", "array", "headset", "realtek", "usb"] {
        if lower.contains(keyword) {
            score += 40;
        }
    }

    for keyword in [
        "stereo mix",
        "speaker",
        "output",
        "monitor",
        "loopback",
        "virtual",
        "cable",
        "voicemeeter",
        "what u hear",
    ] {
        if lower.contains(keyword) {
            score -= 120;
        }
    }

    score
}

fn build_input_stream_typed<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    input_sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
    telemetry: Arc<RecordingTelemetryState>,
) -> Result<cpal::Stream>
where
    T: Sample + SizedSample + Send + Copy + 'static,
    f32: FromSample<T>,
{
    let stream = device.build_input_stream(
        &config,
        move |data: &[T], _| {
            let processed = downmix_and_resample(data, channels, input_sample_rate);
            if let Ok(mut buffer) = samples.try_lock() {
                let limit = TARGET_SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;
                let remaining = limit.saturating_sub(buffer.len());
                let input_level = recent_input_level(&processed);
                buffer.extend(processed.into_iter().take(remaining));
                telemetry.update(buffer.len(), input_level);
            }
        },
        |error| log::warn!("Flow microphone input stream error: {error}"),
        None,
    )?;
    Ok(stream)
}

fn downmix_and_resample<T>(input: &[T], channels: usize, input_sample_rate: u32) -> Vec<f32>
where
    T: Sample + Copy,
    f32: FromSample<T>,
{
    let channels = channels.max(1);
    let frame_count = input.len() / channels;
    let mut mono = Vec::with_capacity(frame_count);
    for frame in input.chunks(channels) {
        let sum = frame
            .iter()
            .map(|sample| (*sample).to_sample::<f32>())
            .sum::<f32>();
        mono.push(sum / frame.len() as f32);
    }

    if input_sample_rate == TARGET_SAMPLE_RATE || mono.is_empty() {
        return mono;
    }

    let ratio = input_sample_rate as f32 / TARGET_SAMPLE_RATE as f32;
    let target_len = ((mono.len() as f32) / ratio).max(1.0) as usize;
    (0..target_len)
        .filter_map(|index| mono.get((index as f32 * ratio) as usize).copied())
        .collect()
}

fn recent_input_level(samples: &[f32]) -> f32 {
    let window = (TARGET_SAMPLE_RATE as usize / 5).max(1);
    let start = samples.len().saturating_sub(window);
    let recent = &samples[start..];
    if recent.is_empty() {
        return 0.0;
    }

    let mean_square = recent
        .iter()
        .map(|sample| {
            let value = sample.clamp(-1.0, 1.0);
            value * value
        })
        .sum::<f32>()
        / recent.len() as f32;
    (mean_square.sqrt() * 4.0).clamp(0.0, 1.0)
}

fn write_wav_i16(path: &Path, sample_rate: u32, samples: &[f32]) -> Result<()> {
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn parse_transcript_output(output: Output) -> Result<String> {
    if !output.status.success() {
        return Err(command_error("Flow STT transcription failed", output));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_stt_transcript(&stdout).with_context(|| "Flow STT finished without a transcript")
}

fn extract_stt_transcript(stdout: &str) -> Result<String> {
    let marker = "[stt] \"";
    let start = stdout
        .rfind(marker)
        .ok_or_else(|| anyhow!("Flow STT transcript marker is missing"))?
        + marker.len();
    let payload = &stdout[start..];
    let end = payload
        .rfind("\"\r\n")
        .or_else(|| payload.rfind("\"\n"))
        .or_else(|| payload.rfind('"'))
        .ok_or_else(|| anyhow!("Flow STT transcript marker is missing a closing quote"))?;
    let text = trim_stt_transcript_payload(&payload[..end]);

    if text.trim().is_empty() {
        Err(anyhow!("Flow STT transcript marker is empty"))
    } else {
        Ok(text)
    }
}

fn trim_stt_transcript_payload(payload: &str) -> String {
    payload.to_string()
}

#[cfg(test)]
mod tests {
    use super::extract_stt_transcript;

    #[test]
    fn extracts_multiline_stt_transcript_after_status_lines() {
        let stdout =
            "[stt] preloading Parakeet...\n[stt] Parakeet ready in 0.1s\n[stt] \"hello\nworld\"\n";

        assert_eq!(extract_stt_transcript(stdout).unwrap(), "hello\nworld");
    }

    #[test]
    fn preserves_raw_backslash_sequences_from_flow_output() {
        let stdout = r#"[stt] "C:\new\notes literal \n and \"quote\""
"#;

        assert_eq!(
            extract_stt_transcript(stdout).unwrap(),
            r#"C:\new\notes literal \n and \"quote\""#
        );
    }

    #[test]
    fn preserves_raw_edge_whitespace_from_flow_output() {
        let stdout = "[stt] \"  indented code  \"\n";

        assert_eq!(extract_stt_transcript(stdout).unwrap(), "  indented code  ");
    }

    #[test]
    fn rejects_status_lines_without_transcript_marker() {
        let stdout = "[stt] preloading Parakeet...\n[stt] Parakeet ready in 0.1s\n";

        let error = extract_stt_transcript(stdout).unwrap_err().to_string();

        assert!(error.contains("Flow STT transcript marker is missing"));
    }

    #[test]
    fn rejects_empty_transcript_marker() {
        let stdout = "[stt] Parakeet ready in 0.1s\n[stt] \"\"\n";

        let error = extract_stt_transcript(stdout).unwrap_err().to_string();

        assert!(error.contains("Flow STT transcript marker is empty"));
    }
}

fn command_error(label: &str, output: Output) -> anyhow::Error {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    anyhow!(
        "{label}: {}{}",
        stderr.trim(),
        if stdout.trim().is_empty() {
            String::new()
        } else {
            format!(" {}", stdout.trim())
        }
    )
}

fn run_command_with_timeout(
    mut command: Command,
    timeout: Duration,
    label: &str,
    cancellation: Option<&FlowSpeechCancellation>,
) -> Result<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to start {label}"))?;
    let process_tree = create_flow_speech_process_tree_guard(&child)
        .with_context(|| format!("Failed to protect {label} process tree"))?;
    let started_at = Instant::now();

    loop {
        if let Some(cancellation) = cancellation
            && cancellation.is_cancelled()
        {
            process_tree.terminate();
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!("{label} was canceled"));
        }

        if child.try_wait()?.is_some() {
            return child
                .wait_with_output()
                .with_context(|| format!("Failed to collect {label} output"));
        }

        if started_at.elapsed() >= timeout {
            process_tree.terminate();
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!("{label} timed out after {}s", timeout.as_secs()));
        }

        thread::sleep(COMMAND_POLL_INTERVAL);
    }
}

#[cfg(target_os = "windows")]
struct FlowSpeechProcessTreeGuard {
    job: windows::Win32::Foundation::HANDLE,
}

#[cfg(target_os = "windows")]
// SAFETY: Windows job object handles are process-owned kernel handles that may
// be closed or terminated from any thread. This guard has unique ownership of
// the handle and never exposes references to the raw handle.
unsafe impl Send for FlowSpeechProcessTreeGuard {}

#[cfg(not(target_os = "windows"))]
struct FlowSpeechProcessTreeGuard;

#[cfg(target_os = "windows")]
impl FlowSpeechProcessTreeGuard {
    fn terminate(&self) {
        use windows::Win32::System::JobObjects::TerminateJobObject;

        let _ = unsafe { TerminateJobObject(self.job, 1) };
    }
}

#[cfg(not(target_os = "windows"))]
impl FlowSpeechProcessTreeGuard {
    fn terminate(&self) {}
}

#[cfg(target_os = "windows")]
impl Drop for FlowSpeechProcessTreeGuard {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;

        let _ = unsafe { CloseHandle(self.job) };
    }
}

fn create_flow_speech_process_tree_guard(child: &Child) -> Result<FlowSpeechProcessTreeGuard> {
    create_platform_process_tree_guard(child)
}

#[cfg(target_os = "windows")]
fn create_platform_process_tree_guard(child: &Child) -> Result<FlowSpeechProcessTreeGuard> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{
        Foundation::HANDLE,
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        },
    };

    let job = unsafe { CreateJobObjectW(None, None)? };
    let guard = FlowSpeechProcessTreeGuard { job };
    let mut job_info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    job_info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

    unsafe {
        SetInformationJobObject(
            guard.job,
            JobObjectExtendedLimitInformation,
            &job_info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;

        AssignProcessToJobObject(guard.job, HANDLE(child.as_raw_handle() as _))?;
    }

    Ok(guard)
}

#[cfg(not(target_os = "windows"))]
fn create_platform_process_tree_guard(_child: &Child) -> Result<FlowSpeechProcessTreeGuard> {
    Ok(FlowSpeechProcessTreeGuard)
}

fn apply_tts_process_env(command: &mut Command, data_root: &Path) {
    let hf_home = data_root.join("huggingface");
    let torch_home = data_root.join("torch");
    let tts_threads = tts_thread_count();
    command
        .env("PYTHONUTF8", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("HF_HOME", &hf_home)
        .env("HUGGINGFACE_HUB_CACHE", hf_home.join("hub"))
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env("HF_HUB_OFFLINE", "1")
        .env("TRANSFORMERS_OFFLINE", "1")
        .env("TRANSFORMERS_CACHE", hf_home.join("transformers"))
        .env("TORCH_HOME", torch_home)
        .env("TOKENIZERS_PARALLELISM", "false")
        .env("OMP_NUM_THREADS", &tts_threads)
        .env("MKL_NUM_THREADS", &tts_threads)
        .env("NUMEXPR_NUM_THREADS", &tts_threads)
        .env("FLOW_TTS_TORCH_THREADS", &tts_threads);
}

fn tts_thread_count() -> String {
    [
        "DX_FLOW_TTS_TORCH_THREADS",
        "FLOW_TTS_TORCH_THREADS",
        "OMP_NUM_THREADS",
    ]
    .into_iter()
    .find_map(|name| {
        env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| {
                value
                    .parse::<usize>()
                    .is_ok_and(|count| (1..=8).contains(&count))
            })
    })
    .unwrap_or_else(|| "2".to_string())
}

fn apply_windows_process_flags(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
    }
}

fn candidate_flow_data_roots(flow_root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_unique_data_root(
        &mut roots,
        env::var_os("DX_FLOW_DATA_ROOT").map(PathBuf::from),
    );
    push_unique_data_root(&mut roots, env::var_os("FLOW_DATA_DIR").map(PathBuf::from));
    push_unique_data_root(&mut roots, Some(flow_root.join("data")));

    #[cfg(target_os = "windows")]
    {
        push_unique_data_root(&mut roots, same_drive_flow_data_root(flow_root));

        if let Some(local_app_data) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            push_unique_data_root(&mut roots, Some(local_app_data.join("com.flow.data")));
        }

        if env::var_os("DX_SCAN_FLOW_DRIVES").is_some() {
            for drive in b'D'..=b'Z' {
                let root = PathBuf::from(format!("{}:\\Flow", drive as char));
                if root.exists() || root.join("data").exists() {
                    push_unique_data_root(&mut roots, Some(root));
                }
            }
        }
    }

    roots
}

#[cfg(target_os = "windows")]
fn same_drive_flow_data_root(flow_root: &Path) -> Option<PathBuf> {
    flow_root
        .ancestors()
        .find(|path| path.parent().is_none())
        .map(|drive_root| drive_root.join("Flow"))
}

fn push_unique_data_root(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    push_unique_path(paths, path.map(normalize_flow_data_root));
}

fn normalize_flow_data_root(path: PathBuf) -> PathBuf {
    if path
        .file_name()
        .map(|name| name.to_string_lossy().eq_ignore_ascii_case("data"))
        .unwrap_or(false)
    {
        return path;
    }

    let nested_data = path.join("data");
    if nested_data.join(KOKORO_RUNNER_SCRIPT).is_file()
        || nested_data.join("models").join("tts").exists()
        || path
            .file_name()
            .map(|name| name.to_string_lossy().eq_ignore_ascii_case("Flow"))
            .unwrap_or(false)
    {
        nested_data
    } else {
        path
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    if let Some(path) = path.map(canonicalize_candidate_path)
        && !paths
            .iter()
            .any(|existing| candidate_paths_equal(existing, &path))
    {
        paths.push(path);
    }
}

fn canonicalize_candidate_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn candidate_paths_equal(left: &Path, right: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        left.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }

    #[cfg(not(target_os = "windows"))]
    {
        left == right
    }
}

fn find_kokoro_python(data_root: &Path) -> Option<PathBuf> {
    let python = expected_kokoro_python_path(data_root);
    file_is_nonempty(&python).then_some(python)
}

fn find_kokoro_model_dir(data_root: &Path) -> Option<PathBuf> {
    let installed_model = data_root.join("models").join("tts").join(KOKORO_MODEL_KEY);
    if kokoro_model_dir_ready(&installed_model) {
        return Some(installed_model);
    }

    let snapshots = data_root
        .join("huggingface")
        .join("hub")
        .join("models--hexgrad--Kokoro-82M")
        .join("snapshots");
    fs::read_dir(snapshots)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| kokoro_model_dir_ready(path))
}

fn kokoro_model_dir_ready(path: &Path) -> bool {
    kokoro_model_required_artifacts(path)
        .iter()
        .all(|(_, path)| file_is_nonempty(path))
}

fn file_is_nonempty(path: &Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn find_binary(flow_root: &Path, name: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    ["release", "debug"]
        .iter()
        .map(|profile| flow_root.join("target").join(profile).join(&exe))
        .filter(|path| file_is_nonempty(path))
        .max_by_key(|path| binary_modified_at(path))
}

fn binary_modified_at(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn flow_root_from_dictate_binary(binary: &Path) -> Option<PathBuf> {
    binary
        .ancestors()
        .find(|candidate| flow_root_ready(candidate))
        .map(Path::to_path_buf)
}

fn env_flow_root() -> Option<PathBuf> {
    [env::var_os("DX_FLOW_ROOT"), env::var_os("FLOW_ROOT")]
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .find(|path| flow_root_ready(path))
}

fn resolve_flow_root(env_root: Option<PathBuf>, binary_root: Option<PathBuf>) -> Option<PathBuf> {
    match (env_root, binary_root) {
        (Some(env_root), Some(binary_root)) if !candidate_paths_equal(&env_root, &binary_root) => {
            log::warn!(
                "Configured Flow root {} does not match DX_FLOW_DICTATE_BINARY root {}; using the Flow root inferred from DX_FLOW_DICTATE_BINARY",
                env_root.display(),
                binary_root.display()
            );
            Some(binary_root)
        }
        (Some(env_root), _) => Some(env_root),
        (_, Some(binary_root)) => Some(binary_root),
        (None, None) => None,
    }
}

fn default_flow_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shared_flow_root = DxProjectContext::shared_fallback_root().join("flow");
    let mut roots = Vec::new();

    push_unique_flow_root(
        &mut roots,
        manifest_dir
            .ancestors()
            .nth(3)
            .map(|dx_root| dx_root.join("flow")),
    );
    push_unique_flow_root(&mut roots, Some(shared_flow_root.clone()));

    roots.into_iter().next().unwrap_or(shared_flow_root)
}

fn push_unique_flow_root(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    if let Some(path) = path
        && flow_root_ready(&path)
    {
        push_unique_path(paths, Some(path));
    }
}

fn flow_root_ready(path: &Path) -> bool {
    path.join("src")
        .join("bin")
        .join("flow-dictate.rs")
        .is_file()
}
