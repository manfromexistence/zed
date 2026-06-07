use super::{DxSourceKind, DxSourceSetSnapshot};

#[derive(Clone, Default)]
pub(crate) struct DxSourceAttachmentSummary {
    pub workspace_roots: usize,
    pub managed_receipts: usize,
    pub produced_files: usize,
    pub restore_previews: usize,
    pub attachable_sources: usize,
}

impl DxSourceSetSnapshot {
    pub(crate) fn attachment_summary(&self) -> DxSourceAttachmentSummary {
        let mut summary = DxSourceAttachmentSummary::default();

        for source in self.sets.iter().flat_map(|set| set.sources.iter()) {
            match source.kind {
                DxSourceKind::WorkspaceRoot => summary.workspace_roots += 1,
                DxSourceKind::MetasearchSourcePack | DxSourceKind::ReducedContextReceipt => {
                    summary.managed_receipts += 1;
                    summary.attachable_sources += 1;
                }
                DxSourceKind::MediaOutput => {
                    summary.produced_files += 1;
                    summary.attachable_sources += 1;
                }
                DxSourceKind::ForgeRestorePreview => {
                    summary.restore_previews += 1;
                    summary.attachable_sources += 1;
                }
                DxSourceKind::DxToolchainConfig => {
                    summary.attachable_sources += 1;
                }
            }
        }

        summary
    }
}
