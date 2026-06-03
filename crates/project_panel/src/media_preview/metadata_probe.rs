use std::path::{Path, PathBuf};

use super::{MediaPreviewItem, MediaPreviewKind, stable_text_hash};

pub(super) const PROJECT_PANEL_MEDIA_METADATA_PROBE_SCHEMA: &str =
    "zed.project_panel.media_metadata_probe";

const MAX_PROJECT_PANEL_MEDIA_METADATA_PROBE_ACTIONS: usize = 8;
const MANAGED_MEDIA_METADATA_CACHE_NAMESPACE: &str = "project-panel-media-metadata";
const CENTER_FRAME_TIMESTAMP_TEMPLATE: &str = "<duration_seconds/2>";
const MANAGED_CENTER_FRAME_OUTPUT_TEMPLATE: &str = "<managed-media-cache>";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MediaMetadataProbePlan {
    schema: &'static str,
    source_folder: PathBuf,
    managed_cache_namespace: &'static str,
    tool_execution_allowed: bool,
    writes_user_project_files: bool,
    cap_hit: bool,
    actions: Vec<MediaMetadataProbeAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MediaMetadataProbeAction {
    item_path: PathBuf,
    kind: MediaPreviewKind,
    needs_duration: bool,
    needs_center_frame: bool,
    ffprobe_argument_vector: Vec<String>,
    ffmpeg_center_frame_argument_template: Option<Vec<String>>,
    expected_manifest_fields: Vec<&'static str>,
}

impl MediaMetadataProbePlan {
    pub(crate) fn summary_label(&self) -> String {
        let duration_count = self
            .actions
            .iter()
            .filter(|action| action.needs_duration)
            .count();
        let center_frame_count = self
            .actions
            .iter()
            .filter(|action| action.needs_center_frame)
            .count();
        let planned_video_count = self
            .actions
            .iter()
            .filter(|action| action.kind == MediaPreviewKind::Video)
            .count();
        let planned_audio_count = self
            .actions
            .iter()
            .filter(|action| action.kind == MediaPreviewKind::Audio)
            .count();

        let safety = if self.is_non_executing_managed_plan() {
            "metadata will stay in the managed cache"
        } else {
            "metadata needs review before refresh"
        };
        let cap = if self.cap_hit { ", capped" } else { "" };

        format!(
            "Some media details are unavailable: {duration_count} durations, {center_frame_count} thumbnails ({planned_video_count} video, {planned_audio_count} audio{cap}; {safety})"
        )
    }

    fn is_non_executing_managed_plan(&self) -> bool {
        self.schema == PROJECT_PANEL_MEDIA_METADATA_PROBE_SCHEMA
            && self.managed_cache_namespace == MANAGED_MEDIA_METADATA_CACHE_NAMESPACE
            && !self.tool_execution_allowed
            && !self.writes_user_project_files
            && !self.source_folder.as_os_str().is_empty()
            && self.actions.iter().all(|action| {
                !action.item_path.as_os_str().is_empty()
                    && !action.ffprobe_argument_vector.is_empty()
                    && !action.expected_manifest_fields.is_empty()
                    && (!action.needs_center_frame
                        || action.ffmpeg_center_frame_argument_template.is_some())
            })
    }
}

pub(crate) fn build_media_metadata_probe_plan(
    parent_abs_path: &Path,
    items: &[MediaPreviewItem],
) -> Option<MediaMetadataProbePlan> {
    let mut actions = Vec::new();
    let mut cap_hit = false;

    for item in items {
        let source_path = item.absolute_path.clone();

        let action = match item.kind {
            MediaPreviewKind::Image => continue,
            MediaPreviewKind::Video => {
                let needs_duration = item.duration_label.is_none();
                let needs_center_frame = item.video_frame_preview.is_none();
                if !needs_duration && !needs_center_frame {
                    continue;
                }

                let source_path_text = source_path.display().to_string();
                let ffprobe_argument_vector = ffprobe_duration_argument_vector(&source_path_text);
                let output_hint = managed_center_frame_output_hint(&source_path_text);
                MediaMetadataProbeAction {
                    item_path: source_path,
                    kind: item.kind,
                    needs_duration,
                    needs_center_frame,
                    ffprobe_argument_vector,
                    ffmpeg_center_frame_argument_template: needs_center_frame.then(|| {
                        ffmpeg_center_frame_argument_template(&source_path_text, &output_hint)
                    }),
                    expected_manifest_fields: vec!["path", "duration_seconds", "center_frame"],
                }
            }
            MediaPreviewKind::Audio => {
                let needs_duration = item.duration_label.is_none();
                if !needs_duration {
                    continue;
                }

                let source_path_text = source_path.display().to_string();
                let ffprobe_argument_vector = ffprobe_duration_argument_vector(&source_path_text);
                MediaMetadataProbeAction {
                    item_path: source_path,
                    kind: item.kind,
                    needs_duration,
                    needs_center_frame: false,
                    ffprobe_argument_vector,
                    ffmpeg_center_frame_argument_template: None,
                    expected_manifest_fields: vec!["path", "duration_seconds"],
                }
            }
        };

        if actions.len() >= MAX_PROJECT_PANEL_MEDIA_METADATA_PROBE_ACTIONS {
            cap_hit = true;
            break;
        }

        actions.push(action);
    }

    (!actions.is_empty()).then_some(MediaMetadataProbePlan {
        schema: PROJECT_PANEL_MEDIA_METADATA_PROBE_SCHEMA,
        source_folder: parent_abs_path.to_path_buf(),
        managed_cache_namespace: MANAGED_MEDIA_METADATA_CACHE_NAMESPACE,
        tool_execution_allowed: false,
        writes_user_project_files: false,
        cap_hit,
        actions,
    })
}

fn ffprobe_duration_argument_vector(source_path: &str) -> Vec<String> {
    vec![
        "ffprobe".to_string(),
        "-v".to_string(),
        "error".to_string(),
        "-show_entries".to_string(),
        "format=duration".to_string(),
        "-of".to_string(),
        "json".to_string(),
        source_path.to_string(),
    ]
}

fn ffmpeg_center_frame_argument_template(source_path: &str, output_hint: &str) -> Vec<String> {
    vec![
        "ffmpeg".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-ss".to_string(),
        CENTER_FRAME_TIMESTAMP_TEMPLATE.to_string(),
        "-i".to_string(),
        source_path.to_string(),
        "-frames:v".to_string(),
        "1".to_string(),
        output_hint.to_string(),
    ]
}

fn managed_center_frame_output_hint(source_path: &str) -> String {
    format!(
        "{MANAGED_CENTER_FRAME_OUTPUT_TEMPLATE}/{:016x}-center-frame.png",
        stable_text_hash(source_path)
    )
}
