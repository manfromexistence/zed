use std::path::PathBuf;

use serde_json::Value;

use super::{
    array_field, bool_field, redact_action_scalar, string_array_field, string_field, usize_field,
};

const MAX_WORKFLOW_NODE_ROWS: usize = 768;
const MAX_WORKFLOW_NODE_CANDIDATES: usize = 2048;
const MAX_CONFIGURED_PLUGIN_ROWS: usize = 12;
const MAX_DETAIL_ITEMS: usize = 8;
const MAX_DISPLAY_CHARS: usize = 180;
const SERIALIZER_MACHINE_FORMAT: &str = "dx.serializer.machine";

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
}

#[derive(Clone)]
pub(crate) struct DxConfiguredPluginSummary {
    pub id: String,
    pub node_id: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub status: String,
    pub credential_status: String,
    pub run_command: String,
    pub action_id: String,
    pub receipt_id: String,
    pub action_label: String,
}

pub(super) fn workflow_node_catalog_summary(
    value: Option<&Value>,
    catalog_path: PathBuf,
    root_exists: bool,
) -> DxWorkflowNodeCatalogSummary {
    let nodes = value.map(workflow_node_rows).unwrap_or_default();
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
            .unwrap_or_else(|| "dx.workflow_nodes.catalog.v1".to_string()),
        serializer_format: value
            .and_then(|value| {
                display_string_field(value, &["serializer_format"])
                    .or_else(|| display_string_field(value, &["machine_format"]))
            })
            .unwrap_or_else(|| SERIALIZER_MACHINE_FORMAT.to_string()),
        catalog_path,
        node_count: value
            .and_then(|value| usize_field(value, &["node_count"]))
            .or_else(|| {
                value
                    .and_then(|value| array_field(value, &["nodes"]))
                    .map(Vec::len)
            })
            .unwrap_or_default(),
        configured_plugin_count: value
            .and_then(|value| usize_field(value, &["configured_plugin_count"]))
            .or_else(|| {
                value
                    .and_then(|value| array_field(value, &["configured_plugins"]))
                    .map(Vec::len)
            })
            .unwrap_or(configured_plugins.len()),
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

fn workflow_node_rows(value: &Value) -> Vec<DxWorkflowNodeSummary> {
    array_field(value, &["nodes"])
        .map(|nodes| {
            nodes
                .iter()
                .take(MAX_WORKFLOW_NODE_CANDIDATES)
                .filter_map(workflow_node_row)
                .take(MAX_WORKFLOW_NODE_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

fn workflow_node_row(value: &Value) -> Option<DxWorkflowNodeSummary> {
    let id = display_string_field(value, &["id"])?;
    let display_name = display_string_field(value, &["name"])
        .or_else(|| display_string_field(value, &["display_name"]))
        .unwrap_or_else(|| id.clone());
    let credential_types =
        display_string_array_field(value, &["credential_types"], MAX_DETAIL_ITEMS);
    let configured = bool_field(value, &["configured"]).unwrap_or(false);

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
            .unwrap_or_else(|| "workflow-node-catalog".to_string()),
        credential_status: display_string_field(value, &["credential_status"]).unwrap_or_else(
            || {
                if credential_types.is_empty() || configured {
                    "not_required".to_string()
                } else {
                    "needs_configuration".to_string()
                }
            },
        ),
        credential_types,
        input_count: usize_field(value, &["input_count"]).unwrap_or_default(),
        output_count: usize_field(value, &["output_count"]).unwrap_or_default(),
        parameter_count: usize_field(value, &["parameter_count"]).unwrap_or_default(),
        dynamic_option_count: usize_field(value, &["dynamic_option_count"]).unwrap_or_default(),
        configured,
        configure_action: display_string_field(value, &["configure_action"])
            .unwrap_or_else(|| "Open credential configuration".to_string()),
    })
}

fn configured_plugin_rows(value: &Value) -> Vec<DxConfiguredPluginSummary> {
    array_field(value, &["configured_plugins"])
        .or_else(|| array_field(value, &["enabled_plugins"]))
        .map(|plugins| {
            plugins
                .iter()
                .filter_map(configured_plugin_row)
                .take(MAX_CONFIGURED_PLUGIN_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

fn configured_plugin_row(value: &Value) -> Option<DxConfiguredPluginSummary> {
    let id = display_string_field(value, &["id"])?;
    let node_id = display_string_field(value, &["node_id"]).unwrap_or_else(|| id.clone());
    Some(DxConfiguredPluginSummary {
        display_name: display_string_field(value, &["name"])
            .or_else(|| display_string_field(value, &["display_name"]))
            .unwrap_or_else(|| id.clone()),
        icon: display_string_field(value, &["icon"]),
        status: display_string_field(value, &["status"]).unwrap_or_else(|| "unknown".to_string()),
        credential_status: display_string_field(value, &["credential_status"])
            .unwrap_or_else(|| "missing_receipt_field".to_string()),
        run_command: display_string_field(value, &["run_command"])
            .unwrap_or_else(|| "missing_run_command".to_string()),
        action_id: display_string_field(value, &["action_id"]).unwrap_or_else(|| id.clone()),
        receipt_id: display_string_field(value, &["receipt_id"])
            .unwrap_or_else(|| "missing_receipt_id".to_string()),
        action_label: display_string_field(value, &["action_label"])
            .unwrap_or_else(|| "Use plugin".to_string()),
        id,
        node_id,
    })
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
