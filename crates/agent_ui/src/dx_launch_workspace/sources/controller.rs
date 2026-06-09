use gpui::{AnyElement, App, prelude::*};
use ui::prelude::*;

use crate::dx_source_sets::DxSourceSetSnapshot;

use super::super::compact_status_row;

pub(in crate::dx_launch_workspace) fn source_controller_state(
    snapshot: &DxSourceSetSnapshot,
    cx: &App,
) -> AnyElement {
    let summary = snapshot.attachment_summary();

    v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-source-controller-roots",
            dx_icon(DxUiIcon::Project),
            "Workspace roots",
            summary.workspace_roots.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-controller-total",
            dx_icon(DxUiIcon::Source),
            "Total sources",
            snapshot.total_sources.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-controller-attachable",
            IconName::Paperclip,
            "Attachable",
            summary.attachable_sources.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-controller-receipts",
            dx_icon(DxUiIcon::Receipts),
            "Managed receipts",
            summary.managed_receipts.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-controller-media",
            dx_icon(DxUiIcon::Media),
            "Media outputs",
            summary.produced_files.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-controller-restores",
            IconName::Download,
            "Restore previews",
            summary.restore_previews.to_string(),
            cx,
        ))
        .into_any_element()
}
