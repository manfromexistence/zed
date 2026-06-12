use serde_json::Value;

use self::runtime_display::{display_string_array_field, display_string_field};
use super::{
    DxAgentRowAction, DxAgentSocialAccount, DxAgentSocialActionSummary, DxConnectedAccountsSummary,
    array_field, bool_field, is_dx_agents_command, is_public_dx_agents_command,
    is_safe_platform_arg, is_secret_like_arg, public_command_for_runtime, string_field,
    usize_field,
};

#[path = "runtime_catalog.rs"]
mod runtime_catalog;
#[path = "runtime_catalog_fields.rs"]
mod runtime_catalog_fields;
#[path = "runtime_display.rs"]
mod runtime_display;
#[path = "runtime_provider_models.rs"]
mod runtime_provider_models;

pub(super) use self::{
    runtime_catalog::catalog_summary,
    runtime_provider_models::{models, providers},
};

pub(super) fn connected_accounts_summary(value: &Value) -> DxConnectedAccountsSummary {
    let needs_connection = usize_field(value, &["needs_connection"]).unwrap_or_default();
    DxConnectedAccountsSummary {
        supported: usize_field(value, &["supported"]).unwrap_or_default(),
        configured: usize_field(value, &["configured"]).unwrap_or_default(),
        connected: usize_field(value, &["connected"]).unwrap_or_default(),
        needs_connection,
        needs_auth: usize_field(value, &["needs_auth"]).unwrap_or(needs_connection),
        qr_connect_supported: usize_field(value, &["qr_connect_supported"]).unwrap_or_default(),
    }
}

pub(super) fn social_accounts(value: &Value) -> Vec<DxAgentSocialAccount> {
    array_field(value, &["accounts"])
        .map(|accounts| {
            accounts
                .iter()
                .take(12)
                .map(|account| DxAgentSocialAccount {
                    provider_id: display_string_field(account, &["provider_id"])
                        .or_else(|| display_string_field(account, &["provider"]))
                        .or_else(|| display_string_field(account, &["platform"]))
                        .unwrap_or_else(|| "unknown-provider".to_string()),
                    platform: display_string_field(account, &["platform"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    label: display_string_field(account, &["label"])
                        .unwrap_or_else(|| "Account".to_string()),
                    status: display_string_field(account, &["status"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    account_state: display_string_field(account, &["account_state"])
                        .unwrap_or_else(|| {
                            if bool_field(account, &["connected"]).unwrap_or(false) {
                                "connected".to_string()
                            } else if bool_field(account, &["configured"]).unwrap_or(false) {
                                "configured".to_string()
                            } else {
                                "missing_auth".to_string()
                            }
                        }),
                    auth_method: display_string_field(account, &["auth_method"])
                        .or_else(|| display_string_field(account, &["connect_method"]))
                        .unwrap_or_else(|| "unknown".to_string()),
                    qr_capability: display_string_field(account, &["qr_capability"])
                        .unwrap_or_else(|| {
                            if bool_field(account, &["qr_connect_supported"]).unwrap_or(false) {
                                "available".to_string()
                            } else {
                                "unavailable".to_string()
                            }
                        }),
                    credential_health: display_string_field(account, &["credential_health"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    credential_expires_at: display_string_field(
                        account,
                        &["credential_expires_at"],
                    ),
                    credential_error: display_string_field(account, &["credential_error"])
                        .or_else(|| display_string_field(account, &["last_error"])),
                    receipt_history: display_string_array_field(account, &["receipt_history"], 8),
                    configured: bool_field(account, &["configured"]).unwrap_or(false),
                    connected: bool_field(account, &["connected"]).unwrap_or(false),
                    qr_connect_supported: bool_field(account, &["qr_connect_supported"])
                        .unwrap_or(false),
                    actions: social_row_actions(account),
                    next_action: display_string_field(account, &["next_action"])
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Copy)]
pub(super) enum DxAgentSocialActionKind {
    Connect,
    Disconnect,
}

pub(super) fn social_action_summary(
    value: Option<&Value>,
    root_exists: bool,
    kind: DxAgentSocialActionKind,
) -> DxAgentSocialActionSummary {
    let action = match kind {
        DxAgentSocialActionKind::Connect => "connect",
        DxAgentSocialActionKind::Disconnect => "disconnect",
    };
    let command = match kind {
        DxAgentSocialActionKind::Connect => "dx agents social connect --json",
        DxAgentSocialActionKind::Disconnect => "dx agents social disconnect --json",
    };
    let waiting_status = match kind {
        DxAgentSocialActionKind::Connect => "waiting_for_social_connect_receipt",
        DxAgentSocialActionKind::Disconnect => "waiting_for_social_disconnect_receipt",
    };
    let account = value.and_then(|value| value.get("account"));
    let flow = value.and_then(|value| value.get("flow"));

    DxAgentSocialActionSummary {
        action,
        present: value.is_some(),
        status: value
            .and_then(|value| display_string_field(value, &["status"]))
            .unwrap_or_else(|| {
                if root_exists {
                    waiting_status.to_string()
                } else {
                    "missing_receipt_root".to_string()
                }
            }),
        platform: account
            .and_then(|account| display_string_field(account, &["platform"]))
            .unwrap_or_else(|| "unknown".to_string()),
        label: account
            .and_then(|account| display_string_field(account, &["label"]))
            .unwrap_or_else(|| "Social account".to_string()),
        connected: account.and_then(|account| bool_field(account, &["connected"])),
        connect_supported: flow
            .and_then(|flow| bool_field(flow, &["connect_supported"]))
            .unwrap_or(false),
        disconnect_supported: flow
            .and_then(|flow| bool_field(flow, &["disconnect_supported"]))
            .unwrap_or(false),
        qr_supported: flow
            .and_then(|flow| bool_field(flow, &["qr_supported"]))
            .unwrap_or(false),
        link_supported: flow
            .and_then(|flow| bool_field(flow, &["link_supported"]))
            .unwrap_or(false),
        connect_method: flow
            .and_then(|flow| display_string_field(flow, &["connect_method"]))
            .unwrap_or_else(|| "none".to_string()),
        manual_revoke_required: flow
            .and_then(|flow| bool_field(flow, &["manual_revoke_required"]))
            .unwrap_or(false),
        explicit_user_action_required: flow
            .and_then(|flow| bool_field(flow, &["explicit_user_action_required"]))
            .unwrap_or(false),
        safe_config_state: flow
            .and_then(|flow| display_string_field(flow, &["safe_config_state"]))
            .unwrap_or_else(|| "unknown".to_string()),
        next_action: value
            .and_then(|value| display_string_field(value, &["next_action"]))
            .unwrap_or_else(|| command.to_string()),
    }
}

fn social_row_actions(value: &Value) -> Vec<DxAgentRowAction> {
    row_actions(value, |id, command, receipt_filename, refresh_command| {
        is_dx_agents_command(refresh_command, "social list --json")
            && match id {
                "connect" => {
                    receipt_filename == "social-connect-latest.json"
                        && is_social_action_command(command, "connect")
                }
                "disconnect" => {
                    receipt_filename == "social-disconnect-latest.json"
                        && is_social_action_command(command, "disconnect")
                }
                "refresh" => {
                    receipt_filename == "social-list-latest.json"
                        && is_dx_agents_command(command, "social list --json")
                }
                _ => false,
            }
    })
}

fn row_actions<F>(value: &Value, is_allowed: F) -> Vec<DxAgentRowAction>
where
    F: Fn(&str, &str, &str, &str) -> bool,
{
    array_field(value, &["actions"])
        .map(|actions| {
            actions
                .iter()
                .take(8)
                .filter_map(|action| row_action(action, &is_allowed))
                .collect()
        })
        .unwrap_or_default()
}

fn row_action<F>(value: &Value, is_allowed: &F) -> Option<DxAgentRowAction>
where
    F: Fn(&str, &str, &str, &str) -> bool,
{
    let id = string_field(value, &["id"])?;
    let command = string_field(value, &["command"])?;
    let public_command = string_field(value, &["public_command"])
        .unwrap_or_else(|| public_command_for_runtime(&command));
    let receipt_filename = string_field(value, &["receipt_filename"])?;
    let refresh_command = string_field(value, &["refresh_command"])?;
    let public_refresh_command = string_field(value, &["public_refresh_command"])
        .unwrap_or_else(|| public_command_for_runtime(&refresh_command));
    let secrets_exposed = bool_field(value, &["secrets_exposed"]).unwrap_or(true);
    let writes_receipt = bool_field(value, &["writes_receipt"]).unwrap_or(false);

    if !writes_receipt
        || secrets_exposed
        || is_secret_like_arg(&command)
        || is_secret_like_arg(&public_command)
        || is_secret_like_arg(&receipt_filename)
        || is_secret_like_arg(&refresh_command)
        || is_secret_like_arg(&public_refresh_command)
        || !is_public_dx_agents_command(&public_command)
        || !is_public_dx_agents_command(&public_refresh_command)
        || !is_allowed(&id, &command, &receipt_filename, &refresh_command)
        || !is_allowed(
            &id,
            &public_command,
            &receipt_filename,
            &public_refresh_command,
        )
    {
        return None;
    }

    Some(DxAgentRowAction {
        label: string_field(value, &["label"]).unwrap_or_else(|| id.clone()),
        id,
        automation_id: None,
        command,
        public_command,
        enabled: bool_field(value, &["enabled"]).unwrap_or(false),
        user_action_required: bool_field(value, &["user_action_required"]).unwrap_or(false),
        writes_receipt,
        receipt_filename,
        refresh_command,
        public_refresh_command,
        secrets_exposed,
    })
}

fn is_social_action_command(command: &str, action: &str) -> bool {
    let runtime_prefix = format!("dx-agents agents social {action}");
    let public_prefix = format!("dx agents social {action}");
    [runtime_prefix.as_str(), public_prefix.as_str()]
        .into_iter()
        .any(|prefix| social_action_command_matches_prefix(command, prefix))
}

fn social_action_command_matches_prefix(command: &str, prefix: &str) -> bool {
    if command == format!("{prefix} --json") {
        return true;
    }

    let platform_prefix = format!("{prefix} --platform ");
    command
        .strip_prefix(&platform_prefix)
        .and_then(|value| value.strip_suffix(" --json"))
        .is_some_and(|platform| is_safe_platform_arg(platform))
}

#[cfg(test)]
#[path = "runtime_catalog_tests.rs"]
mod runtime_catalog_tests;
#[cfg(test)]
#[path = "runtime_connection_tests.rs"]
mod runtime_connection_tests;
#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
