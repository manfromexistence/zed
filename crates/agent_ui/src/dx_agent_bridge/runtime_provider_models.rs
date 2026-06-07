use serde_json::Value;

use super::super::{DxAgentModel, DxAgentProvider};
use super::{array_field, bool_field, string_array_field, string_field, usize_field};

const MAX_RUNTIME_ROWS: usize = 24;
const MAX_PROVIDER_ALIASES: usize = 4;

pub(super) fn providers(value: &Value) -> Vec<DxAgentProvider> {
    let root_status = nonblank_string_field(value, &["status"]);
    array_field(value, &["providers"])
        .map(|providers| {
            providers
                .iter()
                .take(MAX_RUNTIME_ROWS)
                .map(|provider| provider_row(provider, root_status.as_deref()))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn models(value: &Value) -> Vec<DxAgentModel> {
    let flat_models = flat_model_rows(value);
    if !flat_models.is_empty() {
        return flat_models;
    }

    grouped_model_rows(value)
}

fn provider_row(provider: &Value, root_status: Option<&str>) -> DxAgentProvider {
    DxAgentProvider {
        id: nonblank_string_field(provider, &["id"]).unwrap_or_else(|| "provider".to_string()),
        display_name: nonblank_string_field(provider, &["display_name"])
            .unwrap_or_else(|| "Provider".to_string()),
        status: nonblank_string_field(provider, &["status"])
            .or_else(|| root_status.map(ToString::to_string))
            .unwrap_or_else(|| "unknown".to_string()),
        account_state: nonblank_string_field(provider, &["account_state"]).unwrap_or_else(|| {
            if bool_field(provider, &["configured"]).unwrap_or(false) {
                "configured".to_string()
            } else {
                "missing_auth".to_string()
            }
        }),
        auth_method: nonblank_string_field(provider, &["auth_method"])
            .or_else(|| nonblank_string_field(provider, &["auth"]))
            .unwrap_or_else(|| "unknown".to_string()),
        credential_health: nonblank_string_field(provider, &["credential_health"])
            .unwrap_or_else(|| "unknown".to_string()),
        credential_expires_at: nonblank_string_field(provider, &["credential_expires_at"]),
        credential_error: nonblank_string_field(provider, &["credential_error"])
            .or_else(|| nonblank_string_field(provider, &["catalog_error"])),
        qr_connect_supported: bool_field(provider, &["qr_connect_supported"]).unwrap_or(false),
        configured: bool_field(provider, &["configured"]).unwrap_or(false),
        active: bool_field(provider, &["active"]).unwrap_or(false),
        local: bool_field(provider, &["local"]).unwrap_or(false),
        compatibility: provider_detail_tags(provider),
    }
}

fn flat_model_rows(value: &Value) -> Vec<DxAgentModel> {
    array_field(value, &["models"])
        .map(|models| {
            models
                .iter()
                .take(MAX_RUNTIME_ROWS)
                .map(|model| DxAgentModel {
                    id: nonblank_string_field(model, &["id"])
                        .unwrap_or_else(|| "model".to_string()),
                    provider_id: nonblank_string_field(model, &["provider_id"])
                        .unwrap_or_else(|| "provider".to_string()),
                    model_id: nonblank_string_field(model, &["model_id"])
                        .unwrap_or_else(|| "model".to_string()),
                    status: nonblank_string_field(model, &["status"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    active: bool_field(model, &["active"]).unwrap_or(false),
                    compatibility: string_array_field(model, &["compatibility"]),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn grouped_model_rows(value: &Value) -> Vec<DxAgentModel> {
    let root_status = nonblank_string_field(value, &["status"]);
    let Some(providers) = array_field(value, &["providers"]) else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    for provider in providers.iter().take(MAX_RUNTIME_ROWS) {
        let provider_id =
            nonblank_string_field(provider, &["id"]).unwrap_or_else(|| "provider".to_string());
        let provider_status = nonblank_string_field(provider, &["status"])
            .or_else(|| root_status.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let compatibility = provider_model_tags(provider);
        let Some(models) = array_field(provider, &["models"]) else {
            continue;
        };

        for model in models {
            if rows.len() >= MAX_RUNTIME_ROWS {
                return rows;
            }
            let Some(model_id) = grouped_model_id(model) else {
                continue;
            };
            let id = model_row_id(&provider_id, &model_id);
            rows.push(DxAgentModel {
                id,
                provider_id: provider_id.clone(),
                model_id,
                status: nonblank_string_field(model, &["status"])
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
        .map(ToString::to_string)
        .or_else(|| nonblank_string_field(model, &["model_id"]))
        .or_else(|| nonblank_string_field(model, &["id"]))
}

fn model_row_id(provider_id: &str, model_id: &str) -> String {
    if model_id.contains('/') {
        model_id.to_string()
    } else {
        format!("{provider_id}/{model_id}")
    }
}

fn provider_detail_tags(provider: &Value) -> Vec<String> {
    let mut tags = string_array_field(provider, &["compatibility"]);
    tags.extend(provider_model_tags(provider));
    tags.extend(
        string_array_field(provider, &["configured_aliases"])
            .into_iter()
            .filter(|alias| !alias.trim().is_empty())
            .take(MAX_PROVIDER_ALIASES)
            .map(|alias| format!("alias {}", alias.trim())),
    );
    tags
}

fn provider_model_tags(provider: &Value) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(source) = nonblank_string_field(provider, &["source"]) {
        tags.push(format!("source {source}"));
    }
    if let Some(model_count) = usize_field(provider, &["model_count"]) {
        tags.push(model_count_label(model_count));
    }
    tags
}

fn model_compatibility(model: &Value, provider_tags: &[String]) -> Vec<String> {
    let row_tags = string_array_field(model, &["compatibility"]);
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

fn nonblank_string_field(value: &Value, path: &[&str]) -> Option<String> {
    string_field(value, path)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
