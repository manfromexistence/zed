use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::prelude::*;

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use self::rows::dx_agent_social_row;
use super::super::screen_chrome::{screen_detail_row, screen_empty_state};
use super::connection_rows::connection_unavailable_rows;
use super::social_actions::dx_agent_social_action_row;

mod rows;

pub(in super::super) fn dx_agent_social_state(
    snapshot: &DxAgentBridgeSnapshot,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex()
        .gap_1()
        .child(screen_detail_row(
            "dx-agent-social-supported".into(),
            dx_icon(DxUiIcon::Connections),
            "Supported",
            snapshot.connected_accounts_summary.supported.to_string(),
        ))
        .child(screen_detail_row(
            "dx-agent-social-needs-auth".into(),
            dx_icon(DxUiIcon::Credentials),
            "Needs auth",
            snapshot.connected_accounts_summary.needs_auth.to_string(),
        ))
        .child(screen_detail_row(
            "dx-agent-social-qr-ready".into(),
            dx_icon(DxUiIcon::Channels),
            "QR-ready",
            snapshot
                .connected_accounts_summary
                .qr_connect_supported
                .to_string(),
        ))
        .children(connection_unavailable_rows());

    if snapshot.social_accounts.is_empty() {
        stack = stack.child(screen_empty_state(
            "dx-agent-social-empty",
            dx_icon(DxUiIcon::Receipts),
            "Run social list receipt",
            cx,
        ));
    } else {
        for (ix, account) in snapshot.social_accounts.iter().take(3).enumerate() {
            stack = stack.child(dx_agent_social_row(
                SharedString::from(format!("dx-agent-social-{ix}")),
                account,
                cx,
            ));
        }
    }

    if snapshot.social_connect.present {
        stack = stack.child(dx_agent_social_action_row(
            SharedString::from("dx-agent-social-connect-receipt"),
            &snapshot.social_connect,
            cx,
        ));
    }

    if snapshot.social_disconnect.present {
        stack = stack.child(dx_agent_social_action_row(
            SharedString::from("dx-agent-social-disconnect-receipt"),
            &snapshot.social_disconnect,
            cx,
        ));
    }

    stack.into_any_element()
}
