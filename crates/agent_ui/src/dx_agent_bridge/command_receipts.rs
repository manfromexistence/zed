use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, anyhow};
use serde_json::{Value, json};

use super::{MAX_RECEIPT_BYTES, redact_action_scalar, string_field};

const MAX_FAILED_COMMAND_STDERR_BYTES: usize = 2048;
const MAX_FAILED_COMMAND_STDERR_CHARS: usize = 500;
const MAX_ACTION_ERROR_DISPLAY_CHARS: usize = 500;

pub(super) fn failed_command_stderr_display(stderr: &[u8]) -> String {
    let truncated_bytes = stderr.len() > MAX_FAILED_COMMAND_STDERR_BYTES;
    let visible_len = stderr.len().min(MAX_FAILED_COMMAND_STDERR_BYTES);
    let decoded = String::from_utf8_lossy(&stderr[..visible_len]);
    let compact = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    let truncated_chars = compact.chars().count() > MAX_FAILED_COMMAND_STDERR_CHARS;

    if !truncated_bytes && !truncated_chars {
        return compact;
    }

    let mut display = compact
        .chars()
        .take(MAX_FAILED_COMMAND_STDERR_CHARS.saturating_sub(3))
        .collect::<String>();
    display.push_str("...");
    display
}

pub(super) fn write_json_receipt(path: &Path, stdout: &[u8], expected_schema: &str) -> Result<()> {
    if u64::try_from(stdout.len()).unwrap_or(u64::MAX) > MAX_RECEIPT_BYTES {
        return Err(anyhow!("DX Agents metadata response is too large"));
    }

    let value: Value = serde_json::from_slice(stdout)
        .context("DX Agents metadata command returned invalid JSON")?;
    let schema_version = string_field(&value, &["schema_version"])
        .ok_or_else(|| anyhow!("DX Agents metadata JSON is missing schema_version"))?;
    if schema_version != expected_schema {
        return Err(anyhow!(
            "DX Agents metadata JSON schema mismatch: expected {expected_schema}, got {schema_version}"
        ));
    }

    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("DX Agents metadata receipt path has no parent"))?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create DX Agents metadata receipt directory `{}`",
            parent.display()
        )
    })?;

    let bytes = serialized_pretty_receipt(&value, "metadata")?;
    write_receipt_bytes(path, bytes, "metadata")?;
    Ok(())
}

pub(super) fn write_action_error_receipt(
    receipt_root: &Path,
    command: &str,
    error: &anyhow::Error,
) -> Result<()> {
    let path = receipt_root.join("action-error-latest.json");
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("DX Agents action error receipt path has no parent"))?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create DX Agents action error receipt directory `{}`",
            parent.display()
        )
    })?;

    let generated_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let value = json!({
        "schema_version": "dx.agents.zed.action_error.v1",
        "command": action_error_display_field(command),
        "status": "missing_config",
        "generated_at": generated_at_ms.to_string(),
        "generated_at_ms": generated_at_ms,
        "error": action_error_display_field(&error.to_string()),
        "next_action": "review_dx_agents_cli_path_or_receipt_root",
        "redaction": {
            "exports_secret_values": false,
            "exports_provider_credentials": false,
            "exports_receipt_bodies": false
        }
    });
    let bytes = serialized_pretty_receipt(&value, "action error")?;
    write_receipt_bytes(&path, bytes, "action error")?;
    Ok(())
}

pub(super) fn clear_action_error_receipt(receipt_root: &Path) {
    let path = receipt_root.join("action-error-latest.json");
    if path.is_file() {
        let _ = fs::remove_file(path);
    }
}

fn action_error_display_field(value: &str) -> String {
    let redacted = redact_action_scalar(value);
    let display = failed_command_stderr_display(redacted.as_bytes());
    debug_assert!(display.chars().count() <= MAX_ACTION_ERROR_DISPLAY_CHARS);
    display
}

fn serialized_pretty_receipt(value: &Value, receipt_kind: &str) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .with_context(|| format!("failed to serialize DX Agents {receipt_kind} JSON"))?;
    bytes.push(b'\n');
    ensure_serialized_receipt_bytes(receipt_kind, &bytes)?;
    Ok(bytes)
}

fn write_receipt_bytes(path: &Path, bytes: Vec<u8>, receipt_kind: &str) -> Result<()> {
    let temp_path = temp_receipt_path(path)?;
    fs::write(&temp_path, bytes).with_context(|| {
        format!(
            "failed to write DX Agents {receipt_kind} temporary receipt `{}`",
            temp_path.display()
        )
    })?;

    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(rename_error)
            if rename_error.kind() == std::io::ErrorKind::AlreadyExists && path.is_file() =>
        {
            fs::remove_file(path).with_context(|| {
                format!(
                    "failed to replace existing DX Agents {receipt_kind} receipt `{}`",
                    path.display()
                )
            })?;
            fs::rename(&temp_path, path).with_context(|| {
                let _ = fs::remove_file(&temp_path);
                format!(
                    "failed to move DX Agents {receipt_kind} receipt `{}` into place after replace error: {rename_error}",
                    path.display()
                )
            })
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(error).with_context(|| {
                format!(
                    "failed to move DX Agents {receipt_kind} receipt `{}` into place",
                    path.display()
                )
            })
        }
    }
}

fn temp_receipt_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("DX Agents receipt path has no file name"))?
        .to_string_lossy();
    let generated_at_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    Ok(path.with_file_name(format!(
        ".{file_name}.{}.{generated_at_nanos}.tmp",
        std::process::id()
    )))
}

fn ensure_serialized_receipt_bytes(receipt_kind: &str, bytes: &[u8]) -> Result<()> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_RECEIPT_BYTES {
        return Err(anyhow!(
            "DX Agents {receipt_kind} receipt is too large after serialization"
        ));
    }
    Ok(())
}
