use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::{Color, Tooltip, prelude::*};

use crate::dx_source_sets::DxSourceItem;

use super::drilldowns::source_receipt_drilldown_rows;
use super::kinds::source_kind_icon;
use super::signals::source_signal_rows;

pub(super) fn source_item_row(
    id: SharedString,
    source: &DxSourceItem,
    source_row_control: Option<AnyElement>,
    cx: &App,
) -> AnyElement {
    let tooltip_label = source.label.clone();
    let tooltip_meta = match (source.detail.is_empty(), source.path.is_empty()) {
        (true, true) => source.label.clone(),
        (true, false) => source.path.clone(),
        (false, true) => source.detail.clone(),
        (false, false) => format!("{} - {}", source.detail, source.path),
    };
    let has_detail = !source.detail.is_empty();
    let mut stack = v_flex()
        .id(id)
        .gap_1()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .tooltip(move |_window, cx| {
            Tooltip::with_meta(tooltip_label.clone(), None, tooltip_meta.clone(), cx)
        })
        .child(
            h_flex()
                .gap_1p5()
                .min_w_0()
                .items_center()
                .child(
                    Icon::new(source_kind_icon(source.kind))
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    v_flex()
                        .min_w_0()
                        .gap_0p5()
                        .child(
                            Label::new(source.label.clone())
                                .size(LabelSize::Small)
                                .color(Color::Default)
                                .truncate(),
                        )
                        .when(has_detail, |this| {
                            this.child(
                                Label::new(source.detail.clone())
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                                    .truncate(),
                            )
                        }),
                ),
        );

    if let Some(source_row_control) = source_row_control {
        stack = stack.child(source_row_control);
    }

    stack
        .children(source_receipt_drilldown_rows(source, cx))
        .children(source_signal_rows(source))
        .into_any_element()
}
