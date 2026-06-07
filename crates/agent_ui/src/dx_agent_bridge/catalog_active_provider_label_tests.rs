use std::path::PathBuf;

use super::super::{DxAgentCatalogSummary, DxAgentProvider};
use super::{catalog_active_provider_label, catalog_active_provider_value_label};

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

fn provider(display_name: &str) -> DxAgentProvider {
    DxAgentProvider {
        id: "nvidia".to_string(),
        display_name: display_name.to_string(),
        status: "not_configured".to_string(),
        configured: false,
        active: false,
        local: false,
        compatibility: vec!["remote".to_string()],
    }
}

#[test]
fn catalog_active_provider_label_prefers_display_name() {
    let providers = vec![provider("NVIDIA NIM")];

    assert_eq!(
        catalog_active_provider_label(&catalog_summary(), &providers),
        "Active provider: NVIDIA NIM (nvidia)"
    );
    assert_eq!(
        catalog_active_provider_value_label(&catalog_summary(), &providers),
        "NVIDIA NIM (nvidia)"
    );
}

#[test]
fn catalog_active_provider_label_ignores_blank_display_name() {
    let providers = vec![provider("  ")];

    assert_eq!(
        catalog_active_provider_label(&catalog_summary(), &providers),
        "Active provider: nvidia"
    );
    assert_eq!(
        catalog_active_provider_value_label(&catalog_summary(), &providers),
        "nvidia"
    );
}

#[test]
fn catalog_active_provider_label_treats_blank_active_id_as_empty_state() {
    let mut summary = catalog_summary();
    summary.active_provider_id = Some("  ".to_string());

    assert_eq!(
        catalog_active_provider_label(&summary, &[]),
        "Active provider: none"
    );
    assert_eq!(catalog_active_provider_value_label(&summary, &[]), "none");
}

#[test]
fn catalog_active_provider_label_reports_known_empty_state() {
    let mut summary = catalog_summary();
    summary.active_provider_id = None;

    assert_eq!(
        catalog_active_provider_label(&summary, &[]),
        "Active provider: none"
    );
    assert_eq!(catalog_active_provider_value_label(&summary, &[]), "none");
}

#[test]
fn catalog_active_provider_label_waits_for_receipt_before_claiming_none() {
    let mut summary = catalog_summary();
    summary.receipt_status = "waiting_for_provider_receipt".to_string();
    summary.active_provider_id = None;

    assert_eq!(
        catalog_active_provider_label(&summary, &[]),
        "Active provider unknown until provider receipt loads"
    );
    assert_eq!(
        catalog_active_provider_value_label(&summary, &[]),
        "unknown until provider receipt loads"
    );
}
