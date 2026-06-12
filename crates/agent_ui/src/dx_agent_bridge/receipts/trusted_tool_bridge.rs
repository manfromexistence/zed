use serde_json::Value;

use super::super::{DxAgentTrustedToolBridgeSummary, array_field, usize_field};
use super::receipt_strings::{receipt_string_array_field, receipt_string_field};

pub(in crate::dx_agent_bridge) fn trusted_tool_bridge_summary(
    status_value: Option<&Value>,
    contract_value: Option<&Value>,
    import_summary_value: Option<&Value>,
    release_gate_value: Option<&Value>,
    root_exists: bool,
) -> DxAgentTrustedToolBridgeSummary {
    let bridge = trusted_tool_bridge_value(&[
        status_value,
        contract_value,
        import_summary_value,
        release_gate_value,
    ]);
    let status = bridge
        .and_then(|value| receipt_string_field(value, &["status"]))
        .unwrap_or_else(|| {
            if root_exists {
                "waiting_for_trusted_tool_bridge_receipt".to_string()
            } else {
                "missing_receipt_root".to_string()
            }
        });
    let receipt_count = bridge
        .and_then(|value| usize_field(value, &["receipt_count"]))
        .or_else(|| {
            bridge
                .and_then(|value| array_field(value, &["receipts"]))
                .map(|receipts| receipts.len())
        })
        .unwrap_or_default();

    DxAgentTrustedToolBridgeSummary {
        present: bridge.is_some(),
        status,
        trust_policy: bridge
            .and_then(|value| receipt_string_field(value, &["trust_policy"]))
            .unwrap_or_else(|| {
                if bridge.is_some() {
                    "receipt_authorized_only".to_string()
                } else {
                    "unavailable".to_string()
                }
            }),
        approved_plugin_tool_count: bridge
            .and_then(|value| usize_field(value, &["approved_plugin_tool_count"]))
            .unwrap_or_default(),
        approved_automation_tool_count: bridge
            .and_then(|value| usize_field(value, &["approved_automation_tool_count"]))
            .unwrap_or_default(),
        blocked_tool_count: bridge
            .and_then(|value| usize_field(value, &["blocked_tool_count"]))
            .unwrap_or_default(),
        receipt_count,
        bridge_contract_id: bridge
            .and_then(|value| {
                receipt_string_field(value, &["bridge_contract_id"])
                    .or_else(|| receipt_string_field(value, &["contract_id"]))
            })
            .unwrap_or_else(|| "missing_tool_bridge_contract".to_string()),
        next_action: bridge
            .and_then(|value| receipt_string_field(value, &["next_action"]))
            .unwrap_or_else(|| "dx agents contract --json".to_string()),
        trusted_tool_ids: bridge
            .map(|value| receipt_string_array_field(value, &["trusted_tool_ids"]))
            .unwrap_or_default(),
    }
}

fn trusted_tool_bridge_value<'a>(values: &[Option<&'a Value>]) -> Option<&'a Value> {
    values.iter().find_map(|value| {
        value.and_then(|value| {
            value
                .get("trusted_tool_bridge")
                .or_else(|| value.get("tool_bridge"))
        })
    })
}
