use gpui::{AnyElement, App, IntoElement, SharedString};
use ui::{
    Headline, HeadlineSize, IconName, ListHeader, ListItem, ListItemSpacing, Tooltip, prelude::*,
};

pub(super) fn workspace_page_header(
    icon: IconName,
    title: &'static str,
    detail: &'static str,
    stats: Vec<AnyElement>,
    _cx: &App,
) -> AnyElement {
    h_flex()
        .w_full()
        .items_start()
        .justify_between()
        .gap_3()
        .child(
            h_flex()
                .min_w_0()
                .flex_1()
                .items_start()
                .gap_2()
                .child(Icon::new(icon).size(IconSize::Medium).color(Color::Muted))
                .child(
                    v_flex()
                        .min_w_0()
                        .gap_1()
                        .child(Headline::new(title).size(HeadlineSize::Large))
                        .child(
                            Label::new(detail)
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                ),
        )
        .child(h_flex().flex_none().flex_wrap().gap_2().children(stats))
        .into_any_element()
}

pub(super) fn workspace_stat(
    id: &'static str,
    label: &'static str,
    value: impl Into<SharedString>,
    cx: &App,
) -> AnyElement {
    div()
        .id(id)
        .min_w(rems_from_px(104.))
        .max_w(rems_from_px(176.))
        .rounded_md()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().elevated_surface_background.opacity(0.5))
        .px_2()
        .py_1p5()
        .child(
            v_flex()
                .gap_0p5()
                .child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                )
                .child(
                    Label::new(value.into())
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .into_any_element()
}

pub(super) fn screen_section(
    id: &'static str,
    title: &'static str,
    icon: IconName,
    summary: impl Into<SharedString>,
    body: AnyElement,
    cx: &App,
) -> AnyElement {
    let summary = summary.into();

    v_flex()
        .id(id)
        .w_full()
        .min_w_0()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .bg(cx.theme().colors().elevated_surface_background.opacity(0.5))
        .overflow_hidden()
        .child(
            ListHeader::new(title)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .end_slot(
                    Label::new(summary)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(div().w_full().min_w_0().p_2().child(body))
        .into_any_element()
}

pub(super) fn screen_empty_state(
    id: &'static str,
    icon: IconName,
    label: impl Into<SharedString>,
    _cx: &App,
) -> AnyElement {
    let label = label.into();
    let tooltip = label.as_ref().to_string();

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}

pub(super) fn screen_detail_stack(rows: Vec<AnyElement>) -> AnyElement {
    v_flex().gap_0p5().children(rows).into_any_element()
}

pub(super) fn screen_detail_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    let label = label.into();
    let detail = detail.into();
    let tooltip = format!("{}: {}", label.as_ref(), detail.as_ref());

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .end_slot(
            Label::new(detail)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .tooltip(Tooltip::text(tooltip))
        .into_any_element()
}
