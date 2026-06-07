use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

const DX_ICON_INDEX_ENV: &str = "DX_ICON_INDEX";
const DX_ICON_DATA_ENV: &str = "DX_ICON_DATA";
const DX_ICON_ROOT_ENV: &str = "DX_ICON_ROOT";
const LEGACY_DX_ICONS_DATA_DIR_ENV: &str = "DX_ICONS_DATA_DIR";
const DX_HOME_ENV: &str = "DX_HOME";
const USERPROFILE_ENV: &str = "USERPROFILE";
const SHARED_DX_ICON_ROOT: &str = "G:/Dx/icon";

pub fn dx_icon_data_dir() -> PathBuf {
    dx_icon_data_dir_candidates()
        .into_iter()
        .find_map(|candidate| resolve_dx_icon_data_dir(candidate.as_path()))
        .unwrap_or_else(|| PathBuf::from(SHARED_DX_ICON_ROOT).join("data"))
}

fn dx_icon_data_dir_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    push_env_path(&mut candidates, DX_ICON_INDEX_ENV);
    push_env_path(&mut candidates, DX_ICON_DATA_ENV);
    push_env_path(&mut candidates, DX_ICON_ROOT_ENV);
    push_env_path(&mut candidates, LEGACY_DX_ICONS_DATA_DIR_ENV);

    if let Some(dx_home) = env_path(DX_HOME_ENV) {
        candidates.push(dx_home.join("icon"));
    }

    candidates.push(PathBuf::from(SHARED_DX_ICON_ROOT));

    if let Some(user_profile) = env_path(USERPROFILE_ENV) {
        candidates.push(user_profile.join(".dx").join("icon"));
    }

    candidates
}

fn push_env_path(candidates: &mut Vec<PathBuf>, name: &str) {
    if let Some(path) = env_path(name) {
        candidates.push(path);
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    let value = std::env::var_os(name)?;
    let path = PathBuf::from(value);
    (!path.as_os_str().is_empty()).then_some(path)
}

fn resolve_dx_icon_data_dir(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_dir() {
        if candidate
            .file_name()
            .is_some_and(|name| name == OsStr::new("data"))
        {
            return Some(candidate.to_path_buf());
        }
        let nested_data = candidate.join("data");
        if nested_data.is_dir() {
            return Some(nested_data);
        }
        if candidate
            .file_name()
            .is_some_and(|name| name == OsStr::new("index"))
        {
            return candidate
                .parent()
                .map(|parent| parent.join("data"))
                .filter(|data| data.is_dir());
        }
    }

    candidate
        .parent()
        .filter(|parent| {
            parent
                .file_name()
                .is_some_and(|name| name == OsStr::new("index"))
        })
        .and_then(|index_dir| index_dir.parent())
        .map(|root| root.join("data"))
        .filter(|data| data.is_dir())
}
