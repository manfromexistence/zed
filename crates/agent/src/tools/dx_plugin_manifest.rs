mod entries;

use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

pub(crate) const DX_PLUGIN_MANIFEST_SCHEMA: &str = "zed.dx_plugins.manifest.v1";
pub(crate) const DX_PLUGIN_CATALOG_SCHEMA: &str = "zed.dx_plugins.catalog.v1";
pub(crate) const DX_PLUGIN_CATALOG_DISCOVERY_SCHEMA: &str = "zed.dx_plugins.catalog_discovery.v1";
pub(crate) const DX_PLUGIN_CATALOG_SUMMARY_SCHEMA: &str = "zed.dx_plugins.catalog_summary.v1";

#[derive(Clone)]
pub(crate) struct DxPluginCatalogPaths {
    pub(crate) workspace_plugin_root: Option<PathBuf>,
    pub(crate) workspace_tools_root: Option<PathBuf>,
    pub(crate) zed_data_plugin_root: PathBuf,
}

#[derive(Serialize)]
pub(crate) struct DxPluginCatalog {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) source: &'static str,
    pub(crate) product_surface: &'static str,
    pub(crate) workflow_surface: &'static str,
    pub(crate) node_surface: &'static str,
    pub(crate) credential_surface: &'static str,
    pub(crate) receipt_surface: &'static str,
    pub(crate) available_to: Vec<&'static str>,
    pub(crate) discovery: DxPluginCatalogDiscovery,
    pub(crate) bridge: DxPluginBridge,
    pub(crate) manifests: Vec<DxPluginManifest>,
}

#[derive(Serialize)]
pub(crate) struct DxPluginCatalogDiscovery {
    pub(crate) schema: &'static str,
    pub(crate) source_root_policy: &'static str,
    pub(crate) scanner_status: &'static str,
    pub(crate) allowlisted_source_roots: Vec<DxPluginSourceRoot>,
    pub(crate) catalog_source: &'static str,
    pub(crate) rejects_non_allowlisted_roots: bool,
    pub(crate) caches_outside_render_paths: bool,
}

#[derive(Serialize)]
pub(crate) struct DxPluginSourceRoot {
    pub(crate) id: &'static str,
    pub(crate) path: String,
    pub(crate) ownership: &'static str,
    pub(crate) runtime_use: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DxPluginBridge {
    pub(crate) catalog_tool: &'static str,
    pub(crate) runtime_status_tool: &'static str,
    pub(crate) panels_ready_for: Vec<&'static str>,
    pub(crate) agents_bridge_ready: bool,
    pub(crate) exposes_enabled_trusted_only: bool,
}

#[derive(Serialize)]
pub(crate) struct DxPluginManifest {
    pub(crate) schema: &'static str,
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) category: &'static str,
    pub(crate) description: &'static str,
    pub(crate) permissions: Vec<DxPluginPermission>,
    pub(crate) runtime: DxPluginRuntime,
    pub(crate) inputs: Vec<DxPluginPort>,
    pub(crate) outputs: Vec<DxPluginPort>,
    pub(crate) credentials: Vec<DxPluginCredential>,
    pub(crate) trust_status: DxPluginTrustStatus,
    pub(crate) receipts: Vec<DxPluginReceipt>,
    pub(crate) source_root_ids: Vec<&'static str>,
    pub(crate) available_to: Vec<&'static str>,
}

#[derive(Serialize)]
pub(crate) struct DxPluginPermission {
    pub(crate) id: &'static str,
    pub(crate) level: &'static str,
    pub(crate) receipt_required: bool,
    pub(crate) description: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DxPluginRuntime {
    pub(crate) runtime: &'static str,
    pub(crate) engine: &'static str,
    pub(crate) source_root_id: &'static str,
    pub(crate) entrypoint: &'static str,
    pub(crate) cancellation: &'static str,
    pub(crate) receipt_root: String,
    pub(crate) dxjs_required: bool,
}

#[derive(Serialize)]
pub(crate) struct DxPluginPort {
    pub(crate) id: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) required: bool,
    pub(crate) description: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DxPluginCredential {
    pub(crate) id: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) credential_status: &'static str,
    pub(crate) required: bool,
    pub(crate) receipt_required: bool,
    pub(crate) description: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DxPluginTrustStatus {
    pub(crate) status: &'static str,
    pub(crate) source_owned: bool,
    pub(crate) first_party: bool,
    pub(crate) enabled_by_default: bool,
    pub(crate) requires_user_enablement_for_input: bool,
}

#[derive(Serialize)]
pub(crate) struct DxPluginReceipt {
    pub(crate) id: &'static str,
    pub(crate) schema: &'static str,
    pub(crate) receipt_root: String,
    pub(crate) required_for: &'static str,
}

pub(crate) fn dx_first_party_plugin_catalog(
    paths: DxPluginCatalogPaths,
    catalog_tool: &'static str,
    runtime_status_tool: &'static str,
) -> Value {
    let receipt_root = plugin_receipt_root(&paths);
    let catalog = DxPluginCatalog {
        schema: DX_PLUGIN_CATALOG_SCHEMA,
        status: "first_party_manifest_foundation_ready",
        source: "dx_first_party_manifest_entries",
        product_surface: "Plugins",
        workflow_surface: "Workflows",
        node_surface: "Nodes",
        credential_surface: "Credentials",
        receipt_surface: "Receipts",
        available_to: vec!["zed_plugins_panel", "dx_agents_bridge", "agent_panel"],
        discovery: dx_plugin_catalog_discovery(&paths),
        bridge: DxPluginBridge {
            catalog_tool,
            runtime_status_tool,
            panels_ready_for: vec!["zed_plugins_panel", "dx_agents_bridge"],
            agents_bridge_ready: true,
            exposes_enabled_trusted_only: true,
        },
        manifests: entries::first_party_plugin_manifests(receipt_root),
    };

    serde_json::to_value(catalog).unwrap_or_else(|error| {
        serde_json::json!({
            "schema": DX_PLUGIN_CATALOG_SCHEMA,
            "status": "serialization_error",
            "error": error.to_string(),
        })
    })
}

pub(crate) fn dx_first_party_plugin_catalog_summary(catalog: &Value) -> Value {
    let manifests = catalog
        .get("manifests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let plugin_ids = manifests
        .iter()
        .filter_map(|manifest| manifest.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let categories = manifests
        .iter()
        .filter_map(|manifest| manifest.get("category").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let source_root_count = catalog
        .pointer("/discovery/allowlisted_source_roots")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let source_root_policy = catalog
        .pointer("/discovery/source_root_policy")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    serde_json::json!({
        "schema": DX_PLUGIN_CATALOG_SUMMARY_SCHEMA,
        "source_schema": DX_PLUGIN_CATALOG_SCHEMA,
        "status": catalog.get("status").and_then(Value::as_str),
        "plugin_count": manifests.len(),
        "plugin_ids": plugin_ids,
        "categories": categories,
        "default_enabled_plugins": ["dx.browser", "dx.computer", "dx.driven"],
        "source_root_policy": source_root_policy,
        "allowlisted_source_root_count": source_root_count,
        "available_to": catalog.get("available_to").cloned().unwrap_or(Value::Null),
        "read_only": true,
    })
}

fn dx_plugin_catalog_discovery(paths: &DxPluginCatalogPaths) -> DxPluginCatalogDiscovery {
    DxPluginCatalogDiscovery {
        schema: DX_PLUGIN_CATALOG_DISCOVERY_SCHEMA,
        source_root_policy: "dx_owned_source_roots_only",
        scanner_status: "allowlist_manifest_ready",
        allowlisted_source_roots: vec![
            source_root(
                "repo_agent_tools",
                "crates/agent/src/tools",
                "repo_first_party",
                "manifest_model_and_agent_tool_bridge",
            ),
            source_root(
                "repo_agent_ui_bridge",
                "crates/agent_ui/src/dx_agent_bridge",
                "repo_first_party",
                "plugins_panel_and_dx_agents_status_bridge",
            ),
            source_root(
                "repo_web_preview",
                "crates/web_preview/src",
                "repo_first_party",
                "browser_plugin_ui_and_receipt_handoffs",
            ),
            source_root(
                "workspace_agent_plugins",
                paths
                    .workspace_plugin_root
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("tools/agent-plugins")),
                "dx_workspace_managed",
                "managed_plugin_receipts_and_assets",
            ),
            source_root(
                "dxjs_runtime",
                paths
                    .workspace_tools_root
                    .clone()
                    .map(|root| root.join("dxjs"))
                    .unwrap_or_else(|| PathBuf::from("G:\\Dx\\js")),
                "dx_runtime_owned",
                "future_bun_dxjs_workflow_execution",
            ),
        ],
        catalog_source: "first_party_manifest_entries_plus_managed_dx_roots",
        rejects_non_allowlisted_roots: true,
        caches_outside_render_paths: true,
    }
}

fn source_root(
    id: &'static str,
    path: impl IntoSourceRootPath,
    ownership: &'static str,
    runtime_use: &'static str,
) -> DxPluginSourceRoot {
    DxPluginSourceRoot {
        id,
        path: path.into_source_root_path(),
        ownership,
        runtime_use,
    }
}

trait IntoSourceRootPath {
    fn into_source_root_path(self) -> String;
}

impl IntoSourceRootPath for &'static str {
    fn into_source_root_path(self) -> String {
        self.to_string()
    }
}

impl IntoSourceRootPath for PathBuf {
    fn into_source_root_path(self) -> String {
        self.display().to_string()
    }
}

fn plugin_receipt_root(paths: &DxPluginCatalogPaths) -> String {
    paths
        .workspace_plugin_root
        .clone()
        .unwrap_or_else(|| paths.zed_data_plugin_root.clone())
        .join("receipts")
        .display()
        .to_string()
}
