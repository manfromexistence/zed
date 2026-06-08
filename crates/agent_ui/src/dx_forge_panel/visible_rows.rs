use super::{
    panel::DxForgePanelTab,
    snapshot::{DxForgePanelSnapshot, DxForgeReceiptRow, DxForgeSourceRow},
};

const REMOTE_TARGET_GROUP_KEYS: [&str; 3] = ["code", "storage", "media"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DxForgeVisibleRow {
    pub(super) item_key: DxForgeRowKey,
    pub(super) kind: DxForgeVisibleRowKind,
}

impl DxForgeVisibleRow {
    fn new(item_key: DxForgeRowKey, kind: DxForgeVisibleRowKind) -> Self {
        Self { item_key, kind }
    }

    pub(super) fn item_key(&self) -> &DxForgeRowKey {
        &self.item_key
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DxForgeVisibleRowKind {
    Receipt,
    PackageStatus,
    MachineCache,
    MediaOutput,
    RestorePreview,
    RemoteTarget,
    RemoteRegistry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum DxForgeRowKey {
    Receipt(String),
    Source(String),
    RemoteGroup(String),
}

pub(super) fn receipt_item_key(receipt: &DxForgeReceiptRow) -> DxForgeRowKey {
    DxForgeRowKey::Receipt(receipt.source_path.clone())
}

pub(super) fn source_item_key(source: &DxForgeSourceRow) -> DxForgeRowKey {
    DxForgeRowKey::Source(source.open_path.clone())
}

pub(super) fn remote_target_item_key(group_key: &str) -> DxForgeRowKey {
    DxForgeRowKey::RemoteGroup(group_key.to_string())
}

pub(super) fn visible_row_count_for_tab(
    snapshot: &DxForgePanelSnapshot,
    active_tab: DxForgePanelTab,
) -> usize {
    match active_tab {
        DxForgePanelTab::Repository => snapshot.latest_receipts.len(),
        DxForgePanelTab::Packages => {
            snapshot.package_statuses.len() + snapshot.machine_caches.len()
        }
        DxForgePanelTab::Media => snapshot.media_outputs.len() + snapshot.restore_previews.len(),
        DxForgePanelTab::Remotes => {
            REMOTE_TARGET_GROUP_KEYS.len() + snapshot.remote_registries.len()
        }
    }
}

pub(super) fn visible_rows_for_tab(
    snapshot: &DxForgePanelSnapshot,
    active_tab: DxForgePanelTab,
) -> Vec<DxForgeVisibleRow> {
    let mut visible_rows = Vec::new();

    match active_tab {
        DxForgePanelTab::Repository => {
            for receipt in &snapshot.latest_receipts {
                visible_rows.push(DxForgeVisibleRow::new(
                    receipt_item_key(receipt),
                    DxForgeVisibleRowKind::Receipt,
                ));
            }
        }
        DxForgePanelTab::Packages => {
            push_source_rows(
                &mut visible_rows,
                &snapshot.package_statuses,
                DxForgeVisibleRowKind::PackageStatus,
            );
            push_source_rows(
                &mut visible_rows,
                &snapshot.machine_caches,
                DxForgeVisibleRowKind::MachineCache,
            );
        }
        DxForgePanelTab::Media => {
            push_source_rows(
                &mut visible_rows,
                &snapshot.media_outputs,
                DxForgeVisibleRowKind::MediaOutput,
            );
            push_source_rows(
                &mut visible_rows,
                &snapshot.restore_previews,
                DxForgeVisibleRowKind::RestorePreview,
            );
        }
        DxForgePanelTab::Remotes => {
            for group_key in REMOTE_TARGET_GROUP_KEYS {
                visible_rows.push(DxForgeVisibleRow::new(
                    remote_target_item_key(group_key),
                    DxForgeVisibleRowKind::RemoteTarget,
                ));
            }
            push_source_rows(
                &mut visible_rows,
                &snapshot.remote_registries,
                DxForgeVisibleRowKind::RemoteRegistry,
            );
        }
    }

    visible_rows
}

fn push_source_rows(
    visible_rows: &mut Vec<DxForgeVisibleRow>,
    rows: &[DxForgeSourceRow],
    kind: DxForgeVisibleRowKind,
) {
    for source in rows {
        visible_rows.push(DxForgeVisibleRow::new(source_item_key(source), kind));
    }
}
