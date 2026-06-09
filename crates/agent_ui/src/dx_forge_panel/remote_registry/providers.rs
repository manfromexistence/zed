use serde_json::Value;

use crate::dx_forge_panel::snapshot::DxForgeRemoteProvider;

pub(super) fn remote_providers(
    remotes: &[&Value],
    primary: Option<&str>,
    registry_path: &str,
    registry_open_path: &str,
    limit: usize,
) -> Vec<DxForgeRemoteProvider> {
    remotes
        .iter()
        .take(limit)
        .filter_map(|remote| {
            let kind = string_field(remote, &["kind"])?;
            let (provider_id, group_key, label) = catalog_provider_info(&kind)?;
            let remote_name = string_field(remote, &["name"]).unwrap_or_else(|| label.to_string());
            let mapping_count = array_len_field(remote, &["branch_mappings"]).unwrap_or(0);
            let has_auth_backend = string_field(remote, &["auth_backend"]).is_some();
            let enabled = bool_field(remote, &["enabled"]).unwrap_or(true);
            Some(DxForgeRemoteProvider {
                provider_id: provider_id.to_string(),
                group_key: group_key.to_string(),
                label: label.to_string(),
                remote_name: remote_name.clone(),
                registry_path: registry_path.to_string(),
                registry_open_path: registry_open_path.to_string(),
                detail: provider_detail(&kind, mapping_count, has_auth_backend),
                enabled,
                primary: primary == Some(remote_name.as_str()),
            })
        })
        .collect()
}

pub(super) fn catalog_provider_info(
    kind: &str,
) -> Option<(&'static str, &'static str, &'static str)> {
    match remote_kind_key(kind).as_str() {
        "github" => Some(("github", "code", "GitHub")),
        "gitlab" => Some(("gitlab", "code", "GitLab")),
        "bitbucket" => Some(("bitbucket", "code", "Bitbucket")),
        "gdrive" | "googledrive" => Some(("drive", "storage", "Google Drive")),
        "dropbox" => Some(("dropbox", "storage", "Dropbox")),
        "youtube" => Some(("youtube", "media", "YouTube")),
        "soundcloud" => Some(("soundcloud", "media", "SoundCloud")),
        "soundbox" => Some(("soundbox", "media", "SoundBox")),
        _ => None,
    }
}

pub(super) fn canonical_kind_label(kind: &str) -> &'static str {
    match remote_kind_key(kind).as_str() {
        "forge" | "forgetransport" | "quic" => "ForgeTransport",
        "github" => "GitHub",
        "gitlab" => "GitLab",
        "bitbucket" => "Bitbucket",
        "r2" => "R2",
        "gdrive" | "googledrive" => "GoogleDrive",
        "dropbox" => "Dropbox",
        "mega" => "Mega",
        "youtube" => "YouTube",
        "pinterest" => "Pinterest",
        "soundcloud" => "SoundCloud",
        "soundbox" => "SoundBox",
        "sketchfab" => "Sketchfab",
        _ => "Unknown",
    }
}

fn provider_detail(kind: &str, mapping_count: usize, has_auth_backend: bool) -> String {
    let auth = if has_auth_backend {
        "auth backend configured"
    } else {
        "auth backend not configured"
    };
    format!(
        "{} · {} mapping(s) · {} · health unchecked",
        canonical_kind_label(kind),
        mapping_count,
        auth,
    )
}

fn remote_kind_key(kind: &str) -> String {
    kind.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn string_field(value: &Value, path: &[&str]) -> Option<String> {
    field(value, path)
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
