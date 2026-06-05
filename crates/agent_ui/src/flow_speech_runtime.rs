use anyhow::{Context as _, Result, anyhow};
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MAX_RECORDING_SECONDS: usize = 90;
const MIN_RECORDING_SAMPLES: usize = TARGET_SAMPLE_RATE as usize / 4;
const PARAKEET_MODEL_DIR: &str = "models/stt/parakeet-tdt-0.6b-v3-int8";
const KOKORO_MODEL_FILE: &str = "models/tts/kokoro-v1.0.int8.onnx";

#[derive(Clone, Debug)]
pub(crate) struct FlowSpeechRuntime {
    flow_root: PathBuf,
    flow_binary: Option<PathBuf>,
    flow_dictate_binary: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct RecordedSpeech {
    samples: Vec<f32>,
}

pub(crate) struct FlowRecordingSession {
    runtime: FlowSpeechRuntime,
    samples: Arc<Mutex<Vec<f32>>>,
    _stream: cpal::Stream,
    started_at: Instant,
}

impl FlowSpeechRuntime {
    pub(crate) fn detect() -> Self {
        let flow_root = env::var_os("DX_FLOW_ROOT")
            .or_else(|| env::var_os("FLOW_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(default_flow_root);

        let flow_binary = env::var_os("DX_FLOW_BINARY")
            .map(PathBuf::from)
            .filter(|path| path.exists())
            .or_else(|| find_binary(&flow_root, "flow"));

        let flow_dictate_binary = env::var_os("DX_FLOW_DICTATE_BINARY")
            .map(PathBuf::from)
            .filter(|path| path.exists())
            .or_else(|| find_binary(&flow_root, "flow-dictate"));

        Self {
            flow_root,
            flow_binary,
            flow_dictate_binary,
        }
    }

    pub(crate) fn start_recording(&self) -> Result<FlowRecordingSession> {
        self.ensure_parakeet_ready()?;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No microphone input device is available")?;
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
        self.ensure_parakeet_ready()?;
        let audio_path = self.write_recording_wav(&recording)?;
        let output = if let Some(binary) = &self.flow_dictate_binary {
            Command::new(binary)
                .current_dir(&self.flow_root)
                .arg("--file")
                .arg(&audio_path)
                .stdin(Stdio::null())
                .output()
                .with_context(|| format!("Failed to start {}", binary.display()))?
        } else if let Some(binary) = &self.flow_binary {
            Command::new(binary)
                .current_dir(&self.flow_root)
                .arg("--transcribe")
                .arg(&audio_path)
                .stdin(Stdio::null())
                .output()
                .with_context(|| format!("Failed to start {}", binary.display()))?
        } else {
            return Err(anyhow!(
                "Flow Parakeet runtime is not built. Build flow-dictate in {} or set DX_FLOW_DICTATE_BINARY.",
                self.flow_root.display()
            ));
        };

        let transcript = parse_transcript_output(output);
        let _ = fs::remove_file(audio_path);
        transcript
    }

    pub(crate) fn speak_text(&self, text: &str) -> Result<()> {
        self.ensure_kokoro_ready()?;
        let binary = self.flow_binary.as_ref().ok_or_else(|| {
            anyhow!(
                "Flow Kokoro runtime is not built. Build flow in {} or set DX_FLOW_BINARY.",
                self.flow_root.display()
            )
        })?;

        let output = Command::new(binary)
            .current_dir(&self.flow_root)
            .arg("--speak")
            .arg(text)
            .stdin(Stdio::null())
            .output()
            .with_context(|| format!("Failed to start {}", binary.display()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(command_error("Flow Kokoro TTS failed", output))
        }
    }

    pub(crate) fn status_summary(&self) -> String {
        let stt = if self.parakeet_ready() {
            "Parakeet ready"
        } else {
            "Parakeet model missing"
        };
        let tts = if self.kokoro_ready() {
            "Kokoro ready"
        } else {
            "Kokoro model missing"
        };
        let runtime = if self.flow_binary.is_some() || self.flow_dictate_binary.is_some() {
            "Flow runtime found"
        } else {
            "Flow runtime executable missing"
        };

        format!("{stt}; {tts}; {runtime}")
    }

    fn write_recording_wav(&self, recording: &RecordedSpeech) -> Result<PathBuf> {
        let tmp_dir = self.flow_root.join("tmp").join("zed-voice");
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

    fn ensure_kokoro_ready(&self) -> Result<()> {
        if self.kokoro_ready() {
            Ok(())
        } else {
            Err(anyhow!(
                "Flow Kokoro model file is missing at {}",
                self.flow_root.join(KOKORO_MODEL_FILE).display()
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

    fn kokoro_ready(&self) -> bool {
        self.flow_root.join(KOKORO_MODEL_FILE).exists()
            && self.flow_root.join("models/tts/voices-v1.0.bin").exists()
            && self.flow_root.join("models/tts/config.json").exists()
    }
}

impl FlowRecordingSession {
    pub(crate) fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
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

fn parse_transcript_output(output: std::process::Output) -> Result<String> {
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

fn command_error(label: &str, output: std::process::Output) -> anyhow::Error {
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

fn find_binary(flow_root: &Path, name: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    ["release", "debug"]
        .iter()
        .map(|profile| flow_root.join("target").join(profile).join(&exe))
        .find(|path| path.exists())
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
