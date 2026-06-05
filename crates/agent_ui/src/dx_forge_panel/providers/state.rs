use ui::{Color, IconName};

use super::{
    super::snapshot::{DxForgePanelSnapshot, DxForgeRemoteProvider},
    catalog::{ForgeProvider, ProviderGroup},
};

pub(super) struct RemoteTargetState {
    pub(super) label: &'static str,
    pub(super) detail: String,
    pub(super) color: Color,
    pub(super) icon: IconName,
}

pub(super) fn remote_target_state(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
) -> RemoteTargetState {
    if snapshot.workspace_roots.is_empty() {
        return target_state(
            "offline",
            "Open a workspace to inspect Forge targets",
            Color::Muted,
            IconName::Info,
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
            "offline",
            "Open a workspace to inspect Forge targets",
            Color::Muted,
            IconName::Info,
        );
    }

    let Some(remote) = snapshot.remote_provider_for(provider.id) else {
        return target_state(
            "catalog",
            format!(
                "{} is available in the DX icon catalog; no local remote is registered",
                provider.label
            ),
            Color::Muted,
            IconName::Circle,
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
            "review",
            format!(
                "{} registered {}; none enabled",
                registry_count,
                plural(registry_count, "remote", "remotes"),
            ),
            Color::Warning,
            IconName::Warning,
        );
    }

    target_state(
        "configured",
        format!(
            "{} of {} registered {} enabled; live remote health unchecked",
            configured_count,
            registry_count,
            plural(registry_count, "remote", "remotes"),
        ),
        Color::Success,
        IconName::Check,
    )
}

fn provider_state_from_remote(remote: &DxForgeRemoteProvider) -> RemoteTargetState {
    if !remote.enabled {
        return target_state(
            "disabled",
            format!(
                "{} remote '{}' is registered but disabled",
                remote.label, remote.remote_name
            ),
            Color::Warning,
            IconName::Warning,
        );
    }

    target_state(
        if remote.primary {
            "primary"
        } else {
            "configured"
        },
        format!(
            "{} remote '{}' configured; live remote health unchecked",
            remote.label, remote.remote_name
        ),
        Color::Success,
        IconName::Check,
    )
}

fn code_target_state(snapshot: &DxForgePanelSnapshot) -> RemoteTargetState {
    if !snapshot.history_root_exists {
        return target_state(
            "waiting",
            "Missing tools/dx-forge receipt root",
            Color::Muted,
            IconName::Info,
        );
    }

    if snapshot.receipt_count > 0 && snapshot.summarized_receipt_count == 0 {
        return target_state(
            "review",
            "Forge receipts found; no known summaries readable",
            Color::Warning,
            IconName::Warning,
        );
    }

    if snapshot.visible_blocker_count > 0 {
        return target_state(
            "review",
            format!(
                "{} visible Forge {}",
                snapshot.visible_blocker_count,
                plural(snapshot.visible_blocker_count, "blocker", "blockers"),
            ),
            Color::Warning,
            IconName::Warning,
        );
    }

    if snapshot.receipt_count > 0 {
        return target_state(
            "ready",
            format!(
                "{} Forge {} available",
                snapshot.receipt_count,
                plural(snapshot.receipt_count, "receipt", "receipts"),
            ),
            Color::Success,
            IconName::Check,
        );
    }

    target_state(
        "waiting",
        "Forge root configured; no receipts yet",
        Color::Muted,
        IconName::Circle,
    )
}

fn storage_target_state(snapshot: &DxForgePanelSnapshot) -> RemoteTargetState {
    let restore_preview_count = snapshot.restore_previews.len();

    if !snapshot.history_root_exists {
        return target_state(
            "waiting",
            "Waiting for tools/dx-forge/restores",
            Color::Muted,
            IconName::Info,
        );
    }

    if snapshot.visible_restore_warning_count > 0 {
        return target_state(
            "review",
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
            IconName::Warning,
        );
    }

    if restore_preview_count != 0 {
        return target_state(
            "ready",
            format!(
                "{} restore {} available",
                restore_preview_count,
                plural(restore_preview_count, "preview", "previews"),
            ),
            Color::Success,
            IconName::Check,
        );
    }

    target_state(
        "waiting",
        "No restore previews found",
        Color::Muted,
        IconName::Circle,
    )
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
            "review",
            format!(
                "{} media {}",
                warning_count,
                plural(warning_count, "warning", "warnings"),
            ),
            Color::Warning,
            IconName::Warning,
        );
    }

    if media_output_count != 0 {
        return target_state(
            "ready",
            format!(
                "{} media {} available",
                media_output_count,
                plural(media_output_count, "output", "outputs"),
            ),
            Color::Success,
            IconName::Check,
        );
    }

    target_state(
        "waiting",
        "No media outputs found",
        Color::Muted,
        IconName::Circle,
    )
}

fn target_state(
    label: &'static str,
    detail: impl Into<String>,
    color: Color,
    icon: IconName,
) -> RemoteTargetState {
    RemoteTargetState {
        label,
        detail: detail.into(),
        color,
        icon,
    }
}

fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 { singular } else { plural }
}
