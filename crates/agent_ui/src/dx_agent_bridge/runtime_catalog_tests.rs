use std::path::PathBuf;

use serde_json::json;

use super::catalog_summary;

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

#[test]
fn catalog_summary_reports_waiting_for_provider_receipt() {
    let summary = catalog_summary(
        None,
        None,
        PathBuf::from("G:\\fallback\\provider-model-catalog.rkyv"),
        true,
    );

    assert_eq!(summary.receipt_status, "waiting_for_provider_receipt");
    assert_eq!(summary.configured_provider_count, 0);
    assert!(summary.generated_at.is_none());
    assert!(summary.active_provider_id.is_none());
}
#[test]
fn catalog_summary_waits_for_provider_receipt_when_only_model_receipt_exists() {
    let models_receipt = json!({"status": "partial"});
    let summary = catalog_summary(
        None,
        Some(&models_receipt),
        PathBuf::from("G:\\fallback\\provider-model-catalog.rkyv"),
        true,
    );
    assert_eq!(summary.receipt_status, "waiting_for_provider_receipt");
}
