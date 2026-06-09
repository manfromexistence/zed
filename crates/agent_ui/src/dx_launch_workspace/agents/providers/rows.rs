use gpui::{AnyElement, App, SharedString};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, Color, ListItem, ListItemSpacing,
    prelude::*,
};

use crate::dx_agent_bridge::{DxAgentModel, DxAgentProvider};

use super::super::connection_rows::{connection_detail_row, connection_detail_stack};
use super::super::provider_labels::{
    model_detail_label, model_state_label, provider_detail_label, provider_state_label,
};

pub(super) fn dx_agent_provider_row(
    id: SharedString,
    provider: &DxAgentProvider,
    _cx: &App,
) -> AnyElement {
    let mut details = vec![
        connection_detail_row(
            format!("{}-provider-id", id).into(),
            dx_icon(DxUiIcon::Connections),
            "Provider",
            provider_detail_label(&provider.id, &provider.compatibility),
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-auth", id).into(),
            dx_icon(DxUiIcon::Credentials),
            "Auth",
            provider_connection_detail(provider),
        )
        .into_any_element(),
    ];

    if let Some(expires_at) = provider.credential_expires_at.as_ref() {
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

    if let Some(error) = provider.credential_error.as_ref() {
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

    AiSettingItem::new(
        id,
        provider.display_name.clone(),
        provider_status(provider),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Gateway))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(provider_state_label(
        provider.active,
        provider.configured,
        provider.local,
        &provider.status,
    ))
    .details(connection_detail_stack(details))
    .into_any_element()
}

pub(super) fn dx_agent_model_row(id: SharedString, model: &DxAgentModel, _cx: &App) -> AnyElement {
    ListItem::new(id)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(dx_icon(DxUiIcon::Ai))
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            v_flex()
                .min_w_0()
                .gap_0p5()
                .child(
                    h_flex()
                        .min_w_0()
                        .justify_between()
                        .gap_2()
                        .child(Label::new(model.model_id.clone()).size(LabelSize::Small))
                        .child(
                            Label::new(model_state_label(model.active, &model.status))
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                )
                .child(
                    Label::new(model_detail_label(
                        &model.provider_id,
                        &model.id,
                        &model.compatibility,
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate(),
                ),
        )
        .into_any_element()
}

fn provider_connection_detail(provider: &DxAgentProvider) -> String {
    let qr = if provider.qr_connect_supported {
        "QR capable; login payload pending receipt"
    } else {
        "QR unavailable"
    };
    format!(
        "Account {}, auth {}, credential {}, {}",
        provider.account_state, provider.auth_method, provider.credential_health, qr
    )
}

fn provider_status(provider: &DxAgentProvider) -> AiSettingItemStatus {
    if provider.credential_error.is_some() {
        AiSettingItemStatus::Error
    } else if provider.active || provider.local {
        AiSettingItemStatus::Running
    } else if provider.configured {
        AiSettingItemStatus::Stopped
    } else {
        AiSettingItemStatus::AuthRequired
    }
}
