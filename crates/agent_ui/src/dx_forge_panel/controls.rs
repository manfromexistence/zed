use gpui::{AnyElement, App, TaskExt, WeakEntity, Window};
use std::path::PathBuf;
use ui::{IconButtonShape, Tooltip, prelude::*};
use workspace::{OpenOptions, Workspace};

use super::{panel::DxForgePanel, snapshot::DxForgePanelSnapshot};

pub(super) fn status_actions(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    _cx: &App,
) -> AnyElement {
    let history_path = snapshot
        .history_root_path
        .as_deref()
        .and_then(exact_abs_path);
    let history_enabled = history_path.as_ref().is_some_and(|path| path.exists());

    h_flex()
        .id("dx-forge-status-actions")
        .flex_none()
        .gap_0p5()
        .child(
            IconButton::new("dx-forge-open-history", IconName::FolderOpen)
                .shape(IconButtonShape::Square)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .style(ButtonStyle::Subtle)
                .tab_index(0_isize)
                .disabled(!history_enabled)
                .tooltip(Tooltip::text(if history_enabled {
                    "Open Forge history"
                } else {
                    "Forge history is unavailable"
                }))
                .on_click({
                    let workspace = workspace.clone();
                    move |_, window, cx| {
                        cx.stop_propagation();
                        if let Some(path) = history_path.clone().filter(|path| path.exists()) {
                            open_exact_abs_path(workspace.clone(), path, window, cx);
                        }
                    }
                }),
        )
        .child(
            IconButton::new("dx-forge-refresh", IconName::RotateCw)
                .shape(IconButtonShape::Square)
                .icon_size(IconSize::Small)
                .style(ButtonStyle::Subtle)
                .tab_index(0_isize)
                .tooltip(Tooltip::text("Refresh Forge"))
                .on_click({
                    let panel = panel.clone();
                    move |_, _, cx| {
                        cx.stop_propagation();
                        panel.update(cx, |panel, cx| panel.refresh(cx)).ok();
                    }
                }),
        )
        .into_any_element()
}

pub(super) fn open_exact_abs_path_button(
    id: impl Into<ElementId>,
    tooltip: &'static str,
    path: &str,
    workspace: &WeakEntity<Workspace>,
) -> AnyElement {
    let path = exact_abs_path(path);
    let enabled = path.as_ref().is_some_and(|path| path.exists());
    let tooltip_text = if enabled {
        tooltip.to_string()
    } else {
        format!("{tooltip} unavailable")
    };

    IconButton::new(id, IconName::ArrowUpRight)
        .shape(IconButtonShape::Square)
        .icon_size(IconSize::Small)
        .icon_color(Color::Muted)
        .style(ButtonStyle::Subtle)
        .tab_index(0_isize)
        .disabled(!enabled)
        .tooltip(Tooltip::text(tooltip_text))
        .on_click({
            let workspace = workspace.clone();
            move |_, window, cx| {
                cx.stop_propagation();
                if let Some(path) = path.clone().filter(|path| path.exists()) {
                    open_exact_abs_path(workspace.clone(), path, window, cx);
                }
            }
        })
        .into_any_element()
}

pub(super) fn exact_abs_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() {
        return None;
    }

    let path = PathBuf::from(path);
    path.is_absolute().then_some(path)
}

pub(super) fn open_exact_abs_path(
    workspace: WeakEntity<Workspace>,
    path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    if !path.is_absolute() || !path.exists() {
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
