use std::path::Path;

use super::snapshot::DxForgePanelState;

const MAX_WORKSPACE_ROOTS: usize = 4;

pub(super) struct ForgeStateInputs<'a> {
    pub(super) workspace_roots: &'a [String],
    pub(super) history_root_exists: bool,
    pub(super) configured_root_count: usize,
    pub(super) remote_registry_count: usize,
    pub(super) machine_cache_count: usize,
    pub(super) package_status_count: usize,
    pub(super) remote_registry_label: &'static str,
    pub(super) machine_caches_label: &'static str,
    pub(super) package_status_label: &'static str,
    pub(super) receipt_count: usize,
    pub(super) summarized_receipt_count: usize,
    pub(super) visible_blocker_count: usize,
    pub(super) visible_remote_registry_warning_count: usize,
    pub(super) visible_machine_cache_warning_count: usize,
    pub(super) visible_package_status_warning_count: usize,
    pub(super) visible_restore_warning_count: usize,
}

pub(super) fn forge_state(input: ForgeStateInputs<'_>) -> (DxForgePanelState, String) {
    if input.workspace_roots.is_empty() {
        return (
            DxForgePanelState::NoWorkspace,
            "Open a workspace to inspect Forge history".to_string(),
        );
    }
    if !input.history_root_exists
        && input.package_status_count == 0
        && input.remote_registry_count == 0
        && input.machine_cache_count == 0
    {
        return (
            DxForgePanelState::Missing,
            "Missing Forge receipt, remote-registry, package-status, or machine-cache root"
                .to_string(),
        );
    }
    if input.receipt_count > 0 && input.summarized_receipt_count == 0 {
        return (
            DxForgePanelState::Attention,
            "Receipt summaries unavailable".to_string(),
        );
    }
    if input.visible_package_status_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!(
                "{} package status warning(s)",
                input.visible_package_status_warning_count
            ),
        );
    }
    if input.visible_remote_registry_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!(
                "{} remote warning(s)",
                input.visible_remote_registry_warning_count
            ),
        );
    }
    if input.visible_machine_cache_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!(
                "{} visible machine cache warning(s) need review",
                input.visible_machine_cache_warning_count
            ),
        );
    }
    if input.visible_blocker_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{} blocker(s)", input.visible_blocker_count),
        );
    }
    if input.visible_restore_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{} restore warning(s)", input.visible_restore_warning_count),
        );
    }
    if input.receipt_count == 0 {
        if input.configured_root_count < input.workspace_roots.len() {
            return (
                DxForgePanelState::Attention,
                configured_root_scope(input.workspace_roots, input.configured_root_count),
            );
        }

        if input.package_status_count > 0 {
            return (
                DxForgePanelState::Evidence,
                format!(
                    "{} {} available",
                    input.package_status_count,
                    source_label(input.package_status_label, input.package_status_count),
                ),
            );
        }

        if input.remote_registry_count > 0 {
            return (
                DxForgePanelState::Evidence,
                format!(
                    "{} {} available",
                    input.remote_registry_count,
                    source_label(input.remote_registry_label, input.remote_registry_count),
                ),
            );
        }

        if input.machine_cache_count > 0 {
            return (
                DxForgePanelState::Evidence,
                format!(
                    "{} {} available",
                    input.machine_cache_count,
                    source_label(input.machine_caches_label, input.machine_cache_count),
                ),
            );
        }

        return (
            DxForgePanelState::Empty,
            "History configured; no receipts yet".to_string(),
        );
    }

    (
        DxForgePanelState::Ready,
        format!("{} receipt(s) available", input.receipt_count),
    )
}

pub(super) fn workspace_scope(workspace_roots: &[String]) -> String {
    match workspace_roots.len() {
        0 => "No roots".to_string(),
        1 => "1 root".to_string(),
        count if count <= MAX_WORKSPACE_ROOTS => format!("{count} roots"),
        count => format!("{MAX_WORKSPACE_ROOTS} of {count} roots"),
    }
}

pub(super) fn configured_root_scope(
    workspace_roots: &[String],
    configured_root_count: usize,
) -> String {
    if workspace_roots.is_empty() {
        return "No workspace roots".to_string();
    }

    let scanned_roots = workspace_roots.len().min(MAX_WORKSPACE_ROOTS);
    format!("{configured_root_count} of {scanned_roots} roots")
}

pub(super) fn forge_history_root_path(
    workspace_roots: &[String],
    history_root_exists: bool,
) -> Option<String> {
    if !history_root_exists || workspace_roots.len() != 1 {
        return None;
    }

    Some(
        Path::new(&workspace_roots[0])
            .join("tools")
            .join("dx-forge")
            .display()
            .to_string(),
    )
}

fn source_label(label: &'static str, count: usize) -> String {
    match label {
        "Machine Caches" if count == 1 => "machine cache".to_string(),
        "Machine Caches" => "machine caches".to_string(),
        "Package Status" if count == 1 => "package status".to_string(),
        "Package Status" => "package statuses".to_string(),
        "Remote Registry" if count == 1 => "remote registry".to_string(),
        "Remote Registry" => "remote registries".to_string(),
        _ => label.to_ascii_lowercase(),
    }
}
