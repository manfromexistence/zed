use serde_json::Value;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use super::snapshot::{DxForgeReceiptDrilldown, DxForgeSourceRow};

const MAX_PACKAGE_STATUS_BYTES: u64 = 1024 * 1024;
const MAX_WORKSPACE_ROOTS: usize = 4;

pub(super) fn package_status_rows(workspace_roots: &[String]) -> Vec<DxForgeSourceRow> {
    workspace_roots
        .iter()
        .take(MAX_WORKSPACE_ROOTS)
        .filter_map(|root| {
            let path = Path::new(root)
                .join(".dx")
                .join("forge")
                .join("package-status.json");
            let value = read_package_status_json(&path)?;
            Some(package_status_row(root, &path, &value))
        })
        .collect()
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

fn package_status_row(workspace_root: &str, path: &Path, value: &Value) -> DxForgeSourceRow {
    let status = string_field(value, &["status"]).unwrap_or_else(|| "unknown".to_string());
    let package_count =
        usize_field(value, &["package_count"]).unwrap_or_else(|| package_rows(value).len());
    let current_receipts = current_receipt_count(value);
    let warnings = warning_count(value);
    let proof_detail = proof_detail(value);

    DxForgeSourceRow {
        label: package_status_label(&status),
        detail: status_detail(
            value,
            &status,
            package_count,
            current_receipts,
            &proof_detail,
        ),
        path: display_path(workspace_root, path),
        receipts: vec![DxForgeReceiptDrilldown {
            label: "Read model".to_string(),
            detail: format!("{status} package-status; {proof_detail}"),
        }],
        warnings: if warnings == 0 {
            Vec::new()
        } else {
            vec![format!("{warnings} package status warning(s)")]
        },
    }
}

fn status_detail(
    value: &Value,
    status: &str,
    package_count: usize,
    current_receipts: usize,
    proof_detail: &str,
) -> String {
    let node_modules = if bool_field(value, &["no_node_modules_required"]).unwrap_or(false) {
        "no node_modules required"
    } else {
        "node_modules policy unknown"
    };
    format!(
        "{package_count} packages · {status} · {current_receipts} receipt hashes current · {node_modules} · {proof_detail}"
    )
}

fn package_status_label(status: &str) -> String {
    if status.contains("visibility") {
        "Root package visibility".to_string()
    } else if status.contains("lock") {
        "Lock-backed packages".to_string()
    } else {
        "Package status".to_string()
    }
}

fn proof_detail(value: &Value) -> String {
    let proof_count = package_rows(value)
        .iter()
        .filter(|row| {
            bool_field(row, &["browser_proof"]).unwrap_or(false)
                || bool_field(row, &["live_provider_proof"]).unwrap_or(false)
                || bool_field(row, &["receipt_hash_refresh", "runtime_execution"]).unwrap_or(false)
        })
        .count();

    if proof_count == 0 {
        "runtime/provider proof pending".to_string()
    } else {
        format!("{proof_count} runtime/provider evidence flag(s)")
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

fn display_path(workspace_root: &str, path: &Path) -> String {
    let root = PathBuf::from(workspace_root);
    path.strip_prefix(&root)
        .unwrap_or(path)
        .display()
        .to_string()
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
