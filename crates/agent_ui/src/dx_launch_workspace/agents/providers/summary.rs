use gpui::AnyElement;

use crate::dx_agent_bridge::{
    DxAgentBridgeSnapshot, catalog_active_provider_label, catalog_cache_state_label,
    catalog_detail_label,
};

use super::super::super::metric_row;

pub(super) fn dx_agent_provider_summary_rows(
    snapshot: &DxAgentBridgeSnapshot,
    model_count: usize,
) -> Vec<AnyElement> {
    let mut rows = vec![
        metric_row("Catalog summary", catalog_detail_label(&snapshot.catalog)),
        metric_row(
            "Catalog providers",
            snapshot.catalog.provider_count.to_string(),
        ),
        metric_row(
            "Configured providers",
            snapshot.catalog.configured_provider_count.to_string(),
        ),
        metric_row(
            "Enabled candidates",
            snapshot.catalog.enabled_provider_count.to_string(),
        ),
        metric_row("Provider rows shown", snapshot.providers.len().to_string()),
        metric_row("Catalog models", model_count.to_string()),
        metric_row("Model rows shown", snapshot.models.len().to_string()),
        metric_row(
            "Active provider",
            catalog_active_provider_label(&snapshot.catalog, &snapshot.providers),
        ),
        metric_row("Receipt status", snapshot.catalog.receipt_status.clone()),
        metric_row("Catalog path", snapshot.catalog.path.display().to_string()),
        metric_row("Fast cache", catalog_cache_state_label(&snapshot.catalog)),
    ];

    if let Some(generated_at) = snapshot.catalog.generated_at.as_ref() {
        rows.push(metric_row("Generated at", generated_at.clone()));
    }

    rows
}
