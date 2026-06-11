use gpui::{App, Entity, Window};
use ui::prelude::*;
use ui_input::InputField;

use crate::dx_agent_bridge::{
    DxWorkflowNodeCredentialInputSummary, DxWorkflowNodeCredentialSummary, DxWorkflowNodeSummary,
};

pub(super) const MAX_PLUGIN_CREDENTIAL_INPUT_BYTES: usize = 16 * 1024;
const MAX_PLUGIN_CREDENTIAL_INPUT_CHARS: usize = 16 * 1024;
const MAX_PLUGIN_CREDENTIAL_LABEL_CHARS: usize = 96;

pub(super) struct DxPluginCredentialInputField {
    pub credential_id: String,
    pub input_id: String,
    pub label: String,
    pub required: bool,
    pub keychain_handle: String,
    pub input: Entity<InputField>,
}

pub(super) fn credential_input_fields(
    node: &DxWorkflowNodeSummary,
    window: &mut Window,
    cx: &mut App,
    keychain_url: impl Fn(&str, &str, &str) -> String,
) -> Vec<DxPluginCredentialInputField> {
    let mut tab_index = 1;
    node.credentials
        .iter()
        .flat_map(|credential| {
            credential
                .inputs
                .iter()
                .map(move |input| (credential, input))
        })
        .map(|(credential, input)| {
            let field = credential_input_field(
                credential,
                input,
                tab_index,
                node,
                window,
                cx,
                &keychain_url,
            );
            tab_index += 1;
            field
        })
        .collect()
}

pub(super) fn clear_secret_input_fields(
    fields: &[DxPluginCredentialInputField],
    window: &mut Window,
    cx: &mut App,
) {
    for field in fields {
        field.input.update(cx, |input, cx| input.clear(window, cx));
    }
}

pub(super) fn input_text_within_limit(
    input: &Entity<InputField>,
    label: &str,
    required: bool,
    cx: &App,
) -> Result<Option<String>, SharedString> {
    let text = input.read(cx).text(cx);
    if text.len() > MAX_PLUGIN_CREDENTIAL_INPUT_BYTES {
        return Err(format!(
            "{label} is too large (maximum {MAX_PLUGIN_CREDENTIAL_INPUT_BYTES} bytes)"
        )
        .into());
    }
    if text.chars().count() > MAX_PLUGIN_CREDENTIAL_INPUT_CHARS {
        return Err(format!(
            "{label} is too large (maximum {MAX_PLUGIN_CREDENTIAL_INPUT_CHARS} characters)"
        )
        .into());
    }
    if text.trim().is_empty() {
        if required {
            return Err(format!("{label} is required").into());
        }
        return Ok(None);
    }
    Ok(Some(text))
}

fn credential_input_field(
    credential: &DxWorkflowNodeCredentialSummary,
    input: &DxWorkflowNodeCredentialInputSummary,
    tab_index: isize,
    node: &DxWorkflowNodeSummary,
    window: &mut Window,
    cx: &mut App,
    keychain_url: &impl Fn(&str, &str, &str) -> String,
) -> DxPluginCredentialInputField {
    let label = credential_input_label(credential, input, tab_index);
    let keychain_handle = keychain_url(&node.id, &credential.id, &input.id);
    let placeholder = credential_input_placeholder(tab_index);
    let input_field = cx.new(|cx| {
        InputField::new(window, cx, placeholder.as_str())
            .label(label.clone())
            .tab_index(tab_index)
            .tab_stop(true)
            .masked(true)
    });

    DxPluginCredentialInputField {
        credential_id: credential.id.clone(),
        input_id: input.id.clone(),
        label,
        required: input.required,
        keychain_handle,
        input: input_field,
    }
}

fn credential_input_placeholder(tab_index: isize) -> String {
    format!("Stored securely in OS keychain {tab_index}")
}

fn credential_input_label(
    credential: &DxWorkflowNodeCredentialSummary,
    input: &DxWorkflowNodeCredentialInputSummary,
    tab_index: isize,
) -> String {
    let candidate = if credential.credential_type == input.label {
        credential.credential_type.clone()
    } else {
        format!("{} / {}", credential.credential_type, input.label)
    };
    safe_credential_label(candidate, tab_index)
}

fn safe_credential_label(candidate: String, tab_index: isize) -> String {
    let compact = candidate.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty()
        || compact.chars().count() > MAX_PLUGIN_CREDENTIAL_LABEL_CHARS
        || looks_like_secret_value(&compact)
    {
        format!("Credential field {tab_index}")
    } else {
        compact
    }
}

fn looks_like_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let normalized = lower.replace(['-', ' '], "_");
    SECRET_METADATA_MARKERS
        .iter()
        .any(|marker| lower.contains(marker) || normalized.contains(marker))
        || has_high_entropy_shape(value)
}

fn has_high_entropy_shape(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    compact.len() >= 32
        && compact.chars().any(|ch| ch.is_ascii_lowercase())
        && compact.chars().any(|ch| ch.is_ascii_uppercase())
        && compact.chars().any(|ch| ch.is_ascii_digit())
}

const SECRET_METADATA_MARKERS: &[&str] = &[
    concat!("sk", "-"),
    concat!("gh", "p_"),
    concat!("gh", "o_"),
    concat!("xox", "b-"),
    concat!("xox", "p-"),
    concat!("sec", "ret"),
    concat!("tok", "en"),
    concat!("pass", "word"),
    concat!("pass", "wd"),
    concat!("cook", "ie"),
    concat!("author", "ization"),
    concat!("bear", "er "),
    concat!("api", "_key"),
    concat!("api", "key"),
    concat!("access", "_key"),
    concat!("access", "_token"),
    concat!("refresh", "_token"),
    concat!("private", "_key"),
];
