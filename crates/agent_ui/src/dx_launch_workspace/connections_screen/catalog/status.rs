use ui::AiSettingItemStatus;

use crate::dx_agent_bridge::{
    DxAgentBridgeSnapshot, DxAgentProvider, DxAgentSocialAccount, DxAgentTrustedToolBridgeSummary,
};

pub(super) fn bridge_status(bridge: &DxAgentTrustedToolBridgeSummary) -> AiSettingItemStatus {
    if bridge.blocked_tool_count > 0 {
        AiSettingItemStatus::AuthRequired
    } else if bridge.present {
        AiSettingItemStatus::Running
    } else {
        AiSettingItemStatus::Stopped
    }
}

pub(super) fn provider_status(provider: &DxAgentProvider) -> AiSettingItemStatus {
    if provider.credential_error.is_some() {
        AiSettingItemStatus::Error
    } else if provider_needs_auth(provider) {
        AiSettingItemStatus::AuthRequired
    } else if provider.active || provider.configured {
        AiSettingItemStatus::Running
    } else {
        AiSettingItemStatus::Stopped
    }
}

pub(super) fn social_status(account: &DxAgentSocialAccount) -> AiSettingItemStatus {
    if account.credential_error.is_some() {
        AiSettingItemStatus::Error
    } else if social_needs_auth(account) {
        AiSettingItemStatus::AuthRequired
    } else if account.connected {
        AiSettingItemStatus::Running
    } else {
        AiSettingItemStatus::Stopped
    }
}

pub(super) fn credential_status(snapshot: &DxAgentBridgeSnapshot) -> AiSettingItemStatus {
    if credential_issue_count(snapshot) > 0 {
        AiSettingItemStatus::Error
    } else if snapshot.providers.is_empty() && snapshot.social_accounts.is_empty() {
        AiSettingItemStatus::Stopped
    } else {
        AiSettingItemStatus::Running
    }
}

pub(super) fn provider_needs_auth(provider: &DxAgentProvider) -> bool {
    !provider.configured
        || provider
            .credential_health
            .to_ascii_lowercase()
            .contains("auth")
}

pub(super) fn social_needs_auth(account: &DxAgentSocialAccount) -> bool {
    !account.connected
        || account
            .credential_health
            .to_ascii_lowercase()
            .contains("auth")
}

pub(super) fn provider_detail_label(provider: &DxAgentProvider) -> String {
    if provider.active {
        "active provider".to_string()
    } else if provider.configured {
        "configured provider".to_string()
    } else {
        provider.account_state.clone()
    }
}

pub(super) fn social_detail_label(account: &DxAgentSocialAccount) -> String {
    if account.connected {
        "connected account".to_string()
    } else if account.configured {
        "configured account".to_string()
    } else {
        account.account_state.clone()
    }
}

pub(super) fn credential_health_label(snapshot: &DxAgentBridgeSnapshot) -> String {
    format!(
        "{} issues / {} expiry rows",
        credential_issue_count(snapshot),
        credential_expiry_count(snapshot)
    )
}

pub(super) fn credential_issue_count(snapshot: &DxAgentBridgeSnapshot) -> usize {
    snapshot
        .providers
        .iter()
        .filter(|provider| provider.credential_error.is_some())
        .count()
        + snapshot
            .social_accounts
            .iter()
            .filter(|account| account.credential_error.is_some())
            .count()
}

pub(super) fn credential_expiry_count(snapshot: &DxAgentBridgeSnapshot) -> usize {
    snapshot
        .providers
        .iter()
        .filter(|provider| provider.credential_expires_at.is_some())
        .count()
        + snapshot
            .social_accounts
            .iter()
            .filter(|account| account.credential_expires_at.is_some())
            .count()
}

pub(super) fn bounded_join(values: &[String], limit: usize) -> String {
    if values.is_empty() {
        return "None".to_string();
    }

    let mut items = values.iter().take(limit).cloned().collect::<Vec<_>>();
    if values.len() > limit {
        items.push(format!("+{} more", values.len() - limit));
    }
    items.join(", ")
}

pub(super) fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
