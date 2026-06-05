use gpui::{
    AnyElement, App, EntityId, InteractiveElement, IntoElement, ScrollHandle, SharedString,
    WeakEntity, Window,
};
use ui::{WithScrollbar, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::{
    controls::{open_path_button, toolbar},
    panel::DxForgePanel,
    providers::remote_target_strip,
    rows::{empty_row, receipt_row, section_header, source_row, status_strip},
    snapshot::{DxForgePanelSnapshot, DxForgeSourceRow},
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
        .child(remote_target_strip(snapshot, workspace, cx))
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
                        .child(package_status_section(snapshot, workspace, cx))
                        .child(machine_cache_section(snapshot, workspace, cx))
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
                .child(Icon::new(IconName::Forgejo).size(IconSize::Small))
                .child(Label::new("Forge").size(LabelSize::Small).truncate()),
        )
        .child(side_panel_header_controls(
            "dx-forge-panel",
            workspace.clone(),
            panel_id,
            cx,
        ))
}

fn package_status_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    source_section(
        SourceSection {
            header_id: "dx-forge-package-status-header",
            title: "Package Status",
            icon: IconName::Box,
            empty_id: "dx-forge-package-status-empty",
            workspace_empty: "Open a workspace to read Forge package status",
            empty: "No Forge package status found",
            row_id: "dx-forge-package-status",
            open_id: "dx-forge-open-package-status",
            open_tooltip: "Open package status",
        },
        &snapshot.package_statuses,
        snapshot,
        workspace,
        cx,
    )
}

fn machine_cache_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    source_section(
        SourceSection {
            header_id: "dx-forge-machine-caches-header",
            title: "Machine Caches",
            icon: IconName::Binary,
            empty_id: "dx-forge-machine-caches-empty",
            workspace_empty: "Open a workspace to read Forge machine caches",
            empty: "No Forge machine caches found",
            row_id: "dx-forge-machine-cache",
            open_id: "dx-forge-open-machine-cache-root",
            open_tooltip: "Open machine cache root",
        },
        &snapshot.machine_caches,
        snapshot,
        workspace,
        cx,
    )
}

struct SourceSection {
    header_id: &'static str,
    title: &'static str,
    icon: IconName,
    empty_id: &'static str,
    workspace_empty: &'static str,
    empty: &'static str,
    row_id: &'static str,
    open_id: &'static str,
    open_tooltip: &'static str,
}

fn source_section(
    section: SourceSection,
    rows: &[DxForgeSourceRow],
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        section.header_id,
        section.title,
        section.icon,
        rows.len(),
        cx,
    ));

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row(section.empty_id, section.workspace_empty, cx));
    } else if rows.is_empty() {
        stack = stack.child(empty_row(section.empty_id, section.empty, cx));
    } else {
        for (ix, row) in rows.iter().enumerate() {
            stack = stack.child(source_row(
                SharedString::from(format!("{}-{ix}", section.row_id)),
                section.icon,
                row,
                Some(open_path_button(
                    format!("{}-{ix}", section.open_id),
                    section.open_tooltip,
                    &row.path,
                    &snapshot.workspace_roots,
                    workspace,
                )),
                cx,
            ));
        }
    }

    stack.into_any_element()
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
    source_section(
        SourceSection {
            header_id: "dx-forge-restores-header",
            title: "Restore Previews",
            icon: IconName::Download,
            empty_id: "dx-forge-restores-empty",
            workspace_empty: "Open a workspace to read restore previews",
            empty: "No restore previews found",
            row_id: "dx-forge-restore",
            open_id: "dx-forge-open-restore",
            open_tooltip: "Open restore target",
        },
        &snapshot.restore_previews,
        snapshot,
        workspace,
        cx,
    )
}

fn media_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    source_section(
        SourceSection {
            header_id: "dx-forge-media-header",
            title: "Media Outputs",
            icon: IconName::Image,
            empty_id: "dx-forge-media-empty",
            workspace_empty: "Open a workspace to read media outputs",
            empty: "No media outputs found",
            row_id: "dx-forge-media",
            open_id: "dx-forge-open-media",
            open_tooltip: "Open media output",
        },
        &snapshot.media_outputs,
        snapshot,
        workspace,
        cx,
    )
}
