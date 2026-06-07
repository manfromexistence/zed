use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

use futures::future::{Either, select};
use gpui::BackgroundExecutor;
use rodio::{Decoder, Source};

use super::{
    GeneratedMediaMetadataIndex, MediaPreviewItem, MediaPreviewKind,
    generated_video_frame::generate_video_center_frame, metadata::GeneratedMediaMetadataRecord,
};

pub(crate) const GENERATED_MEDIA_METADATA_RUNNER_SCHEMA: &str =
    "zed.project_panel.generated_media_metadata_runner";

const MAX_GENERATED_MEDIA_METADATA_JOBS: usize = 8;
const MAX_GENERATED_MEDIA_METADATA_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_GENERATED_MEDIA_METADATA_PATH_TEXT_BYTES: usize = 4096;
const GENERATED_AUDIO_DURATION_PROBE_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeneratedMediaMetadataJobBatch {
    schema: &'static str,
    jobs: Vec<GeneratedMediaMetadataJob>,
}

#[derive(Clone, Debug, PartialEq)]
struct GeneratedMediaMetadataJob {
    path_text: String,
    path: PathBuf,
    kind: MediaPreviewKind,
    size: u64,
}

impl GeneratedMediaMetadataJob {
    fn is_safe_managed_job(&self) -> bool {
        self.size > 0
            && self.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES
            && self.path.is_absolute()
            && !self.path_text.is_empty()
            && self.path_text.len() <= MAX_GENERATED_MEDIA_METADATA_PATH_TEXT_BYTES
            && !matches!(self.kind, MediaPreviewKind::Image)
    }
}

impl GeneratedMediaMetadataJobBatch {
    fn safe_jobs(self) -> impl Iterator<Item = GeneratedMediaMetadataJob> {
        self.jobs
            .into_iter()
            .take(MAX_GENERATED_MEDIA_METADATA_JOBS)
            .filter(GeneratedMediaMetadataJob::is_safe_managed_job)
    }
}

pub(crate) fn build_generated_media_metadata_job_batch(
    items: &[MediaPreviewItem],
) -> Option<GeneratedMediaMetadataJobBatch> {
    let mut jobs = Vec::new();

    for item in items {
        if jobs.len() >= MAX_GENERATED_MEDIA_METADATA_JOBS {
            break;
        }

        let should_generate = match item.kind {
            MediaPreviewKind::Audio => {
                item.duration_label.is_none()
                    && item.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES
            }
            MediaPreviewKind::Video => {
                (item.video_frame_preview.is_none() || item.duration_label.is_none())
                    && item.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES
            }
            MediaPreviewKind::Image => false,
        };

        if !should_generate {
            continue;
        }

        let job = GeneratedMediaMetadataJob {
            path_text: item.absolute_path.display().to_string(),
            path: item.absolute_path.clone(),
            kind: item.kind,
            size: item.size,
        };

        if job.is_safe_managed_job() {
            jobs.push(job);
        }
    }

    (!jobs.is_empty()).then_some(GeneratedMediaMetadataJobBatch {
        schema: GENERATED_MEDIA_METADATA_RUNNER_SCHEMA,
        jobs,
    })
}

pub(crate) async fn collect_generated_media_metadata(
    batch: GeneratedMediaMetadataJobBatch,
    executor: BackgroundExecutor,
) -> GeneratedMediaMetadataIndex {
    if batch.schema != GENERATED_MEDIA_METADATA_RUNNER_SCHEMA {
        return GeneratedMediaMetadataIndex::default();
    }

    let mut records = Vec::with_capacity(batch.jobs.len().min(MAX_GENERATED_MEDIA_METADATA_JOBS));

    for job in batch.safe_jobs() {
        match job.kind {
            MediaPreviewKind::Audio => {
                if let Some(duration_seconds) =
                    audio_duration_seconds_for_path(&job.path, &executor).await
                {
                    records.push(GeneratedMediaMetadataRecord {
                        path_text: job.path_text,
                        duration_label: None,
                        duration_seconds: Some(duration_seconds),
                        center_frame_path: None,
                        preview_frame_path: None,
                    });
                }
            }
            MediaPreviewKind::Video => {
                if let Some(video_frame_metadata) =
                    generate_video_center_frame(&job.path, &job.path_text, job.size, &executor)
                        .await
                {
                    records.push(GeneratedMediaMetadataRecord {
                        path_text: job.path_text,
                        duration_label: None,
                        duration_seconds: video_frame_metadata.duration_seconds,
                        center_frame_path: Some(video_frame_metadata.center_frame_path),
                        preview_frame_path: None,
                    });
                }
            }
            MediaPreviewKind::Image => {}
        }
    }

    GeneratedMediaMetadataIndex::from_records(records)
}

async fn audio_duration_seconds_for_path(
    path: &Path,
    executor: &BackgroundExecutor,
) -> Option<f64> {
    let path = path.to_path_buf();
    let duration_task = executor.spawn(async move { audio_duration_seconds_for_path_sync(&path) });
    let timeout = executor.timer(GENERATED_AUDIO_DURATION_PROBE_TIMEOUT);
    futures::pin_mut!(duration_task);
    futures::pin_mut!(timeout);

    match select(duration_task, timeout).await {
        Either::Left((duration_seconds, _)) => duration_seconds,
        Either::Right((_, _)) => None,
    }
}

fn audio_duration_seconds_for_path_sync(path: &Path) -> Option<f64> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).ok()?;
    source
        .total_duration()
        .map(|duration| duration.as_secs_f64())
}
