use {serde_json::Value, std::collections::HashSet};

use super::super::{DxAgentModel, DxAgentProvider};
use super::runtime_display::{
    dedupe_display_strings, display_string, display_string_array_field, display_string_field,
};
use super::{array_field, bool_field, usize_field};

const MAX_RUNTIME_ROWS: usize = 24;
const MAX_RUNTIME_CANDIDATES: usize = 256;
const MAX_PROVIDER_ALIASES: usize = 4;
const MAX_PROVIDER_TAGS: usize = 8;

pub(in super::super) fn providers(value: &Value) -> Vec<DxAgentProvider> {
    let root_status = display_string_field(value, &["status"]);
    array_field(value, &["providers"])
        .map(|providers| {
            let mut seen = HashSet::new();
            providers
                .iter()
                .take(MAX_RUNTIME_CANDIDATES)
                .filter_map(|provider| provider_row(provider, root_status.as_deref()))
                .filter(|provider| seen.insert(provider.id.clone()))
                .take(MAX_RUNTIME_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(in super::super) fn models(value: &Value) -> Vec<DxAgentModel> {
    let flat_models = flat_model_rows(value);
    if !flat_models.is_empty() {
        return flat_models;
    }

    grouped_model_rows(value)
}

fn provider_row(provider: &Value, root_status: Option<&str>) -> Option<DxAgentProvider> {
    let id = display_string_field(provider, &["id"])?;
    let display_name =
        display_string_field(provider, &["display_name"]).unwrap_or_else(|| id.clone());
    Some(DxAgentProvider {
        id,
        display_name,
        status: display_string_field(provider, &["status"])
            .or_else(|| root_status.map(ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string()),
        account_state: display_string_field(provider, &["account_state"]).unwrap_or_else(|| {
            if bool_field(provider, &["configured"]).unwrap_or(false) {
                "configured".to_string()
            } else {
                "missing_auth".to_string()
            }
        }),
        auth_method: display_string_field(provider, &["auth_method"])
            .or_else(|| display_string_field(provider, &["auth"]))
            .unwrap_or_else(|| "unknown".to_string()),
        credential_health: display_string_field(provider, &["credential_health"])
            .unwrap_or_else(|| "unknown".to_string()),
        credential_expires_at: display_string_field(provider, &["credential_expires_at"]),
        credential_error: display_string_field(provider, &["credential_error"])
            .or_else(|| display_string_field(provider, &["catalog_error"])),
        qr_connect_supported: bool_field(provider, &["qr_connect_supported"]).unwrap_or(false),
        configured: bool_field(provider, &["configured"]).unwrap_or(false),
        active: bool_field(provider, &["active"]).unwrap_or(false),
        local: bool_field(provider, &["local"]).unwrap_or(false),
        compatibility: provider_detail_tags(provider),
    })
}

fn flat_model_rows(value: &Value) -> Vec<DxAgentModel> {
    array_field(value, &["models"])
        .map(|models| {
            let mut seen = HashSet::new();
            models
                .iter()
                .take(MAX_RUNTIME_CANDIDATES)
                .filter_map(flat_model_row)
                .filter(|model| seen.insert(model.id.clone()))
                .take(MAX_RUNTIME_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

fn flat_model_row(model: &Value) -> Option<DxAgentModel> {
    let provider_id = display_string_field(model, &["provider_id"])?;
    let model_id = display_string_field(model, &["model_id"])?;
    let id = display_string_field(model, &["id"])
        .unwrap_or_else(|| model_row_id(&provider_id, &model_id));
    Some(DxAgentModel {
        id,
        provider_id,
        model_id,
        status: display_string_field(model, &["status"]).unwrap_or_else(|| "unknown".to_string()),
        active: bool_field(model, &["active"]).unwrap_or(false),
        compatibility: display_string_array_field(model, &["compatibility"], MAX_PROVIDER_TAGS),
    })
}

fn grouped_model_rows(value: &Value) -> Vec<DxAgentModel> {
    let root_status = display_string_field(value, &["status"]);
    let Some(providers) = array_field(value, &["providers"]) else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    for provider in providers.iter().take(MAX_RUNTIME_CANDIDATES) {
        let Some(provider_id) = display_string_field(provider, &["id"]) else {
            continue;
        };
        let provider_status = display_string_field(provider, &["status"])
            .or_else(|| root_status.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let compatibility = provider_model_tags(provider);
        let Some(models) = array_field(provider, &["models"]) else {
            continue;
        };

        for model in models.iter().take(MAX_RUNTIME_CANDIDATES) {
            if rows.len() >= MAX_RUNTIME_ROWS {
                return rows;
            }
            let Some(model_id) = grouped_model_id(model) else {
                continue;
            };
            let id = model_row_id(&provider_id, &model_id);
            if !seen.insert(id.clone()) {
                continue;
            }
            rows.push(DxAgentModel {
                id,
                provider_id: provider_id.clone(),
                model_id,
                status: display_string_field(model, &["status"])
                    .unwrap_or_else(|| provider_status.clone()),
                active: bool_field(model, &["active"]).unwrap_or(false),
                compatibility: model_compatibility(model, &compatibility),
            });
        }
    }

    rows
}

fn grouped_model_id(model: &Value) -> Option<String> {
    model
        .as_str()
        .and_then(|model| display_string(model.to_string()))
        .or_else(|| display_string_field(model, &["model_id"]))
        .or_else(|| display_string_field(model, &["id"]))
}

fn model_row_id(provider_id: &str, model_id: &str) -> String {
    if model_id.contains('/') {
        model_id.to_string()
    } else {
        format!("{provider_id}/{model_id}")
    }
}

fn provider_detail_tags(provider: &Value) -> Vec<String> {
    let mut tags = display_string_array_field(provider, &["compatibility"], MAX_PROVIDER_TAGS);
    tags.extend(provider_model_tags(provider));
    tags.extend(
        display_string_array_field(provider, &["configured_aliases"], MAX_PROVIDER_ALIASES)
            .into_iter()
            .map(|alias| format!("alias {alias}")),
    );
    dedupe_display_strings(tags, MAX_PROVIDER_TAGS)
}

fn provider_model_tags(provider: &Value) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(source) = display_string_field(provider, &["source"]) {
        tags.push(format!("source {source}"));
    }
    if let Some(model_count) = usize_field(provider, &["model_count"]) {
        tags.push(model_count_label(model_count));
    }
    tags
}

fn model_compatibility(model: &Value, provider_tags: &[String]) -> Vec<String> {
    let row_tags = display_string_array_field(model, &["compatibility"], MAX_PROVIDER_TAGS);
    if row_tags.is_empty() {
        provider_tags.to_vec()
    } else {
        row_tags
    }
}

fn model_count_label(model_count: usize) -> String {
    if model_count == 1 {
        "1 model".to_string()
    } else {
        format!("{model_count} models")
    }
}

#[cfg(test)]
#[path = "runtime_provider_models_tests.rs"]
mod tests;
