use super::{DxAgentCatalogSummary, DxAgentProvider};

pub(crate) fn catalog_active_provider_label(
    summary: &DxAgentCatalogSummary,
    providers: &[DxAgentProvider],
) -> String {
    let value = catalog_active_provider_value_label(summary, providers);
    if value == "unknown until provider receipt loads" {
        "Active provider unknown until provider receipt loads".to_string()
    } else {
        format!("Active provider: {value}")
    }
}

pub(crate) fn catalog_active_provider_value_label(
    summary: &DxAgentCatalogSummary,
    providers: &[DxAgentProvider],
) -> String {
    let Some(active_provider_id) = summary
        .active_provider_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return empty_active_provider_value_label(summary);
    };
    let Some(provider) = providers
        .iter()
        .find(|provider| provider.id.trim() == active_provider_id)
    else {
        return active_provider_id.to_string();
    };

    let display_name = provider.display_name.trim();
    if display_name.is_empty() || display_name == active_provider_id {
        active_provider_id.to_string()
    } else {
        format!("{display_name} ({active_provider_id})")
    }
}

fn empty_active_provider_value_label(summary: &DxAgentCatalogSummary) -> String {
    if provider_receipt_pending(summary) {
        "unknown until provider receipt loads"
    } else {
        "none"
    }
    .to_string()
}

fn provider_receipt_pending(summary: &DxAgentCatalogSummary) -> bool {
    matches!(
        summary.receipt_status.trim(),
        "missing_receipt_root" | "waiting_for_provider_receipt"
    )
}

#[cfg(test)]
#[path = "catalog_active_provider_label_tests.rs"]
mod catalog_active_provider_label_tests;
