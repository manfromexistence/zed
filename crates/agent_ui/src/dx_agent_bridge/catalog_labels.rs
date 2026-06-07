use super::{DxAgentCatalogSummary, DxAgentProvider};

pub(crate) fn catalog_cache_state_label(summary: &DxAgentCatalogSummary) -> &'static str {
    if summary.present && !summary.stale {
        "fast cache ready"
    } else if summary.present {
        "cache stale"
    } else {
        "cache missing"
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
        summary.receipt_status
    )
}

pub(crate) fn catalog_active_provider_label(
    summary: &DxAgentCatalogSummary,
    providers: &[DxAgentProvider],
) -> String {
    let Some(active_provider_id) = summary.active_provider_id.as_deref() else {
        return "No active DX provider".to_string();
    };
    let Some(provider) = providers
        .iter()
        .find(|provider| provider.id == active_provider_id)
    else {
        return format!("Active provider: {active_provider_id}");
    };

    if provider.display_name == active_provider_id {
        format!("Active provider: {active_provider_id}")
    } else {
        format!(
            "Active provider: {} ({active_provider_id})",
            provider.display_name
        )
    }
}

#[cfg(test)]
#[path = "catalog_labels_tests.rs"]
mod tests;
