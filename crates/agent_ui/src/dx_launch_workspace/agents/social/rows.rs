use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::{Color, prelude::*};

use crate::dx_agent_bridge::DxAgentSocialAccount;

use super::super::super::metric_row;
use super::super::actions::dx_agent_action_line;

pub(super) fn dx_agent_social_row(
    id: SharedString,
    account: &DxAgentSocialAccount,
    cx: &App,
) -> AnyElement {
    v_flex()
        .id(id)
        .gap_0p5()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .child(metric_row(
            account.label.clone(),
            account.account_state.clone(),
        ))
        .child(
            Label::new(format!(
                "{} / {} - {}",
                account.provider_id, account.platform, account.status
            ))
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .truncate(),
        )
        .child(
            Label::new(format!(
                "Auth {}, QR {}, credential {}",
                account.auth_method, account.qr_capability, account.credential_health
            ))
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .truncate(),
        )
        .when_some(
            account.credential_expires_at.as_ref(),
            |this, expires_at| {
                this.child(
                    Label::new(format!("Credential expires {expires_at}"))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                )
            },
        )
        .when_some(account.credential_error.as_ref(), |this, error| {
            this.child(
                Label::new(format!("Credential issue: {error}"))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when(!account.receipt_history.is_empty(), |this| {
            this.child(
                Label::new(format!(
                    "Receipts {}",
                    account
                        .receipt_history
                        .iter()
                        .take(3)
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
            )
        })
        .when(!account.next_action.is_empty(), |this| {
            this.child(
                Label::new(account.next_action.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when_some(
            dx_agent_action_line(&account.actions),
            |this, action_line| {
                this.child(
                    Label::new(action_line)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                )
            },
        )
        .into_any_element()
}
