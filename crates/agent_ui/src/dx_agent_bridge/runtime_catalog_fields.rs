use serde_json::Value;

use super::runtime_display::display_string_field;
use super::{array_field, bool_field, usize_field};

pub(super) struct CatalogHonestyFields {
    pub generated_at: Option<String>,
    pub receipt_status: String,
    pub configured_provider_count: usize,
    pub enabled_provider_count: usize,
    pub active_provider_id: Option<String>,
}

pub(super) fn catalog_honesty_fields(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    catalog: Option<&Value>,
    root_exists: bool,
) -> CatalogHonestyFields {
    CatalogHonestyFields {
        generated_at: generated_at(provider_value, model_value, catalog),
        receipt_status: receipt_status(provider_value, model_value, root_exists),
        configured_provider_count: readiness_count(
            provider_value,
            model_value,
            "configured_provider_count",
            "configured",
        ),
        enabled_provider_count: readiness_count(
            provider_value,
            model_value,
            "enabled_provider_count",
            "enabled",
        ),
        active_provider_id: active_provider_id(provider_value, model_value),
    }
}

fn generated_at(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    catalog: Option<&Value>,
) -> Option<String> {
    first_receipt_string(provider_value, model_value, &["generated_at"])
        .or_else(|| catalog.and_then(|catalog| display_string_field(catalog, &["generated_at"])))
        .or_else(|| {
            catalog
                .and_then(|catalog| display_string_field(catalog, &["binary_cache_generated_at"]))
        })
}

fn receipt_status(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    root_exists: bool,
) -> String {
    if provider_value.is_none() {
        return receipt_unavailable_status(root_exists).to_string();
    }

    first_receipt_string(provider_value, model_value, &["status"])
        .unwrap_or_else(|| receipt_unavailable_status(root_exists).to_string())
}

fn receipt_unavailable_status(root_exists: bool) -> &'static str {
    match root_exists {
        true => "waiting_for_provider_receipt",
        false => "missing_receipt_root",
    }
}

fn readiness_count(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    receipt_field: &str,
    row_field: &str,
) -> usize {
    provider_value
        .and_then(|value| usize_field(value, &[receipt_field]))
        .or_else(|| model_value.and_then(|value| usize_field(value, &[receipt_field])))
        .or_else(|| provider_value.and_then(|value| count_provider_rows(value, row_field)))
        .or_else(|| model_value.and_then(|value| count_provider_rows(value, row_field)))
        .unwrap_or_default()
}

fn active_provider_id(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
) -> Option<String> {
    first_receipt_string(provider_value, model_value, &["active_provider_id"])
        .or_else(|| first_receipt_string(provider_value, model_value, &["active_provider", "id"]))
        .or_else(|| provider_value.and_then(active_provider_row_id))
        .or_else(|| model_value.and_then(active_provider_row_id))
}

fn active_provider_row_id(value: &Value) -> Option<String> {
    array_field(value, &["providers"])?
        .iter()
        .find(|provider| bool_field(provider, &["active"]).unwrap_or(false))
        .and_then(|provider| display_string_field(provider, &["id"]))
}

fn count_provider_rows(value: &Value, row_field: &str) -> Option<usize> {
    Some(
        array_field(value, &["providers"])?
            .iter()
            .filter(|provider| bool_field(provider, &[row_field]).unwrap_or(false))
            .count(),
    )
}

fn first_receipt_string(
    provider_value: Option<&Value>,
    model_value: Option<&Value>,
    path: &[&str],
) -> Option<String> {
    provider_value
        .and_then(|value| display_string_field(value, path))
        .or_else(|| model_value.and_then(|value| display_string_field(value, path)))
}
