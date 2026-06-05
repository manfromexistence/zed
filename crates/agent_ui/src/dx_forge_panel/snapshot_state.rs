use std::path::Path;

use super::snapshot::DxForgePanelState;

const MAX_WORKSPACE_ROOTS: usize = 4;

pub(super) struct ForgeStateInputs<'a> {
    pub(super) workspace_roots: &'a [String],
    pub(super) history_root_exists: bool,
    pub(super) configured_root_count: usize,
    pub(super) machine_cache_count: usize,
    pub(super) machine_caches_label: &'static str,
    pub(super) package_status_label: &'static str,
    pub(super) package_status_count: usize,
    pub(super) receipt_count: usize,
    pub(super) summarized_receipt_count: usize,
    pub(super) visible_blocker_count: usize,
    pub(super) visible_machine_cache_warning_count: usize,
    pub(super) visible_package_status_warning_count: usize,
    pub(super) visible_restore_warning_count: usize,
}

pub(super) fn forge_state(input: ForgeStateInputs<'_>) -> (DxForgePanelState, String) {
    if input.workspace_roots.is_empty() {
        return (
            DxForgePanelState::NoWorkspace,
            "Open a workspace to read Forge receipts".to_string(),
        );
    }
    if !input.history_root_exists
        && input.package_status_count == 0
        && input.machine_cache_count == 0
    {
        return (
            DxForgePanelState::Missing,
            "Missing Forge receipt, package-status, or machine-cache root".to_string(),
        );
    }
    if input.receipt_count > 0 && input.summarized_receipt_count == 0 {
        return (
            DxForgePanelState::Attention,
            "Forge receipts exist, but no known receipt summaries were readable".to_string(),
        );
    }
    if input.visible_package_status_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!(
                "{} visible package status warning(s) need review",
                input.visible_package_status_warning_count
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
            format!(
                "{} visible Forge blocker(s) need review",
                input.visible_blocker_count
            ),
        );
    }
    if input.visible_restore_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!(
                "{} visible restore warning(s) need review",
                input.visible_restore_warning_count
            ),
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
                DxForgePanelState::Ready,
                format!(
                    "{} {} file(s) available",
                    input.package_status_count, input.package_status_label
                ),
            );
        }

        if input.machine_cache_count > 0 {
            return (
                DxForgePanelState::Ready,
                format!(
                    "{} {} row(s) available",
                    input.machine_cache_count, input.machine_caches_label
                ),
            );
        }

        return (
            DxForgePanelState::Empty,
            "Forge is configured, but no receipts were found".to_string(),
        );
    }

    (
        DxForgePanelState::Ready,
        format!("{} Forge receipt(s) available", input.receipt_count),
    )
}

pub(super) fn workspace_scope(workspace_roots: &[String]) -> String {
    match workspace_roots.len() {
        0 => "No roots".to_string(),
        1 => "1 root scanned".to_string(),
        count if count <= MAX_WORKSPACE_ROOTS => format!("{count} roots scanned"),
        count => format!("first {MAX_WORKSPACE_ROOTS} of {count} roots scanned"),
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
    format!("{configured_root_count} of {scanned_roots} scanned roots configured")
}

pub(super) fn configured_forge_root_count(workspace_roots: &[String]) -> usize {
    workspace_roots
        .iter()
        .take(MAX_WORKSPACE_ROOTS)
        .filter(|root| {
            let root = Path::new(root);
            root.join("tools").join("dx-forge").is_dir() || root.join(".dx").join("forge").is_dir()
        })
        .count()
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
