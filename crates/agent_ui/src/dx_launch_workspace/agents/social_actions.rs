use gpui::{AnyElement, App, SharedString};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, Color, prelude::*};

use crate::dx_agent_bridge::DxAgentSocialActionSummary;

use super::connection_rows::{connection_detail_row, connection_detail_stack};

pub(super) fn dx_agent_social_action_row(
    id: SharedString,
    receipt: &DxAgentSocialActionSummary,
    _cx: &App,
) -> AnyElement {
    let connected = if receipt.connected.unwrap_or(false) {
        "connected"
    } else {
        "not connected"
    };
    let detail = if receipt.action == "connect" {
        let support = if receipt.connect_supported {
            "supported"
        } else {
            "unsupported"
        };
        let qr = if receipt.qr_supported {
            "QR supported"
        } else {
            "QR unavailable"
        };
        let link = if receipt.link_supported {
            "link supported"
        } else {
            "link unavailable"
        };
        format!(
            "{} connect {}, via {}, {}, {}, {}",
            receipt.label, support, receipt.connect_method, qr, link, connected
        )
    } else {
        let support = if receipt.disconnect_supported {
            "supported"
        } else {
            "not needed"
        };
        let revoke = if receipt.manual_revoke_required {
            "provider revoke"
        } else {
            "no revoke"
        };
        format!(
            "{} disconnect {}, {}, {}, config {}",
            receipt.label, support, revoke, connected, receipt.safe_config_state
        )
    };

    AiSettingItem::new(
        id.clone(),
        format!("Last {}", receipt.action),
        social_action_status(receipt),
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Connections))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(receipt.status.clone())
    .details(connection_detail_stack(vec![
        connection_detail_row(
            format!("{}-detail", id).into(),
            dx_icon(DxUiIcon::Gateway),
            "Receipt",
            detail,
        )
        .into_any_element(),
        connection_detail_row(
            format!("{}-next-action", id).into(),
            dx_icon(DxUiIcon::Commands),
            "Next",
            receipt.next_action.clone(),
        )
        .into_any_element(),
    ]))
    .into_any_element()
}

fn social_action_status(receipt: &DxAgentSocialActionSummary) -> AiSettingItemStatus {
    if receipt.status.eq_ignore_ascii_case("error") {
        AiSettingItemStatus::Error
    } else if receipt.connected.unwrap_or(false) {
        AiSettingItemStatus::Running
    } else if receipt.explicit_user_action_required
        || receipt.qr_supported
        || receipt.link_supported
    {
        AiSettingItemStatus::AuthRequired
    } else {
        AiSettingItemStatus::Stopped
    }
}
