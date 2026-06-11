use super::{
    catalog::{ForgeProvider, ProviderGroup},
    state::RemoteTargetState,
};
use crate::dx_forge_panel::snapshot::DxForgePanelSnapshot;

pub(super) fn provider_tooltip_meta(
    provider: &ForgeProvider,
    snapshot: &DxForgePanelSnapshot,
    state: &RemoteTargetState,
    target_path: Option<&str>,
    enabled: bool,
) -> String {
    let mut lines = vec![format!("State: {}", state.label), state.detail.clone()];

    if let Some(remote) = snapshot.remote_provider_for(provider.id) {
        lines.push(format!("Configured remote: {}", remote.remote_name));
        lines.push(format!("Registry: {}", remote.registry_path));
        lines.push(remote.detail.clone());
    } else if let Some(path) = target_path {
        lines.push(format!("Registry: {path}"));
    } else {
        lines.push("No remote found".to_string());
    }

    if !enabled {
        if target_path.is_some() {
            lines.push("Registry path is unavailable".to_string());
        } else {
            lines.push("Open a workspace with remotes.json".to_string());
        }
    }

    lines.join("\n")
}

pub(super) fn remote_target_tooltip(
    group: ProviderGroup,
    state: &RemoteTargetState,
    target_path: Option<&str>,
    enabled: bool,
) -> String {
    let mut lines = vec![
        state.label.to_string(),
        state.detail.clone(),
        format!("Providers: {}", group.provider_labels()),
    ];

    if let Some(path) = target_path {
        lines.push(format!("Path: {path}"));
    }

    if !enabled {
        if target_path.is_some() {
            lines.push("Path unavailable".to_string());
        } else {
            lines.push("No path found".to_string());
        }
    }

    lines.join("\n")
}
