use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use super::package_status::scan_package_status_rows;
use super::snapshot::DxForgeSourceRow;

const PACKAGE_STATUS_CACHE_TTL: Duration = Duration::from_secs(5);

static PACKAGE_STATUS_CACHE: OnceLock<
    Mutex<Option<(Instant, Vec<String>, Vec<DxForgeSourceRow>)>>,
> = OnceLock::new();

pub(super) fn package_status_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    let cache = PACKAGE_STATUS_CACHE.get_or_init(|| Mutex::new(None));
    let now = Instant::now();

    if let Ok(mut cache) = cache.lock() {
        if let Some((cached_at, cached_roots, rows)) = cache.as_ref()
            && cached_roots == workspace_roots
            && now.duration_since(*cached_at) <= PACKAGE_STATUS_CACHE_TTL
        {
            return rows.clone();
        }

        let rows = scan_package_status_rows(workspace_roots);
        *cache = Some((now, workspace_roots.to_vec(), rows.clone()));
        return rows;
    }

    scan_package_status_rows(workspace_roots)
}

pub(super) fn invalidate_package_status_snapshot_cache() {
    let cache = PACKAGE_STATUS_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        *cache = None;
    }
}
