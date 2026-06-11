use gpui::AnyElement;
use ui::{AiSettingItemStatus, IconName, prelude::*};

use crate::dx_agent_bridge::{DxAgentBridgeSnapshot, DxAgentProvider, DxAgentSocialAccount};

use super::ConnectionCatalogFilter;
use super::details::{
    channels_details, credential_details, gateway_details, provider_details,
    receipt_authority_details, social_details, trusted_tool_bridge_details,
};
use super::status::{
    bridge_status, credential_health_label, credential_issue_count, credential_status,
    provider_detail_label, provider_needs_auth, provider_status, social_detail_label,
    social_needs_auth, social_status,
};

#[derive(Clone, Copy)]
pub(super) enum ConnectionCatalogEntry {
    ReceiptAuthority,
    TrustedToolBridge,
    Channels,
    Gateway,
    CredentialHealth,
    Provider(usize),
    SocialAccount(usize),
}

pub(super) fn all_connection_entries(
    snapshot: &DxAgentBridgeSnapshot,
) -> Vec<ConnectionCatalogEntry> {
    let mut entries = vec![
        ConnectionCatalogEntry::ReceiptAuthority,
        ConnectionCatalogEntry::TrustedToolBridge,
        ConnectionCatalogEntry::Channels,
        ConnectionCatalogEntry::Gateway,
        ConnectionCatalogEntry::CredentialHealth,
    ];
    entries.extend(
        snapshot
            .providers
            .iter()
            .enumerate()
            .map(|(index, _)| ConnectionCatalogEntry::Provider(index)),
    );
    entries.extend(
        snapshot
            .social_accounts
            .iter()
            .enumerate()
            .map(|(index, _)| ConnectionCatalogEntry::SocialAccount(index)),
    );
    entries
}

impl ConnectionCatalogEntry {
    pub(super) fn id(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => "receipt-authority".to_string(),
            ConnectionCatalogEntry::TrustedToolBridge => "trusted-tool-bridge".to_string(),
            ConnectionCatalogEntry::Channels => "channels".to_string(),
            ConnectionCatalogEntry::Gateway => "gateway".to_string(),
            ConnectionCatalogEntry::CredentialHealth => "credential-health".to_string(),
            ConnectionCatalogEntry::Provider(index) => snapshot
                .providers
                .get(index)
                .map(|provider| format!("provider-{}", provider.id))
                .unwrap_or_else(|| format!("provider-{index}")),
            ConnectionCatalogEntry::SocialAccount(index) => snapshot
                .social_accounts
                .get(index)
                .map(|account| format!("social-{}", account.platform))
                .unwrap_or_else(|| format!("social-{index}")),
        }
    }

    pub(super) fn icon(self) -> IconName {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => dx_icon(DxUiIcon::Receipts),
            ConnectionCatalogEntry::TrustedToolBridge => dx_icon(DxUiIcon::Permissions),
            ConnectionCatalogEntry::Channels => dx_icon(DxUiIcon::Channels),
            ConnectionCatalogEntry::Gateway => dx_icon(DxUiIcon::Gateway),
            ConnectionCatalogEntry::CredentialHealth => dx_icon(DxUiIcon::Credentials),
            ConnectionCatalogEntry::Provider(_) => dx_icon(DxUiIcon::Gateway),
            ConnectionCatalogEntry::SocialAccount(_) => dx_icon(DxUiIcon::Connections),
        }
    }

    pub(super) fn title(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => "Receipt authority".to_string(),
            ConnectionCatalogEntry::TrustedToolBridge => "Trusted tool bridge".to_string(),
            ConnectionCatalogEntry::Channels => "Channels".to_string(),
            ConnectionCatalogEntry::Gateway => "Provider gateway".to_string(),
            ConnectionCatalogEntry::CredentialHealth => "Credential health".to_string(),
            ConnectionCatalogEntry::Provider(index) => snapshot
                .providers
                .get(index)
                .map(|provider| provider.display_name.clone())
                .unwrap_or_else(|| "Provider".to_string()),
            ConnectionCatalogEntry::SocialAccount(index) => snapshot
                .social_accounts
                .get(index)
                .map(|account| account.label.clone())
                .unwrap_or_else(|| "Social account".to_string()),
        }
    }

    pub(super) fn detail_label(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => snapshot.receipt_index.status.clone(),
            ConnectionCatalogEntry::TrustedToolBridge => {
                snapshot.trusted_tool_bridge.status.clone()
            }
            ConnectionCatalogEntry::Channels => format!(
                "{} supported / {} needs auth",
                snapshot.connected_accounts_summary.supported,
                snapshot.connected_accounts_summary.needs_auth
            ),
            ConnectionCatalogEntry::Gateway => snapshot.status.clone(),
            ConnectionCatalogEntry::CredentialHealth => credential_health_label(snapshot),
            ConnectionCatalogEntry::Provider(index) => snapshot
                .providers
                .get(index)
                .map(provider_detail_label)
                .unwrap_or_else(|| "missing provider".to_string()),
            ConnectionCatalogEntry::SocialAccount(index) => snapshot
                .social_accounts
                .get(index)
                .map(social_detail_label)
                .unwrap_or_else(|| "missing account".to_string()),
        }
    }

    pub(super) fn status(self, snapshot: &DxAgentBridgeSnapshot) -> AiSettingItemStatus {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => {
                if snapshot.root_exists && snapshot.receipt_index.present {
                    AiSettingItemStatus::Running
                } else {
                    AiSettingItemStatus::Stopped
                }
            }
            ConnectionCatalogEntry::TrustedToolBridge => {
                bridge_status(&snapshot.trusted_tool_bridge)
            }
            ConnectionCatalogEntry::Channels => {
                if snapshot.connected_accounts_summary.needs_auth > 0 {
                    AiSettingItemStatus::AuthRequired
                } else if snapshot.connected_accounts_summary.supported > 0 {
                    AiSettingItemStatus::Running
                } else {
                    AiSettingItemStatus::Stopped
                }
            }
            ConnectionCatalogEntry::Gateway => {
                if snapshot.last_error.is_some() {
                    AiSettingItemStatus::Error
                } else if snapshot.enabled {
                    AiSettingItemStatus::Running
                } else {
                    AiSettingItemStatus::Stopped
                }
            }
            ConnectionCatalogEntry::CredentialHealth => credential_status(snapshot),
            ConnectionCatalogEntry::Provider(index) => snapshot
                .providers
                .get(index)
                .map(provider_status)
                .unwrap_or(AiSettingItemStatus::Stopped),
            ConnectionCatalogEntry::SocialAccount(index) => snapshot
                .social_accounts
                .get(index)
                .map(social_status)
                .unwrap_or(AiSettingItemStatus::Stopped),
        }
    }

    pub(super) fn detail_body(self, snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
        match self {
            ConnectionCatalogEntry::ReceiptAuthority => receipt_authority_details(snapshot),
            ConnectionCatalogEntry::TrustedToolBridge => trusted_tool_bridge_details(snapshot),
            ConnectionCatalogEntry::Channels => channels_details(snapshot),
            ConnectionCatalogEntry::Gateway => gateway_details(snapshot),
            ConnectionCatalogEntry::CredentialHealth => credential_details(snapshot),
            ConnectionCatalogEntry::Provider(index) => snapshot
                .providers
                .get(index)
                .map(provider_details)
                .unwrap_or_else(|| v_flex().into_any_element()),
            ConnectionCatalogEntry::SocialAccount(index) => snapshot
                .social_accounts
                .get(index)
                .map(social_details)
                .unwrap_or_else(|| v_flex().into_any_element()),
        }
    }

    pub(super) fn matches_filter(
        self,
        snapshot: &DxAgentBridgeSnapshot,
        filter: ConnectionCatalogFilter,
    ) -> bool {
        match filter {
            ConnectionCatalogFilter::All => true,
            ConnectionCatalogFilter::Connected => match self {
                ConnectionCatalogEntry::Provider(index) => snapshot
                    .providers
                    .get(index)
                    .is_some_and(|provider| provider.active || provider.configured),
                ConnectionCatalogEntry::SocialAccount(index) => snapshot
                    .social_accounts
                    .get(index)
                    .is_some_and(|account| account.connected),
                ConnectionCatalogEntry::ReceiptAuthority => snapshot.root_exists,
                _ => false,
            },
            ConnectionCatalogFilter::NeedsAuth => match self {
                ConnectionCatalogEntry::Provider(index) => snapshot
                    .providers
                    .get(index)
                    .is_some_and(provider_needs_auth),
                ConnectionCatalogEntry::SocialAccount(index) => snapshot
                    .social_accounts
                    .get(index)
                    .is_some_and(social_needs_auth),
                ConnectionCatalogEntry::Channels => {
                    snapshot.connected_accounts_summary.needs_auth > 0
                }
                ConnectionCatalogEntry::CredentialHealth => credential_issue_count(snapshot) > 0,
                _ => false,
            },
            ConnectionCatalogFilter::Receipts => match self {
                ConnectionCatalogEntry::ReceiptAuthority
                | ConnectionCatalogEntry::TrustedToolBridge
                | ConnectionCatalogEntry::Gateway
                | ConnectionCatalogEntry::CredentialHealth => true,
                ConnectionCatalogEntry::SocialAccount(index) => snapshot
                    .social_accounts
                    .get(index)
                    .is_some_and(|account| !account.receipt_history.is_empty()),
                ConnectionCatalogEntry::Provider(_) | ConnectionCatalogEntry::Channels => false,
            },
        }
    }

    pub(super) fn matches_search(self, snapshot: &DxAgentBridgeSnapshot, query: &str) -> bool {
        let mut haystack = vec![self.title(snapshot), self.detail_label(snapshot)];
        if let Some(provider) = self.provider(snapshot) {
            haystack.extend([
                provider.id.clone(),
                provider.status.clone(),
                provider.account_state.clone(),
                provider.auth_method.clone(),
                provider.credential_health.clone(),
            ]);
        }
        if let Some(account) = self.social_account(snapshot) {
            haystack.extend([
                account.provider_id.clone(),
                account.platform.clone(),
                account.status.clone(),
                account.account_state.clone(),
                account.auth_method.clone(),
                account.credential_health.clone(),
                account.next_action.clone(),
            ]);
        }
        haystack
            .iter()
            .any(|value| value.to_lowercase().contains(query))
    }

    fn provider<'a>(self, snapshot: &'a DxAgentBridgeSnapshot) -> Option<&'a DxAgentProvider> {
        match self {
            ConnectionCatalogEntry::Provider(index) => snapshot.providers.get(index),
            _ => None,
        }
    }

    fn social_account<'a>(
        self,
        snapshot: &'a DxAgentBridgeSnapshot,
    ) -> Option<&'a DxAgentSocialAccount> {
        match self {
            ConnectionCatalogEntry::SocialAccount(index) => snapshot.social_accounts.get(index),
            _ => None,
        }
    }
}
