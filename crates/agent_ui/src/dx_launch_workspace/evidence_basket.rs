use gpui::{AnyElement, App, prelude::*};
use ui::prelude::*;

use crate::dx_evidence_basket::DxEvidenceBasket;

use super::list_labels::{bounded_items, yes_no};
use super::{metric_row, muted_card};

pub(super) fn evidence_basket_state(basket: &DxEvidenceBasket, cx: &App) -> AnyElement {
    let source_preview_labels = basket.source_preview_labels();
    let flow_binary = basket
        .flow
        .dictate_binary
        .clone()
        .unwrap_or_else(|| "missing".to_string());

    v_flex()
        .gap_1()
        .child(metric_row(
            "Workspace roots",
            basket.workspace_roots.to_string(),
        ))
        .child(metric_row(
            "Total sources",
            basket.total_sources.to_string(),
        ))
        .child(metric_row(
            "Attachable",
            basket.attachable_sources.to_string(),
        ))
        .child(metric_row(
            "Managed receipts",
            basket.managed_receipts.to_string(),
        ))
        .child(metric_row(
            "Receipt buckets",
            basket.receipt_buckets.to_string(),
        ))
        .child(metric_row(
            "Receipt files",
            basket.receipt_count.to_string(),
        ))
        .child(metric_row(
            "Media outputs",
            basket.media_outputs.to_string(),
        ))
        .child(metric_row(
            "Restore previews",
            basket.forge_restore_previews.to_string(),
        ))
        .child(metric_row(
            "Source rows",
            basket.evidence_source_count().to_string(),
        ))
        .child(metric_row(
            "Source signals",
            basket.source_signal_count().to_string(),
        ))
        .child(metric_row(
            "Drilldowns",
            basket.source_drilldowns.to_string(),
        ))
        .child(metric_row("Proof notes", basket.source_proofs.to_string()))
        .child(metric_row("Warnings", basket.source_warnings.to_string()))
        .child(metric_row(
            "Preview",
            bounded_items(&source_preview_labels, 2, "No evidence rows"),
        ))
        .child(metric_row("Flow root", basket.flow.flow_root.clone()))
        .child(metric_row("Flow binary", flow_binary))
        .child(metric_row("STT model", basket.flow.stt_model.clone()))
        .child(metric_row(
            "Flow STT",
            format!(
                "{} - {}",
                yes_no(basket.flow.stt_ready),
                basket.flow.stt_detail
            ),
        ))
        .child(metric_row(
            "Flow TTS",
            format!(
                "{} - {}",
                yes_no(basket.flow.kokoro_ready),
                basket.flow.kokoro_detail
            ),
        ))
        .child(metric_row(
            "Flow input",
            basket.flow.input_device_detail.clone(),
        ))
        .when(basket.evidence_source_count() == 0, |this| {
            this.child(muted_card("No evidence receipts found", cx))
        })
        .into_any_element()
}
