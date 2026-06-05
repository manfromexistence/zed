use super::DxSourceSetSnapshot;
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const SOURCE_SET_CACHE_TTL: Duration = Duration::from_secs(5);
static SOURCE_SET_CACHE: OnceLock<Mutex<Option<(Instant, Vec<String>, DxSourceSetSnapshot)>>> =
    OnceLock::new();

pub(super) fn cached_source_set_snapshot(
    workspace_roots: &[String],
) -> Option<DxSourceSetSnapshot> {
    let cache = SOURCE_SET_CACHE.get_or_init(|| Mutex::new(None));
    let Ok(cache) = cache.lock() else {
        return None;
    };
    let Some((cached_at, cached_roots, snapshot)) = cache.as_ref() else {
        return None;
    };
    if cached_roots == workspace_roots && cached_at.elapsed() <= SOURCE_SET_CACHE_TTL {
        Some(snapshot.clone())
    } else {
        None
    }
}

pub(super) fn store_source_set_snapshot(
    workspace_roots: &[String],
    snapshot: &DxSourceSetSnapshot,
) {
    let cache = SOURCE_SET_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        *cache = Some((Instant::now(), workspace_roots.to_vec(), snapshot.clone()));
    }
}

pub(crate) fn invalidate_source_set_snapshot_cache() {
    let cache = SOURCE_SET_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        *cache = None;
    }
}
