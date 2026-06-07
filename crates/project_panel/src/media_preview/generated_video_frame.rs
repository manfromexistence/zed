use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{self, Output},
    time::{Duration, UNIX_EPOCH},
};

use futures::future::{Either, select};
use gpui::BackgroundExecutor;
use util::command::Stdio;

pub(super) const PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR: &str = "project-panel-media-frames";
const DX_FFMPEG_PATH_ENV: &str = "DX_FFMPEG_PATH";
const DX_FFPROBE_PATH_ENV: &str = "DX_FFPROBE_PATH";
const GENERATED_VIDEO_DURATION_PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const GENERATED_VIDEO_FRAME_EXTRACTION_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_GENERATED_VIDEO_DURATION_STDOUT_BYTES: usize = 256;
const MAX_GENERATED_VIDEO_DURATION_SECONDS: f64 = 24. * 60. * 60.;
const MAX_GENERATED_VIDEO_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_GENERATED_VIDEO_PATH_TEXT_BYTES: usize = 4096;
const MAX_GENERATED_VIDEO_FRAME_BYTES: u64 = 8 * 1024 * 1024;

pub(super) struct GeneratedVideoFrameMetadata {
    pub(super) center_frame_path: PathBuf,
    pub(super) duration_seconds: Option<f64>,
}

pub(super) async fn generate_video_center_frame(
    source_path: &Path,
    path_text: &str,
    size: u64,
    executor: &BackgroundExecutor,
) -> Option<GeneratedVideoFrameMetadata> {
    if !is_safe_generated_video_source(source_path, path_text, size) {
        return None;
    }

    let modified_at = video_frame_cache_modified_at(source_path);
    let output_path = managed_video_frame_cache_path(path_text, size, modified_at);
    let duration_seconds = probe_video_duration_seconds(source_path, executor).await;
    if output_path.is_file() {
        return Some(GeneratedVideoFrameMetadata {
            center_frame_path: output_path,
            duration_seconds,
        });
    }

    fs::create_dir_all(output_path.parent()?).ok()?;

    let duration_seconds = duration_seconds?;
    let center_seconds = duration_seconds / 2.;
    let temporary_output_path = temporary_video_frame_path(&output_path)?;
    let _ = fs::remove_file(&temporary_output_path);

    if !extract_video_center_frame(
        source_path,
        &temporary_output_path,
        center_seconds,
        executor,
    )
    .await
    {
        let _ = fs::remove_file(&temporary_output_path);
        return None;
    }

    let frame_size = fs::metadata(&temporary_output_path).ok()?.len();
    if frame_size == 0 || frame_size > MAX_GENERATED_VIDEO_FRAME_BYTES {
        let _ = fs::remove_file(&temporary_output_path);
        return None;
    }

    if output_path.is_file() {
        let _ = fs::remove_file(&temporary_output_path);
        return Some(GeneratedVideoFrameMetadata {
            center_frame_path: output_path,
            duration_seconds: Some(duration_seconds),
        });
    }

    match fs::rename(&temporary_output_path, &output_path) {
        Ok(()) => Some(GeneratedVideoFrameMetadata {
            center_frame_path: output_path,
            duration_seconds: Some(duration_seconds),
        }),
        Err(_) if output_path.is_file() => {
            let _ = fs::remove_file(&temporary_output_path);
            Some(GeneratedVideoFrameMetadata {
                center_frame_path: output_path,
                duration_seconds: Some(duration_seconds),
            })
        }
        Err(_) => {
            let _ = fs::remove_file(&temporary_output_path);
            None
        }
    }
}

fn is_safe_generated_video_source(source_path: &Path, path_text: &str, size: u64) -> bool {
    size > 0
        && size <= MAX_GENERATED_VIDEO_SOURCE_BYTES
        && source_path.is_absolute()
        && !path_text.is_empty()
        && path_text.len() <= MAX_GENERATED_VIDEO_PATH_TEXT_BYTES
}

fn managed_video_frame_cache_path(path_text: &str, size: u64, modified_at: u64) -> PathBuf {
    paths::temp_dir()
        .join(PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR)
        .join(format!(
            "{:016x}.jpg",
            stable_video_frame_cache_key(path_text, size, modified_at)
        ))
}

fn video_frame_cache_modified_at(source_path: &Path) -> u64 {
    fs::metadata(source_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified_at| modified_at.duration_since(UNIX_EPOCH).ok())
        .map(|duration| {
            duration
                .as_secs()
                .saturating_mul(1_000_000_000)
                .saturating_add(u64::from(duration.subsec_nanos()))
        })
        .unwrap_or_default()
}

async fn probe_video_duration_seconds(
    source_path: &Path,
    executor: &BackgroundExecutor,
) -> Option<f64> {
    let output = run_media_command_output(
        executor,
        ffprobe_binary()?,
        &[
            OsString::from("-v"),
            OsString::from("error"),
            OsString::from("-show_entries"),
            OsString::from("format=duration"),
            OsString::from("-of"),
            OsString::from("default=noprint_wrappers=1:nokey=1"),
            source_path.as_os_str().to_os_string(),
        ],
        GENERATED_VIDEO_DURATION_PROBE_TIMEOUT,
    )
    .await?;

    if !output.status.success() || output.stdout.len() > MAX_GENERATED_VIDEO_DURATION_STDOUT_BYTES {
        return None;
    }

    let duration_text = String::from_utf8(output.stdout).ok()?;
    let duration_seconds = duration_text.trim().parse::<f64>().ok()?;
    (duration_seconds.is_finite()
        && duration_seconds > 0.
        && duration_seconds <= MAX_GENERATED_VIDEO_DURATION_SECONDS)
        .then_some(duration_seconds)
}

async fn extract_video_center_frame(
    source_path: &Path,
    output_path: &Path,
    center_seconds: f64,
    executor: &BackgroundExecutor,
) -> bool {
    let output = run_media_command_output(
        executor,
        match ffmpeg_binary() {
            Some(binary) => binary,
            None => return false,
        },
        &[
            OsString::from("-hide_banner"),
            OsString::from("-loglevel"),
            OsString::from("error"),
            OsString::from("-nostdin"),
            OsString::from("-y"),
            OsString::from("-ss"),
            OsString::from(format_video_timestamp(center_seconds)),
            OsString::from("-i"),
            source_path.as_os_str().to_os_string(),
            OsString::from("-frames:v"),
            OsString::from("1"),
            OsString::from("-vf"),
            OsString::from("scale=480:-2"),
            OsString::from("-q:v"),
            OsString::from("4"),
            output_path.as_os_str().to_os_string(),
        ],
        GENERATED_VIDEO_FRAME_EXTRACTION_TIMEOUT,
    )
    .await;

    output.is_some_and(|output| output.status.success())
}

async fn run_media_command_output(
    executor: &BackgroundExecutor,
    program: OsString,
    args: &[OsString],
    timeout: Duration,
) -> Option<Output> {
    let mut command = util::command::new_command(program);
    command.args(args);
    command.stdin(Stdio::null());
    command.kill_on_drop(true);

    let output = command.output();
    let timeout = executor.timer(timeout);
    futures::pin_mut!(output);
    futures::pin_mut!(timeout);

    match select(output, timeout).await {
        Either::Left((output, _)) => output.ok(),
        Either::Right((_, _)) => None,
    }
}

fn ffmpeg_binary() -> Option<OsString> {
    media_binary_from_env(DX_FFMPEG_PATH_ENV, "ffmpeg")
}

fn ffprobe_binary() -> Option<OsString> {
    media_binary_from_env(DX_FFPROBE_PATH_ENV, "ffprobe")
}

fn media_binary_from_env(env_var: &str, fallback: &str) -> Option<OsString> {
    let binary = env::var_os(env_var).unwrap_or_else(|| OsString::from(fallback));
    (!media_binary_is_shell(&binary)).then_some(binary)
}

fn media_binary_is_shell(binary: &OsString) -> bool {
    let Some(name) = Path::new(binary.as_os_str())
        .file_stem()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return true;
    };

    matches!(
        name.as_str(),
        "cmd" | "powershell" | "pwsh" | "sh" | "bash" | "zsh"
    )
}

fn temporary_video_frame_path(output_path: &Path) -> Option<PathBuf> {
    let extension = format!("{}.tmp.jpg", process::id());
    Some(output_path.with_extension(extension))
}

fn format_video_timestamp(seconds: f64) -> String {
    format!("{:.3}", seconds.max(0.))
}

fn stable_video_frame_cache_key(path_text: &str, size: u64, modified_at: u64) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    let size_bytes = size.to_le_bytes();
    let modified_at_bytes = modified_at.to_le_bytes();
    for byte in path_text
        .as_bytes()
        .iter()
        .chain(size_bytes.iter())
        .chain(modified_at_bytes.iter())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
