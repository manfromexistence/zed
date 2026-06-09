use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use self::providers::{canonical_kind_label, catalog_provider_info, remote_providers};
use super::roots::forge_root_contexts;
use super::snapshot::{DxForgeReceiptDrilldown, DxForgeRemoteProvider, DxForgeSourceRow};

#[path = "remote_registry/providers.rs"]
mod providers;

const REMOTE_REGISTRY_CACHE_TTL: Duration = Duration::from_secs(5);
const MAX_REMOTE_REGISTRY_BYTES: u64 = 256 * 1024;
const MAX_WORKSPACE_ROOTS: usize = 4;
const MAX_REMOTE_ROWS: usize = 64;

static REMOTE_REGISTRY_CACHE: OnceLock<
    Mutex<Option<(Instant, Vec<String>, ForgeRemoteRegistrySnapshot)>>,
> = OnceLock::new();

#[derive(Clone, Default)]
pub(super) struct ForgeRemoteRegistrySnapshot {
    pub(super) rows: Vec<DxForgeSourceRow>,
    pub(super) providers: Vec<DxForgeRemoteProvider>,
    pub(super) warning_count: usize,
}

pub(super) fn remote_registry_snapshot(workspace_roots: &[String]) -> ForgeRemoteRegistrySnapshot {
    let cache = REMOTE_REGISTRY_CACHE.get_or_init(|| Mutex::new(None));
    let now = Instant::now();

    if let Ok(mut cache) = cache.lock() {
        if let Some((cached_at, cached_roots, snapshot)) = cache.as_ref()
            && cached_roots == workspace_roots
            && now.duration_since(*cached_at) <= REMOTE_REGISTRY_CACHE_TTL
        {
            return snapshot.clone();
        }

        let snapshot = scan_remote_registries(workspace_roots);
        *cache = Some((now, workspace_roots.to_vec(), snapshot.clone()));
        return snapshot;
    }

    scan_remote_registries(workspace_roots)
}

pub(super) fn invalidate_remote_registry_snapshot_cache() {
    let cache = REMOTE_REGISTRY_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        *cache = None;
    }
}

fn scan_remote_registries(workspace_roots: &[String]) -> ForgeRemoteRegistrySnapshot {
    let mut snapshot = ForgeRemoteRegistrySnapshot::default();

    for context in forge_root_contexts(workspace_roots)
        .into_iter()
        .take(MAX_WORKSPACE_ROOTS)
    {
        let path = context.forge_remote_registry_path();
        if !path.is_file() {
            continue;
        }

        let Some(value) = read_remote_registry_json(&path) else {
            snapshot
                .rows
                .push(unreadable_registry_row(context.workspace_root(), &path));
            snapshot.warning_count += 1;
            continue;
        };

        let (row, providers) = remote_registry_row(context.workspace_root(), &path, &value);
        snapshot.warning_count += row.warnings.len();
        snapshot.providers.extend(providers);
        snapshot.rows.push(row);
    }

    snapshot
}

fn read_remote_registry_json(path: &Path) -> Option<Value> {
    let mut file = File::open(path).ok()?;
    let mut buffer = Vec::new();
    file.by_ref()
        .take(MAX_REMOTE_REGISTRY_BYTES + 1)
        .read_to_end(&mut buffer)
        .ok()?;
    if buffer.len() as u64 > MAX_REMOTE_REGISTRY_BYTES {
        return None;
    }
    serde_json::from_slice(&buffer).ok()
}

fn remote_registry_row(
    workspace_root: &Path,
    path: &Path,
    value: &Value,
) -> (DxForgeSourceRow, Vec<DxForgeRemoteProvider>) {
    let remotes = remote_values(value);
    let primary = primary_field(value);
    let enabled_count = remotes.iter().filter(|remote| enabled(remote)).count();
    let disabled_count = remotes.len().saturating_sub(enabled_count);
    let branch_mapping_count = remotes
        .iter()
        .map(|remote| array_len_field(remote, &["branch_mappings"]).unwrap_or(0))
        .sum::<usize>();
    let auth_backend_count = remotes
        .iter()
        .filter(|remote| string_field(remote, &["auth_backend"]).is_some())
        .count();
    let kinds = kind_counts(&remotes);
    let warnings = registry_warnings(&remotes, primary.as_deref(), disabled_count);
    let path_label = display_path(workspace_root, path);
    let registry_open_path = path.display().to_string();
    let providers = remote_providers(
        &remotes,
        primary.as_deref(),
        &path_label,
        &registry_open_path,
        MAX_REMOTE_ROWS,
    );
    let primary_detail = primary
        .as_deref()
        .map(|name| format!("primary {name}"))
        .unwrap_or_else(|| "no primary".to_string());

    (
        DxForgeSourceRow {
            label: "Forge remotes".to_string(),
            detail: format!(
                "{} {} · {} enabled · {} · {} · {} auth backend(s) · health unchecked",
                remotes.len(),
                plural(remotes.len(), "remote", "remotes"),
                enabled_count,
                primary_detail,
                kind_detail(&kinds),
                auth_backend_count,
            ),
            path: path_label,
            open_path: registry_open_path.clone(),
            receipts: vec![
                DxForgeReceiptDrilldown {
                    label: "Kinds".to_string(),
                    detail: kind_detail(&kinds),
                },
                DxForgeReceiptDrilldown {
                    label: "Mappings".to_string(),
                    detail: format!("{branch_mapping_count} mapping(s)"),
                },
            ],
            warnings,
        },
        providers,
    )
}

fn unreadable_registry_row(workspace_root: &Path, path: &Path) -> DxForgeSourceRow {
    DxForgeSourceRow {
        label: "Forge remotes".to_string(),
        detail: "remote registry unreadable or above bounded read limit".to_string(),
        path: display_path(workspace_root, path),
        open_path: path.display().to_string(),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Registry".to_string(),
            detail: "registry file only; health unchecked".to_string(),
        }],
        warnings: vec![format!(
            "remote registry could not be read within {} bytes",
            MAX_REMOTE_REGISTRY_BYTES
        )],
    }
}

fn registry_warnings(
    remotes: &[&Value],
    primary: Option<&str>,
    disabled_count: usize,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if remotes.is_empty() {
        warnings.push("remote registry has no remotes".to_string());
    }
    if disabled_count > 0 {
        warnings.push(format!("{disabled_count} disabled remote(s)"));
    }

    let names = remotes
        .iter()
        .filter_map(|remote| string_field(remote, &["name"]))
        .collect::<BTreeSet<_>>();
    if let Some(primary) = primary
        && !names.contains(primary)
    {
        warnings.push(format!("primary remote '{primary}' is not listed"));
    }

    let unknown_count = remotes
        .iter()
        .filter(|remote| {
            string_field(remote, &["kind"])
                .as_deref()
                .and_then(catalog_provider_info)
                .is_none()
        })
        .count();
    if unknown_count > 0 {
        warnings.push(format!(
            "{unknown_count} remote kind(s) not in provider icon catalog"
        ));
    }

    warnings
}

fn kind_counts(remotes: &[&Value]) -> Vec<String> {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for remote in remotes {
        let kind = string_field(remote, &["kind"])
            .map(|kind| canonical_kind_label(&kind))
            .unwrap_or("Unknown");
        *counts.entry(kind).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("{kind} {count}"))
        .collect()
}

fn kind_detail(kinds: &[String]) -> String {
    if kinds.is_empty() {
        "no remote kinds".to_string()
    } else {
        kinds.join(", ")
    }
}

fn remote_values(value: &Value) -> Vec<&Value> {
    value
        .get("remotes")
        .and_then(Value::as_array)
        .map(|remotes| remotes.iter().take(MAX_REMOTE_ROWS).collect())
        .unwrap_or_default()
}

fn enabled(value: &Value) -> bool {
    bool_field(value, &["enabled"]).unwrap_or(true)
}

fn display_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn string_field(value: &Value, path: &[&str]) -> Option<String> {
    field(value, path)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn primary_field(value: &Value) -> Option<String> {
    value
        .get("primary")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    field(value, path).and_then(Value::as_bool)
}

fn array_len_field(value: &Value, path: &[&str]) -> Option<usize> {
    field(value, path).and_then(Value::as_array).map(Vec::len)
}

fn field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 { singular } else { plural }
}
