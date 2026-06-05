use crate::{DxCatalogError, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) const DEFAULT_AUTH_PROFILE_MAX_BYTES: u64 = 1024 * 1024;
pub(crate) const DEFAULT_MODEL_CATALOG_MAX_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const DEFAULT_PROVIDER_ARCHIVE_MAX_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const DEFAULT_PROVIDER_METADATA_SIDECAR_MAX_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) fn read_to_string_with_limit(path: &Path, max_bytes: u64) -> Result<String> {
    ensure_file_with_limit(path, max_bytes)?;

    Ok(fs::read_to_string(path)?)
}

pub(crate) fn ensure_file_with_limit(path: &Path, max_bytes: u64) -> Result<u64> {
    let len = fs::metadata(path)?.len();
    if len > max_bytes {
        return Err(DxCatalogError::FileTooLarge {
            path: path.to_path_buf(),
            len,
            max_len: max_bytes,
        });
    }

    Ok(len)
}

pub(crate) fn file_too_large_reason(path: &PathBuf, len: u64, max_len: u64) -> String {
    format!(
        "{} is {len} byte(s), which exceeds the configured {max_len} byte read limit",
        path.display()
    )
}
