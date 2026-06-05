use crate::dx_receipt_history::{DxToolHistoryBucket, tool_history_snapshot};
use crate::dx_source_sets::{DxSourceItem, source_set_snapshot};
use std::path::Path;

const FORGE_HISTORY_LABEL: &str = "Forge History";
const RESTORE_PREVIEWS_LABEL: &str = "Restore Previews";
const MEDIA_OUTPUTS_LABEL: &str = "Media Outputs";
const MAX_WORKSPACE_ROOTS: usize = 4;
const MAX_PANEL_ROWS: usize = 4;

#[derive(Clone)]
pub(super) struct DxForgePanelSnapshot {
    pub(super) workspace_roots: Vec<String>,
    pub(super) workspace_scope: String,
    pub(super) configured_root_scope: String,
    pub(super) state: DxForgePanelState,
    pub(super) state_detail: String,
    pub(super) history_root_label: String,
    pub(super) history_root_exists: bool,
    pub(super) receipt_count: usize,
    pub(super) summarized_receipt_count: usize,
    pub(super) visible_blocker_count: usize,
    pub(super) visible_restore_warning_count: usize,
    pub(super) latest_receipts: Vec<DxForgeReceiptRow>,
    pub(super) restore_previews: Vec<DxForgeSourceRow>,
    pub(super) media_outputs: Vec<DxForgeSourceRow>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DxForgePanelState {
    NoWorkspace,
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
    let configured_root_count = configured_forge_root_count(workspace_roots);
    let history = tool_history
        .buckets
        .iter()
        .find(|bucket| bucket.label == FORGE_HISTORY_LABEL);
    let latest_receipts = history.map(receipt_rows).unwrap_or_default();
    let summarized_receipt_count = latest_receipts.len();
    let receipt_count = history.map(|bucket| bucket.count).unwrap_or_default();
    let visible_blocker_count = latest_receipts
        .iter()
        .map(|receipt| receipt.blocker_count)
        .sum();
    let restore_previews = source_rows_for_set(&source_sets.sets, RESTORE_PREVIEWS_LABEL);
    let media_outputs = source_rows_for_set(&source_sets.sets, MEDIA_OUTPUTS_LABEL);
    let visible_restore_warning_count = restore_previews
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
        configured_root_count,
        receipt_count,
        summarized_receipt_count,
        visible_blocker_count,
        visible_restore_warning_count,
    );

    DxForgePanelSnapshot {
        workspace_roots: workspace_roots.to_vec(),
        workspace_scope: workspace_scope(workspace_roots),
        configured_root_scope: configured_root_scope(workspace_roots, configured_root_count),
        state,
        state_detail,
        history_root_label,
        history_root_exists,
        receipt_count,
        summarized_receipt_count,
        visible_blocker_count,
        visible_restore_warning_count,
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
    configured_root_count: usize,
    receipt_count: usize,
    summarized_receipt_count: usize,
    visible_blocker_count: usize,
    visible_restore_warning_count: usize,
) -> (DxForgePanelState, String) {
    if workspace_roots.is_empty() {
        return (
            DxForgePanelState::NoWorkspace,
            "Open a workspace to read Forge receipts".to_string(),
        );
    }
    if !history_root_exists {
        return (
            DxForgePanelState::Missing,
            "Missing tools/dx-forge receipt root".to_string(),
        );
    }
    if receipt_count > 0 && summarized_receipt_count == 0 {
        return (
            DxForgePanelState::Attention,
            "Forge receipts exist, but no known receipt summaries were readable".to_string(),
        );
    }
    if visible_blocker_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{visible_blocker_count} visible Forge blocker(s) need review"),
        );
    }
    if visible_restore_warning_count > 0 {
        return (
            DxForgePanelState::Attention,
            format!("{visible_restore_warning_count} visible restore warning(s) need review"),
        );
    }
    if receipt_count == 0 {
        if configured_root_count < workspace_roots.len() {
            return (
                DxForgePanelState::Attention,
                configured_root_scope(workspace_roots, configured_root_count),
            );
        }

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

fn workspace_scope(workspace_roots: &[String]) -> String {
    match workspace_roots.len() {
        0 => "No roots".to_string(),
        1 => "1 root scanned".to_string(),
        count if count <= MAX_WORKSPACE_ROOTS => format!("{count} roots scanned"),
        count => format!("first {MAX_WORKSPACE_ROOTS} of {count} roots scanned"),
    }
}

fn configured_root_scope(workspace_roots: &[String], configured_root_count: usize) -> String {
    if workspace_roots.is_empty() {
        return "No workspace roots".to_string();
    }

    let scanned_roots = workspace_roots.len().min(MAX_WORKSPACE_ROOTS);
    format!("{configured_root_count} of {scanned_roots} scanned roots configured")
}

fn configured_forge_root_count(workspace_roots: &[String]) -> usize {
    workspace_roots
        .iter()
        .take(MAX_WORKSPACE_ROOTS)
        .filter(|root| Path::new(root).join("tools").join("dx-forge").is_dir())
        .count()
}
