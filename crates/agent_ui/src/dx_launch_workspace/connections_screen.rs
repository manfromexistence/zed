use gpui::{AnyElement, App, IntoElement, SharedString};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, ListItem, ListItemSpacing,
    prelude::*,
};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::{DxLaunchWorkspaceStatus, agents, metric_row, muted_card, section_title};

pub(crate) fn render_connections_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    cx: &mut App,
) -> AnyElement {
    let body = if let Some(status) = status {
        let snapshot = &status.agent_bridge;
        v_flex()
            .gap_2()
            .child(section_title("Providers", dx_icon(DxUiIcon::Gateway)))
            .child(agents::dx_agent_provider_state(snapshot, cx))
            .child(section_title("Channels", dx_icon(DxUiIcon::Channels)))
            .child(channel_state(snapshot, cx))
            .child(section_title("Social", dx_icon(DxUiIcon::Connections)))
            .child(agents::dx_agent_social_state(snapshot, cx))
            .child(section_title("Gateway", dx_icon(DxUiIcon::Gateway)))
            .child(gateway_state(snapshot))
            .child(section_title("Credentials", dx_icon(DxUiIcon::Credentials)))
            .child(credential_state(snapshot))
            .into_any_element()
    } else {
        muted_card("Loading provider, social, and credential receipts", cx)
    };

    div()
        .id("dx-connections-screen")
        .size_full()
        .min_w_0()
        .overflow_y_scroll()
        .bg(cx.theme().colors().panel_background)
        .child(
            v_flex()
                .gap_3()
                .p_4()
                .child(screen_header(
                    dx_icon(DxUiIcon::Connections),
                    "Connections",
                    "Providers, channels, social accounts, gateway readiness, and credential health.",
                ))
                .child(body),
        )
        .into_any_element()
}

fn channel_state(snapshot: &DxAgentBridgeSnapshot, cx: &App) -> AnyElement {
    v_flex()
        .gap_1()
        .child(metric_row(
            "Supported",
            snapshot.connected_accounts_summary.supported.to_string(),
        ))
        .child(metric_row(
            "Configured",
            snapshot.connected_accounts_summary.configured.to_string(),
        ))
        .child(metric_row(
            "Connected",
            snapshot.connected_accounts_summary.connected.to_string(),
        ))
        .child(metric_row(
            "Needs auth",
            snapshot.connected_accounts_summary.needs_auth.to_string(),
        ))
        .child(unavailable_row(
            "dx-connections-channels-schema",
            dx_icon(DxUiIcon::Channels),
            "Channel receipts",
            "Unavailable",
            "No DX Agents channel receipt/schema is available yet.",
            "Run social list receipts until the channel contract lands.",
        ))
        .when(snapshot.social_accounts.is_empty(), |stack| {
            stack.child(muted_card("Run social list receipt", cx))
        })
        .into_any_element()
}

fn gateway_state(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    v_flex()
        .gap_1()
        .child(metric_row("Bridge", snapshot.status.clone()))
        .child(metric_row(
            "Provider catalog",
            snapshot.contract_summary.provider_catalog_source.clone(),
        ))
        .child(metric_row(
            "Catalog receipts",
            snapshot
                .contract_summary
                .provider_catalog_receipt_count
                .to_string(),
        ))
        .child(unavailable_row(
            "dx-connections-gateway-health",
            dx_icon(DxUiIcon::Gateway),
            "Provider gateway",
            "Unavailable",
            "No first-class provider gateway health receipt is available yet.",
            snapshot.contract_summary.next_action.clone(),
        ))
        .into_any_element()
}

fn credential_state(snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
    let provider_error_count = snapshot
        .providers
        .iter()
        .filter(|provider| provider.credential_error.is_some())
        .count();
    let social_error_count = snapshot
        .social_accounts
        .iter()
        .filter(|account| account.credential_error.is_some())
        .count();
    let expiry_count = snapshot
        .providers
        .iter()
        .filter(|provider| provider.credential_expires_at.is_some())
        .count()
        + snapshot
            .social_accounts
            .iter()
            .filter(|account| account.credential_expires_at.is_some())
            .count();

    let status = if provider_error_count + social_error_count > 0 {
        AiSettingItemStatus::Error
    } else if snapshot.providers.is_empty() && snapshot.social_accounts.is_empty() {
        AiSettingItemStatus::Stopped
    } else {
        AiSettingItemStatus::Running
    };

    AiSettingItem::new(
        "dx-connections-credential-health",
        "Credential health",
        status,
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Credentials))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(match status {
        AiSettingItemStatus::Running => "Healthy",
        AiSettingItemStatus::Error => "Needs attention",
        _ => "Receipt required",
    })
    .details(connection_detail_stack(vec![
        connection_detail_row(
            "dx-connections-credential-provider-errors".into(),
            IconName::Warning,
            "Provider errors",
            provider_error_count.to_string(),
        ),
        connection_detail_row(
            "dx-connections-credential-social-errors".into(),
            IconName::Warning,
            "Social errors",
            social_error_count.to_string(),
        ),
        connection_detail_row(
            "dx-connections-credential-expiry".into(),
            IconName::Clock,
            "Expiry rows",
            expiry_count.to_string(),
        ),
        connection_detail_row(
            "dx-connections-credential-action".into(),
            IconName::FileTextOutlined,
            "Next action",
            snapshot.receipt_index.next_action.clone(),
        ),
    ]))
    .into_any_element()
}

fn unavailable_row(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    detail_label: &'static str,
    state: impl Into<SharedString>,
    next_action: impl Into<SharedString>,
) -> AnyElement {
    AiSettingItem::new(
        id,
        label,
        AiSettingItemStatus::Stopped,
        AiSettingItemSource::Custom,
    )
    .icon(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
    .detail_label(detail_label)
    .details(connection_detail_stack(vec![
        connection_detail_row(
            format!("{id}-state").into(),
            IconName::Warning,
            "State",
            state,
        ),
        connection_detail_row(
            format!("{id}-action").into(),
            IconName::FileTextOutlined,
            "Next action",
            next_action,
        ),
    ]))
    .into_any_element()
}

fn screen_header(icon: IconName, title: &'static str, detail: &'static str) -> AnyElement {
    h_flex()
        .items_start()
        .gap_2()
        .child(Icon::new(icon).size(IconSize::Medium).color(Color::Muted))
        .child(
            v_flex()
                .gap_0p5()
                .child(
                    Label::new(title)
                        .size(LabelSize::Default)
                        .color(Color::Default),
                )
                .child(
                    Label::new(detail)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
        .into_any_element()
}

fn connection_detail_stack(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().gap_1().children(rows).into_any_element()
}

fn connection_detail_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(label.into())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(detail.into())
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .into_any_element()
}
