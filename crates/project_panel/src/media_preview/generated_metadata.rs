use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use rodio::{Decoder, Source};

use super::{
    GeneratedMediaMetadataIndex, MediaPreviewItem, MediaPreviewKind,
    generated_video_frame::generate_video_center_frame, metadata::GeneratedMediaMetadataRecord,
};

pub(crate) const GENERATED_MEDIA_METADATA_RUNNER_SCHEMA: &str =
    "zed.project_panel.generated_media_metadata_runner";

const MAX_GENERATED_MEDIA_METADATA_JOBS: usize = 8;
const MAX_GENERATED_MEDIA_METADATA_FILE_BYTES: u64 = 64 * 1024 * 1024;

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
                item.video_frame_preview.is_none()
                    && item.size <= MAX_GENERATED_MEDIA_METADATA_FILE_BYTES
            }
            MediaPreviewKind::Image => false,
        };

        if !should_generate {
            continue;
        }

        jobs.push(GeneratedMediaMetadataJob {
            path_text: item.absolute_path.display().to_string(),
            path: item.absolute_path.clone(),
            kind: item.kind,
            size: item.size,
        });
    }

    (!jobs.is_empty()).then_some(GeneratedMediaMetadataJobBatch {
        schema: GENERATED_MEDIA_METADATA_RUNNER_SCHEMA,
        jobs,
    })
}

pub(crate) async fn collect_generated_media_metadata(
    batch: GeneratedMediaMetadataJobBatch,
) -> GeneratedMediaMetadataIndex {
    if batch.schema != GENERATED_MEDIA_METADATA_RUNNER_SCHEMA {
        return GeneratedMediaMetadataIndex::default();
    }

    let mut records = Vec::with_capacity(batch.jobs.len());

    for job in batch.jobs {
        match job.kind {
            MediaPreviewKind::Audio => {
                if let Some(duration_seconds) = audio_duration_seconds_for_path(&job.path) {
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
                if let Some(center_frame_path) =
                    generate_video_center_frame(&job.path, &job.path_text, job.size).await
                {
                    records.push(GeneratedMediaMetadataRecord {
                        path_text: job.path_text,
                        duration_label: None,
                        duration_seconds: None,
                        center_frame_path: Some(center_frame_path),
                        preview_frame_path: None,
                    });
                }
            }
            MediaPreviewKind::Image => {}
        }
    }

    GeneratedMediaMetadataIndex::from_records(records)
}

fn audio_duration_seconds_for_path(path: &Path) -> Option<f64> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).ok()?;
    source
        .total_duration()
        .map(|duration| duration.as_secs_f64())
}
