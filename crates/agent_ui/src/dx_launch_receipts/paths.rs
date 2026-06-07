use super::DX_LAUNCH_STATUS_PREFIX;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_LAUNCH_SNAPSHOT_DIR_ENTRIES: usize = 512;
const MAX_LAUNCH_SNAPSHOT_PATHS: usize = 128;

pub(super) struct LaunchSnapshotPaths {
    pub(super) paths: Vec<PathBuf>,
    pub(super) scan_truncated: bool,
}

pub(super) fn launch_snapshot_paths(root: &Path) -> LaunchSnapshotPaths {
    let Ok(entries) = fs::read_dir(root) else {
        return LaunchSnapshotPaths {
            paths: Vec::new(),
            scan_truncated: false,
        };
    };

    let mut candidates = Vec::new();
    let mut scan_truncated = false;
    for (index, entry) in entries.flatten().enumerate() {
        if index >= MAX_LAUNCH_SNAPSHOT_DIR_ENTRIES {
            scan_truncated = true;
            break;
        }

        let path = entry.path();
        if !is_launch_snapshot_path(&path) {
            continue;
        }

        let Some(order_ms) = receipt_order_ms(&path) else {
            continue;
        };
        candidates.push((order_ms, path));
    }

    candidates.sort_by(|(left_ms, left_path), (right_ms, right_path)| {
        right_ms
            .cmp(left_ms)
            .then_with(|| file_name(right_path).cmp(&file_name(left_path)))
    });

    if candidates.len() > MAX_LAUNCH_SNAPSHOT_PATHS {
        scan_truncated = true;
    }

    LaunchSnapshotPaths {
        paths: candidates
            .into_iter()
            .take(MAX_LAUNCH_SNAPSHOT_PATHS)
            .map(|(_, path)| path)
            .collect(),
        scan_truncated,
    }
}

fn is_launch_snapshot_path(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.starts_with(DX_LAUNCH_STATUS_PREFIX) && name.ends_with(".json")
            })
}

pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}

pub(super) fn receipt_order_ms(path: &Path) -> Option<u64> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix(DX_LAUNCH_STATUS_PREFIX))
        .and_then(|suffix| suffix.parse::<u64>().ok())
}

pub(super) fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}
