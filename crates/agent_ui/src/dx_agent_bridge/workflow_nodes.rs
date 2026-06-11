use std::path::PathBuf;

use serde_json::Value;

use super::{
    array_field, bool_field, redact_action_scalar, string_array_field, string_field, usize_field,
};

mod configured;
mod configured_authorization;
mod contract;
mod credentials;

pub(crate) use self::configured::DxConfiguredPluginSummary;
use self::configured::{ConfiguredPluginIndex, configured_plugin_index, configured_plugin_rows};
pub(crate) use self::contract::{
    DxWorkflowNodeActionSummary, DxWorkflowNodeDynamicOptionSummary,
    DxWorkflowNodePermissionSummary, DxWorkflowNodePortSummary, DxWorkflowNodeReceiptSummary,
    DxWorkflowNodeTrustSummary,
};
use self::contract::{
    workflow_node_action_rows, workflow_node_dynamic_option_rows, workflow_node_permission_rows,
    workflow_node_port_rows, workflow_node_receipt_rows, workflow_node_trust_summary,
};
use self::credentials::workflow_node_credential_rows;
pub(crate) use self::credentials::{
    DxWorkflowNodeCredentialInputSummary, DxWorkflowNodeCredentialSummary,
};

const MAX_WORKFLOW_NODE_ROWS: usize = 768;
const MAX_WORKFLOW_NODE_CANDIDATES: usize = 2048;
const MAX_DETAIL_ITEMS: usize = 8;
const MAX_DISPLAY_CHARS: usize = 180;

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeCatalogSummary {
    pub present: bool,
    pub status: String,
    pub schema_version: String,
    pub serializer_format: String,
    pub catalog_path: PathBuf,
    pub node_count: usize,
    pub configured_plugin_count: usize,
    pub generated_at: Option<String>,
    pub source_packages: Vec<String>,
    pub nodes: Vec<DxWorkflowNodeSummary>,
    pub configured_plugins: Vec<DxConfiguredPluginSummary>,
    pub next_action: String,
}

#[derive(Clone)]
pub(crate) struct DxWorkflowNodeSummary {
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
    pub icon: Option<String>,
    pub runtime: String,
    pub trust_status: String,
    pub source_package: String,
    pub credential_types: Vec<String>,
    pub credential_status: String,
    pub input_count: usize,
    pub output_count: usize,
    pub parameter_count: usize,
    pub dynamic_option_count: usize,
    pub configured: bool,
    pub configure_action: String,
    pub permissions: Vec<DxWorkflowNodePermissionSummary>,
    pub inputs: Vec<DxWorkflowNodePortSummary>,
    pub outputs: Vec<DxWorkflowNodePortSummary>,
    pub credentials: Vec<DxWorkflowNodeCredentialSummary>,
    pub dynamic_options: Vec<DxWorkflowNodeDynamicOptionSummary>,
    pub receipts: Vec<DxWorkflowNodeReceiptSummary>,
    pub actions: Vec<DxWorkflowNodeActionSummary>,
    pub trust: DxWorkflowNodeTrustSummary,
    pub source_package_version: String,
    pub source_root_id: String,
    pub source_path: String,
}

pub(super) fn workflow_node_catalog_summary(
    value: Option<&Value>,
    catalog_path: PathBuf,
    root_exists: bool,
) -> DxWorkflowNodeCatalogSummary {
    let configured_index = value.map(configured_plugin_index).unwrap_or_default();
    let nodes = value
        .map(|value| workflow_node_rows(value, &configured_index))
        .unwrap_or_default();
    let configured_plugins = value.map(configured_plugin_rows).unwrap_or_default();

    DxWorkflowNodeCatalogSummary {
        present: value.is_some(),
        status: value
            .and_then(|value| display_string_field(value, &["status"]))
            .unwrap_or_else(|| {
                if root_exists {
                    "waiting_for_workflow_node_catalog_receipt".to_string()
                } else {
                    "missing_receipt_root".to_string()
                }
            }),
        schema_version: value
            .and_then(|value| display_string_field(value, &["schema_version"]))
            .unwrap_or_else(|| "missing_schema_version".to_string()),
        serializer_format: value
            .and_then(|value| {
                display_string_field(value, &["serializer_format"])
                    .or_else(|| display_string_field(value, &["machine_format"]))
            })
            .unwrap_or_else(|| "missing_serializer_format".to_string()),
        catalog_path,
        node_count: value
            .and_then(|value| usize_field(value, &["node_count"]))
            .or_else(|| {
                value
                    .and_then(|value| array_field(value, &["nodes"]))
                    .map(Vec::len)
            })
            .unwrap_or_default(),
        configured_plugin_count: nodes.iter().filter(|node| node.configured).count(),
        generated_at: value.and_then(|value| display_string_field(value, &["generated_at"])),
        source_packages: value
            .map(|value| display_string_array_field(value, &["source_packages"], MAX_DETAIL_ITEMS))
            .unwrap_or_default(),
        nodes,
        configured_plugins,
        next_action: value
            .and_then(|value| display_string_field(value, &["next_action"]))
            .unwrap_or_else(|| "dx agents plugins workflow-nodes refresh --json".to_string()),
    }
}

fn workflow_node_rows(
    value: &Value,
    configured_index: &ConfiguredPluginIndex,
) -> Vec<DxWorkflowNodeSummary> {
    array_field(value, &["nodes"])
        .map(|nodes| {
            nodes
                .iter()
                .take(MAX_WORKFLOW_NODE_CANDIDATES)
                .filter_map(|value| workflow_node_row(value, configured_index))
                .take(MAX_WORKFLOW_NODE_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

fn workflow_node_row(
    value: &Value,
    configured_index: &ConfiguredPluginIndex,
) -> Option<DxWorkflowNodeSummary> {
    let id = display_string_field(value, &["id"])?;
    let display_name = display_string_field(value, &["name"])
        .or_else(|| display_string_field(value, &["display_name"]))
        .unwrap_or_else(|| id.clone());
    let inputs = workflow_node_port_rows(value, &["inputs"]);
    let outputs = workflow_node_port_rows(value, &["outputs"]);
    let credentials = workflow_node_credential_rows(value);
    let credential_types = credential_type_values(value, &credentials);
    let dynamic_options = workflow_node_dynamic_option_rows(value);
    let source_root_id = display_string_field(value, &["source_root_id"])
        .unwrap_or_else(|| "missing_source_root_id".to_string());
    let source_path = display_string_field(value, &["source_path"])
        .unwrap_or_else(|| "missing_source_path".to_string());
    let configured = if configured_index.has_configured_plugin_data() {
        configured_index.contains_node(&id, &source_root_id, &source_path)
    } else {
        bool_field(value, &["configured"]).unwrap_or(false)
    };

    Some(DxWorkflowNodeSummary {
        id,
        display_name,
        category: display_string_field(value, &["category"])
            .unwrap_or_else(|| "workflow".to_string()),
        description: display_string_field(value, &["description"]).unwrap_or_default(),
        icon: display_string_field(value, &["icon"]),
        runtime: display_string_field(value, &["runtime"]).unwrap_or_else(|| "dx-js".to_string()),
        trust_status: display_string_field(value, &["trust_status"])
            .unwrap_or_else(|| "unverified".to_string()),
        source_package: display_string_field(value, &["source_package"])
            .unwrap_or_else(|| "unknown_source_package".to_string()),
        credential_status: display_string_field(value, &["credential_status"])
            .unwrap_or_else(|| "missing_credential_metadata".to_string()),
        credential_types,
        input_count: usize_field(value, &["input_count"]).unwrap_or(inputs.len()),
        output_count: usize_field(value, &["output_count"]).unwrap_or(outputs.len()),
        parameter_count: usize_field(value, &["parameter_count"]).unwrap_or_default(),
        dynamic_option_count: usize_field(value, &["dynamic_option_count"])
            .unwrap_or(dynamic_options.len()),
        configured,
        configure_action: display_string_field(value, &["configure_action"])
            .unwrap_or_else(|| "Open credential configuration".to_string()),
        permissions: workflow_node_permission_rows(value),
        inputs,
        outputs,
        credentials,
        dynamic_options,
        receipts: workflow_node_receipt_rows(value),
        actions: workflow_node_action_rows(value),
        trust: workflow_node_trust_summary(value),
        source_package_version: display_string_field(value, &["source_package_version"])
            .unwrap_or_else(|| "missing_source_package_version".to_string()),
        source_root_id,
        source_path,
    })
}

fn credential_type_values(
    value: &Value,
    credentials: &[DxWorkflowNodeCredentialSummary],
) -> Vec<String> {
    let declared = display_string_array_field(value, &["credential_types"], MAX_DETAIL_ITEMS);
    if !declared.is_empty() {
        return declared;
    }

    credentials
        .iter()
        .filter_map(|credential| display_string(credential.credential_type.clone()))
        .take(MAX_DETAIL_ITEMS)
        .collect()
}

fn display_string_field(value: &Value, path: &[&str]) -> Option<String> {
    string_field(value, path).and_then(display_string)
}

fn display_string_array_field(value: &Value, path: &[&str], limit: usize) -> Vec<String> {
    string_array_field(value, path)
        .into_iter()
        .filter_map(display_string)
        .take(limit)
        .collect()
}

fn display_string(value: String) -> Option<String> {
    let compact = redact_action_scalar(&value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if compact.is_empty() {
        return None;
    }
    if compact.chars().count() <= MAX_DISPLAY_CHARS {
        return Some(compact);
    }

    let mut display = compact
        .chars()
        .take(MAX_DISPLAY_CHARS.saturating_sub(3))
        .collect::<String>();
    display.push_str("...");
    Some(display)
}
