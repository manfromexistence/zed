use crate::dx_receipt_history::{DxToolHistoryBucket, tool_history_snapshot};
use crate::dx_source_sets::{DxSourceItem, source_set_snapshot};

const FORGE_HISTORY_LABEL: &str = "Forge History";
const RESTORE_PREVIEWS_LABEL: &str = "Restore Previews";
const MEDIA_OUTPUTS_LABEL: &str = "Media Outputs";
const MAX_PANEL_ROWS: usize = 4;

#[derive(Clone)]
pub(super) struct DxForgePanelSnapshot {
    pub(super) workspace_roots: Vec<String>,
    pub(super) state: DxForgePanelState,
    pub(super) state_detail: String,
    pub(super) history_root_label: String,
    pub(super) history_root_exists: bool,
    pub(super) receipt_count: usize,
    pub(super) blocker_count: usize,
    pub(super) restore_warning_count: usize,
    pub(super) latest_receipts: Vec<DxForgeReceiptRow>,
    pub(super) restore_previews: Vec<DxForgeSourceRow>,
    pub(super) media_outputs: Vec<DxForgeSourceRow>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DxForgePanelState {
    Ready,
    Attention,
    Empty,
    Missing,
}

#[derive(Clone)]
pub(super) struct DxForgeReceiptRow {
    pub(super) label: String,
    pub(super) kind: String,
    pub(super) headline: String,
    pub(super) detail: String,
    pub(super) target_path: Option<String>,
    pub(super) restore_destination_root: Option<String>,
    pub(super) blocker_count: usize,
}

#[derive(Clone)]
pub(super) struct DxForgeSourceRow {
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) path: String,
    pub(super) receipts: Vec<DxForgeReceiptDrilldown>,
    pub(super) warnings: Vec<String>,
}

#[derive(Clone)]
pub(super) struct DxForgeReceiptDrilldown {
    pub(super) label: String,
    pub(super) detail: String,
}

pub(super) fn forge_panel_snapshot(workspace_roots: &[String]) -> DxForgePanelSnapshot {
    let tool_history = tool_history_snapshot(workspace_roots);
    let source_sets = source_set_snapshot(workspace_roots);
    let history = tool_history
        .buckets
        .iter()
        .find(|bucket| bucket.label == FORGE_HISTORY_LABEL);
    let latest_receipts = history.map(receipt_rows).unwrap_or_default();
    let receipt_count = history.map(|bucket| bucket.count).unwrap_or_default();
    let blocker_count = latest_receipts
        .iter()
        .map(|receipt| receipt.blocker_count)
        .sum();
    let restore_previews = source_rows_for_set(&source_sets.sets, RESTORE_PREVIEWS_LABEL);
    let media_outputs = source_rows_for_set(&source_sets.sets, MEDIA_OUTPUTS_LABEL);
    let restore_warning_count = restore_previews
        .iter()
        .map(|preview| preview.warnings.len())
        .sum();
    let history_root_exists = history.map(|bucket| bucket.root_exists).unwrap_or(false);
    let history_root_label = history
        .map(|bucket| bucket.root_label.clone())
        .unwrap_or_else(|| "No workspace".to_string());
    let (state, state_detail) = forge_state(
        workspace_roots,
        history_root_exists,
        receipt_count,
        blocker_count,
        restore_warning_count,
    );

    DxForgePanelSnapshot {
        workspace_roots: workspace_roots.to_vec(),
        state,
        state_detail,
        history_root_label,
        history_root_exists,
        receipt_count,
        blocker_count,
        restore_warning_count,
        latest_receipts,
        restore_previews,
        media_outputs,
    }
}

fn receipt_rows(bucket: &DxToolHistoryBucket) -> Vec<DxForgeReceiptRow> {
    bucket
        .latest_summaries
        .iter()
        .take(MAX_PANEL_ROWS)
        .map(|summary| DxForgeReceiptRow {
            label: summary.label.clone(),
            kind: summary.kind.clone(),
            headline: summary.headline.clone(),
            detail: summary.detail.clone(),
            target_path: summary.target_path.clone(),
            restore_destination_root: summary.restore_destination_root.clone(),
            blocker_count: summary.blocker_count,
        })
        .collect()
}

fn source_rows_for_set(
    sets: &[crate::dx_source_sets::DxSourceSet],
    label: &str,
) -> Vec<DxForgeSourceRow> {
    sets.iter()
        .find(|set| set.label == label)
        .map(|set| {
            set.sources
                .iter()
                .take(MAX_PANEL_ROWS)
                .map(source_row)
                .collect()
        })
        .unwrap_or_default()
}

fn source_row(source: &DxSourceItem) -> DxForgeSourceRow {
    DxForgeSourceRow {
        label: source.label.clone(),
        detail: source.detail.clone(),
        path: source.path.clone(),
        receipts: source
            .receipt_drilldowns
            .iter()
            .map(|receipt| DxForgeReceiptDrilldown {
                label: receipt.label.clone(),
                detail: receipt.detail.clone(),
            })
            .collect(),
        warnings: source.warnings.clone(),
    }
}

fn forge_state(
    workspace_roots: &[String],
    history_root_exists: bool,
    receipt_count: usize,
    blocker_count: usize,
    restore_warning_count: usize,
) -> (DxForgePanelState, String) {
    if workspace_roots.is_empty() {
        return (
            DxForgePanelState::Missing,
            "Open a workspace to read Forge receipts".to_string(),
        );
    }
    if !history_root_exists {
        return (
            DxForgePanelState::Missing,
            "Missing tools/dx-forge receipt root".to_string(),
        );
    }
    if blocker_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{blocker_count} Forge blocker(s) need review"),
        );
    }
    if restore_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{restore_warning_count} restore warning(s) need review"),
        );
    }
    if receipt_count == 0 {
        return (
            DxForgePanelState::Empty,
            "Forge is configured, but no receipts were found".to_string(),
        );
    }

    (
        DxForgePanelState::Ready,
        format!("{receipt_count} Forge receipt(s) available"),
    )
}
