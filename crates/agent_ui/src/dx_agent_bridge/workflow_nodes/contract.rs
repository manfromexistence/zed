use serde_json::Value;

use super::super::{array_field, bool_field};
use super::display_string_field;

pub(super) const MAX_WORKFLOW_NODE_PORT_ROWS: usize = 12;
pub(super) const MAX_WORKFLOW_NODE_PERMISSION_ROWS: usize = 12;
pub(super) const MAX_WORKFLOW_NODE_DYNAMIC_OPTION_ROWS: usize = 12;
pub(super) const MAX_WORKFLOW_NODE_RECEIPT_ROWS: usize = 12;
pub(super) const MAX_WORKFLOW_NODE_ACTION_ROWS: usize = 8;

#[derive(Clone)]
pub(crate) struct DxWorkflowNodePortSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub required: bool,
    pub description: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodePermissionSummary {
    pub id: String,
    pub level: String,
    pub receipt_required: bool,
    pub status: String,
    pub description: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeDynamicOptionSummary {
    pub id: String,
    pub parameter_id: String,
    pub label: String,
    pub source: String,
    pub status: String,
    pub action_id: String,
    pub receipt_id: String,
    pub credential_required: bool,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeReceiptSummary {
    pub id: String,
    pub schema: String,
    pub status: String,
    pub required_for: String,
    pub generated_at: Option<String>,
    pub path: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeActionSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub risk: String,
    pub requires_approval: bool,
    pub writes_receipts: bool,
    pub receipt_id: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeTrustSummary {
    pub status: String,
    pub source_owned: bool,
    pub first_party: bool,
    pub enabled_by_default: bool,
    pub requires_user_enablement_for_input: bool,
    pub approved_by_trusted_bridge: bool,
    pub trust_policy: String,
}

pub(super) fn workflow_node_permission_rows(value: &Value) -> Vec<DxWorkflowNodePermissionSummary> {
    array_field(value, &["permissions"])
        .map(|items| {
            items
                .iter()
                .filter_map(permission_row)
                .take(MAX_WORKFLOW_NODE_PERMISSION_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn workflow_node_port_rows(
    value: &Value,
    path: &[&str],
) -> Vec<DxWorkflowNodePortSummary> {
    array_field(value, path)
        .map(|items| {
            items
                .iter()
                .filter_map(port_row)
                .take(MAX_WORKFLOW_NODE_PORT_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn workflow_node_dynamic_option_rows(
    value: &Value,
) -> Vec<DxWorkflowNodeDynamicOptionSummary> {
    array_field(value, &["dynamic_options"])
        .map(|items| {
            items
                .iter()
                .filter_map(dynamic_option_row)
                .take(MAX_WORKFLOW_NODE_DYNAMIC_OPTION_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn workflow_node_receipt_rows(value: &Value) -> Vec<DxWorkflowNodeReceiptSummary> {
    array_field(value, &["receipts"])
        .map(|items| {
            items
                .iter()
                .filter_map(receipt_row)
                .take(MAX_WORKFLOW_NODE_RECEIPT_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn workflow_node_action_rows(value: &Value) -> Vec<DxWorkflowNodeActionSummary> {
    array_field(value, &["actions"])
        .map(|items| {
            items
                .iter()
                .filter_map(action_row)
                .take(MAX_WORKFLOW_NODE_ACTION_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn workflow_node_trust_summary(value: &Value) -> DxWorkflowNodeTrustSummary {
    let trust = value.get("trust").unwrap_or(value);
    DxWorkflowNodeTrustSummary {
        status: display_string_field(trust, &["status"])
            .or_else(|| display_string_field(value, &["trust_status"]))
            .unwrap_or_else(|| "unverified".to_string()),
        source_owned: bool_field(trust, &["source_owned"]).unwrap_or(false),
        first_party: bool_field(trust, &["first_party"]).unwrap_or(false),
        enabled_by_default: bool_field(trust, &["enabled_by_default"]).unwrap_or(false),
        requires_user_enablement_for_input: bool_field(
            trust,
            &["requires_user_enablement_for_input"],
        )
        .unwrap_or(true),
        approved_by_trusted_bridge: bool_field(trust, &["approved_by_trusted_bridge"])
            .unwrap_or(false),
        trust_policy: display_string_field(trust, &["trust_policy"])
            .unwrap_or_else(|| "missing_trust_policy".to_string()),
    }
}

fn permission_row(value: &Value) -> Option<DxWorkflowNodePermissionSummary> {
    let id = display_string_field(value, &["id"])?;
    Some(DxWorkflowNodePermissionSummary {
        level: display_string_field(value, &["level"]).unwrap_or_else(|| "missing".to_string()),
        receipt_required: bool_field(value, &["receipt_required"]).unwrap_or(true),
        status: display_string_field(value, &["status"])
            .unwrap_or_else(|| "missing_permissions".to_string()),
        description: display_string_field(value, &["description"]).unwrap_or_default(),
        id,
    })
}

fn port_row(value: &Value) -> Option<DxWorkflowNodePortSummary> {
    let id = display_string_field(value, &["id"])?;
    Some(DxWorkflowNodePortSummary {
        name: display_string_field(value, &["name"]).unwrap_or_else(|| id.clone()),
        kind: display_string_field(value, &["kind"]).unwrap_or_else(|| "unknown".to_string()),
        required: bool_field(value, &["required"]).unwrap_or(false),
        description: display_string_field(value, &["description"]).unwrap_or_default(),
        id,
    })
}

fn dynamic_option_row(value: &Value) -> Option<DxWorkflowNodeDynamicOptionSummary> {
    let id = display_string_field(value, &["id"])?;
    Some(DxWorkflowNodeDynamicOptionSummary {
        parameter_id: display_string_field(value, &["parameter_id"])
            .unwrap_or_else(|| "missing_parameter_id".to_string()),
        label: display_string_field(value, &["label"]).unwrap_or_else(|| id.clone()),
        source: display_string_field(value, &["source"]).unwrap_or_else(|| "missing".to_string()),
        status: display_string_field(value, &["status"])
            .unwrap_or_else(|| "missing_dynamic_option_metadata".to_string()),
        action_id: display_string_field(value, &["action_id"])
            .unwrap_or_else(|| "missing_action_id".to_string()),
        receipt_id: display_string_field(value, &["receipt_id"])
            .unwrap_or_else(|| "missing_receipt_id".to_string()),
        credential_required: bool_field(value, &["credential_required"]).unwrap_or(true),
        id,
    })
}

fn receipt_row(value: &Value) -> Option<DxWorkflowNodeReceiptSummary> {
    let id = display_string_field(value, &["id"])?;
    Some(DxWorkflowNodeReceiptSummary {
        schema: display_string_field(value, &["schema"]).unwrap_or_else(|| "missing".to_string()),
        status: display_string_field(value, &["status"]).unwrap_or_else(|| "missing".to_string()),
        required_for: display_string_field(value, &["required_for"])
            .unwrap_or_else(|| "plugin_contract".to_string()),
        generated_at: display_string_field(value, &["generated_at"]),
        path: display_string_field(value, &["path"]).unwrap_or_default(),
        id,
    })
}

fn action_row(value: &Value) -> Option<DxWorkflowNodeActionSummary> {
    let id = display_string_field(value, &["id"])?;
    Some(DxWorkflowNodeActionSummary {
        label: display_string_field(value, &["label"]).unwrap_or_else(|| id.clone()),
        kind: display_string_field(value, &["kind"]).unwrap_or_else(|| "metadata".to_string()),
        risk: display_string_field(value, &["risk"]).unwrap_or_else(|| "unknown".to_string()),
        requires_approval: bool_field(value, &["requires_approval"]).unwrap_or(true),
        writes_receipts: bool_field(value, &["writes_receipts"]).unwrap_or(false),
        receipt_id: display_string_field(value, &["receipt_id"])
            .unwrap_or_else(|| "missing_receipt_id".to_string()),
        id,
    })
}
