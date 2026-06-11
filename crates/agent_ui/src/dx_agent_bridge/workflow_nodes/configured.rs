use std::collections::HashSet;

use serde_json::Value;

use super::super::{array_field, bool_field};
use super::configured_authorization::configured_plugin_row_is_authorized;
use super::display_string_field;

const MAX_CONFIGURED_PLUGIN_ROWS: usize = 12;
const MAX_CONFIGURED_PLUGIN_INDEX_ROWS: usize = 2048;

#[derive(Clone, PartialEq, Eq)]
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
    pub source_root_id: String,
    pub source_path: String,
    pub trust_policy: String,
    pub approved_by_trusted_bridge: bool,
    pub writes_receipt: bool,
    pub secrets_exposed: bool,
}

pub(super) fn configured_plugin_rows(value: &Value) -> Vec<DxConfiguredPluginSummary> {
    configured_plugin_values(value)
        .map(|plugins| {
            plugins
                .iter()
                .filter_map(configured_plugin_row)
                .take(MAX_CONFIGURED_PLUGIN_ROWS)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Default)]
pub(super) struct ConfiguredPluginIndex {
    has_configured_plugin_data: bool,
    node_keys: HashSet<(String, String, String)>,
}

impl ConfiguredPluginIndex {
    pub(super) fn has_configured_plugin_data(&self) -> bool {
        self.has_configured_plugin_data
    }

    pub(super) fn contains_node(
        &self,
        node_id: &str,
        source_root_id: &str,
        source_path: &str,
    ) -> bool {
        self.node_keys.iter().any(
            |(configured_node_id, configured_source_root_id, configured_source_path)| {
                configured_node_id == node_id
                    && configured_source_root_id == source_root_id
                    && configured_source_path == source_path
            },
        )
    }
}

pub(super) fn configured_plugin_index(value: &Value) -> ConfiguredPluginIndex {
    let Some(plugins) = configured_plugin_values(value) else {
        return ConfiguredPluginIndex {
            has_configured_plugin_data: false,
            node_keys: HashSet::new(),
        };
    };
    let node_keys = plugins
        .iter()
        .take(MAX_CONFIGURED_PLUGIN_INDEX_ROWS)
        .filter_map(configured_plugin_key)
        .collect();

    ConfiguredPluginIndex {
        has_configured_plugin_data: true,
        node_keys,
    }
}

fn configured_plugin_values(value: &Value) -> Option<&Vec<Value>> {
    array_field(value, &["configured_plugins"]).or_else(|| array_field(value, &["enabled_plugins"]))
}

fn configured_plugin_key(value: &Value) -> Option<(String, String, String)> {
    let plugin = configured_plugin_row(value)?;
    Some((plugin.node_id, plugin.source_root_id, plugin.source_path))
}

fn configured_plugin_row(value: &Value) -> Option<DxConfiguredPluginSummary> {
    let id = display_string_field(value, &["id"])?;
    let node_id = display_string_field(value, &["node_id"]).unwrap_or_else(|| id.clone());
    let row = DxConfiguredPluginSummary {
        display_name: display_string_field(value, &["name"])
            .or_else(|| display_string_field(value, &["display_name"]))
            .unwrap_or_else(|| id.clone()),
        icon: display_string_field(value, &["icon"]),
        status: display_string_field(value, &["status"]).unwrap_or_else(|| "unknown".to_string()),
        credential_status: display_string_field(value, &["credential_status"])
            .unwrap_or_else(|| "missing_receipt_field".to_string()),
        run_command: display_string_field(value, &["run_command"])
            .unwrap_or_else(|| "missing_run_command".to_string()),
        action_id: display_string_field(value, &["action_id"])
            .unwrap_or_else(|| "missing_action_id".to_string()),
        receipt_id: display_string_field(value, &["receipt_id"])
            .unwrap_or_else(|| "missing_receipt_id".to_string()),
        action_label: display_string_field(value, &["action_label"])
            .unwrap_or_else(|| "Use plugin".to_string()),
        source_root_id: display_string_field(value, &["source_root_id"])
            .unwrap_or_else(|| "missing_source_root_id".to_string()),
        source_path: display_string_field(value, &["source_path"])
            .unwrap_or_else(|| "missing_source_path".to_string()),
        trust_policy: display_string_field(value, &["trust_policy"])
            .unwrap_or_else(|| "missing_trust_policy".to_string()),
        approved_by_trusted_bridge: bool_field(value, &["approved_by_trusted_bridge"])
            .unwrap_or(false),
        writes_receipt: bool_field(value, &["writes_receipt"])
            .or_else(|| bool_field(value, &["writes_receipts"]))
            .unwrap_or(false),
        secrets_exposed: bool_field(value, &["secrets_exposed"]).unwrap_or(true),
        id,
        node_id,
    };

    configured_plugin_row_is_authorized(&row).then_some(row)
}
