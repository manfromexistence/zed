use gpui::{AnyElement, SharedString};
use ui::{IconName, prelude::*};

use crate::dx_agent_bridge::{DxAgentBridgeSnapshot, DxAgentProvider, DxAgentSocialAccount};

use super::super::super::screen_chrome::{screen_detail_row, screen_detail_stack};
use super::status::{
    bounded_join, credential_expiry_count, credential_health_label, credential_issue_count, yes_no,
};

pub(super) fn receipt_authority_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let latest = snapshot
        .latest_receipts
        .first()
        .cloned()
        .or_else(|| snapshot.receipt_index.latest_receipt_path.clone())
        .unwrap_or_else(|| "No latest receipt".to_string());

    screen_detail_stack(vec![
        detail(
            "receipt-status",
            dx_icon(DxUiIcon::Receipts),
            "Status",
            snapshot.receipt_index.status.clone(),
        ),
        detail(
            "receipt-root",
            IconName::Folder,
            "Root",
            snapshot.receipt_root.display().to_string(),
        ),
        detail(
            "receipt-root-exists",
            IconName::Check,
            "Root exists",
            yes_no(snapshot.root_exists),
        ),
        detail(
            "receipt-count",
            dx_icon(DxUiIcon::Receipts),
            "Indexed receipts",
            snapshot.receipt_index.receipt_count.to_string(),
        ),
        detail(
            "receipt-returned",
            dx_icon(DxUiIcon::Receipts),
            "Returned receipts",
            snapshot.receipt_index.returned_receipt_count.to_string(),
        ),
        detail(
            "receipt-latest",
            IconName::FileTextOutlined,
            "Latest",
            latest,
        ),
        detail(
            "receipt-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.receipt_index.next_action.clone(),
        ),
    ])
}

pub(super) fn trusted_tool_bridge_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let bridge = &snapshot.trusted_tool_bridge;
    screen_detail_stack(vec![
        detail(
            "trusted-status",
            dx_icon(DxUiIcon::Permissions),
            "Trusted tool bridge",
            bridge.status.clone(),
        ),
        detail(
            "trusted-policy",
            dx_icon(DxUiIcon::Permissions),
            "Trust policy",
            bridge.trust_policy.clone(),
        ),
        detail(
            "trusted-plugin-tools",
            dx_icon(DxUiIcon::Plugins),
            "Approved plugin tools",
            bridge.approved_plugin_tool_count.to_string(),
        ),
        detail(
            "trusted-automation-tools",
            dx_icon(DxUiIcon::Automations),
            "Approved automation tools",
            bridge.approved_automation_tool_count.to_string(),
        ),
        detail(
            "trusted-blocked",
            IconName::Warning,
            "Blocked tools",
            bridge.blocked_tool_count.to_string(),
        ),
        detail(
            "trusted-receipts",
            dx_icon(DxUiIcon::Receipts),
            "Bridge receipts",
            bridge.receipt_count.to_string(),
        ),
        detail(
            "trusted-contract",
            IconName::FileTextOutlined,
            "Contract",
            bridge.bridge_contract_id.clone(),
        ),
        detail(
            "trusted-tools",
            dx_icon(DxUiIcon::Plugins),
            "Trusted tools",
            bounded_join(&bridge.trusted_tool_ids, 4),
        ),
        detail(
            "trusted-action",
            IconName::ArrowRight,
            "Next action",
            bridge.next_action.clone(),
        ),
    ])
}

pub(super) fn channels_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    screen_detail_stack(vec![
        detail(
            "channels-supported",
            dx_icon(DxUiIcon::Channels),
            "Supported",
            snapshot.connected_accounts_summary.supported.to_string(),
        ),
        detail(
            "channels-configured",
            dx_icon(DxUiIcon::Credentials),
            "Configured",
            snapshot.connected_accounts_summary.configured.to_string(),
        ),
        detail(
            "channels-connected",
            dx_icon(DxUiIcon::Connections),
            "Connected",
            snapshot.connected_accounts_summary.connected.to_string(),
        ),
        detail(
            "channels-needs-auth",
            IconName::Warning,
            "Needs auth",
            snapshot.connected_accounts_summary.needs_auth.to_string(),
        ),
        detail(
            "channels-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.social_connect.next_action.clone(),
        ),
    ])
}

pub(super) fn gateway_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    screen_detail_stack(vec![
        detail(
            "gateway-status",
            dx_icon(DxUiIcon::Gateway),
            "Bridge",
            snapshot.status.clone(),
        ),
        detail(
            "gateway-source",
            IconName::FileTextOutlined,
            "Provider source",
            snapshot.contract_summary.provider_catalog_source.clone(),
        ),
        detail(
            "gateway-receipts",
            dx_icon(DxUiIcon::Receipts),
            "Catalog receipts",
            snapshot
                .contract_summary
                .provider_catalog_receipt_count
                .to_string(),
        ),
        detail(
            "gateway-catalog",
            dx_icon(DxUiIcon::Gateway),
            "Catalog status",
            snapshot.catalog.receipt_status.clone(),
        ),
        detail(
            "gateway-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.contract_summary.next_action.clone(),
        ),
    ])
}

pub(super) fn credential_details(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    screen_detail_stack(vec![
        detail(
            "credentials-summary",
            dx_icon(DxUiIcon::Credentials),
            "Credential health",
            credential_health_label(snapshot),
        ),
        detail(
            "credentials-issues",
            IconName::Warning,
            "Issue rows",
            credential_issue_count(snapshot).to_string(),
        ),
        detail(
            "credentials-expiry",
            IconName::Clock,
            "Expiry rows",
            credential_expiry_count(snapshot).to_string(),
        ),
        detail(
            "credentials-action",
            IconName::ArrowRight,
            "Next action",
            snapshot.receipt_index.next_action.clone(),
        ),
    ])
}

pub(super) fn provider_details(provider: &DxAgentProvider) -> AnyElement {
    screen_detail_stack(vec![
        provider_detail(
            provider,
            "status",
            dx_icon(DxUiIcon::Gateway),
            "Status",
            provider.status.clone(),
        ),
        provider_detail(
            provider,
            "account",
            dx_icon(DxUiIcon::Connections),
            "Account",
            provider.account_state.clone(),
        ),
        provider_detail(
            provider,
            "auth",
            dx_icon(DxUiIcon::Credentials),
            "Auth",
            provider.auth_method.clone(),
        ),
        provider_detail(
            provider,
            "credential",
            dx_icon(DxUiIcon::Credentials),
            "Credential",
            provider.credential_health.clone(),
        ),
        provider_detail(
            provider,
            "active",
            IconName::Check,
            "Active",
            yes_no(provider.active),
        ),
        provider_detail(
            provider,
            "local",
            IconName::FileTextOutlined,
            "Local",
            yes_no(provider.local),
        ),
        provider_detail(
            provider,
            "compatibility",
            dx_icon(DxUiIcon::Plugins),
            "Compatibility",
            bounded_join(&provider.compatibility, 4),
        ),
    ])
}

pub(super) fn social_details(account: &DxAgentSocialAccount) -> AnyElement {
    screen_detail_stack(vec![
        social_detail(
            account,
            "platform",
            dx_icon(DxUiIcon::Connections),
            "Platform",
            account.platform.clone(),
        ),
        social_detail(
            account,
            "status",
            dx_icon(DxUiIcon::Connections),
            "Status",
            account.status.clone(),
        ),
        social_detail(
            account,
            "account",
            dx_icon(DxUiIcon::Connections),
            "Account",
            account.account_state.clone(),
        ),
        social_detail(
            account,
            "auth",
            dx_icon(DxUiIcon::Credentials),
            "Auth",
            account.auth_method.clone(),
        ),
        social_detail(
            account,
            "qr",
            dx_icon(DxUiIcon::Channels),
            "QR capability",
            account.qr_capability.clone(),
        ),
        social_detail(
            account,
            "credential",
            dx_icon(DxUiIcon::Credentials),
            "Credential",
            account.credential_health.clone(),
        ),
        social_detail(
            account,
            "receipts",
            dx_icon(DxUiIcon::Receipts),
            "Receipt history",
            bounded_join(&account.receipt_history, 3),
        ),
        social_detail(
            account,
            "action",
            IconName::ArrowRight,
            "Next action",
            account.next_action.clone(),
        ),
    ])
}

fn detail(
    suffix: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
) -> AnyElement {
    screen_detail_row(
        format!("dx-connections-detail-{suffix}").into(),
        icon,
        label,
        value,
    )
}

fn provider_detail(
    provider: &DxAgentProvider,
    suffix: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
) -> AnyElement {
    screen_detail_row(
        format!("dx-connections-provider-{}-{suffix}", provider.id).into(),
        icon,
        label,
        value,
    )
}

fn social_detail(
    account: &DxAgentSocialAccount,
    suffix: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
) -> AnyElement {
    screen_detail_row(
        format!("dx-connections-social-{}-{suffix}", account.platform).into(),
        icon,
        label,
        value,
    )
}
