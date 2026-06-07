use gpui::AnyElement;

use crate::dx_agent_bridge::{
    DxAgentBridgeSnapshot, catalog_active_provider_value_label, catalog_cache_state_label,
    catalog_detail_label, catalog_receipt_status_label,
};

use super::super::super::metric_row;
use super::{VISIBLE_MODEL_ROW_LIMIT, VISIBLE_PROVIDER_ROW_LIMIT};

pub(super) fn dx_agent_provider_summary_rows(snapshot: &DxAgentBridgeSnapshot) -> Vec<AnyElement> {
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
        metric_row(
            "Provider rows shown",
            visible_row_count_label(snapshot.providers.len(), VISIBLE_PROVIDER_ROW_LIMIT),
        ),
        metric_row("Catalog models", snapshot.catalog.model_count.to_string()),
        metric_row(
            "Model rows shown",
            visible_row_count_label(snapshot.models.len(), VISIBLE_MODEL_ROW_LIMIT),
        ),
        metric_row(
            "Catalog active provider",
            catalog_active_provider_value_label(&snapshot.catalog, &snapshot.providers),
        ),
        metric_row(
            "Receipt status",
            catalog_receipt_status_label(&snapshot.catalog.receipt_status),
        ),
        metric_row("Catalog path", snapshot.catalog.path.display().to_string()),
        metric_row("Fast cache", catalog_cache_state_label(&snapshot.catalog)),
    ];

    if let Some(generated_at) = snapshot.catalog.generated_at.as_ref() {
        rows.push(metric_row("Generated at", generated_at.clone()));
    }

    rows
}

fn visible_row_count_label(total: usize, limit: usize) -> String {
    let visible = total.min(limit);
    if total > limit {
        format!("{visible} of {total}")
    } else {
        visible.to_string()
    }
}
