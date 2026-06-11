use std::path::Path;

use gpui::{AnyElement, SharedString, prelude::*};
use ui::{IconName, ListHeader, ListItem, ListItemSpacing, Tooltip, prelude::*};

pub(super) fn style_section(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    count_label: impl Into<SharedString>,
) -> AnyElement {
    div()
        .id(id)
        .child(
            ListHeader::new(title)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .end_slot(
                    Label::new(count_label.into())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .into_any_element()
}

pub(super) fn style_detail_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    icon: IconName,
    color: Color,
) -> AnyElement {
    let label = label.into();
    let value = value.into();
    let tooltip = format!("{}: {}", label.as_ref(), value.as_ref());

    ListItem::new(id.into())
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .end_slot(
            Label::new(value)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn style_path_row(
    id: impl Into<SharedString>,
    label: &'static str,
    present: bool,
    path: &Path,
) -> AnyElement {
    let value = if present {
        path.file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("present")
            .to_string()
    } else {
        "missing".to_string()
    };
    let (icon, color) = if present {
        (IconName::FileTextOutlined, Color::Muted)
    } else {
        (IconName::Warning, Color::Warning)
    };

    let tooltip = format!("{label}: {}", path.display());

    ListItem::new(id.into())
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .end_slot(
            Label::new(value)
                .size(LabelSize::Small)
                .color(if present {
                    Color::Default
                } else {
                    Color::Warning
                })
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn style_note_row(
    id: impl Into<SharedString>,
    icon: IconName,
    color: Color,
    label: impl Into<SharedString>,
) -> AnyElement {
    let label = label.into();

    ListItem::new(id.into())
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(label.clone())
                .size(LabelSize::Small)
                .color(color)
                .truncate(),
        )
        .tooltip(Tooltip::text(label))
        .into_any_element()
}
