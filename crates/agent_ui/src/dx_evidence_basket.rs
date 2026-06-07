use crate::dx_receipt_history::{DxToolHistorySnapshot, tool_history_snapshot};
use crate::dx_source_sets::{DxSourceKind, DxSourceSetSnapshot, source_set_snapshot};
use crate::flow_speech_runtime::{FlowSpeechReadinessSnapshot, FlowSpeechRuntime};

const MAX_EVIDENCE_SOURCE_ROWS: usize = 4;

#[derive(Clone)]
pub(crate) struct DxEvidenceBasket {
    pub workspace_roots: usize,
    pub total_sources: usize,
    pub managed_receipts: usize,
    pub media_outputs: usize,
    pub forge_restore_previews: usize,
    pub attachable_sources: usize,
    pub receipt_buckets: usize,
    pub receipt_count: usize,
    pub source_drilldowns: usize,
    pub source_proofs: usize,
    pub source_warnings: usize,
    pub metasearch_sources: Vec<DxEvidenceBasketSource>,
    pub reduced_context_sources: Vec<DxEvidenceBasketSource>,
    pub media_output_sources: Vec<DxEvidenceBasketSource>,
    pub forge_restore_sources: Vec<DxEvidenceBasketSource>,
    pub flow: DxEvidenceBasketFlowReadiness,
}

#[derive(Clone)]
pub(crate) struct DxEvidenceBasketSource {
    pub label: String,
    pub detail: String,
    pub path: String,
    pub open_path: String,
    pub kind: DxEvidenceBasketSourceKind,
    pub receipt_drilldowns: usize,
    pub proofs: usize,
    pub warnings: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DxEvidenceBasketSourceKind {
    WorkspaceRoot,
    MetasearchSourcePack,
    ReducedContextReceipt,
    MediaOutput,
    ForgeRestorePreview,
    DxToolchainConfig,
}

#[derive(Clone)]
pub(crate) struct DxEvidenceBasketFlowReadiness {
    pub flow_root: String,
    pub dictate_binary: Option<String>,
    pub stt_model: String,
    pub stt_ready: bool,
    pub stt_detail: String,
    pub kokoro_ready: bool,
    pub kokoro_detail: String,
    pub input_device_detail: String,
}

pub(crate) fn dx_evidence_basket(workspace_roots: &[String]) -> DxEvidenceBasket {
    let source_sets = source_set_snapshot(workspace_roots);
    let tool_history = tool_history_snapshot(workspace_roots);
    let flow = FlowSpeechRuntime::detect().readiness_snapshot();
    let attachment_summary = source_sets.attachment_summary();
    let (source_drilldowns, source_proofs, source_warnings) = source_totals(&source_sets);

    DxEvidenceBasket {
        workspace_roots: attachment_summary.workspace_roots,
        total_sources: source_sets.total_sources,
        managed_receipts: attachment_summary.managed_receipts,
        media_outputs: attachment_summary.produced_files,
        forge_restore_previews: attachment_summary.restore_previews,
        attachable_sources: attachment_summary.attachable_sources,
        receipt_buckets: receipt_bucket_count(&tool_history),
        receipt_count: tool_history.buckets.iter().map(|bucket| bucket.count).sum(),
        source_drilldowns,
        source_proofs,
        source_warnings,
        metasearch_sources: source_rows_for_set(&source_sets, "Metasearch"),
        reduced_context_sources: source_rows_for_set(&source_sets, "Reduced Context"),
        media_output_sources: source_rows_for_set(&source_sets, "Media Outputs"),
        forge_restore_sources: source_rows_for_set(&source_sets, "Restore Previews"),
        flow: DxEvidenceBasketFlowReadiness::from(flow),
    }
}

impl DxEvidenceBasket {
    pub(crate) fn evidence_source_count(&self) -> usize {
        self.metasearch_sources.len()
            + self.reduced_context_sources.len()
            + self.media_output_sources.len()
            + self.forge_restore_sources.len()
    }

    pub(crate) fn source_signal_count(&self) -> usize {
        self.source_groups()
            .into_iter()
            .flatten()
            .map(DxEvidenceBasketSource::signal_count)
            .sum()
    }

    pub(crate) fn source_preview_labels(&self) -> Vec<String> {
        self.source_groups()
            .into_iter()
            .flatten()
            .take(MAX_EVIDENCE_SOURCE_ROWS)
            .map(DxEvidenceBasketSource::preview_label)
            .collect()
    }

    fn source_groups(&self) -> [&[DxEvidenceBasketSource]; 4] {
        [
            &self.metasearch_sources,
            &self.reduced_context_sources,
            &self.media_output_sources,
            &self.forge_restore_sources,
        ]
    }
}

impl DxEvidenceBasketSource {
    fn preview_label(&self) -> String {
        format!("{}: {} - {}", self.kind.label(), self.label, self.detail)
    }

    fn signal_count(&self) -> usize {
        self.receipt_drilldowns
            + self.proofs
            + self.warnings
            + usize::from(!self.path.trim().is_empty())
            + usize::from(!self.open_path.trim().is_empty())
    }
}

impl DxEvidenceBasketSourceKind {
    fn label(self) -> &'static str {
        match self {
            DxEvidenceBasketSourceKind::WorkspaceRoot => "Workspace",
            DxEvidenceBasketSourceKind::MetasearchSourcePack => "Metasearch",
            DxEvidenceBasketSourceKind::ReducedContextReceipt => "Reduced context",
            DxEvidenceBasketSourceKind::MediaOutput => "Media",
            DxEvidenceBasketSourceKind::ForgeRestorePreview => "Forge restore",
            DxEvidenceBasketSourceKind::DxToolchainConfig => "DX toolchain",
        }
    }
}

impl From<FlowSpeechReadinessSnapshot> for DxEvidenceBasketFlowReadiness {
    fn from(snapshot: FlowSpeechReadinessSnapshot) -> Self {
        Self {
            flow_root: snapshot.flow_root,
            dictate_binary: snapshot.dictate_binary,
            stt_model: snapshot.stt_model,
            stt_ready: snapshot.stt_ready,
            stt_detail: snapshot.stt_detail,
            kokoro_ready: snapshot.kokoro_ready,
            kokoro_detail: snapshot.kokoro_detail,
            input_device_detail: snapshot.input_device_detail,
        }
    }
}

fn source_rows_for_set(
    source_sets: &DxSourceSetSnapshot,
    label: &'static str,
) -> Vec<DxEvidenceBasketSource> {
    source_sets
        .sets
        .iter()
        .find(|set| set.label == label)
        .map(|set| {
            set.sources
                .iter()
                .take(MAX_EVIDENCE_SOURCE_ROWS)
                .map(|source| DxEvidenceBasketSource {
                    label: source.label.clone(),
                    detail: source.detail.clone(),
                    path: source.path.clone(),
                    open_path: source.open_path.clone(),
                    kind: source_kind(source.kind),
                    receipt_drilldowns: source.receipt_drilldowns.len(),
                    proofs: source.proofs.len(),
                    warnings: source.warnings.len(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn source_totals(source_sets: &DxSourceSetSnapshot) -> (usize, usize, usize) {
    let mut drilldowns = 0;
    let mut proofs = 0;
    let mut warnings = 0;

    for source in source_sets.sets.iter().flat_map(|set| set.sources.iter()) {
        drilldowns += source.receipt_drilldowns.len();
        proofs += source.proofs.len();
        warnings += source.warnings.len();
    }

    (drilldowns, proofs, warnings)
}

fn source_kind(kind: DxSourceKind) -> DxEvidenceBasketSourceKind {
    match kind {
        DxSourceKind::WorkspaceRoot => DxEvidenceBasketSourceKind::WorkspaceRoot,
        DxSourceKind::MetasearchSourcePack => DxEvidenceBasketSourceKind::MetasearchSourcePack,
        DxSourceKind::ReducedContextReceipt => DxEvidenceBasketSourceKind::ReducedContextReceipt,
        DxSourceKind::MediaOutput => DxEvidenceBasketSourceKind::MediaOutput,
        DxSourceKind::ForgeRestorePreview => DxEvidenceBasketSourceKind::ForgeRestorePreview,
        DxSourceKind::DxToolchainConfig => DxEvidenceBasketSourceKind::DxToolchainConfig,
    }
}

fn receipt_bucket_count(tool_history: &DxToolHistorySnapshot) -> usize {
    tool_history
        .buckets
        .iter()
        .filter(|bucket| bucket.count > 0)
        .count()
}
