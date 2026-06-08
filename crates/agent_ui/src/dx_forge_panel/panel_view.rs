use gpui::{
    AnyElement, App, EntityId, InteractiveElement, IntoElement, ScrollHandle, WeakEntity, Window,
};
use ui::{WithScrollbar, prelude::*};
use workspace::{Workspace, dock::side_panel_header_controls};

use super::{
    controls::{open_exact_abs_path_button, toolbar},
    panel::{DxForgePanel, DxForgePanelTab},
    providers::remote_target_strip,
    rows::{empty_row, section_header, status_strip},
    snapshot::DxForgePanelSnapshot,
    source_section::{SourceSection, source_section},
    tabs::render_tab_bar,
    workflow_rows::selectable_receipt_row,
};

pub(super) fn render_panel(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    panel_id: EntityId,
    active_tab: DxForgePanelTab,
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
        .child(render_tab_bar(snapshot, active_tab, panel, cx))
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
                        .children(match active_tab {
                            DxForgePanelTab::Repository => {
                                vec![repository_section(snapshot, workspace, panel, cx)]
                            }
                            DxForgePanelTab::Packages => vec![
                                package_status_section(snapshot, workspace, panel, cx),
                                machine_cache_section(snapshot, workspace, panel, cx),
                            ],
                            DxForgePanelTab::Media => vec![
                                media_section(snapshot, workspace, panel, cx),
                                restore_section(snapshot, workspace, panel, cx),
                            ],
                            DxForgePanelTab::Remotes => vec![
                                remote_target_strip(snapshot, workspace, panel, cx)
                                    .into_any_element(),
                                remote_registry_section(snapshot, workspace, panel, cx),
                            ],
                        })
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

fn remote_registry_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    source_section(
        SourceSection {
            header_id: "dx-forge-remote-registry-header",
            title: "Remote Registry",
            icon: IconName::CloudDownload,
            empty_id: "dx-forge-remote-registry-empty",
            workspace_empty: "Open a workspace to read Forge remotes",
            empty: "No Forge remote registry found",
            row_id: "dx-forge-remote-registry",
            open_id: "dx-forge-open-remote-registry",
            open_tooltip: "Open remote registry",
        },
        &snapshot.remote_registries,
        snapshot,
        workspace,
        panel,
        cx,
    )
}

fn package_status_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
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
        panel,
        cx,
    )
}

fn machine_cache_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
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
        panel,
        cx,
    )
}

fn repository_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    receipt_section(snapshot, workspace, panel, cx)
}

fn receipt_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        "dx-forge-receipts-header",
        "Repository History",
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
            stack = stack.child(selectable_receipt_row(
                ix,
                receipt,
                panel,
                Some(open_exact_abs_path_button(
                    format!("dx-forge-open-receipt-{ix}"),
                    "Open receipt",
                    &receipt.source_path,
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
    panel: &WeakEntity<DxForgePanel>,
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
        panel,
        cx,
    )
}

fn media_section(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    source_section(
        SourceSection {
            header_id: "dx-forge-media-header",
            title: "Media Outputs",
            icon: dx_icon(DxUiIcon::Media),
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
        panel,
        cx,
    )
}
