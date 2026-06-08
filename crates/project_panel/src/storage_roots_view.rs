use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Stateful,
    Styled, WeakEntity,
};
use ui::{Color, DxUiIcon, Icon, IconSize, Label, LabelSize, Tooltip, dx_icon, prelude::*, v_flex};
use util::ResultExt;

use crate::{ProjectPanel, storage_roots};

pub(crate) fn render_storage_root_strip(
    shortcuts: Vec<storage_roots::StorageRootShortcut>,
    panel: WeakEntity<ProjectPanel>,
    cx: &mut Context<ProjectPanel>,
) -> Option<AnyElement> {
    if shortcuts.is_empty() {
        return None;
    }

    let rows = shortcuts
        .into_iter()
        .map(|shortcut| render_storage_root_strip_row(shortcut, panel.clone(), cx))
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
                h_flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Icon::new(dx_icon(DxUiIcon::Storage))
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new("Storage roots")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .child(
                div()
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

    h_flex()
        .id(SharedString::from(format!(
            "dx-explorer-storage-root-{}",
            shortcut.id
        )))
        .flex_none()
        .items_center()
        .gap_1()
        .px_1()
        .py_0p5()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant.opacity(0.5))
        .bg(cx.theme().colors().element_background.opacity(0.35))
        .tooltip(move |_window, cx| Tooltip::with_meta("Storage root", None, tooltip.clone(), cx))
        .when(available, |this| {
            this.cursor_pointer()
                .hover(|style| style.bg(cx.theme().colors().element_hover.opacity(0.6)))
                .on_click(move |_, window, cx| {
                    panel
                        .update_in(cx, |this, window, cx| {
                            this.open_dx_explorer_storage_root(path.clone(), window, cx);
                        })
                        .log_err();
                })
        })
        .when(!available, |this| this.cursor_not_allowed().opacity(0.55))
        .child(Icon::new(icon).size(IconSize::XSmall).color(if available {
            Color::Muted
        } else {
            Color::Disabled
        }))
        .child(
            Label::new(shortcut.label)
                .size(LabelSize::XSmall)
                .color(if available {
                    Color::Default
                } else {
                    Color::Muted
                })
                .single_line(),
        )
        .child(
            Label::new(status_label)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .single_line(),
        )
        .into_any_element()
}
