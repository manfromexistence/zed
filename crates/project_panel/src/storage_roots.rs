use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
};

use crate::storage;

pub(crate) const MAX_PROJECT_PANEL_STORAGE_ROOT_STRIP_ITEMS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StorageRootKind {
    Drive,
    DxHub,
    OneDrive,
    GoogleDrive,
    Dropbox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DriveCapacity {
    pub available_bytes: u64,
    pub total_bytes: u64,
}

impl DriveCapacity {
    pub(crate) fn used_bytes(&self) -> u64 {
        self.total_bytes
            .saturating_sub(self.available_bytes.min(self.total_bytes))
    }

    pub(crate) fn capacity_label(&self) -> String {
        format!(
            "{} free / {}",
            storage::format_file_size(self.available_bytes),
            storage::format_file_size(self.total_bytes)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StorageRootShortcut {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
    pub kind: StorageRootKind,
    pub capacity: Option<DriveCapacity>,
    pub available: bool,
    pub tooltip: String,
}

impl StorageRootShortcut {
    pub(crate) fn is_available(&self) -> bool {
        self.available && self.path.is_absolute()
    }

    pub(crate) fn status_label(&self) -> String {
        if let Some(capacity) = self.capacity.as_ref() {
            capacity.capacity_label()
        } else if self.is_available() {
            "Available".to_string()
        } else {
            "Unavailable".to_string()
        }
    }
}

pub(crate) fn collect_storage_root_shortcuts() -> Vec<StorageRootShortcut> {
    let known_roots = known_root_shortcuts();
    let mut shortcuts = Vec::new();
    let drive_limit = MAX_PROJECT_PANEL_STORAGE_ROOT_STRIP_ITEMS.saturating_sub(known_roots.len());
    collect_drive_shortcuts(&mut shortcuts, drive_limit);
    shortcuts.extend(known_roots);

    dedupe_and_cap(shortcuts)
}

fn known_root_shortcuts() -> Vec<StorageRootShortcut> {
    vec![
        known_root_shortcut(
            StorageRootKind::DxHub,
            "DX Hub",
            "dx-hub",
            &["DX_HOME", "DX_ROOT"],
            &[PathBuf::from(r"G:\Dx"), PathBuf::from(r"D:\Dx")],
        ),
        known_root_shortcut(
            StorageRootKind::OneDrive,
            "OneDrive",
            "onedrive",
            &["OneDrive", "OneDriveConsumer", "OneDriveCommercial"],
            &user_profile_fallbacks(&["OneDrive"]),
        ),
        known_root_shortcut(
            StorageRootKind::GoogleDrive,
            "Google Drive",
            "google-drive",
            &["GOOGLE_DRIVE", "GoogleDrive", "GOOGLE_DRIVE_ROOT"],
            &user_profile_fallbacks(&["Google Drive", "My Drive", "GoogleDrive"]),
        ),
        known_root_shortcut(
            StorageRootKind::Dropbox,
            "Dropbox",
            "dropbox",
            &["DROPBOX", "Dropbox", "DROPBOX_ROOT"],
            &user_profile_fallbacks(&["Dropbox"]),
        ),
    ]
}

fn collect_drive_shortcuts(shortcuts: &mut Vec<StorageRootShortcut>, limit: usize) {
    if limit == 0 {
        return;
    }

    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let path = disk.mount_point().to_path_buf();
        if !path.is_absolute() || disk.total_space() == 0 {
            continue;
        }

        let label = drive_label(&path);
        let capacity = DriveCapacity {
            available_bytes: disk.available_space(),
            total_bytes: disk.total_space(),
        };
        let tooltip = format!(
            "Open drive at {} ({})",
            path.display(),
            capacity.capacity_label()
        );
        shortcuts.push(StorageRootShortcut {
            id: format!("drive-{}", storage_root_id_part(&label)),
            label,
            path,
            kind: StorageRootKind::Drive,
            capacity: Some(capacity),
            available: true,
            tooltip,
        });
        if shortcuts.len() >= limit {
            break;
        }
    }
}

fn known_root_shortcut(
    kind: StorageRootKind,
    label: &'static str,
    id: &'static str,
    env_names: &[&str],
    fallbacks: &[PathBuf],
) -> StorageRootShortcut {
    let configured = env_names.iter().find_map(valid_env_dir_path);
    let path = configured
        .or_else(|| fallbacks.iter().find(|path| path.is_dir()).cloned())
        .or_else(|| fallbacks.first().cloned())
        .unwrap_or_default();
    let available = path.is_absolute() && path.is_dir();
    let tooltip = if available {
        format!("Open {label} at {}", path.display())
    } else {
        format!(
            "{label} folder not found. Set {} or use Open Project.",
            env_names.first().copied().unwrap_or("the matching root")
        )
    };

    StorageRootShortcut {
        id: id.to_string(),
        label: label.to_string(),
        path,
        kind,
        capacity: None,
        available,
        tooltip,
    }
}

fn env_path(name: &&str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn valid_env_dir_path(name: &&str) -> Option<PathBuf> {
    env_path(name).filter(|path| path.is_absolute() && path.is_dir())
}

fn user_profile_fallbacks(children: &[&str]) -> Vec<PathBuf> {
    let Some(profile) = env::var_os("USERPROFILE").map(PathBuf::from) else {
        return Vec::new();
    };

    children.iter().map(|child| profile.join(child)).collect()
}

fn dedupe_and_cap(shortcuts: Vec<StorageRootShortcut>) -> Vec<StorageRootShortcut> {
    let mut seen = HashSet::new();
    let mut retained = Vec::new();

    for shortcut in shortcuts {
        let key = storage_root_key(&shortcut.path, &shortcut.id);
        if seen.insert(key) {
            retained.push(shortcut);
        }
        if retained.len() >= MAX_PROJECT_PANEL_STORAGE_ROOT_STRIP_ITEMS {
            break;
        }
    }

    retained
}

fn storage_root_key(path: &Path, fallback: &str) -> String {
    if path.as_os_str().is_empty() {
        return fallback.to_string();
    }

    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

fn drive_label(path: &Path) -> String {
    let display = path.display().to_string();
    let label = display.trim_end_matches(['\\', '/']);
    if label.is_empty() {
        display
    } else {
        label.to_string()
    }
}

fn storage_root_id_part(label: &str) -> String {
    let mut id = String::new();
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        }
    }
    if id.is_empty() {
        "root".to_string()
    } else {
        id
    }
}
