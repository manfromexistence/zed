use super::DxAgentCatalogSummary;

pub(crate) fn catalog_cache_state_label(summary: &DxAgentCatalogSummary) -> &'static str {
    match (summary.present, summary.stale) {
        (true, false) => "fast cache ready",
        (true, true) => "cache stale",
        (false, _) => "cache missing",
    }
}

pub(crate) fn catalog_detail_label(summary: &DxAgentCatalogSummary) -> String {
    format!(
        "{} cataloged providers, {} configured, {} enabled candidates, {} models, {}, receipt {}",
        summary.provider_count,
        summary.configured_provider_count,
        summary.enabled_provider_count,
        summary.model_count,
        catalog_cache_state_label(summary),
        catalog_receipt_status_label(&summary.receipt_status)
    )
}

pub(crate) fn catalog_receipt_status_label(status: &str) -> String {
    let status = status.trim();
    if status.is_empty() {
        return "unknown".to_string();
    }

    match status {
        "missing_receipt_root" => "receipt root missing".to_string(),
        "waiting_for_provider_receipt" => "waiting for provider receipt".to_string(),
        status => status.replace('_', " "),
    }
}

#[cfg(test)]
#[path = "catalog_labels_tests.rs"]
mod catalog_labels_tests;
