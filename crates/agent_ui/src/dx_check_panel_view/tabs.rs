use gpui::{App, IntoElement, WeakEntity};
use ui::{Divider, Tab, prelude::*};

use super::DxCheckPanel;
use super::view_rows::count_chip;
use crate::dx_check_panel::DxCheckPanelSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DxCheckPanelTab {
    Overview,
    Findings,
    Receipt,
}

pub(super) fn render_tab_bar(
    snapshot: &DxCheckPanelSnapshot,
    active_tab: DxCheckPanelTab,
    panel: WeakEntity<DxCheckPanel>,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .id("dx-check-tab-bar")
        .h(Tab::container_height(cx))
        .w_full()
        .border_b_1()
        .border_color(cx.theme().colors().border.opacity(0.6))
        .child(render_tab(
            "dx-check-tab-overview",
            "Overview",
            snapshot.sections.len(),
            DxCheckPanelTab::Overview,
            active_tab,
            panel.clone(),
            cx,
        ))
        .child(Divider::vertical().color(ui::DividerColor::BorderFaded))
        .child(render_tab(
            "dx-check-tab-findings",
            "Findings",
            snapshot.blockers.len()
                + snapshot.warnings.len()
                + snapshot.quick_fixes.len()
                + snapshot.web_audits.len(),
            DxCheckPanelTab::Findings,
            active_tab,
            panel.clone(),
            cx,
        ))
        .child(Divider::vertical().color(ui::DividerColor::BorderFaded))
        .child(render_tab(
            "dx-check-tab-receipt",
            "Receipt",
            usize::from(snapshot.receipt_present),
            DxCheckPanelTab::Receipt,
            active_tab,
            panel,
            cx,
        ))
}

fn render_tab(
    id: &'static str,
    label: &'static str,
    count: usize,
    tab: DxCheckPanelTab,
    active_tab: DxCheckPanelTab,
    panel: WeakEntity<DxCheckPanel>,
    cx: &App,
) -> impl IntoElement {
    let selected = active_tab == tab;

    h_flex()
        .id(id)
        .h_full()
        .flex_1()
        .min_w_0()
        .justify_center()
        .gap_1()
        .px_1()
        .cursor_pointer()
        .border_b_1()
        .when(selected, |this| {
            this.border_color(cx.theme().colors().text_accent)
        })
        .when(!selected, |this| {
            this.bg(cx.theme().colors().editor_background.opacity(0.6))
                .border_color(cx.theme().colors().border.opacity(0.6))
        })
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .when(!selected, |this| this.color(Color::Muted))
                .truncate(),
        )
        .child(count_chip(
            count,
            if selected {
                Color::Accent
            } else {
                Color::Muted
            },
            cx,
        ))
        .on_click(move |_, _, cx| {
            panel
                .update(cx, |panel, cx| panel.set_active_tab(tab, cx))
                .ok();
        })
}
