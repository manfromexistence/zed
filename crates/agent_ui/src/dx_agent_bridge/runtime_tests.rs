use std::path::PathBuf;

use serde_json::json;

use super::{catalog_summary, models, providers};

#[test]
fn provider_rows_read_agent_cli_provider_receipts() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.providers_list.v1",
        "status": "ok",
        "providers": [{
            "id": "deepseek",
            "display_name": "DeepSeek",
            "local": false,
            "source": "dx_providers_catalog",
            "model_count": 2,
            "configured": true,
            "configured_aliases": ["deepseek-chat"]
        }]
    });

    let rows = providers(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "deepseek");
    assert_eq!(rows[0].display_name, "DeepSeek");
    assert_eq!(rows[0].status, "ok");
    assert!(rows[0].configured);
    assert!(!rows[0].local);
    assert_eq!(
        rows[0].compatibility,
        vec![
            "source dx_providers_catalog".to_string(),
            "2 models".to_string(),
            "alias deepseek-chat".to_string()
        ]
    );
}

#[test]
fn model_rows_flatten_agent_cli_provider_model_groups() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.models_list.v1",
        "status": "partial",
        "providers": [
            {
                "id": "deepseek",
                "display_name": "DeepSeek",
                "source": "dx_providers_catalog",
                "model_count": 2,
                "models": ["deepseek-chat", "deepseek-reasoner"],
                "catalog_error": null
            },
            {
                "id": "missing-provider",
                "display_name": "missing-provider",
                "source": "native",
                "model_count": 0,
                "models": [],
                "catalog_error": "provider was not found in the provider catalog"
            }
        ]
    });

    let rows = models(&receipt);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "deepseek/deepseek-chat");
    assert_eq!(rows[0].provider_id, "deepseek");
    assert_eq!(rows[0].model_id, "deepseek-chat");
    assert_eq!(rows[0].status, "partial");
    assert!(!rows[0].active);
    assert_eq!(
        rows[0].compatibility,
        vec![
            "source dx_providers_catalog".to_string(),
            "2 models".to_string()
        ]
    );
    assert_eq!(rows[1].id, "deepseek/deepseek-reasoner");
}

#[test]
fn legacy_flat_model_rows_still_parse() {
    let receipt = json!({
        "models": [{
            "id": "openai/gpt-5",
            "provider_id": "openai",
            "model_id": "gpt-5",
            "status": "ready",
            "active": true,
            "compatibility": ["tools"]
        }]
    });

    let rows = models(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "openai/gpt-5");
    assert_eq!(rows[0].provider_id, "openai");
    assert_eq!(rows[0].model_id, "gpt-5");
    assert_eq!(rows[0].status, "ready");
    assert!(rows[0].active);
    assert_eq!(rows[0].compatibility, vec!["tools".to_string()]);
}

#[test]
fn catalog_summary_reads_agent_cli_catalog_diagnostics() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.providers_list.v1",
        "generated_at": "2026-05-22T07:09:39.822686+00:00",
        "status": "warning",
        "configured_provider_count": 3,
        "enabled_provider_count": 10,
        "active_provider_id": "openai",
        "catalog": {
            "loaded": false,
            "path": "G:\\Dx\\.dx\\catalog\\agents\\provider-model-catalog.rkyv",
            "provider_count": 75,
            "model_count": 1200,
            "error": "provider catalog file not found"
        }
    });

    let summary = catalog_summary(
        Some(&receipt),
        None,
        PathBuf::from("G:\\fallback\\provider-model-catalog.rkyv"),
        true,
    );

    assert_eq!(
        summary.path,
        PathBuf::from("G:\\Dx\\.dx\\catalog\\agents\\provider-model-catalog.rkyv")
    );
    assert!(!summary.present);
    assert!(summary.stale);
    assert_eq!(summary.provider_count, 75);
    assert_eq!(summary.model_count, 1200);
    assert_eq!(
        summary.generated_at.as_deref(),
        Some("2026-05-22T07:09:39.822686+00:00")
    );
    assert_eq!(summary.receipt_status, "warning");
    assert_eq!(summary.configured_provider_count, 3);
    assert_eq!(summary.enabled_provider_count, 10);
    assert_eq!(summary.active_provider_id.as_deref(), Some("openai"));
    assert_eq!(
        summary.error.as_deref(),
        Some("provider catalog file not found")
    );
}

#[test]
fn catalog_summary_derives_provider_honesty_from_provider_rows() {
    let providers_receipt = json!({
        "schema_version": "dx.agents.zed.providers_list.v1",
        "provider_count": 75,
        "providers": [
            {"id": "openai", "configured": true, "enabled": true, "active": false},
            {"id": "ollama", "configured": false, "enabled": true, "active": true},
            {"id": "nvidia", "configured": false, "enabled": false, "active": false}
        ]
    });
    let models_receipt = json!({
        "schema_version": "dx.agents.zed.models_list.v1",
        "status": "partial",
        "provider_count": 70,
        "model_count": 1200,
        "providers": []
    });

    let summary = catalog_summary(
        Some(&providers_receipt),
        Some(&models_receipt),
        PathBuf::from("G:\\fallback\\provider-model-catalog.rkyv"),
        true,
    );

    assert_eq!(summary.provider_count, 75);
    assert_eq!(summary.model_count, 1200);
    assert_eq!(summary.receipt_status, "partial");
    assert_eq!(summary.configured_provider_count, 1);
    assert_eq!(summary.enabled_provider_count, 2);
    assert_eq!(summary.active_provider_id.as_deref(), Some("ollama"));
}

#[test]
fn catalog_summary_reports_missing_receipt_root() {
    let summary = catalog_summary(
        None,
        None,
        PathBuf::from("G:\\fallback\\provider-model-catalog.rkyv"),
        false,
    );

    assert_eq!(summary.receipt_status, "missing_receipt_root");
    assert_eq!(summary.configured_provider_count, 0);
    assert_eq!(summary.enabled_provider_count, 0);
    assert!(summary.generated_at.is_none());
    assert!(summary.active_provider_id.is_none());
}
