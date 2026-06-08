use gpui::{AnyElement, App, MouseButton, SharedString, WeakEntity, px};
use ui::{
    Checkbox, ElevationIndex, IconName, ListItem, ListItemSpacing, ToggleState, Tooltip, prelude::*,
};

use super::{
    panel::DxForgePanel,
    snapshot::{DxForgeReceiptRow, DxForgeSourceRow},
};

pub(super) fn selectable_receipt_row(
    ix: usize,
    receipt: &DxForgeReceiptRow,
    panel: &WeakEntity<DxForgePanel>,
    open_button: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let id = SharedString::from(format!("dx-forge-repository-receipt-{ix}"));
    let item_key = format!("receipt:{}", receipt.source_path);
    let selected = item_selected(panel, &item_key, cx);
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
        selected,
        IconName::FileTextOutlined,
        icon_color,
        receipt.headline.clone(),
        receipt.detail.clone(),
        receipt.source_path.clone(),
        panel,
        open_button,
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
    let item_key = format!("source:{}", source.open_path);
    let selected = item_selected(panel, &item_key, cx);
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
        selected,
        icon,
        icon_color,
        source.label.clone(),
        source.detail.clone(),
        source.path.clone(),
        panel,
        open_button,
    )
    .tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
    .into_any_element()
}

pub(super) fn selection_checkbox(
    id: SharedString,
    item_key: String,
    selected: bool,
    panel: &WeakEntity<DxForgePanel>,
) -> AnyElement {
    let toggle_state = if selected {
        ToggleState::Selected
    } else {
        ToggleState::Unselected
    };
    let panel = panel.clone();
    let checkbox_key = item_key.clone();

    h_flex()
        .id(SharedString::from(format!("dx-forge-selection-slot-{id}")))
        .child(
            Checkbox::new(
                SharedString::from(format!("dx-forge-select-{id}")),
                toggle_state,
            )
            .fill()
            .elevation(ElevationIndex::Surface)
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                panel
                    .update(cx, |panel, cx| {
                        panel.toggle_item_selection(checkbox_key.clone(), cx)
                    })
                    .ok();
            }),
        )
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .into_any_element()
}

fn selectable_row(
    id: SharedString,
    item_key: String,
    selected: bool,
    icon: IconName,
    icon_color: Color,
    title: String,
    detail: String,
    path: String,
    panel: &WeakEntity<DxForgePanel>,
    open_button: Option<AnyElement>,
) -> ListItem {
    let panel_for_row = panel.clone();
    let row_key = item_key.clone();
    let selection_checkbox = selection_checkbox(id.clone(), item_key, selected, panel);
    let mut row = ListItem::new(id)
        .inset(true)
        .height(px(52.0))
        .spacing(ListItemSpacing::Sparse)
        .toggle_state(selected)
        .start_slot(selection_checkbox)
        .child(
            h_flex()
                .w_full()
                .min_w_0()
                .flex_1()
                .gap_2()
                .child(Icon::new(icon).size(IconSize::Small).color(icon_color))
                .child(
                    v_flex()
                        .w_full()
                        .min_w_0()
                        .flex_1()
                        .gap_0p5()
                        .child(Label::new(title).size(LabelSize::Small).truncate())
                        .child(
                            Label::new(detail)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate(),
                        )
                        .child(
                            Label::new(path)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate_start(),
                        ),
                ),
        )
        .on_click(move |_, _, cx| {
            panel_for_row
                .update(cx, |panel, cx| {
                    panel.toggle_item_selection(row_key.clone(), cx)
                })
                .ok();
        });

    if let Some(open_button) = open_button {
        row = row.end_slot(open_button);
    }

    row
}

fn item_selected(panel: &WeakEntity<DxForgePanel>, item_key: &str, cx: &App) -> bool {
    panel
        .upgrade()
        .is_some_and(|panel| panel.read(cx).item_selected(item_key))
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
