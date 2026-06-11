mod entries;

use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) const DX_PLUGIN_MANIFEST_SCHEMA: &str = "zed.dx_plugins.manifest.v1";
pub(crate) const DX_PLUGIN_CATALOG_SCHEMA: &str = "zed.dx_plugins.catalog.v1";
pub(crate) const DX_PLUGIN_CATALOG_DISCOVERY_SCHEMA: &str = "zed.dx_plugins.catalog_discovery.v1";
pub(crate) const DX_PLUGIN_CATALOG_SUMMARY_SCHEMA: &str = "zed.dx_plugins.catalog_summary.v1";
pub(crate) const DX_PLUGIN_SOURCE_ROOT_INTEGRITY_SCHEMA: &str =
    "zed.dx_plugins.source_root_integrity.v1";
pub(crate) const DX_PLUGIN_RUNTIME_STATUS_ALIAS_SCHEMA: &str =
    "zed.dx_plugins.runtime_status_alias.v1";
pub(crate) const DX_PLUGIN_RUNTIME_STATUS_ALIASES_SCHEMA: &str =
    "zed.dx_plugins.runtime_status_aliases.v1";

#[derive(Clone)]
pub(crate) struct DxPluginCatalogPaths {
    pub(crate) project_root: Option<PathBuf>,
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
    pub(crate) source_integrity: DxPluginSourceRootIntegrity,
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

#[derive(Clone, Serialize)]
pub(crate) struct DxPluginSourceRoot {
    pub(crate) id: &'static str,
    pub(crate) path: String,
    pub(crate) ownership: &'static str,
    pub(crate) runtime_use: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DxPluginSourceRootIntegrity {
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) valid: bool,
    pub(crate) checked_manifest_count: usize,
    pub(crate) checked_runtime_count: usize,
    pub(crate) allowlisted_source_root_count: usize,
    pub(crate) violation_count: usize,
    pub(crate) source_root_policy: &'static str,
    pub(crate) canonicalization_mode: &'static str,
    pub(crate) symlink_escape_protection: &'static str,
    pub(crate) manifest_source_roots_known: bool,
    pub(crate) runtime_source_roots_known: bool,
    pub(crate) runtime_roots_declared_by_manifest: bool,
    pub(crate) runtime_entrypoints_under_source_roots: bool,
    pub(crate) default_enabled_plugins_safe: bool,
    pub(crate) violations: Vec<DxPluginSourceRootIntegrityViolation>,
}

#[derive(Serialize)]
pub(crate) struct DxPluginSourceRootIntegrityViolation {
    pub(crate) plugin_id: String,
    pub(crate) field: &'static str,
    pub(crate) source_root_id: String,
    pub(crate) entrypoint: Option<String>,
    pub(crate) reason: &'static str,
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
    pub(crate) runtime_status_alias: DxPluginRuntimeStatusAlias,
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
pub(crate) struct DxPluginRuntimeStatusAlias {
    pub(crate) schema: &'static str,
    pub(crate) canonical_plugin_id: &'static str,
    pub(crate) maps_to_runtime_plugin_ids: Vec<&'static str>,
    pub(crate) maps_to_lane_ids: Vec<&'static str>,
    pub(crate) runtime_status_fields: Vec<&'static str>,
    pub(crate) readiness_fields: Vec<&'static str>,
    pub(crate) claim_policy: &'static str,
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
    let discovery = dx_plugin_catalog_discovery(&paths);
    let manifests = entries::first_party_plugin_manifests(receipt_root);
    let source_integrity = validate_dx_plugin_catalog_sources(&discovery, &manifests, &paths);
    let source_integrity_valid = source_integrity.valid;
    let catalog = DxPluginCatalog {
        schema: DX_PLUGIN_CATALOG_SCHEMA,
        status: if source_integrity_valid {
            "first_party_manifest_foundation_ready"
        } else {
            "manifest_source_root_validation_failed"
        },
        source: "dx_first_party_manifest_entries",
        product_surface: "Plugins",
        workflow_surface: "Workflows",
        node_surface: "Nodes",
        credential_surface: "Credentials",
        receipt_surface: "Receipts",
        available_to: vec!["zed_plugins_panel", "dx_agents_bridge", "agent_panel"],
        discovery,
        source_integrity,
        bridge: DxPluginBridge {
            catalog_tool,
            runtime_status_tool,
            panels_ready_for: vec!["zed_plugins_panel", "dx_agents_bridge"],
            agents_bridge_ready: source_integrity_valid,
            exposes_enabled_trusted_only: true,
        },
        manifests: if source_integrity_valid {
            manifests
        } else {
            Vec::new()
        },
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
    let source_integrity_valid = catalog
        .pointer("/source_integrity/valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let default_enabled_plugins = if source_integrity_valid {
        vec!["dx.browser", "dx.computer", "dx.driven"]
    } else {
        Vec::new()
    };

    serde_json::json!({
        "schema": DX_PLUGIN_CATALOG_SUMMARY_SCHEMA,
        "source_schema": DX_PLUGIN_CATALOG_SCHEMA,
        "status": catalog.get("status").and_then(Value::as_str),
        "plugin_count": manifests.len(),
        "plugin_ids": plugin_ids,
        "categories": categories,
        "default_enabled_plugins": default_enabled_plugins,
        "source_root_policy": source_root_policy,
        "allowlisted_source_root_count": source_root_count,
        "source_integrity_schema": DX_PLUGIN_SOURCE_ROOT_INTEGRITY_SCHEMA,
        "source_integrity_status": catalog.pointer("/source_integrity/status").and_then(Value::as_str),
        "source_integrity_valid": source_integrity_valid,
        "source_root_violation_count": catalog.pointer("/source_integrity/violation_count").and_then(Value::as_u64).unwrap_or(0),
        "runtime_entrypoints_under_source_roots": catalog.pointer("/source_integrity/runtime_entrypoints_under_source_roots").and_then(Value::as_bool).unwrap_or(false),
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
                "repo_agent_ui_bridge_module",
                "crates/agent_ui/src/dx_agent_bridge.rs",
                "repo_first_party",
                "dx_agents_bridge_module_entrypoint",
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
                "workspace_playwright_runner",
                paths
                    .workspace_tools_root
                    .clone()
                    .map(|root| root.join("playwright").join("zed-managed-chrome-runner"))
                    .unwrap_or_else(|| {
                        PathBuf::from("tools/playwright").join("zed-managed-chrome-runner")
                    }),
                "dx_workspace_managed",
                "managed_chrome_playwright_runner_artifact",
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

fn validate_dx_plugin_catalog_sources(
    discovery: &DxPluginCatalogDiscovery,
    manifests: &[DxPluginManifest],
    paths: &DxPluginCatalogPaths,
) -> DxPluginSourceRootIntegrity {
    let allowlisted_roots = discovery
        .allowlisted_source_roots
        .iter()
        .map(|root| (root.id, root))
        .collect::<HashMap<_, _>>();
    let mut violations = Vec::new();

    for manifest in manifests {
        let manifest_source_roots = manifest
            .source_root_ids
            .iter()
            .copied()
            .collect::<HashSet<_>>();

        for source_root_id in &manifest.source_root_ids {
            if !allowlisted_roots.contains_key(source_root_id) {
                violations.push(source_root_integrity_violation(
                    manifest.id,
                    "source_root_ids",
                    source_root_id,
                    None,
                    "manifest_source_root_id_not_allowlisted",
                ));
            }
        }

        let runtime_source_root_id = manifest.runtime.source_root_id;
        let runtime_source_root = allowlisted_roots.get(runtime_source_root_id);

        if runtime_source_root.is_none() {
            violations.push(source_root_integrity_violation(
                manifest.id,
                "runtime.source_root_id",
                runtime_source_root_id,
                Some(manifest.runtime.entrypoint),
                "runtime_source_root_id_not_allowlisted",
            ));
        }

        if !manifest_source_roots.contains(runtime_source_root_id) {
            violations.push(source_root_integrity_violation(
                manifest.id,
                "runtime.source_root_id",
                runtime_source_root_id,
                Some(manifest.runtime.entrypoint),
                "runtime_source_root_id_not_declared_by_manifest",
            ));
        }

        if let Some(root) = runtime_source_root
            && !runtime_entrypoint_is_under_source_root(paths, root, manifest.runtime.entrypoint)
        {
            violations.push(source_root_integrity_violation(
                manifest.id,
                "runtime.entrypoint",
                runtime_source_root_id,
                Some(manifest.runtime.entrypoint),
                "runtime_entrypoint_outside_source_root_after_canonicalization",
            ));
        }
    }

    let valid = violations.is_empty();

    DxPluginSourceRootIntegrity {
        schema: DX_PLUGIN_SOURCE_ROOT_INTEGRITY_SCHEMA,
        status: if valid {
            "source_root_integrity_verified"
        } else {
            "source_root_integrity_failed"
        },
        valid,
        checked_manifest_count: manifests.len(),
        checked_runtime_count: manifests.len(),
        allowlisted_source_root_count: discovery.allowlisted_source_roots.len(),
        violation_count: violations.len(),
        source_root_policy: discovery.source_root_policy,
        canonicalization_mode: "canonicalize_existing_ancestors_then_normalize_without_directory_scan",
        symlink_escape_protection: "existing_ancestors_are_canonicalized; missing_managed_artifact_suffixes_remain_lexically_bounded",
        manifest_source_roots_known: !violations
            .iter()
            .any(|violation| violation.reason == "manifest_source_root_id_not_allowlisted"),
        runtime_source_roots_known: !violations
            .iter()
            .any(|violation| violation.reason == "runtime_source_root_id_not_allowlisted"),
        runtime_roots_declared_by_manifest: !violations
            .iter()
            .any(|violation| violation.reason == "runtime_source_root_id_not_declared_by_manifest"),
        runtime_entrypoints_under_source_roots: !violations.iter().any(|violation| {
            violation.reason == "runtime_entrypoint_outside_source_root_after_canonicalization"
        }),
        default_enabled_plugins_safe: valid,
        violations,
    }
}

fn source_root_integrity_violation(
    plugin_id: &str,
    field: &'static str,
    source_root_id: &str,
    entrypoint: Option<&str>,
    reason: &'static str,
) -> DxPluginSourceRootIntegrityViolation {
    DxPluginSourceRootIntegrityViolation {
        plugin_id: plugin_id.to_string(),
        field,
        source_root_id: source_root_id.to_string(),
        entrypoint: entrypoint.map(str::to_string),
        reason,
    }
}

fn runtime_entrypoint_is_under_source_root(
    paths: &DxPluginCatalogPaths,
    root: &DxPluginSourceRoot,
    entrypoint: &str,
) -> bool {
    let root_path = resolve_manifest_path(paths, &root.path);
    let entrypoint_path = resolve_manifest_path(paths, entrypoint);

    if path_contains_parent_component(&root_path)
        || path_contains_parent_component(&entrypoint_path)
    {
        return false;
    }

    let root_path = canonicalize_manifest_path_with_ancestors(&root_path);
    let entrypoint_path = canonicalize_manifest_path_with_ancestors(&entrypoint_path);

    normalized_path_starts_with(&entrypoint_path, &root_path)
}

fn resolve_manifest_path(paths: &DxPluginCatalogPaths, path: &str) -> PathBuf {
    let path = PathBuf::from(path);

    if path.is_absolute() {
        path
    } else if let Some(project_root) = paths.project_root.as_ref() {
        project_root.join(path)
    } else {
        path
    }
}

fn canonicalize_manifest_path_with_ancestors(path: &Path) -> PathBuf {
    let mut current = normalize_manifest_path(path);
    let mut missing_suffixes: Vec<PathBuf> = Vec::new();

    loop {
        if current.exists()
            && let Ok(canonical) = fs::canonicalize(&current)
        {
            let mut resolved = canonical;
            for suffix in missing_suffixes.iter().rev() {
                resolved.push(suffix);
            }
            return normalize_manifest_path(&resolved);
        }

        let Some(file_name) = current
            .file_name()
            .map(|name| PathBuf::from(name.to_os_string()))
        else {
            return normalize_manifest_path(path);
        };

        if !current.pop() {
            return normalize_manifest_path(path);
        }

        missing_suffixes.push(file_name);
    }
}

fn normalize_manifest_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::Normal(part) => normalized.push(part),
        }
    }

    normalized
}

fn path_contains_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn normalized_path_starts_with(path: &Path, root: &Path) -> bool {
    let Some(path_parts) = comparable_path_parts(path) else {
        return false;
    };
    let Some(root_parts) = comparable_path_parts(root) else {
        return false;
    };

    !root_parts.is_empty()
        && path_parts.len() >= root_parts.len()
        && path_parts
            .iter()
            .zip(root_parts.iter())
            .all(|(path_part, root_part)| path_part == root_part)
}

fn comparable_path_parts(path: &Path) -> Option<Vec<String>> {
    let mut parts = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return None,
            Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().to_ascii_lowercase());
            }
            Component::RootDir => parts.push(component.as_os_str().to_string_lossy().to_string()),
            Component::Normal(part) => {
                parts.push(part.to_string_lossy().to_ascii_lowercase());
            }
        }
    }

    Some(parts)
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
