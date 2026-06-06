use std::path::PathBuf;

use serde_json::Value;

use super::super::DxAgentCatalogSummary;
use super::{bool_field, string_field, usize_field};

pub(super) fn catalog_summary(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    default_path: PathBuf,
) -> DxAgentCatalogSummary {
    let catalog = provider_value
        .and_then(|value| value.get("catalog"))
        .or_else(|| model_value.and_then(|value| value.get("catalog")));
    let path = catalog_path(catalog, default_path);
    let error = catalog.and_then(|catalog| nonblank_string_field(catalog, &["error"]));

    DxAgentCatalogSummary {
        present: catalog_present(catalog, &path),
        stale: catalog_stale(catalog, error.is_some()),
        provider_count: count_field(catalog, provider_value, model_value, "provider_count"),
        model_count: count_field(catalog, model_value, provider_value, "model_count"),
        source_hash: catalog.and_then(|catalog| nonblank_string_field(catalog, &["source_hash"])),
        error,
        safe_regeneration_command: catalog
            .and_then(|catalog| nonblank_string_field(catalog, &["safe_regeneration_command"]))
            .unwrap_or_else(|| "dx agents providers catalog regenerate --json".to_string()),
        path,
    }
}

fn count_field(
    catalog: Option<&Value>,
    primary_value: Option<&Value>,
    secondary_value: Option<&Value>,
    field: &str,
) -> usize {
    catalog
        .and_then(|catalog| usize_field(catalog, &[field]))
        .or_else(|| primary_value.and_then(|value| usize_field(value, &[field])))
        .or_else(|| secondary_value.and_then(|value| usize_field(value, &[field])))
        .unwrap_or_default()
}

fn catalog_path(catalog: Option<&Value>, default_path: PathBuf) -> PathBuf {
    catalog
        .and_then(|catalog| {
            nonblank_string_field(catalog, &["binary_cache_path"])
                .or_else(|| nonblank_string_field(catalog, &["path"]))
        })
        .map(PathBuf::from)
        .unwrap_or(default_path)
}

fn catalog_present(catalog: Option<&Value>, path: &PathBuf) -> bool {
    catalog
        .and_then(|catalog| {
            bool_field(catalog, &["binary_cache_present"])
                .or_else(|| bool_field(catalog, &["loaded"]))
        })
        .unwrap_or_else(|| path.is_file())
}

fn catalog_stale(catalog: Option<&Value>, has_error: bool) -> bool {
    catalog
        .and_then(|catalog| bool_field(catalog, &["binary_cache_stale"]))
        .unwrap_or_else(|| {
            has_error
                || catalog
                    .and_then(|catalog| bool_field(catalog, &["loaded"]))
                    .map_or(true, |loaded| !loaded)
        })
}

fn nonblank_string_field(value: &Value, path: &[&str]) -> Option<String> {
    string_field(value, path)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
