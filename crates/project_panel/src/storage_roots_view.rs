use gpui::{
    AnyElement, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, WeakEntity,
};
use ui::{
    ButtonLike, ButtonSize, ButtonStyle, Color, DxUiIcon, Icon, IconSize, Label, LabelSize,
    ListHeader, ProgressBar, Tooltip, dx_icon, prelude::*, v_flex,
};
use util::ResultExt;

use crate::{ProjectPanel, storage_roots};

struct DriveCapacityProgress {
    id: String,
    used_percent: f32,
}

pub(crate) fn render_storage_root_strip(
    shortcuts: Vec<storage_roots::StorageRootShortcut>,
    panel: WeakEntity<ProjectPanel>,
    focus_handle: FocusHandle,
    cx: &mut Context<ProjectPanel>,
) -> Option<AnyElement> {
    if shortcuts.is_empty() {
        return None;
    }

    let rows = shortcuts
        .into_iter()
        .map(|shortcut| {
            render_storage_root_strip_row(shortcut, panel.clone(), focus_handle.clone(), cx)
        })
        .collect::<Vec<_>>();

    Some(
        v_flex()
            .id("dx-explorer-storage-root-strip")
            .w_full()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border.opacity(0.6))
            .bg(cx.theme().colors().panel_background)
            .child(
                ListHeader::new("Storage")
                    .start_slot(Icon::new(dx_icon(DxUiIcon::Storage)).size(IconSize::XSmall)),
            )
            .child(
                div()
                    .id("dx-explorer-storage-root-strip-scroll")
                    .w_full()
                    .overflow_x_scroll()
                    .child(h_flex().gap_1().children(rows)),
            )
            .into_any_element(),
    )
}

fn render_storage_root_strip_row(
    shortcut: storage_roots::StorageRootShortcut,
    panel: WeakEntity<ProjectPanel>,
    focus_handle: FocusHandle,
    cx: &mut Context<ProjectPanel>,
) -> AnyElement {
    let icon = match shortcut.kind {
        storage_roots::StorageRootKind::Drive => dx_icon(DxUiIcon::Storage),
        storage_roots::StorageRootKind::DxHub => dx_icon(DxUiIcon::Source),
        storage_roots::StorageRootKind::OneDrive => dx_icon(DxUiIcon::CloudStorage),
        storage_roots::StorageRootKind::GoogleDrive => dx_icon(DxUiIcon::DriveProvider),
        storage_roots::StorageRootKind::Dropbox => dx_icon(DxUiIcon::DropboxProvider),
    };
    let path = shortcut.path.clone();
    let available = shortcut.is_available();
    let tooltip = shortcut.tooltip.clone();
    let status_label = shortcut.status_label();
    let capacity_progress = drive_capacity_progress(&shortcut);

    ButtonLike::new(SharedString::from(format!(
        "dx-explorer-storage-root-{}",
        shortcut.id
    )))
    .style(ButtonStyle::Subtle)
    .size(ButtonSize::Compact)
    .width(rems(18.))
    .disabled(!available)
    .tooltip(move |_window, cx| Tooltip::with_meta("Storage", None, tooltip.clone(), cx))
    .when(available, |this| {
        let row_focus_handle = focus_handle.clone();
        let click_focus_handle = row_focus_handle.clone();
        this.on_click(move |_, window, cx| {
            window.focus(&click_focus_handle, cx);
            panel
                .update_in(cx, |this, window, cx| {
                    this.open_dx_explorer_storage_root(path.clone(), window, cx);
                })
                .log_err();
        })
        .tab_index(0_isize)
        .track_focus(&row_focus_handle)
    })
    .child(Icon::new(icon).size(IconSize::XSmall).color(if available {
        Color::Muted
    } else {
        Color::Disabled
    }))
    .child(
        div().min_w_0().flex_1().child(
            Label::new(shortcut.label)
                .size(LabelSize::XSmall)
                .color(if available {
                    Color::Default
                } else {
                    Color::Muted
                })
                .truncate(),
        ),
    )
    .when_some(capacity_progress, |this, capacity_progress| {
        let progress_color = if capacity_progress.used_percent >= 95.0 {
            Color::Error.color(cx)
        } else if capacity_progress.used_percent >= 85.0 {
            Color::Warning.color(cx)
        } else {
            Color::Info.color(cx)
        };

        this.child(
            div().w(rems(3.5)).flex_none().child(
                ProgressBar::new(
                    capacity_progress.id,
                    capacity_progress.used_percent,
                    100.0_f32,
                    cx,
                )
                .fg_color(progress_color)
                .bg_color(cx.theme().colors().border.opacity(0.35)),
            ),
        )
    })
    .child(
        Label::new(status_label)
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .truncate(),
    )
    .into_any_element()
}

fn drive_capacity_progress(
    shortcut: &storage_roots::StorageRootShortcut,
) -> Option<DriveCapacityProgress> {
    if !matches!(shortcut.kind, storage_roots::StorageRootKind::Drive) {
        return None;
    }

    let capacity = shortcut.capacity.as_ref()?;
    if capacity.total_bytes == 0 {
        return None;
    }

    let used_percent =
        ((capacity.used_bytes() as f32 / capacity.total_bytes as f32) * 100.0).clamp(0.0, 100.0);

    Some(DriveCapacityProgress {
        id: format!("dx-explorer-storage-root-{}-capacity", shortcut.id),
        used_percent,
    })
}
