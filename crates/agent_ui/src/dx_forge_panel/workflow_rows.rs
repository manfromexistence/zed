use gpui::{AnyElement, App, MouseButton, ScrollAnchor, SharedString, WeakEntity, rems};
use ui::{
    Checkbox, ElevationIndex, IconName, ListItem, ListItemSpacing, ToggleState, Tooltip, prelude::*,
};

use super::{
    panel::DxForgePanel,
    snapshot::{DxForgeReceiptRow, DxForgeSourceRow},
    visible_rows::{DxForgeRowKey, receipt_item_key, source_item_key},
};

pub(super) fn selectable_receipt_row(
    ix: usize,
    receipt: &DxForgeReceiptRow,
    panel: &WeakEntity<DxForgePanel>,
    open_button: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let id = SharedString::from(format!("dx-forge-repository-receipt-{ix}"));
    let item_key = receipt_item_key(receipt);
    let checked = item_checked(panel, &item_key, cx);
    let active = item_active(panel, &item_key, cx);
    let icon_color = if receipt.blocker_count > 0 {
        Color::Warning
    } else {
        Color::Muted
    };
    let title = receipt.headline.clone();
    let meta = receipt_tooltip(receipt);

    selectable_row(
        id,
        item_key,
        checked,
        active,
        IconName::FileTextOutlined,
        icon_color,
        receipt.headline.clone(),
        receipt.detail.clone(),
        panel,
        open_button,
        cx,
    )
    .tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
    .into_any_element()
}

pub(super) fn selectable_source_row(
    id: SharedString,
    icon: IconName,
    source: &DxForgeSourceRow,
    panel: &WeakEntity<DxForgePanel>,
    open_button: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let item_key = source_item_key(source);
    let checked = item_checked(panel, &item_key, cx);
    let active = item_active(panel, &item_key, cx);
    let icon_color = if source.warnings.is_empty() {
        Color::Muted
    } else {
        Color::Warning
    };
    let title = source.label.clone();
    let meta = source_tooltip(source);

    selectable_row(
        id,
        item_key,
        checked,
        active,
        icon,
        icon_color,
        source.label.clone(),
        source.detail.clone(),
        panel,
        open_button,
        cx,
    )
    .tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
    .into_any_element()
}

pub(super) fn selection_checkbox(
    id: SharedString,
    item_key: DxForgeRowKey,
    checked: bool,
    panel: &WeakEntity<DxForgePanel>,
) -> AnyElement {
    let toggle_state = if checked {
        ToggleState::Selected
    } else {
        ToggleState::Unselected
    };
    let panel = panel.clone();
    let checkbox_key = item_key.clone();

    h_flex()
        .id(SharedString::from(format!("dx-forge-selection-slot-{id}")))
        .flex_none()
        .occlude()
        .cursor_pointer()
        .child(
            Checkbox::new(
                SharedString::from(format!("dx-forge-select-{id}")),
                toggle_state,
            )
            .fill()
            .elevation(ElevationIndex::Surface)
            .on_click(move |_, window, cx| {
                cx.stop_propagation();
                panel
                    .update(cx, |panel, cx| {
                        panel.focus_panel(window, cx);
                        panel.activate_item(checkbox_key.clone(), cx);
                        panel.toggle_item_checked(checkbox_key.clone(), cx)
                    })
                    .ok();
            }),
        )
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .into_any_element()
}

fn selectable_row(
    id: SharedString,
    item_key: DxForgeRowKey,
    checked: bool,
    active: bool,
    icon: IconName,
    icon_color: Color,
    title: String,
    detail: String,
    panel: &WeakEntity<DxForgePanel>,
    open_button: Option<AnyElement>,
    cx: &App,
) -> ListItem {
    let panel_for_row = panel.clone();
    let row_key = item_key.clone();
    let scroll_anchor = row_scroll_anchor(panel, &row_key, cx);
    let hover_checkbox_id = SharedString::from(format!("{id}-hover"));
    let checkbox = selection_checkbox(id.clone(), item_key.clone(), checked, panel);
    let hover_checkbox = selection_checkbox(hover_checkbox_id, item_key, checked, panel);
    let row_actions = selectable_row_actions(open_button, hover_checkbox);
    ListItem::new(id)
        .anchor_scroll(scroll_anchor)
        .inset(true)
        .height(rems(1.75))
        .spacing(ListItemSpacing::Sparse)
        .toggle_state(active)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(icon_color))
        .child(
            Label::new(title)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .end_slot(checkbox)
        .end_slot_on_hover(row_actions)
        .tooltip(Tooltip::text(detail))
        .on_click(move |_, window, cx| {
            panel_for_row
                .update(cx, |panel, cx| {
                    panel.focus_panel(window, cx);
                    panel.activate_item(row_key.clone(), cx)
                })
                .ok();
        })
}

pub(super) fn row_scroll_anchor(
    panel: &WeakEntity<DxForgePanel>,
    item_key: &DxForgeRowKey,
    cx: &App,
) -> Option<ScrollAnchor> {
    panel
        .upgrade()
        .and_then(|panel| panel.read(cx).row_scroll_anchor(item_key))
}

fn selectable_row_actions(
    open_button: Option<AnyElement>,
    selection_checkbox: AnyElement,
) -> AnyElement {
    let mut actions = h_flex()
        .flex_none()
        .gap_0p5()
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        });

    if let Some(open_button) = open_button {
        actions = actions.child(open_button);
    }

    actions.child(selection_checkbox).into_any_element()
}

fn item_checked(panel: &WeakEntity<DxForgePanel>, item_key: &DxForgeRowKey, cx: &App) -> bool {
    panel
        .upgrade()
        .is_some_and(|panel| panel.read(cx).item_checked(item_key))
}

fn item_active(panel: &WeakEntity<DxForgePanel>, item_key: &DxForgeRowKey, cx: &App) -> bool {
    panel
        .upgrade()
        .is_some_and(|panel| panel.read(cx).item_active(item_key))
}

fn receipt_tooltip(receipt: &DxForgeReceiptRow) -> String {
    let mut lines = vec![
        format!("{} - {}", receipt.kind, receipt.detail),
        receipt.source_path.clone(),
    ];
    if let Some(target_path) = receipt.target_path.as_ref() {
        lines.push(format!("Target: {target_path}"));
    }
    if let Some(destination) = receipt.restore_destination_root.as_ref() {
        lines.push(format!("Restore: {destination}"));
    }
    lines.join("\n")
}

fn source_tooltip(source: &DxForgeSourceRow) -> String {
    let mut lines = vec![source.detail.clone(), source.path.clone()];
    for receipt in source.receipts.iter().take(2) {
        lines.push(format!("{}: {}", receipt.label, receipt.detail));
    }
    for warning in source.warnings.iter().take(2) {
        lines.push(format!("Warning: {warning}"));
    }
    lines.join("\n")
}
