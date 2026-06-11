use gpui::{App, Task};
use ui::prelude::*;

use crate::dx_agent_bridge::DxWorkflowNodeSummary;

use super::fields::{DxPluginCredentialInputField, input_text_within_limit};

const MAX_PLUGIN_CREDENTIAL_KEY_SEGMENT_CHARS: usize = 16;
const DX_PLUGIN_CREDENTIAL_KEYCHAIN_PREFIX: &str = "zed://dx/plugins/credentials";

struct DxPluginCredentialWrite {
    keychain_handle: String,
    username: String,
    value: String,
}

pub(super) struct DxPluginCredentialSavedSummary {
    pub save_reference: String,
    pub saved_field_count: usize,
}

pub(super) fn save_plugin_credentials(
    node: &DxWorkflowNodeSummary,
    fields: &[DxPluginCredentialInputField],
    cx: &mut App,
) -> Task<Result<DxPluginCredentialSavedSummary, SharedString>> {
    let writes = match credential_writes(node, fields, cx) {
        Ok(writes) => writes,
        Err(error) => return Task::ready(Err(error)),
    };
    let saved_field_count = writes.len();
    let save_reference = dx_plugin_credential_save_reference(node, saved_field_count);

    let tasks = writes
        .into_iter()
        .map(|write| {
            let task = cx.write_credentials(
                &write.keychain_handle,
                &write.username,
                write.value.as_bytes(),
            );
            task
        })
        .collect::<Vec<_>>();

    cx.spawn(async move |_| {
        for task in tasks {
            task.await.map_err(|_| {
                SharedString::from("Failed to save plugin credential in the OS keychain")
            })?;
        }
        Ok(DxPluginCredentialSavedSummary {
            save_reference,
            saved_field_count,
        })
    })
}

pub(crate) fn credential_storage_unavailable_reason(
    node: &DxWorkflowNodeSummary,
) -> Option<SharedString> {
    if !node.trust.source_owned || !node.trust.first_party {
        return Some("Only DX-owned first-party plugins can store credentials.".into());
    }
    if !node.trust.approved_by_trusted_bridge {
        return Some("Trusted bridge approval is required before storing credentials.".into());
    }
    if node.source_root_id.starts_with("missing_") || node.source_path.starts_with("missing_") {
        return Some(
            "Plugin source metadata is incomplete; refresh the plugin catalog first.".into(),
        );
    }
    None
}

pub(super) fn dx_plugin_credential_keychain_url(
    node_id: &str,
    credential_id: &str,
    input_id: &str,
) -> String {
    format!(
        "{}/{}/{}/{}",
        DX_PLUGIN_CREDENTIAL_KEYCHAIN_PREFIX,
        credential_key_segment(node_id),
        credential_key_segment(credential_id),
        credential_key_segment(input_id)
    )
}

pub(super) fn credentials_saved_prompt(
    node: &DxWorkflowNodeSummary,
    summary: &DxPluginCredentialSavedSummary,
) -> String {
    format!(
        "Plugin credentials were saved locally for `{display_name}`.\n\
         - plugin_node_id: {node_id}\n\
         - source_package: {source_package}\n\
         - source_package_version: {source_package_version}\n\
         - source_root_id: {source_root_id}\n\
         - source_path: {source_path}\n\
         - local_save_reference: {save_reference}\n\
         - saved_field_count: {saved_field_count}\n\
         Use the DX Agents credential bridge only after bridge import support is active. Do not \
         request, echo, or persist credential values or keychain item names in prompts, settings, \
         logs, or receipts.",
        display_name = node.display_name.as_str(),
        node_id = node.id.as_str(),
        source_package = node.source_package.as_str(),
        source_package_version = node.source_package_version.as_str(),
        source_root_id = node.source_root_id.as_str(),
        source_path = node.source_path.as_str(),
        save_reference = summary.save_reference.as_str(),
        saved_field_count = summary.saved_field_count,
    )
}

fn credential_writes(
    node: &DxWorkflowNodeSummary,
    fields: &[DxPluginCredentialInputField],
    cx: &App,
) -> Result<Vec<DxPluginCredentialWrite>, SharedString> {
    validate_plugin_source(node)?;
    if fields.is_empty() {
        return Err("No credential inputs were declared for this plugin.".into());
    }

    let mut writes = Vec::new();
    for field in fields {
        let Some(value) = input_text_within_limit(&field.input, &field.label, field.required, cx)?
        else {
            continue;
        };
        writes.push(DxPluginCredentialWrite {
            keychain_handle: field.keychain_handle.clone(),
            username: credential_keychain_username(&field.credential_id, &field.input_id),
            value,
        });
    }

    if writes.is_empty() {
        Err("Enter at least one credential value before saving.".into())
    } else {
        Ok(writes)
    }
}

fn validate_plugin_source(node: &DxWorkflowNodeSummary) -> Result<(), SharedString> {
    if let Some(reason) = credential_storage_unavailable_reason(node) {
        return Err(reason);
    }
    Ok(())
}

fn credential_keychain_username(credential_id: &str, input_id: &str) -> String {
    format!(
        "dx-plugin:{}:{}",
        credential_key_segment(credential_id),
        credential_key_segment(input_id)
    )
}

fn dx_plugin_credential_save_reference(
    node: &DxWorkflowNodeSummary,
    saved_field_count: usize,
) -> String {
    format!(
        "dx-plugin-credentials:{:016x}",
        credential_key_hash(&format!(
            "{}:{}:{}:{}:{}",
            node.id, node.source_package, node.source_root_id, node.source_path, saved_field_count
        ))
    )
}

fn credential_key_segment(value: &str) -> String {
    format!("{:016x}", credential_key_hash(value))
        .chars()
        .take(MAX_PLUGIN_CREDENTIAL_KEY_SEGMENT_CHARS)
        .collect()
}

fn credential_key_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
