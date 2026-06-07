use std::path::PathBuf;

use super::super::{DxAgentCatalogSummary, DxAgentProvider};
use super::{catalog_active_provider_label, catalog_detail_label};

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
fn catalog_active_provider_label_prefers_display_name() {
    let providers = vec![DxAgentProvider {
        id: "nvidia".to_string(),
        display_name: "NVIDIA NIM".to_string(),
        status: "not_configured".to_string(),
        configured: false,
        active: false,
        local: false,
        compatibility: vec!["remote".to_string()],
    }];

    assert_eq!(
        catalog_active_provider_label(&catalog_summary(), &providers),
        "Active provider: NVIDIA NIM (nvidia)"
    );
}

#[test]
fn catalog_active_provider_label_reports_missing_active_provider() {
    let mut summary = catalog_summary();
    summary.active_provider_id = None;

    assert_eq!(
        catalog_active_provider_label(&summary, &[]),
        "No active DX provider"
    );
}
