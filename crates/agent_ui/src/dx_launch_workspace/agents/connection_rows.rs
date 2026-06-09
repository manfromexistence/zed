use gpui::{AnyElement, SharedString, prelude::*};
use ui::{
    AiSettingItem, AiSettingItemSource, AiSettingItemStatus, Color, ListItem, ListItemSpacing,
    prelude::*,
};

pub(super) fn connection_detail_stack(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().gap_0p5().pl_4().children(rows).into_any_element()
}

pub(super) fn connection_detail_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    ListItem::new(id)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            h_flex()
                .min_w_0()
                .gap_1()
                .child(
                    Label::new(label.into())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .flex_none(),
                )
                .child(
                    Label::new(detail.into())
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .into_any_element()
}

pub(super) fn unavailable_capability_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    AiSettingItem::new(
        id.clone(),
        label,
        AiSettingItemStatus::Stopped,
        AiSettingItemSource::Custom,
    )
    .icon(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
    .detail_label("Unavailable")
    .details(connection_detail_stack(vec![
        connection_detail_row(
            format!("{}-detail", id).into(),
            IconName::Warning,
            "State",
            detail,
        )
        .into_any_element(),
    ]))
    .into_any_element()
}

pub(super) fn connection_unavailable_rows() -> Vec<AnyElement> {
    vec![
        unavailable_capability_row(
            "dx-agent-channels-unavailable".into(),
            dx_icon(DxUiIcon::Channels),
            "Channels",
            "No DX Agents channel receipt/schema is available yet.",
        ),
        unavailable_capability_row(
            "dx-agent-gateway-unavailable".into(),
            dx_icon(DxUiIcon::Gateway),
            "Gateway",
            "No first-class provider gateway health receipt is available yet.",
        ),
    ]
}
