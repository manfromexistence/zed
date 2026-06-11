use serde_json::Value;

use super::super::{array_field, bool_field};
use super::display_string_field;

const MAX_WORKFLOW_NODE_CREDENTIAL_ROWS: usize = 12;
const MAX_WORKFLOW_NODE_CREDENTIAL_INPUT_ROWS: usize = 12;

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeCredentialInputSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub required: bool,
    pub secret: bool,
    pub placeholder: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeCredentialSummary {
    pub id: String,
    pub kind: String,
    pub credential_type: String,
    pub required: bool,
    pub receipt_required: bool,
    pub status: String,
    pub receipt_id: String,
    pub configure_action_id: String,
    pub inputs: Vec<DxWorkflowNodeCredentialInputSummary>,
}

pub(super) fn workflow_node_credential_rows(value: &Value) -> Vec<DxWorkflowNodeCredentialSummary> {
    array_field(value, &["credentials"])
        .map(|items| {
            items
                .iter()
                .filter_map(credential_row)
                .take(MAX_WORKFLOW_NODE_CREDENTIAL_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

fn credential_row(value: &Value) -> Option<DxWorkflowNodeCredentialSummary> {
    let id =
        display_string_field(value, &["id"]).or_else(|| display_string_field(value, &["name"]))?;
    let credential_type = display_string_field(value, &["credential_type"])
        .or_else(|| display_string_field(value, &["type"]))
        .or_else(|| display_string_field(value, &["kind"]))
        .unwrap_or_else(|| id.clone());
    let inputs = credential_input_rows(value, &id, &credential_type);
    Some(DxWorkflowNodeCredentialSummary {
        kind: display_string_field(value, &["kind"]).unwrap_or_else(|| "credential".to_string()),
        credential_type,
        required: bool_field(value, &["required"]).unwrap_or(true),
        receipt_required: bool_field(value, &["receipt_required"]).unwrap_or(true),
        status: display_string_field(value, &["status"])
            .or_else(|| display_string_field(value, &["credential_status"]))
            .unwrap_or_else(|| "missing_credential_metadata".to_string()),
        receipt_id: display_string_field(value, &["receipt_id"])
            .unwrap_or_else(|| "missing_receipt_id".to_string()),
        configure_action_id: display_string_field(value, &["configure_action_id"])
            .or_else(|| display_string_field(value, &["action_id"]))
            .unwrap_or_else(|| "missing_action_id".to_string()),
        inputs,
        id,
    })
}

fn credential_input_rows(
    value: &Value,
    credential_id: &str,
    credential_type: &str,
) -> Vec<DxWorkflowNodeCredentialInputSummary> {
    let field_values = credential_input_values(value);
    let rows = field_values
        .iter()
        .filter_map(|value| credential_input_row(value))
        .take(MAX_WORKFLOW_NODE_CREDENTIAL_INPUT_ROWS)
        .collect::<Vec<_>>();

    if field_values.is_empty() {
        vec![fallback_credential_input(credential_id, credential_type)]
    } else {
        rows
    }
}

fn credential_input_values(value: &Value) -> Vec<&Value> {
    let mut values = Vec::new();
    collect_array_field(value, &["inputs"], &mut values);
    collect_array_field(value, &["fields"], &mut values);
    collect_object_values(value, &["properties"], &mut values);
    collect_object_values(value, &["schema", "properties"], &mut values);
    if let Some(inputs) = value.get("inputs") {
        collect_array_field(inputs, &["fields"], &mut values);
        collect_object_values(inputs, &["properties"], &mut values);
    }
    values
}

fn collect_array_field<'a>(value: &'a Value, path: &[&str], values: &mut Vec<&'a Value>) {
    if let Some(items) = array_field(value, path) {
        values.extend(items.iter());
    }
}

fn collect_object_values<'a>(value: &'a Value, path: &[&str], values: &mut Vec<&'a Value>) {
    let Some(object) = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
    else {
        return;
    };
    if let Some(object) = object.as_object() {
        values.extend(object.values());
    }
}

fn credential_input_row(value: &Value) -> Option<DxWorkflowNodeCredentialInputSummary> {
    let id =
        display_string_field(value, &["id"]).or_else(|| display_string_field(value, &["name"]))?;
    Some(DxWorkflowNodeCredentialInputSummary {
        label: display_string_field(value, &["label"]).unwrap_or_else(|| id.clone()),
        kind: display_string_field(value, &["kind"])
            .or_else(|| display_string_field(value, &["type"]))
            .unwrap_or_else(|| "text".to_string()),
        required: bool_field(value, &["required"]).unwrap_or(true),
        secret: bool_field(value, &["secret"]).unwrap_or(true),
        placeholder: display_string_field(value, &["placeholder"]).unwrap_or_default(),
        id,
    })
}

fn fallback_credential_input(
    credential_id: &str,
    credential_type: &str,
) -> DxWorkflowNodeCredentialInputSummary {
    DxWorkflowNodeCredentialInputSummary {
        id: credential_id.to_string(),
        label: credential_type.to_string(),
        kind: "text".to_string(),
        required: true,
        secret: true,
        placeholder: String::new(),
    }
}
