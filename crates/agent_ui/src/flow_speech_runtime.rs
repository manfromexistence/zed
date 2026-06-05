use anyhow::{Context as _, Result, anyhow};
use cpal::{
    DeviceId, FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MAX_RECORDING_SECONDS: usize = 90;
const MIN_RECORDING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize / 4;
const FRIDAY_DEFAULT_STT_MODEL_KEY: &str = "parakeet_unified_en_int8";
const FLOW_PARAKEET_EXECUTION_MODEL_KEY: &str = "parakeet-tdt-0.6b-v3-int8";
const PARAKEET_MODEL_DIR: &str = "models/stt/parakeet-tdt-0.6b-v3-int8";
const KOKORO_MODEL_KEY: &str = "kokoro_82m";
const KOKORO_RUNNER_SCRIPT: &str = "tools/qwen3_tts_runner.py";
const DEFAULT_KOKORO_VOICE: &str = "af_bella";
const STT_COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const TTS_COMMAND_TIMEOUT: Duration = Duration::from_secs(180);
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;

#[derive(Clone, Debug)]
pub(crate) struct FlowSpeechRuntime {
    flow_root: PathBuf,
    flow_dictate_binary: Option<PathBuf>,
    kokoro_tts_runtime: Option<KokoroTtsRuntime>,
}

#[derive(Clone, Debug)]
struct KokoroTtsRuntime {
    data_root: PathBuf,
    python: PathBuf,
    runner: PathBuf,
    model_dir: PathBuf,
}

#[derive(Debug)]
pub(crate) struct RecordedSpeech {
    samples: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RecordingTelemetry {
    captured_duration: Duration,
    input_level: f32,
}

pub(crate) struct FlowRecordingSession {
    runtime: FlowSpeechRuntime,
    samples: Arc<Mutex<Vec<f32>>>,
    _stream: cpal::Stream,
    started_at: Instant,
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

impl TemporarySpeechFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporarySpeechFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl FlowSpeechRuntime {
    pub(crate) fn detect() -> Self {
        let flow_root = env::var_os("DX_FLOW_ROOT")
            .or_else(|| env::var_os("FLOW_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(default_flow_root);

        let flow_dictate_binary = env::var_os("DX_FLOW_DICTATE_BINARY")
            .map(PathBuf::from)
            .filter(|path| path.is_file())
            .or_else(|| find_binary(&flow_root, "flow-dictate"));
        let kokoro_tts_runtime = KokoroTtsRuntime::detect(&flow_root);

        Self {
            flow_root,
            flow_dictate_binary,
            kokoro_tts_runtime,
        }
    }

    pub(crate) fn start_recording(
        &self,
        input_device_id: Option<&DeviceId>,
    ) -> Result<FlowRecordingSession> {
        self.ensure_stt_ready()?;

        let device = resolve_input_device(input_device_id)?;
        let config = device.default_input_config()?;
        let channels = config.channels() as usize;
        let input_sample_rate = config.sample_rate().0;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let stream = build_input_stream(
            &device,
            &config,
            channels,
            input_sample_rate,
            Arc::clone(&samples),
        )?;
        stream.play()?;

        Ok(FlowRecordingSession {
            runtime: self.clone(),
            samples,
            _stream: stream,
            started_at: Instant::now(),
        })
    }

    pub(crate) fn transcribe_recording(&self, recording: RecordedSpeech) -> Result<String> {
        self.ensure_stt_ready()?;
        let audio_file = TemporarySpeechFile::new(self.write_recording_wav(&recording)?);
        let binary = self
            .flow_dictate_binary
            .as_ref()
            .context("Flow Parakeet dictation command is not available")?;
        let mut command = Command::new(binary);
        command
            .current_dir(&self.flow_root)
            .arg("--file")
            .arg(audio_file.path());
        apply_windows_process_flags(&mut command);
        let output =
            run_command_with_timeout(command, STT_COMMAND_TIMEOUT, "Flow Parakeet transcription")?;

        parse_transcript_output(output)
    }

    pub(crate) fn speak_text(&self, text: &str) -> Result<PathBuf> {
        self.kokoro_tts_runtime
            .as_ref()
            .context("Friday Kokoro TTS runtime is not available")?
            .synthesize(text)
    }

    pub(crate) fn status_summary(&self) -> String {
        let stt = if self.parakeet_ready() {
            "Parakeet ready"
        } else {
            "Parakeet model missing"
        };
        let tts = if self.kokoro_tts_runtime.is_some() {
            "Friday Kokoro ready"
        } else {
            "Friday Kokoro missing"
        };
        let stt_runtime = if self.flow_dictate_binary.is_some() {
            "Parakeet command ready"
        } else {
            "Parakeet command missing"
        };

        format!("{stt}; {tts}; {stt_runtime}")
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

    fn ensure_parakeet_ready(&self) -> Result<()> {
        if self.parakeet_ready() {
            Ok(())
        } else {
            Err(anyhow!(
                "Flow Parakeet model files are missing under {}",
                self.flow_root.join(PARAKEET_MODEL_DIR).display()
            ))
        }
    }

    fn ensure_stt_ready(&self) -> Result<()> {
        self.ensure_parakeet_ready()?;
        if self.flow_dictate_binary.is_some() {
            Ok(())
        } else {
            Err(anyhow!(
                "Flow Parakeet runtime is not built. Build flow-dictate with the sherpa-stt feature in {} or set DX_FLOW_DICTATE_BINARY. This maps Friday's default STT model {} to {}.",
                self.flow_root.display(),
                FRIDAY_DEFAULT_STT_MODEL_KEY,
                FLOW_PARAKEET_EXECUTION_MODEL_KEY
            ))
        }
    }

    fn parakeet_ready(&self) -> bool {
        let root = self.flow_root.join(PARAKEET_MODEL_DIR);
        [
            "encoder.int8.onnx",
            "decoder.int8.onnx",
            "joiner.int8.onnx",
            "tokens.txt",
        ]
        .iter()
        .all(|file| root.join(file).exists())
    }
}

impl KokoroTtsRuntime {
    fn detect(flow_root: &Path) -> Option<Self> {
        candidate_flow_data_roots(flow_root)
            .into_iter()
            .filter(|root| root.exists())
            .find_map(Self::from_data_root)
    }

    fn from_data_root(data_root: PathBuf) -> Option<Self> {
        let python = env::var_os("FLOW_TTS_PYTHON")
            .or_else(|| env::var_os("DX_KOKORO_TTS_PYTHON"))
            .map(PathBuf::from)
            .filter(|path| path.is_file())
            .or_else(|| find_kokoro_python(&data_root))?;
        let runner = env::var_os("FLOW_TTS_RUNNER")
            .or_else(|| env::var_os("DX_KOKORO_TTS_RUNNER"))
            .map(PathBuf::from)
            .filter(|path| path.is_file())
            .or_else(|| {
                let path = data_root.join(KOKORO_RUNNER_SCRIPT);
                path.is_file().then_some(path)
            })?;
        let model_dir = env::var_os("DX_KOKORO_MODEL_DIR")
            .map(PathBuf::from)
            .filter(|path| kokoro_model_dir_ready(path))
            .or_else(|| find_kokoro_model_dir(&data_root))?;

        Some(Self {
            data_root,
            python,
            runner,
            model_dir,
        })
    }

    fn synthesize(&self, text: &str) -> Result<PathBuf> {
        let output_dir = env::temp_dir().join("zed-kokoro-tts");
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(self.data_root.join("huggingface"))?;
        fs::create_dir_all(self.data_root.join("torch"))?;
        let output_path = output_dir.join(format!(
            "zed-composer-kokoro-{}.wav",
            Uuid::new_v4().as_simple()
        ));

        let mut command = Command::new(&self.python);
        command
            .arg(&self.runner)
            .arg("--model-kind")
            .arg("kokoro")
            .arg("--model-dir")
            .arg(&self.model_dir)
            .arg("--text")
            .arg(text)
            .arg("--output")
            .arg(&output_path)
            .arg("--language")
            .arg("English")
            .arg("--speaker")
            .arg(DEFAULT_KOKORO_VOICE)
            .arg("--device")
            .arg("cpu");
        apply_tts_process_env(&mut command, &self.data_root);
        apply_windows_process_flags(&mut command);

        let output =
            match run_command_with_timeout(command, TTS_COMMAND_TIMEOUT, "Friday Kokoro TTS") {
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
        if !output_path.exists() {
            return Err(anyhow!(
                "Friday Kokoro TTS finished without writing {}",
                output_path.display()
            ));
        }
        let audio_size = match fs::metadata(&output_path) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                let _ = fs::remove_file(&output_path);
                return Err(anyhow!(
                    "Could not inspect {}: {}",
                    output_path.display(),
                    error
                ));
            }
        };
        if audio_size <= 44 {
            let _ = fs::remove_file(&output_path);
            return Err(anyhow!(
                "Friday Kokoro TTS wrote an empty WAV file at {}",
                output_path.display()
            ));
        }

        Ok(output_path)
    }
}

impl FlowRecordingSession {
    pub(crate) fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub(crate) fn telemetry(&self) -> Result<RecordingTelemetry> {
        let samples = self
            .samples
            .lock()
            .map_err(|_| anyhow!("Microphone recording buffer was poisoned"))?;
        let captured_duration =
            Duration::from_secs_f32(samples.len() as f32 / TARGET_SAMPLE_RATE as f32);
        let input_level = recent_input_level(&samples);

        Ok(RecordingTelemetry {
            captured_duration,
            input_level,
        })
    }

    pub(crate) fn finish(self) -> Result<(FlowSpeechRuntime, RecordedSpeech)> {
        let Self {
            runtime,
            samples,
            _stream,
            started_at: _,
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
) -> Result<cpal::Stream> {
    let stream_config = config.clone().into();
    match config.sample_format() {
        SampleFormat::F32 => build_input_stream_typed::<f32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::F64 => build_input_stream_typed::<f64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::I8 => build_input_stream_typed::<i8>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::I16 => build_input_stream_typed::<i16>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::I24 => build_input_stream_typed::<cpal::I24>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::I32 => build_input_stream_typed::<i32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::I64 => build_input_stream_typed::<i64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::U8 => build_input_stream_typed::<u8>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::U16 => build_input_stream_typed::<u16>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::U32 => build_input_stream_typed::<u32>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        SampleFormat::U64 => build_input_stream_typed::<u64>(
            device,
            stream_config,
            channels,
            input_sample_rate,
            samples,
        ),
        other => Err(anyhow!("Unsupported microphone sample format: {other:?}")),
    }
}

fn resolve_input_device(input_device_id: Option<&DeviceId>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    if let Some(device_id) = input_device_id {
        if let Some(device) = host.device_by_id(device_id) {
            return Ok(device);
        }
        log::warn!("Selected Flow microphone device was not found; falling back to default input");
    }

    host.default_input_device()
        .context("No microphone input device is available")
}

fn build_input_stream_typed<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    channels: usize,
    input_sample_rate: u32,
    samples: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream>
where
    T: Sample + SizedSample + Send + Copy + 'static,
    f32: FromSample<T>,
{
    let stream = device.build_input_stream(
        &config,
        move |data: &[T], _| {
            let converted = data
                .iter()
                .map(|sample| sample.to_sample::<f32>())
                .collect::<Vec<_>>();
            let processed = downmix_and_resample(&converted, channels, input_sample_rate);
            if let Ok(mut buffer) = samples.try_lock() {
                let limit = TARGET_SAMPLE_RATE as usize * MAX_RECORDING_SECONDS;
                let remaining = limit.saturating_sub(buffer.len());
                buffer.extend(processed.into_iter().take(remaining));
            }
        },
        |error| log::warn!("Flow microphone input stream error: {error}"),
        None,
    )?;
    Ok(stream)
}

fn downmix_and_resample(input: &[f32], channels: usize, input_sample_rate: u32) -> Vec<f32> {
    let channels = channels.max(1);
    let frame_count = input.len() / channels;
    let mut mono = Vec::with_capacity(frame_count);
    for frame in input.chunks(channels) {
        let sum = frame.iter().copied().sum::<f32>();
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
        return Err(command_error("Flow Parakeet transcription failed", output));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().rev() {
        if let Some(value) = line.strip_prefix("[stt] ") {
            let text = value.trim().trim_matches('"').trim();
            if !text.is_empty() {
                return Ok(text.to_string());
            }
        }
    }

    Err(anyhow!("Flow STT finished without a transcript"))
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
) -> Result<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to start {label}"))?;
    let started_at = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            return child
                .wait_with_output()
                .with_context(|| format!("Failed to collect {label} output"));
        }

        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!("{label} timed out after {}s", timeout.as_secs()));
        }

        thread::sleep(COMMAND_POLL_INTERVAL);
    }
}

fn apply_tts_process_env(command: &mut Command, data_root: &Path) {
    let hf_home = data_root.join("huggingface");
    let torch_home = data_root.join("torch");
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
        .env("OMP_NUM_THREADS", "4")
        .env("MKL_NUM_THREADS", "4")
        .env("NUMEXPR_NUM_THREADS", "4")
        .env("FLOW_TTS_TORCH_THREADS", "4");
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
    push_unique_path(
        &mut roots,
        env::var_os("DX_FLOW_DATA_ROOT").map(PathBuf::from),
    );
    push_unique_path(&mut roots, env::var_os("FLOW_DATA_DIR").map(PathBuf::from));
    push_unique_path(&mut roots, Some(flow_root.join("data")));

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            push_unique_path(&mut roots, Some(local_app_data.join("com.flow.data")));
        }

        for drive in b'D'..=b'Z' {
            let root = PathBuf::from(format!("{}:\\Flow\\data", drive as char));
            if root.exists() {
                push_unique_path(&mut roots, Some(root));
            }
        }
    }

    roots
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    if let Some(path) = path
        && !paths.iter().any(|existing| existing == &path)
    {
        paths.push(path);
    }
}

fn find_kokoro_python(data_root: &Path) -> Option<PathBuf> {
    let flow_home = data_root.parent()?;
    let python = flow_home
        .join("runtime")
        .join("kokoro-tts")
        .join(".venv")
        .join(if cfg!(target_os = "windows") {
            "Scripts/python.exe"
        } else {
            "bin/python"
        });
    python.is_file().then_some(python)
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
    [
        path.join("config.json"),
        path.join("kokoro-v1_0.pth"),
        path.join("voices").join("af_heart.pt"),
        path.join("voices").join("af_bella.pt"),
    ]
    .iter()
    .all(|path| file_is_nonempty(path))
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
        .find(|path| path.is_file())
}

fn default_flow_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .map(|dx_root| dx_root.join("flow"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from(r"G:\Dx\flow"))
}
