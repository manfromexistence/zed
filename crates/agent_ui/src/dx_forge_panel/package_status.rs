use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

use super::package_status_cache;
use super::roots::ForgeRootContext;
use super::roots::forge_root_contexts;
use super::snapshot::{DxForgeReceiptDrilldown, DxForgeSourceRow};

const MAX_PACKAGE_STATUS_BYTES: u64 = 1024 * 1024;
const MAX_WORKSPACE_ROOTS: usize = 4;

pub(super) fn package_status_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    package_status_cache::package_status_rows(workspace_roots)
}

pub(super) fn invalidate_package_status_snapshot_cache() {
    package_status_cache::invalidate_package_status_snapshot_cache();
}

pub(super) fn scan_package_status_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    let mut rows = Vec::new();
    for context in forge_root_contexts(workspace_roots)
        .into_iter()
        .take(MAX_WORKSPACE_ROOTS)
    {
        if let Some(row) = package_status_candidate_rows(&context) {
            rows.push(row);
        }
    }
    rows
}

fn package_status_candidate_rows(context: &ForgeRootContext) -> Option<DxForgeSourceRow> {
    context
        .forge_package_status_candidates()
        .into_iter()
        .find_map(|path| {
            if !path.is_file() {
                return None;
            }

            Some(match read_package_status_json(&path) {
                Some(value) => package_status_row(context.workspace_root(), &path, &value),
                None => unreadable_package_status_row(context.workspace_root(), &path),
            })
        })
}

fn read_package_status_json(path: &Path) -> Option<Value> {
    let mut file = File::open(path).ok()?;
    let mut buffer = Vec::new();
    file.by_ref()
        .take(MAX_PACKAGE_STATUS_BYTES + 1)
        .read_to_end(&mut buffer)
        .ok()?;
    if buffer.len() as u64 > MAX_PACKAGE_STATUS_BYTES {
        return None;
    }
    serde_json::from_slice(&buffer).ok()
}

fn package_status_row(workspace_root: &Path, path: &Path, value: &Value) -> DxForgeSourceRow {
    if string_field(value, &["schema"]).as_deref() == Some("forge.package_status_receipt") {
        return forge_package_status_row(workspace_root, path, value);
    }

    let status = string_field(value, &["status"]).unwrap_or_else(|| "unknown".to_string());
    let package_count =
        usize_field(value, &["package_count"]).unwrap_or_else(|| package_rows(value).len());
    let current_receipts = current_receipt_count(value);
    let warnings = warning_count(value);
    let node_modules = node_modules_detail(value);
    let evidence_detail = package_status_evidence_detail(value);

    DxForgeSourceRow {
        label: package_status_label(&status),
        detail: status_detail(&status, package_count, current_receipts),
        path: display_path(workspace_root, path),
        open_path: path.display().to_string(),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Status file".to_string(),
            detail: format!("{status} package-status; {node_modules}; {evidence_detail}"),
        }],
        warnings: if warnings == 0 {
            Vec::new()
        } else {
            vec![format!("{warnings} package status warning(s)")]
        },
    }
}

fn unreadable_package_status_row(workspace_root: &Path, path: &Path) -> DxForgeSourceRow {
    let warning =
        format!("package status could not be read within {MAX_PACKAGE_STATUS_BYTES} bytes");

    DxForgeSourceRow {
        label: "Package status unavailable".to_string(),
        detail: warning.clone(),
        path: display_path(workspace_root, path),
        open_path: path.display().to_string(),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Status file".to_string(),
            detail: warning.clone(),
        }],
        warnings: vec![warning],
    }
}

fn forge_package_status_row(workspace_root: &Path, path: &Path, value: &Value) -> DxForgeSourceRow {
    let package_count = usize_field(value, &["summary", "package_count"])
        .unwrap_or_else(|| package_rows(value).len());
    let valid_packages = usize_field(value, &["summary", "valid_packages"]).unwrap_or(0);
    let missing_packages = usize_field(value, &["summary", "missing_packages"]).unwrap_or(0);
    let mismatched_packages = usize_field(value, &["summary", "mismatched_packages"]).unwrap_or(0);
    let unsafe_remote_count = usize_field(value, &["summary", "unsafe_remote_count"]).unwrap_or(0);
    let media_asset_count = usize_field(value, &["summary", "media_asset_count"]).unwrap_or(0);
    let tracked_media_assets =
        usize_field(value, &["summary", "tracked_media_assets"]).unwrap_or(0);
    let package_lock_present = bool_field(value, &["package_lock_present"]).unwrap_or(false);
    let integrity_state =
        string_field(value, &["integrity_state"]).unwrap_or_else(|| "unknown".to_string());
    let missing_summary_fields = forge_summary_missing_count(value);
    let warning_count = missing_packages
        + mismatched_packages
        + unsafe_remote_count
        + media_asset_count.saturating_sub(tracked_media_assets)
        + missing_summary_fields;

    DxForgeSourceRow {
        label: "Package receipt".to_string(),
        detail: format!(
            "{valid_packages}/{package_count} valid · {missing_packages} missing · {mismatched_packages} mismatched · lock {} · media {tracked_media_assets}/{media_asset_count} · integrity {integrity_state}",
            if package_lock_present {
                "present"
            } else {
                "missing"
            },
        ),
        path: display_path(workspace_root, path),
        open_path: path.display().to_string(),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Package receipt".to_string(),
            detail: "receipt file only; live checks not executed".to_string(),
        }],
        warnings: forge_package_status_warnings(warning_count, missing_summary_fields),
    }
}

fn forge_package_status_warnings(
    warning_count: usize,
    missing_summary_fields: usize,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if warning_count > 0 {
        warnings.push(format!(
            "{} {}",
            warning_count,
            plural(
                warning_count,
                "package receipt warning",
                "package receipt warnings"
            )
        ));
    }
    if missing_summary_fields > 0 {
        warnings.push(format!("{missing_summary_fields} summary field(s) missing"));
    }
    warnings
}

fn forge_summary_missing_count(value: &Value) -> usize {
    [
        "package_count",
        "valid_packages",
        "missing_packages",
        "mismatched_packages",
        "unsafe_remote_count",
        "media_asset_count",
        "tracked_media_assets",
    ]
    .into_iter()
    .filter(|field_name| field(value, &["summary", field_name]).is_none())
    .count()
}

fn status_detail(status: &str, package_count: usize, current_receipts: usize) -> String {
    format!("{package_count} packages · {status} · {current_receipts} receipt hashes current")
}

fn package_status_label(status: &str) -> String {
    if status.contains("visibility") {
        "Package status".to_string()
    } else if status.contains("lock") {
        "Package status".to_string()
    } else {
        "Package status".to_string()
    }
}

fn node_modules_detail(value: &Value) -> &'static str {
    if bool_field(value, &["no_node_modules_required"]).unwrap_or(false) {
        "no node_modules required"
    } else {
        "node_modules policy unknown"
    }
}

fn package_status_evidence_detail(value: &Value) -> String {
    let evidence_count = package_rows(value)
        .iter()
        .filter(|row| {
            bool_field(row, &["browser_proof"]).unwrap_or(false)
                || bool_field(row, &["live_provider_proof"]).unwrap_or(false)
                || bool_field(row, &["receipt_hash_refresh", "runtime_execution"]).unwrap_or(false)
        })
        .count();

    if evidence_count == 0 {
        "source-only receipt evidence".to_string()
    } else {
        format!(
            "{} {}",
            evidence_count,
            plural(evidence_count, "evidence flag", "evidence flags")
        )
    }
}

fn current_receipt_count(value: &Value) -> usize {
    package_rows(value)
        .iter()
        .filter(|row| {
            string_field(row, &["receipt_hash_refresh", "status"]).as_deref() == Some("current")
        })
        .count()
}

fn warning_count(value: &Value) -> usize {
    package_rows(value)
        .iter()
        .filter(|row| {
            string_field(row, &["status"]).as_deref() != Some("present")
                || string_field(row, &["receipt_status"]).as_deref() != Some("present")
                || string_field(row, &["receipt_hash_refresh", "status"]).as_deref()
                    != Some("current")
                || usize_field(row, &["receipt_hash_refresh", "stale_file_count"]).unwrap_or(0) > 0
                || usize_field(row, &["receipt_hash_refresh", "missing_file_count"]).unwrap_or(0)
                    > 0
                || array_len_field(row, &["blocked_surfaces"]).unwrap_or(0) > 0
        })
        .count()
}

fn package_rows(value: &Value) -> &[Value] {
    value
        .get("package_lane_visibility")
        .and_then(Value::as_array)
        .or_else(|| value.get("packages").and_then(Value::as_array))
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn display_path(workspace_root: &Path, path: &Path) -> String {
    let path = path.strip_prefix(workspace_root).unwrap_or(path);
    path.display().to_string()
}

fn string_field(value: &Value, path: &[&str]) -> Option<String> {
    field(value, path)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn bool_field(value: &Value, path: &[&str]) -> Option<bool> {
    field(value, path).and_then(Value::as_bool)
}

fn usize_field(value: &Value, path: &[&str]) -> Option<usize> {
    field(value, path)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
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
