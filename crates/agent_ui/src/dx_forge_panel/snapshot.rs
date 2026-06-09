use crate::dx_forge_panel::machine_cache::machine_cache_rows;
use crate::dx_forge_panel::package_status::package_status_rows;
use crate::dx_forge_panel::remote_registry::remote_registry_snapshot;
use crate::dx_forge_panel::roots::configured_forge_root_count;
use crate::dx_forge_panel::snapshot_state::{
    ForgeStateInputs, forge_history_root_path, forge_state, workspace_scope,
};
use crate::dx_receipt_history::{DxToolHistoryBucket, tool_history_snapshot};
use crate::dx_source_sets::{DxSourceItem, source_set_snapshot};

const FORGE_HISTORY_LABEL: &str = "Forge History";
pub(super) const PACKAGE_STATUS_LABEL: &str = "Package Status";
pub(super) const MACHINE_CACHES_LABEL: &str = "Machine Caches";
pub(super) const REMOTE_REGISTRY_LABEL: &str = "Remote Registry";
const RESTORE_PREVIEWS_LABEL: &str = "Restore Previews";
const MEDIA_OUTPUTS_LABEL: &str = "Media Outputs";
const MAX_PANEL_ROWS: usize = 4;

#[derive(Clone)]
pub(super) struct DxForgePanelSnapshot {
    pub(super) workspace_roots: Vec<String>,
    pub(super) workspace_scope: String,
    pub(super) state: DxForgePanelState,
    pub(super) state_detail: String,
    pub(super) history_root_path: Option<String>,
    pub(super) history_root_exists: bool,
    pub(super) receipt_count: usize,
    pub(super) summarized_receipt_count: usize,
    pub(super) visible_blocker_count: usize,
    pub(super) visible_restore_warning_count: usize,
    pub(super) machine_caches: Vec<DxForgeSourceRow>,
    pub(super) package_statuses: Vec<DxForgeSourceRow>,
    pub(super) remote_registries: Vec<DxForgeSourceRow>,
    pub(super) remote_providers: Vec<DxForgeRemoteProvider>,
    pub(super) latest_receipts: Vec<DxForgeReceiptRow>,
    pub(super) restore_previews: Vec<DxForgeSourceRow>,
    pub(super) media_outputs: Vec<DxForgeSourceRow>,
}

impl DxForgePanelSnapshot {
    pub(super) fn remote_provider_for(&self, provider_id: &str) -> Option<&DxForgeRemoteProvider> {
        let mut best = None;
        for provider in self
            .remote_providers
            .iter()
            .filter(|provider| provider.provider_id == provider_id)
        {
            if best.is_none_or(|current| {
                remote_provider_rank(provider) > remote_provider_rank(current)
            }) {
                best = Some(provider);
            }
        }
        best
    }

    pub(super) fn remote_registry_count_for_group(&self, group_key: &str) -> usize {
        self.remote_providers
            .iter()
            .filter(|provider| provider.group_key == group_key)
            .count()
    }

    pub(super) fn configured_provider_count_for_group(&self, group_key: &str) -> usize {
        self.remote_providers
            .iter()
            .filter(|provider| provider.group_key == group_key && provider.enabled)
            .count()
    }
}

fn remote_provider_rank(provider: &DxForgeRemoteProvider) -> u8 {
    match (provider.primary, provider.enabled) {
        (true, true) => 3,
        (false, true) => 2,
        (true, false) => 1,
        (false, false) => 0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DxForgePanelState {
    NoWorkspace,
    Ready,
    Evidence,
    Attention,
    Empty,
    Missing,
}

#[derive(Clone)]
pub(super) struct DxForgeReceiptRow {
    pub(super) source_path: String,
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
    pub(super) open_path: String,
    pub(super) receipts: Vec<DxForgeReceiptDrilldown>,
    pub(super) warnings: Vec<String>,
}

#[derive(Clone)]
pub(super) struct DxForgeRemoteProvider {
    pub(super) provider_id: String,
    pub(super) group_key: String,
    pub(super) label: String,
    pub(super) remote_name: String,
    pub(super) registry_path: String,
    pub(super) registry_open_path: String,
    pub(super) detail: String,
    pub(super) enabled: bool,
    pub(super) primary: bool,
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
    let remote_registry = remote_registry_snapshot(workspace_roots);
    let remote_registries = remote_registry.rows;
    let remote_providers = remote_registry.providers;
    let visible_remote_registry_warning_count = remote_registry.warning_count;
    let package_statuses = package_status_rows(workspace_roots);
    let visible_package_status_warning_count = package_statuses
        .iter()
        .map(|status| status.warnings.len())
        .sum();
    let machine_caches = machine_cache_rows(workspace_roots);
    let visible_machine_cache_warning_count = machine_caches
        .iter()
        .map(|cache| cache.warnings.len())
        .sum();
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
    let history_root_path = forge_history_root_path(workspace_roots, history_root_exists);
    let (state, state_detail) = forge_state(ForgeStateInputs {
        workspace_roots,
        history_root_exists,
        configured_root_count,
        remote_registry_count: remote_registries.len(),
        machine_cache_count: machine_caches.len(),
        package_status_count: package_statuses.len(),
        remote_registry_label: REMOTE_REGISTRY_LABEL,
        machine_caches_label: MACHINE_CACHES_LABEL,
        package_status_label: PACKAGE_STATUS_LABEL,
        receipt_count,
        summarized_receipt_count,
        visible_blocker_count,
        visible_remote_registry_warning_count,
        visible_machine_cache_warning_count,
        visible_package_status_warning_count,
        visible_restore_warning_count,
    });

    DxForgePanelSnapshot {
        workspace_roots: workspace_roots.to_vec(),
        workspace_scope: workspace_scope(workspace_roots),
        state,
        state_detail,
        history_root_path,
        history_root_exists,
        receipt_count,
        summarized_receipt_count,
        visible_blocker_count,
        visible_restore_warning_count,
        machine_caches,
        package_statuses,
        remote_registries,
        remote_providers,
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
            source_path: summary.source_path.clone(),
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
        open_path: source.open_path.clone(),
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
