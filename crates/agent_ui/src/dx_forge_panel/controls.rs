use gpui::{AnyElement, App, AppContext as _, TaskExt, WeakEntity, Window, px};
use std::path::PathBuf;
use ui::{IconButtonShape, Tooltip, prelude::*};
use workspace::{OpenOptions, Workspace};

use super::{panel::DxForgePanel, snapshot::DxForgePanelSnapshot};

pub(super) fn toolbar(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    let history_path = snapshot.history_root_path.clone().map(PathBuf::from);
    let history_enabled = history_path.as_ref().is_some_and(|path| path.exists());

    h_flex()
        .id("dx-forge-toolbar")
        .h(px(32.0))
        .w_full()
        .min_w_0()
        .px_1()
        .gap_2()
        .justify_between()
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(
            IconButton::new("dx-forge-open-history", IconName::FolderOpen)
                .shape(IconButtonShape::Square)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .disabled(!history_enabled)
                .tooltip(Tooltip::text(if history_enabled {
                    "Open Forge history root"
                } else {
                    "Forge history root is not available for this workspace scope"
                }))
                .on_click({
                    let workspace = workspace.clone();
                    move |_, window, cx| {
                        if let Some(path) = history_path.clone().filter(|path| path.exists()) {
                            open_workspace_path(workspace.clone(), path, window, cx);
                        }
                    }
                }),
        )
        .child(
            h_flex().gap_1().child(
                IconButton::new("dx-forge-refresh", IconName::RotateCw)
                    .shape(IconButtonShape::Square)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Refresh Forge panel"))
                    .on_click({
                        let panel = panel.clone();
                        move |_, _, cx| {
                            panel.update(cx, |panel, cx| panel.refresh(cx)).ok();
                        }
                    }),
            ),
        )
        .into_any_element()
}

pub(super) fn open_path_button(
    id: impl Into<ElementId>,
    tooltip: &'static str,
    path: &str,
    workspace_roots: &[String],
    workspace: &WeakEntity<Workspace>,
) -> AnyElement {
    let path = workspace_path(path, workspace_roots);
    let enabled = path.as_ref().is_some_and(|path| path.exists());

    IconButton::new(id, IconName::ArrowUpRight)
        .shape(IconButtonShape::Square)
        .icon_size(IconSize::XSmall)
        .icon_color(Color::Muted)
        .disabled(!enabled)
        .tooltip(Tooltip::text(if enabled {
            tooltip
        } else {
            "Source path is not available"
        }))
        .on_click({
            let workspace = workspace.clone();
            move |_, window, cx| {
                if let Some(path) = path.clone().filter(|path| path.exists()) {
                    open_workspace_path(workspace.clone(), path, window, cx);
                }
            }
        })
        .into_any_element()
}

pub(super) fn workspace_path(path: &str, workspace_roots: &[String]) -> Option<PathBuf> {
    if path.is_empty() {
        return None;
    }

    let direct = PathBuf::from(path);
    if direct.is_absolute() {
        return Some(direct);
    }

    for root in workspace_roots {
        let candidate = PathBuf::from(root).join(&direct);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    workspace_roots
        .first()
        .map(|root| PathBuf::from(root).join(direct))
}

pub(super) fn open_workspace_path(
    workspace: WeakEntity<Workspace>,
    path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    if !path.exists() {
        return;
    }

    workspace
        .update(cx, |workspace, cx| {
            workspace
                .open_abs_path(
                    path,
                    OpenOptions {
                        focus: Some(true),
                        ..Default::default()
                    },
                    window,
                    cx,
                )
                .detach_and_log_err(cx);
        })
        .ok();
}
