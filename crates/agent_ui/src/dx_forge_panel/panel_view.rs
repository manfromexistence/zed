use gpui::{
    AnyElement, App, EntityId, IntoElement, ScrollHandle, SharedString, WeakEntity, Window,
};
use ui::{IconName, WithScrollbar, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::{
    rows::{
        empty_row, metric_row, receipt_row, section, source_row, state_presentation, status_row,
    },
    snapshot::DxForgePanelSnapshot,
};

pub(super) fn render_panel(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel_id: EntityId,
    scroll_handle: &ScrollHandle,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    v_flex()
        .id("dx-forge-panel")
        .size_full()
        .min_h_0()
        .min_w_0()
        .bg(cx.theme().colors().panel_background)
        .child(panel_header(workspace, panel_id, cx))
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .gap_2()
                .p_2()
                .child(overview(snapshot, cx))
                .child(receipt_section(snapshot, cx))
                .child(restore_section(snapshot, cx))
                .child(media_section(snapshot, cx))
                .vertical_scrollbar_for(scroll_handle, _window, cx),
        )
}

fn panel_header(
    workspace: &WeakEntity<Workspace>,
    panel_id: EntityId,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .justify_between()
        .gap_2()
        .px_2()
        .py_1()
        .child(
            h_flex()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(Icon::new(IconName::Archive).size(IconSize::Small))
                .child(Label::new("Forge").size(LabelSize::Small).truncate()),
        )
        .child(side_panel_header_controls(
            "dx-forge-panel",
            workspace.clone(),
            panel_id,
            cx,
        ))
}

fn overview(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
    let (icon, color, label) = state_presentation(snapshot.state);
    section(
        "dx-forge-overview",
        "Forge Proof",
        IconName::ToolHammer,
        v_flex()
            .gap_1()
            .child(status_row(
                icon,
                color,
                label,
                snapshot.state_detail.clone(),
            ))
            .child(metric_row(
                "Workspace scope",
                snapshot.workspace_scope.clone(),
            ))
            .child(metric_row(
                "Forge roots",
                snapshot.configured_root_scope.clone(),
            ))
            .child(metric_row(
                "Receipt root",
                snapshot.history_root_label.clone(),
            ))
            .child(metric_row("Receipts", snapshot.receipt_count.to_string()))
            .child(metric_row(
                "Summaries",
                snapshot.summarized_receipt_count.to_string(),
            ))
            .child(metric_row(
                "Visible blockers",
                snapshot.visible_blocker_count.to_string(),
            ))
            .child(metric_row(
                "Visible restore warnings",
                snapshot.visible_restore_warning_count.to_string(),
            ))
            .into_any_element(),
        cx,
    )
}

fn receipt_section(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1();

    if snapshot.latest_receipts.is_empty() {
        stack = stack.child(empty_row(
            if snapshot.workspace_roots.is_empty() {
                "Open a workspace to read Forge receipts"
            } else if snapshot.receipt_count > 0 {
                "Forge receipts found, but no known summaries were readable"
            } else if snapshot.history_root_exists {
                "No Forge receipts found"
            } else {
                "Forge receipt root is missing"
            },
            cx,
        ));
    } else {
        for (ix, receipt) in snapshot.latest_receipts.iter().enumerate() {
            stack = stack.child(receipt_row(ix, receipt, cx));
        }
    }

    section(
        "dx-forge-receipts",
        "Receipts",
        IconName::FileTextOutlined,
        stack.into_any_element(),
        cx,
    )
}

fn restore_section(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1();

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row("Open a workspace to read restore previews", cx));
    } else if snapshot.restore_previews.is_empty() {
        stack = stack.child(empty_row("No restore previews found", cx));
    } else {
        for (ix, preview) in snapshot.restore_previews.iter().enumerate() {
            stack = stack.child(source_row(
                SharedString::from(format!("dx-forge-restore-{ix}")),
                IconName::Download,
                preview,
                cx,
            ));
        }
    }

    section(
        "dx-forge-restores",
        "Restore Previews",
        IconName::Download,
        stack.into_any_element(),
        cx,
    )
}

fn media_section(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1();

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row("Open a workspace to read media outputs", cx));
    } else if snapshot.media_outputs.is_empty() {
        stack = stack.child(empty_row("No media outputs found", cx));
    } else {
        for (ix, output) in snapshot.media_outputs.iter().enumerate() {
            stack = stack.child(source_row(
                SharedString::from(format!("dx-forge-media-{ix}")),
                IconName::Image,
                output,
                cx,
            ));
        }
    }

    section(
        "dx-forge-media",
        "Media Outputs",
        IconName::Image,
        stack.into_any_element(),
        cx,
    )
}
