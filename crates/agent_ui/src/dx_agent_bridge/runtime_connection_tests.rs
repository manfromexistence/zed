use serde_json::json;

use super::super::trusted_tool_bridge_summary;
use super::{DxAgentSocialActionKind, social_accounts, social_action_summary};

#[test]
fn social_connection_cards_parse_auth_health_and_receipt_history() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.social_list.v1",
        "accounts": [{
            "provider_id": "github",
            "platform": "github",
            "label": "essencefromexistence",
            "status": "needs_reauth",
            "account_state": "expired",
            "auth_method": "oauth_device",
            "qr_capability": "blocked_missing_device_code",
            "credential_health": "expired",
            "credential_expires_at": "2026-06-08T00:00:00Z",
            "credential_error": "reauth required",
            "configured": true,
            "connected": false,
            "qr_connect_supported": true,
            "receipt_history": [
                "social-list-latest.json",
                "social-connect-latest.json"
            ],
            "next_action": "dx agents social connect --platform github --json"
        }]
    });

    let accounts = social_accounts(&receipt);

    assert_eq!(accounts.len(), 1);
    let account = &accounts[0];
    assert_eq!(account.provider_id, "github");
    assert_eq!(account.account_state, "expired");
    assert_eq!(account.auth_method, "oauth_device");
    assert_eq!(account.qr_capability, "blocked_missing_device_code");
    assert_eq!(account.credential_health, "expired");
    assert_eq!(
        account.credential_expires_at.as_deref(),
        Some("2026-06-08T00:00:00Z")
    );
    assert_eq!(account.credential_error.as_deref(), Some("reauth required"));
    assert_eq!(account.receipt_history.len(), 2);
}

#[test]
fn social_connection_cards_redact_visible_account_fields() {
    let receipt = json!({
        "accounts": [{
            "label": "token=ghp_secret_user",
            "credential_error": "refresh_token=secret",
            "receipt_history": ["social-list-latest.json", "access_token=secret"],
            "next_action": "dx agents social connect --api-key secret --json"
        }]
    });

    let accounts = social_accounts(&receipt);

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].label, "<redacted>");
    assert_eq!(accounts[0].credential_error.as_deref(), Some("<redacted>"));
    assert_eq!(accounts[0].receipt_history[1], "<redacted>");
    assert_eq!(accounts[0].next_action, "<redacted>");
}

#[test]
fn social_action_summary_redacts_visible_receipt_fields() {
    let receipt = json!({
        "account": {"label": "password=secret"},
        "flow": {"safe_config_state": "token=secret"},
        "next_action": "dx agents social connect --client-secret secret --json"
    });

    let summary = social_action_summary(Some(&receipt), true, DxAgentSocialActionKind::Connect);

    assert_eq!(summary.label, "<redacted>");
    assert_eq!(summary.safe_config_state, "<redacted>");
    assert_eq!(summary.next_action, "<redacted>");
}

#[test]
fn trusted_tool_bridge_summary_requires_receipt_authority() {
    let receipt = json!({
        "schema_version": "dx.agents.zed.contract.v1",
        "tool_bridge": {
            "status": "ready",
            "trust_policy": "receipt_authorized_only",
            "approved_plugin_tool_count": 3,
            "approved_automation_tool_count": 2,
            "blocked_tool_count": 4,
            "receipt_count": 5,
            "bridge_contract_id": "dx.agents.trusted_tool_bridge.v1",
            "trusted_tool_ids": [
                "dx.plugin.browser.navigate",
                "dx.automation.receipt.refresh"
            ],
            "next_action": "dx agents release-gate --json"
        }
    });

    let summary = trusted_tool_bridge_summary(None, Some(&receipt), None, None, true);

    assert!(summary.present);
    assert_eq!(summary.status, "ready");
    assert_eq!(summary.trust_policy, "receipt_authorized_only");
    assert_eq!(summary.approved_plugin_tool_count, 3);
    assert_eq!(summary.approved_automation_tool_count, 2);
    assert_eq!(summary.blocked_tool_count, 4);
    assert_eq!(summary.receipt_count, 5);
    assert_eq!(
        summary.bridge_contract_id,
        "dx.agents.trusted_tool_bridge.v1"
    );
    assert_eq!(summary.trusted_tool_ids.len(), 2);
}
