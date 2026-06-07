use std::collections::HashSet;

use serde_json::Value;

use super::super::{redact_action_scalar, safe_string_field, string_array_field};

const MAX_RUNTIME_DISPLAY_CHARS: usize = 180;

pub(super) fn display_string_field(value: &Value, path: &[&str]) -> Option<String> {
    safe_string_field(value, path).and_then(display_string)
}

pub(super) fn display_string_array_field(
    value: &Value,
    path: &[&str],
    limit: usize,
) -> Vec<String> {
    let values = string_array_field(value, path)
        .into_iter()
        .map(|value| redact_action_scalar(&value))
        .filter_map(display_string)
        .collect::<Vec<_>>();

    dedupe_display_strings(values, limit)
}

pub(super) fn display_string(value: String) -> Option<String> {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact = compact
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    if compact.is_empty() {
        return None;
    }
    if compact.chars().count() <= MAX_RUNTIME_DISPLAY_CHARS {
        return Some(compact);
    }

    let mut display = compact
        .chars()
        .take(MAX_RUNTIME_DISPLAY_CHARS.saturating_sub(3))
        .collect::<String>();
    display.push_str("...");
    Some(display)
}

pub(super) fn dedupe_display_strings(values: Vec<String>, limit: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .take(limit)
        .collect()
}
