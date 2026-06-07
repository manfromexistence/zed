use gpui::{AnyElement, App, SharedString};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, Color, prelude::*};

use crate::dx_agent_bridge::DxAgentSocialAccount;

use super::super::actions::dx_agent_action_line;
use super::super::connection_rows::{connection_detail_row, connection_detail_stack};

pub(super) fn dx_agent_social_row(
    id: SharedString,
    account: &DxAgentSocialAccount,
    _cx: &App,
) -> AnyElement {
    let mut details = vec![
        connection_detail_row(
            format!("{}-provider", id).into(),
            dx_icon(DxUiIcon::Connections),
            "Provider",
            format!(
                "{} / {} - {}",
                account.provider_id, account.platform, account.status
            ),
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-auth", id).into(),
            dx_icon(DxUiIcon::Credentials),
            "Auth",
            format!(
                "{} / credential {}",
                account.auth_method, account.credential_health
            ),
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-qr", id).into(),
            IconName::Public,
            "QR login",
            social_qr_detail(account),
        )
        .into_any_element(),
    ];

    if let Some(expires_at) = account.credential_expires_at.as_ref() {
        details.push(
            connection_detail_row(
                format!("{}-credential-expiry", id).into(),
                IconName::Clock,
                "Expiry",
                format!("Credential expires {expires_at}"),
            )
            .into_any_element(),
        );
    }

    if let Some(error) = account.credential_error.as_ref() {
        details.push(
            connection_detail_row(
                format!("{}-credential-error", id).into(),
                IconName::Warning,
                "Credential issue",
                error.clone(),
            )
            .into_any_element(),
        );
    }

    if !account.receipt_history.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-receipts", id).into(),
                dx_icon(DxUiIcon::Receipts),
                "Receipts",
                account
                    .receipt_history
                    .iter()
                    .take(3)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .into_any_element(),
        );
    }

    if !account.next_action.is_empty() {
        details.push(
            connection_detail_row(
                format!("{}-next-action", id).into(),
                dx_icon(DxUiIcon::Commands),
                "Next",
                account.next_action.clone(),
            )
            .into_any_element(),
        );
    }

    if let Some(action_line) = dx_agent_action_line(&account.actions) {
        details.push(
            connection_detail_row(
                format!("{}-actions", id).into(),
                dx_icon(DxUiIcon::Permissions),
                "Actions",
                action_line,
            )
            .into_any_element(),
        );
    }

    AiSettingItem::new(
        id,
        account.label.clone(),
        social_account_status(account),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Connections))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(format!(
        "{} - {}",
        account.account_state, account.credential_health
    ))
    .details(connection_detail_stack(details))
    .into_any_element()
}

fn social_account_status(account: &DxAgentSocialAccount) -> AiSettingItemStatus {
    if account.credential_error.is_some() {
        AiSettingItemStatus::Error
    } else if account.connected {
        AiSettingItemStatus::Running
    } else if account.configured || account.qr_connect_supported {
        AiSettingItemStatus::AuthRequired
    } else {
        AiSettingItemStatus::Stopped
    }
}

fn social_qr_detail(account: &DxAgentSocialAccount) -> String {
    if account.qr_connect_supported {
        format!(
            "{}; login payload pending DX Agents receipt",
            account.qr_capability
        )
    } else {
        format!("{}; QR flow unavailable", account.qr_capability)
    }
}
