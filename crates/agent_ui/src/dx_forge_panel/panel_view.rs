use gpui::{
    AnyElement, App, EntityId, InteractiveElement, IntoElement, ScrollHandle, SharedString,
    StatefulInteractiveElement, WeakEntity, Window,
};
use ui::{WithScrollbar, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::{
    controls::{open_path_button, toolbar},
    panel::DxForgePanel,
    providers::remote_target_strip,
    rows::{empty_row, receipt_row, section_header, source_row, status_strip},
    snapshot::DxForgePanelSnapshot,
};

pub(super) fn render_panel(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    panel_id: EntityId,
    scroll_handle: &ScrollHandle,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    v_flex()
        .id("dx-forge-panel")
        .size_full()
        .min_h_0()
        .min_w_0()
        .bg(cx.theme().colors().panel_background)
        .child(panel_header(workspace, panel_id, cx))
        .child(status_strip(
            snapshot.state,
            snapshot.state_detail.clone(),
            snapshot.workspace_scope.clone(),
            cx,
        ))
        .child(toolbar(snapshot, workspace, panel, cx))
        .child(remote_target_strip(snapshot, cx))
        .child(
            v_flex()
                .id("dx-forge-panel-content")
                .flex_1()
                .size_full()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .py_1()
                        .child(receipt_section(snapshot, workspace, cx))
                        .child(restore_section(snapshot, workspace, cx))
                        .child(media_section(snapshot, workspace, cx))
                        .vertical_scrollbar_for(scroll_handle, window, cx),
                ),
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

fn receipt_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        "dx-forge-receipts-header",
        "Receipts",
        IconName::FileTextOutlined,
        snapshot.receipt_count,
        cx,
    ));

    if snapshot.latest_receipts.is_empty() {
        stack = stack.child(empty_row(
            "dx-forge-receipts-empty",
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
            stack = stack.child(receipt_row(
                ix,
                receipt,
                Some(open_path_button(
                    format!("dx-forge-open-receipt-{ix}"),
                    "Open receipt",
                    &receipt.source_path,
                    &snapshot.workspace_roots,
                    workspace,
                )),
                cx,
            ));
        }
    }

    stack.into_any_element()
}

fn restore_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        "dx-forge-restores-header",
        "Restore Previews",
        IconName::Download,
        snapshot.restore_previews.len(),
        cx,
    ));

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row(
            "dx-forge-restores-empty",
            "Open a workspace to read restore previews",
            cx,
        ));
    } else if snapshot.restore_previews.is_empty() {
        stack = stack.child(empty_row(
            "dx-forge-restores-empty",
            "No restore previews found",
            cx,
        ));
    } else {
        for (ix, preview) in snapshot.restore_previews.iter().enumerate() {
            stack = stack.child(source_row(
                SharedString::from(format!("dx-forge-restore-{ix}")),
                IconName::Download,
                preview,
                Some(open_path_button(
                    format!("dx-forge-open-restore-{ix}"),
                    "Open restore target",
                    &preview.path,
                    &snapshot.workspace_roots,
                    workspace,
                )),
                cx,
            ));
        }
    }

    stack.into_any_element()
}

fn media_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        "dx-forge-media-header",
        "Media Outputs",
        IconName::Image,
        snapshot.media_outputs.len(),
        cx,
    ));

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row(
            "dx-forge-media-empty",
            "Open a workspace to read media outputs",
            cx,
        ));
    } else if snapshot.media_outputs.is_empty() {
        stack = stack.child(empty_row(
            "dx-forge-media-empty",
            "No media outputs found",
            cx,
        ));
    } else {
        for (ix, output) in snapshot.media_outputs.iter().enumerate() {
            stack = stack.child(source_row(
                SharedString::from(format!("dx-forge-media-{ix}")),
                IconName::Image,
                output,
                Some(open_path_button(
                    format!("dx-forge-open-media-{ix}"),
                    "Open media output",
                    &output.path,
                    &snapshot.workspace_roots,
                    workspace,
                )),
                cx,
            ));
        }
    }

    stack.into_any_element()
}
