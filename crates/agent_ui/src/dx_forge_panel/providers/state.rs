use ui::{Color, IconName};

use super::{super::snapshot::DxForgePanelSnapshot, catalog::ProviderGroup};

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
        ProviderGroup::Code => code_target_state(snapshot),
        ProviderGroup::Storage => storage_target_state(snapshot),
        ProviderGroup::Media => media_target_state(snapshot),
    }
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
