use std::path::PathBuf;

use super::super::DxAgentCatalogSummary;
use super::{catalog_detail_label, catalog_receipt_status_label};

fn catalog_summary() -> DxAgentCatalogSummary {
    DxAgentCatalogSummary {
        path: PathBuf::from("G:\\Dx\\.dx\\catalog\\agents\\provider-model-catalog.rkyv"),
        present: false,
        stale: true,
        provider_count: 67,
        model_count: 0,
        generated_at: Some("2026-05-22T07:09:39.822686+00:00".to_string()),
        receipt_status: "warning".to_string(),
        configured_provider_count: 0,
        enabled_provider_count: 10,
        active_provider_id: Some("nvidia".to_string()),
        source_hash: Some("sha256:provider-catalog".to_string()),
        error: None,
        safe_regeneration_command: "dx agents providers catalog regenerate --json".to_string(),
    }
}

#[test]
fn catalog_detail_label_separates_catalog_from_readiness() {
    assert_eq!(
        catalog_detail_label(&catalog_summary()),
        "67 cataloged providers, 0 configured, 10 enabled candidates, 0 models, cache missing, receipt warning"
    );
}

#[test]
fn catalog_receipt_status_label_humanizes_waiting_states() {
    assert_eq!(
        catalog_receipt_status_label("waiting_for_provider_receipt"),
        "waiting for provider receipt"
    );
    assert_eq!(
        catalog_receipt_status_label(" waiting_for_provider_receipt "),
        "waiting for provider receipt"
    );
    assert_eq!(
        catalog_receipt_status_label("missing_receipt_root"),
        "receipt root missing"
    );
    assert_eq!(catalog_receipt_status_label("warning"), "warning");
    assert_eq!(catalog_receipt_status_label(""), "unknown");
    assert_eq!(catalog_receipt_status_label("  "), "unknown");
    assert_eq!(
        catalog_receipt_status_label("custom_status"),
        "custom status"
    );
}
