use ui::Color;

use super::{
    super::snapshot::{DxForgePanelSnapshot, DxForgeRemoteProvider},
    catalog::{ForgeProvider, ProviderGroup},
};

pub(super) struct RemoteTargetState {
    pub(super) label: &'static str,
    pub(super) detail: String,
    pub(super) color: Color,
}

pub(super) fn remote_target_state(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
) -> RemoteTargetState {
    if snapshot.workspace_roots.is_empty() {
        return target_state(
            "No workspace",
            "Open a workspace to inspect remotes",
            Color::Muted,
        );
    }

    match group {
        ProviderGroup::Code => group_target_state(group, snapshot, code_target_state(snapshot)),
        ProviderGroup::Storage => {
            group_target_state(group, snapshot, storage_target_state(snapshot))
        }
        ProviderGroup::Media => group_target_state(group, snapshot, media_target_state(snapshot)),
    }
}

pub(super) fn provider_target_state(
    provider: &ForgeProvider,
    snapshot: &DxForgePanelSnapshot,
) -> RemoteTargetState {
    if snapshot.workspace_roots.is_empty() {
        return target_state(
            "No workspace",
            "Open a workspace to inspect remotes",
            Color::Muted,
        );
    }

    let Some(remote) = snapshot.remote_provider_for(provider.id) else {
        return target_state(
            "Not configured",
            format!("No {} remote found", provider.label),
            Color::Muted,
        );
    };

    provider_state_from_remote(remote)
}

fn group_target_state(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    fallback: RemoteTargetState,
) -> RemoteTargetState {
    let registry_count = snapshot.remote_registry_count_for_group(group.key());
    if registry_count == 0 {
        return fallback;
    }

    let configured_count = snapshot.configured_provider_count_for_group(group.key());
    if configured_count == 0 {
        return target_state(
            "Review",
            format!(
                "{} registered {}; none enabled",
                registry_count,
                plural(registry_count, "remote", "remotes"),
            ),
            Color::Warning,
        );
    }

    target_state(
        "Configured",
        format!(
            "{}/{} registered {} enabled; health unchecked",
            configured_count,
            registry_count,
            plural(registry_count, "remote", "remotes"),
        ),
        Color::Muted,
    )
}

fn provider_state_from_remote(remote: &DxForgeRemoteProvider) -> RemoteTargetState {
    if !remote.enabled {
        return target_state(
            "Disabled",
            format!(
                "{} '{}' registered but disabled",
                remote.label, remote.remote_name
            ),
            Color::Warning,
        );
    }

    let primary = if remote.primary { " primary" } else { "" };
    target_state(
        "Configured",
        format!(
            "{} '{}'{} configured; health unchecked",
            remote.label, remote.remote_name, primary
        ),
        Color::Muted,
    )
}

fn code_target_state(snapshot: &DxForgePanelSnapshot) -> RemoteTargetState {
    if !snapshot.history_root_exists {
        return target_state("Waiting", "Receipt history is unavailable", Color::Muted);
    }

    if snapshot.receipt_count > 0 && snapshot.summarized_receipt_count == 0 {
        return target_state("Review", "Receipt summaries unavailable", Color::Warning);
    }

    if snapshot.visible_blocker_count > 0 {
        return target_state(
            "Review",
            format!(
                "{} {}",
                snapshot.visible_blocker_count,
                plural(snapshot.visible_blocker_count, "blocker", "blockers"),
            ),
            Color::Warning,
        );
    }

    if snapshot.receipt_count > 0 {
        return target_state(
            "Ready",
            format!(
                "{} {} available",
                snapshot.receipt_count,
                plural(snapshot.receipt_count, "receipt", "receipts"),
            ),
            Color::Success,
        );
    }

    target_state(
        "Waiting",
        "History configured; no receipts yet",
        Color::Muted,
    )
}

fn storage_target_state(snapshot: &DxForgePanelSnapshot) -> RemoteTargetState {
    let restore_preview_count = snapshot.restore_previews.len();

    if !snapshot.history_root_exists {
        return target_state("Waiting", "Waiting for restore previews", Color::Muted);
    }

    if snapshot.visible_restore_warning_count > 0 {
        return target_state(
            "Review",
            format!(
                "{} restore {}",
                snapshot.visible_restore_warning_count,
                plural(
                    snapshot.visible_restore_warning_count,
                    "warning",
                    "warnings"
                ),
            ),
            Color::Warning,
        );
    }

    if restore_preview_count != 0 {
        return target_state(
            "Ready",
            format!(
                "{} restore {} available",
                restore_preview_count,
                plural(restore_preview_count, "preview", "previews"),
            ),
            Color::Success,
        );
    }

    target_state("Waiting", "No restore previews found", Color::Muted)
}

fn media_target_state(snapshot: &DxForgePanelSnapshot) -> RemoteTargetState {
    let media_output_count = snapshot.media_outputs.len();
    let warning_count: usize = snapshot
        .media_outputs
        .iter()
        .map(|output| output.warnings.len())
        .sum();

    if warning_count > 0 {
        return target_state(
            "Review",
            format!(
                "{} media {}",
                warning_count,
                plural(warning_count, "warning", "warnings"),
            ),
            Color::Warning,
        );
    }

    if media_output_count != 0 {
        return target_state(
            "Ready",
            format!(
                "{} media {} available",
                media_output_count,
                plural(media_output_count, "output", "outputs"),
            ),
            Color::Success,
        );
    }

    target_state("Waiting", "No media outputs found", Color::Muted)
}

fn target_state(label: &'static str, detail: impl Into<String>, color: Color) -> RemoteTargetState {
    RemoteTargetState {
        label,
        detail: detail.into(),
        color,
    }
}

fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 { singular } else { plural }
}
