use std::{
    cmp::Ordering,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use collections::HashMap;
use fs::MTime;
use project::{Entry, ProjectEntryId, WorktreeId};

use crate::utils;

pub(crate) const MAX_PROJECT_PANEL_STORAGE_LARGEST_FILES: usize = 3;
pub(crate) const MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FolderStorageFile {
    pub entry_id: ProjectEntryId,
    pub label: String,
    pub file_bytes: u64,
    pub modified_at: Option<MTime>,
}

impl FolderStorageFile {
    fn from_entry(entry: &Entry) -> Option<Self> {
        Some(Self {
            entry_id: entry.id,
            label: utils::bounded_project_panel_label(entry.path.file_name()?.to_string()),
            file_bytes: entry.size,
            modified_at: entry.mtime,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FolderStorageSummary {
    pub file_count: usize,
    pub file_bytes: u64,
    pub latest_modified_at: Option<MTime>,
    pub largest_files: Vec<FolderStorageFile>,
}

impl FolderStorageSummary {
    pub(crate) fn has_recorded_files(&self) -> bool {
        self.file_count > 0 || self.file_bytes > 0
    }

    pub(crate) fn record_file(&mut self, entry: &Entry) {
        self.file_count += 1;
        self.file_bytes = self.file_bytes.saturating_add(entry.size);
        self.latest_modified_at = latest_mtime(self.latest_modified_at, entry.mtime);

        if let Some(file) = FolderStorageFile::from_entry(entry) {
            self.largest_files.push(file);
            rank_largest_files(&mut self.largest_files);
            self.largest_files
                .truncate(MAX_PROJECT_PANEL_STORAGE_LARGEST_FILES);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct StorageOverview {
    pub visible_file_count: usize,
    pub visible_file_bytes: u64,
    pub cached_folder_count: usize,
    pub cached_direct_file_count: usize,
    pub cached_direct_file_bytes: u64,
    pub latest_modified_at: Option<MTime>,
    pub largest_visible_files: Vec<FolderStorageFile>,
}

impl StorageOverview {
    pub(crate) fn record_visible_file(&mut self, entry: &Entry) {
        if !entry.is_file() {
            return;
        }

        self.latest_modified_at = latest_mtime(self.latest_modified_at, entry.mtime);
        if let Some(file) = FolderStorageFile::from_entry(entry) {
            self.largest_visible_files.push(file);
            rank_largest_files(&mut self.largest_visible_files);
            self.largest_visible_files
                .truncate(MAX_PROJECT_PANEL_STORAGE_LARGEST_FILES);
        }
    }

    pub(crate) fn record_cached_folder(&mut self, summary: &FolderStorageSummary) {
        self.cached_folder_count += 1;
        self.cached_direct_file_count = self
            .cached_direct_file_count
            .saturating_add(summary.file_count);
        self.cached_direct_file_bytes = self
            .cached_direct_file_bytes
            .saturating_add(summary.file_bytes);
        self.latest_modified_at = latest_mtime(self.latest_modified_at, summary.latest_modified_at);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StorageFolderItem {
    pub worktree_id: WorktreeId,
    pub entry_id: ProjectEntryId,
    pub label: String,
    pub path_label: String,
    pub file_count: usize,
    pub file_bytes: u64,
    pub latest_modified_at: Option<MTime>,
    pub largest_files: Vec<FolderStorageFile>,
    pub heat_level: u8,
}

impl StorageFolderItem {
    pub(crate) fn from_entry(
        worktree_id: WorktreeId,
        entry: &Entry,
        summary: &FolderStorageSummary,
    ) -> Option<Self> {
        if !entry.kind.is_dir() || !summary.has_recorded_files() {
            return None;
        }

        Some(Self {
            worktree_id,
            entry_id: entry.id,
            label: folder_label(entry),
            path_label: folder_path_label(entry),
            file_count: summary.file_count,
            file_bytes: summary.file_bytes,
            latest_modified_at: summary.latest_modified_at,
            largest_files: summary.largest_files.clone(),
            heat_level: 0,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum StorageSortMode {
    #[default]
    Size,
    FileCount,
    Modified,
}

impl StorageSortMode {
    pub(crate) const ALL: [Self; 3] = [Self::Size, Self::FileCount, Self::Modified];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Size => "Size",
            Self::FileCount => "Files",
            Self::Modified => "Modified",
        }
    }

    pub(crate) fn status_label(self) -> String {
        self.label().to_string()
    }

    pub(crate) fn menu_label(self, current: Self) -> String {
        if self == current {
            format!("{} (current)", self.label())
        } else {
            self.label().to_string()
        }
    }
}

pub(crate) fn rank_storage_folder_items(
    mut items: Vec<StorageFolderItem>,
    storage_sort_mode: StorageSortMode,
) -> Vec<StorageFolderItem> {
    items.sort_by(|left, right| match storage_sort_mode {
        StorageSortMode::Size => compare_by_size(left, right),
        StorageSortMode::FileCount => compare_by_file_count(left, right),
        StorageSortMode::Modified => compare_by_modified(left, right),
    });

    let max_file_bytes = items
        .iter()
        .map(|item| item.file_bytes)
        .max()
        .unwrap_or_default();
    for item in &mut items {
        item.heat_level = storage_heat_level(item.file_bytes, max_file_bytes);
    }

    items
}

pub(crate) fn storage_overview<'a>(
    visible_file_count: usize,
    visible_file_bytes: u64,
    visible_entries: impl IntoIterator<Item = (WorktreeId, &'a Entry)>,
    folder_storage_summaries: &HashMap<(WorktreeId, ProjectEntryId), FolderStorageSummary>,
) -> StorageOverview {
    let mut overview = StorageOverview {
        visible_file_count,
        visible_file_bytes,
        ..Default::default()
    };

    for (worktree_id, entry) in visible_entries {
        overview.record_visible_file(entry);

        if entry.kind.is_dir() {
            let cache_key = (worktree_id, entry.id);
            if let Some(summary) = folder_storage_summaries.get(&cache_key)
                && summary.has_recorded_files()
            {
                overview.record_cached_folder(summary);
            }
        }
    }

    overview
}

pub(crate) fn storage_folder_items<'a>(
    visible_entries: impl IntoIterator<Item = (WorktreeId, &'a Entry)>,
    folder_storage_summaries: &HashMap<(WorktreeId, ProjectEntryId), FolderStorageSummary>,
    storage_sort_mode: StorageSortMode,
) -> Vec<StorageFolderItem> {
    let mut items = Vec::new();

    for (worktree_id, entry) in visible_entries {
        if !entry.kind.is_dir() {
            continue;
        }

        let cache_key = (worktree_id, entry.id);
        let Some(summary) = folder_storage_summaries.get(&cache_key) else {
            continue;
        };
        if let Some(item) = StorageFolderItem::from_entry(worktree_id, entry, summary) {
            items.push(item);
        }
    }

    let mut items = rank_storage_folder_items(items, storage_sort_mode);
    items.truncate(MAX_PROJECT_PANEL_STORAGE_DRILLDOWN_ITEMS);
    items
}

pub(crate) fn storage_heat_level(file_bytes: u64, max_file_bytes: u64) -> u8 {
    if file_bytes == 0 || max_file_bytes == 0 {
        return 0;
    }

    let scaled = ((u128::from(file_bytes) * 4) + (u128::from(max_file_bytes) - 1))
        / u128::from(max_file_bytes);
    scaled.clamp(1, 4) as u8
}

pub(crate) fn format_file_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_ix = 0usize;
    while value >= 1024.0 && unit_ix < UNITS.len() - 1 {
        value /= 1024.0;
        unit_ix += 1;
    }

    if unit_ix == 0 {
        format!("{} {}", bytes, UNITS[unit_ix])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit_ix])
    } else {
        format!("{value:.1} {}", UNITS[unit_ix])
    }
}

fn folder_label(entry: &Entry) -> String {
    utils::bounded_project_panel_label(
        entry
            .path
            .file_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| entry.path.as_unix_str().to_string()),
    )
}

fn folder_path_label(entry: &Entry) -> String {
    utils::bounded_project_panel_label(entry.path.as_unix_str().to_string())
}

pub(crate) fn format_modified_label(mtime: Option<MTime>) -> Option<String> {
    let modified_at = mtime?.timestamp_for_user();
    let age = SystemTime::now()
        .duration_since(modified_at)
        .unwrap_or(Duration::ZERO);

    let label = if age < Duration::from_secs(60) {
        "just now".to_string()
    } else if age < Duration::from_secs(60 * 60) {
        format!("{}m ago", age.as_secs() / 60)
    } else if age < Duration::from_secs(60 * 60 * 24) {
        format!("{}h ago", age.as_secs() / (60 * 60))
    } else {
        format!("{}d ago", age.as_secs() / (60 * 60 * 24))
    };

    Some(label)
}

fn rank_largest_files(files: &mut [FolderStorageFile]) {
    files.sort_by(compare_largest_files);
}

fn compare_largest_files(left: &FolderStorageFile, right: &FolderStorageFile) -> Ordering {
    right
        .file_bytes
        .cmp(&left.file_bytes)
        .then_with(|| compare_mtime_desc(left.modified_at, right.modified_at))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_by_size(left: &StorageFolderItem, right: &StorageFolderItem) -> Ordering {
    right
        .file_bytes
        .cmp(&left.file_bytes)
        .then_with(|| right.file_count.cmp(&left.file_count))
        .then_with(|| compare_mtime_desc(left.latest_modified_at, right.latest_modified_at))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_by_file_count(left: &StorageFolderItem, right: &StorageFolderItem) -> Ordering {
    right
        .file_count
        .cmp(&left.file_count)
        .then_with(|| right.file_bytes.cmp(&left.file_bytes))
        .then_with(|| compare_mtime_desc(left.latest_modified_at, right.latest_modified_at))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_by_modified(left: &StorageFolderItem, right: &StorageFolderItem) -> Ordering {
    compare_mtime_desc(left.latest_modified_at, right.latest_modified_at)
        .then_with(|| right.file_bytes.cmp(&left.file_bytes))
        .then_with(|| right.file_count.cmp(&left.file_count))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_mtime_desc(left: Option<MTime>, right: Option<MTime>) -> Ordering {
    mtime_sort_key(right)
        .cmp(&mtime_sort_key(left))
        .then_with(|| right.is_some().cmp(&left.is_some()))
}

fn latest_mtime(left: Option<MTime>, right: Option<MTime>) -> Option<MTime> {
    match (left, right) {
        (Some(left), Some(right)) => {
            if compare_mtime_desc(Some(left), Some(right)) == Ordering::Greater {
                Some(right)
            } else {
                Some(left)
            }
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn mtime_sort_key(mtime: Option<MTime>) -> Option<(u64, u32)> {
    mtime?
        .timestamp_for_user()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| (duration.as_secs(), duration.subsec_nanos()))
}
