use super::configured::DxConfiguredPluginSummary;

const MAX_CONFIGURED_PLUGIN_ID_CHARS: usize = 96;
const MAX_CONFIGURED_PLUGIN_SOURCE_CHARS: usize = 240;
const TRUSTED_TOOL_POLICY: &str = "receipt_authorized_only";
const DX_CONFIGURED_PLUGIN_SOURCE_ROOTS: &[&str] = &[
    "repo_agent_tools",
    "repo_agent_ui_bridge",
    "repo_agent_ui_bridge_module",
    "repo_web_preview",
    "workspace_agent_plugins",
    "workspace_playwright_runner",
    "dxjs_runtime",
];

pub(super) fn configured_plugin_row_is_authorized(row: &DxConfiguredPluginSummary) -> bool {
    configured_plugin_identity_is_safe(&row.id)
        && configured_plugin_identity_is_safe(&row.node_id)
        && configured_plugin_identity_is_safe(&row.action_id)
        && configured_plugin_identity_is_safe(&row.receipt_id)
        && configured_plugin_source_root_is_allowed(&row.source_root_id)
        && configured_plugin_source_path_is_safe(&row.source_path)
        && row.trust_policy == TRUSTED_TOOL_POLICY
        && row.approved_by_trusted_bridge
        && row.writes_receipt
        && !row.secrets_exposed
        && configured_plugin_state_is_usable(&row.status)
        && configured_plugin_state_is_usable(&row.credential_status)
}

fn configured_plugin_identity_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= MAX_CONFIGURED_PLUGIN_ID_CHARS
        && value.bytes().all(|byte| {
            matches!(
                byte,
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b':' | b'/' | b'@'
            )
        })
        && !value.starts_with("missing_")
        && !matches!(value, "unknown" | "unavailable" | "disabled")
}

fn configured_plugin_source_root_is_allowed(value: &str) -> bool {
    let value = value.trim();
    configured_plugin_identity_is_safe(value) && DX_CONFIGURED_PLUGIN_SOURCE_ROOTS.contains(&value)
}

fn configured_plugin_source_path_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= MAX_CONFIGURED_PLUGIN_SOURCE_CHARS
        && !value.starts_with("missing_")
        && !value.contains('\0')
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value.contains('\\')
        && !has_windows_drive_prefix(value)
        && value
            .split('/')
            .all(|segment| !matches!(segment, "" | "." | ".."))
        && !matches!(value, "unknown" | "unavailable" | "disabled")
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn configured_plugin_state_is_usable(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "active"
            | "authorized"
            | "available"
            | "configured"
            | "connected"
            | "enabled"
            | "healthy"
            | "not_required"
            | "ready"
            | "valid"
    )
}
