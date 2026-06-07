use std::{
    collections::HashMap,
    fs,
    io::Read as _,
    path::{Component, Path, PathBuf},
};

use project::Entry;
use serde_json::Value;

use super::{
    MediaPreviewKind, VideoFramePreview, VideoFramePreviewKind, child_absolute_path,
    generated_video_frame::PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR, media_preview_kind_for_path,
    media_stem_key,
};

pub(super) const MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES: u64 = 256 * 1024;

const MAX_GENERATED_MEDIA_METADATA_RECORDS: usize = 256;
const MEDIA_METADATA_MANIFEST_NAMES: &[&str] = &[
    ".dx-media.json",
    "dx-media.json",
    ".zed-media.json",
    "zed-media.json",
];
const MEDIA_METADATA_LIST_FIELDS: &[&str] = &["items", "media", "entries", "assets", "files"];
const MEDIA_METADATA_PATH_FIELDS: &[&str] = &[
    "path",
    "file",
    "name",
    "source",
    "media_source",
    "relative_path",
];
const MEDIA_METADATA_DURATION_LABEL_FIELDS: &[&str] =
    &["duration_label", "duration", "time", "length"];
const MEDIA_METADATA_DURATION_SECONDS_FIELDS: &[&str] = &[
    "duration_seconds",
    "duration_secs",
    "seconds",
    "length_seconds",
];
const MEDIA_METADATA_CENTER_FRAME_FIELDS: &[&str] = &["center_frame", "middle_frame"];
const MEDIA_METADATA_PREVIEW_FRAME_FIELDS: &[&str] = &[
    "frame_path",
    "thumbnail_path",
    "poster_path",
    "preview_path",
    "frame",
    "thumbnail",
    "poster",
    "preview",
];
const MAX_MEDIA_METADATA_DURATION_LABEL_CHARS: usize = 32;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GeneratedMediaMetadataIndex {
    records: Vec<GeneratedMediaMetadataRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeneratedMediaMetadataRecord {
    pub(crate) path_text: String,
    pub(crate) duration_label: Option<String>,
    pub(crate) duration_seconds: Option<f64>,
    pub(crate) center_frame_path: Option<PathBuf>,
    pub(crate) preview_frame_path: Option<PathBuf>,
}

impl GeneratedMediaMetadataIndex {
    pub(crate) fn from_records(
        records: impl IntoIterator<Item = GeneratedMediaMetadataRecord>,
    ) -> Self {
        Self {
            records: records
                .into_iter()
                .take(MAX_GENERATED_MEDIA_METADATA_RECORDS)
                .collect(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn records(&self) -> impl Iterator<Item = &GeneratedMediaMetadataRecord> {
        self.records.iter()
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct MediaMetadataIndex {
    duration_labels: HashMap<String, String>,
    video_frame_previews: HashMap<String, VideoFramePreview>,
}

impl MediaMetadataIndex {
    pub(super) fn duration_label_for_path(&self, path: &Path) -> Option<String> {
        media_metadata_lookup_keys(path).find_map(|key| self.duration_labels.get(&key).cloned())
    }

    pub(super) fn video_frame_for_path(&self, path: &Path) -> Option<VideoFramePreview> {
        media_metadata_lookup_keys(path)
            .find_map(|key| self.video_frame_previews.get(&key).cloned())
    }

    pub(super) fn merge_generated_media_metadata(
        &mut self,
        _parent_abs_path: &Path,
        generated: &GeneratedMediaMetadataIndex,
    ) {
        for record in generated
            .records()
            .take(MAX_GENERATED_MEDIA_METADATA_RECORDS)
        {
            let keys = media_metadata_lookup_keys_from_text(&record.path_text);
            if keys.is_empty() {
                continue;
            }

            if let Some(duration) = record
                .duration_label
                .as_deref()
                .and_then(normalize_duration_label)
                .or_else(|| record.duration_seconds.map(format_media_duration_seconds))
            {
                for key in &keys {
                    self.duration_labels
                        .entry(key.clone())
                        .or_insert_with(|| duration.clone());
                }
            }

            let frame_preview = record
                .center_frame_path
                .as_ref()
                .and_then(|path| generated_video_frame_preview(path, VideoFramePreviewKind::Center))
                .or_else(|| {
                    record.preview_frame_path.as_ref().and_then(|path| {
                        generated_video_frame_preview(path, VideoFramePreviewKind::Preview)
                    })
                });

            if let Some(frame_preview) = frame_preview {
                for key in &keys {
                    self.video_frame_previews
                        .entry(key.clone())
                        .or_insert_with(|| frame_preview.clone());
                }
            }
        }
    }
}

pub(super) fn build_media_metadata_index(
    parent_abs_path: &Path,
    children: &[&Entry],
) -> MediaMetadataIndex {
    let mut index = MediaMetadataIndex::default();

    for child in children {
        if !child.is_file() || child.size > MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES {
            continue;
        }

        let Some(file_name) = child.path.file_name() else {
            continue;
        };

        if !MEDIA_METADATA_MANIFEST_NAMES
            .iter()
            .any(|candidate| file_name.eq_ignore_ascii_case(candidate))
        {
            continue;
        }

        let manifest_path = child_absolute_path(parent_abs_path, child);
        let Some(contents) = read_bounded_media_metadata_manifest(&manifest_path) else {
            continue;
        };

        let Ok(value) = serde_json::from_str::<Value>(&contents) else {
            continue;
        };
        collect_media_metadata_manifest(parent_abs_path, &value, &mut index);
    }

    index
}

fn read_bounded_media_metadata_manifest(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;

    if bytes.len() as u64 > MAX_PROJECT_PANEL_MEDIA_METADATA_MANIFEST_BYTES {
        return None;
    }

    String::from_utf8(bytes).ok()
}

fn collect_media_metadata_manifest(
    parent_abs_path: &Path,
    value: &Value,
    index: &mut MediaMetadataIndex,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_media_metadata_record(parent_abs_path, item, index, None);
            }
        }
        Value::Object(object) => {
            let mut used_list = false;
            for key in MEDIA_METADATA_LIST_FIELDS {
                if let Some(list) = object.get(*key).and_then(Value::as_array) {
                    used_list = true;
                    for item in list {
                        collect_media_metadata_record(parent_abs_path, item, index, None);
                    }
                }
            }

            if used_list {
                return;
            }

            for (path, record) in object {
                collect_media_metadata_record(parent_abs_path, record, index, Some(path));
            }
        }
        _ => {}
    }
}

fn collect_media_metadata_record(
    parent_abs_path: &Path,
    record: &Value,
    index: &mut MediaMetadataIndex,
    fallback_path: Option<&str>,
) {
    let Some(object) = record.as_object() else {
        return;
    };

    let path_text = first_string_field(object, MEDIA_METADATA_PATH_FIELDS).or(fallback_path);
    let Some(path_text) = path_text else {
        return;
    };

    let keys = media_metadata_lookup_keys_from_text(path_text);
    if keys.is_empty() {
        return;
    }

    if let Some(duration) = media_duration_label_from_record(object) {
        for key in &keys {
            index
                .duration_labels
                .entry(key.clone())
                .or_insert_with(|| duration.clone());
        }
    }

    if let Some(frame_preview) = video_frame_preview_from_record(
        parent_abs_path,
        object,
        MEDIA_METADATA_CENTER_FRAME_FIELDS,
        MEDIA_METADATA_PREVIEW_FRAME_FIELDS,
    ) {
        for key in &keys {
            index
                .video_frame_previews
                .entry(key.clone())
                .or_insert_with(|| frame_preview.clone());
        }
    }
}

fn video_frame_preview_from_record(
    parent_abs_path: &Path,
    object: &serde_json::Map<String, Value>,
    center_frame_fields: &[&str],
    preview_frame_fields: &[&str],
) -> Option<VideoFramePreview> {
    first_string_field(object, center_frame_fields)
        .and_then(|path| {
            resolve_metadata_media_path(parent_abs_path, path).map(|path| VideoFramePreview {
                path,
                kind: VideoFramePreviewKind::Center,
            })
        })
        .or_else(|| {
            first_string_field(object, preview_frame_fields).and_then(|path| {
                resolve_metadata_media_path(parent_abs_path, path).map(|path| VideoFramePreview {
                    path,
                    kind: VideoFramePreviewKind::Preview,
                })
            })
        })
}

fn media_duration_label_from_record(object: &serde_json::Map<String, Value>) -> Option<String> {
    first_string_field(object, MEDIA_METADATA_DURATION_LABEL_FIELDS)
        .and_then(normalize_duration_label)
        .or_else(|| {
            first_number_field(object, MEDIA_METADATA_DURATION_SECONDS_FIELDS)
                .map(format_media_duration_seconds)
        })
}

fn first_string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn first_number_field(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| object.get(*key).and_then(Value::as_f64))
}

fn normalize_duration_label(label: &str) -> Option<String> {
    let label = label.trim();
    if label.is_empty() {
        return None;
    }

    Some(
        label
            .chars()
            .take(MAX_MEDIA_METADATA_DURATION_LABEL_CHARS)
            .collect(),
    )
}

fn format_media_duration_seconds(seconds: f64) -> String {
    let seconds = seconds.max(0.).round() as u64;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn resolve_metadata_media_path(parent_abs_path: &Path, path: &str) -> Option<PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(path);
    if media_preview_kind_for_path(&candidate) != Some(MediaPreviewKind::Image) {
        return None;
    }

    confined_media_preview_path(parent_abs_path, &candidate)
}

fn confined_media_preview_path(parent_abs_path: &Path, candidate: &Path) -> Option<PathBuf> {
    let parent_abs_path = normalize_media_preview_path(parent_abs_path)?;
    let candidate = if candidate.is_absolute() {
        normalize_media_preview_path(candidate)?
    } else {
        normalize_media_preview_path(&parent_abs_path.join(candidate))?
    };

    candidate.starts_with(&parent_abs_path).then_some(candidate)
}

fn normalize_media_preview_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
}

fn generated_video_frame_preview(
    path: &Path,
    kind: VideoFramePreviewKind,
) -> Option<VideoFramePreview> {
    if media_preview_kind_for_path(path) != Some(MediaPreviewKind::Image) {
        return None;
    }

    Some(VideoFramePreview {
        path: resolve_generated_video_frame_path(path)?,
        kind,
    })
}

fn resolve_generated_video_frame_path(path: &Path) -> Option<PathBuf> {
    let path = normalize_media_preview_path(path)?;
    let cache_root = generated_video_frame_cache_root()?;
    path.starts_with(&cache_root).then_some(path)
}

fn generated_video_frame_cache_root() -> Option<PathBuf> {
    normalize_media_preview_path(&paths::temp_dir().join(PROJECT_PANEL_GENERATED_VIDEO_FRAME_DIR))
}

fn media_metadata_lookup_keys(path: &Path) -> impl Iterator<Item = String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase());
    let stem = media_stem_key(path);
    [file_name, stem].into_iter().flatten()
}

fn media_metadata_lookup_keys_from_text(path: &str) -> Vec<String> {
    let path = Path::new(path);
    let mut keys = media_metadata_lookup_keys(path).collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}
