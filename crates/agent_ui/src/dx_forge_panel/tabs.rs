use gpui::{App, IntoElement, WeakEntity};
use ui::{Divider, Tab, prelude::*};

use super::{
    panel::{DxForgePanel, DxForgePanelTab},
    snapshot::DxForgePanelSnapshot,
    visible_rows::visible_row_count_for_tab,
};

pub(super) fn render_tab_bar(
    snapshot: &DxForgePanelSnapshot,
    active_tab: DxForgePanelTab,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .id("dx-forge-tab-bar")
        .h(Tab::container_height(cx))
        .w_full()
        .border_b_1()
        .border_color(cx.theme().colors().border.opacity(0.6))
        .child(forge_tab(
            "dx-forge-tab-repository",
            "Repository",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Repository),
            DxForgePanelTab::Repository,
            active_tab,
            panel,
            cx,
        ))
        .child(Divider::vertical().color(ui::DividerColor::BorderFaded))
        .child(forge_tab(
            "dx-forge-tab-packages",
            "Packages",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Packages),
            DxForgePanelTab::Packages,
            active_tab,
            panel,
            cx,
        ))
        .child(Divider::vertical().color(ui::DividerColor::BorderFaded))
        .child(forge_tab(
            "dx-forge-tab-media",
            "Media",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Media),
            DxForgePanelTab::Media,
            active_tab,
            panel,
            cx,
        ))
        .child(Divider::vertical().color(ui::DividerColor::BorderFaded))
        .child(forge_tab(
            "dx-forge-tab-remotes",
            "Remotes",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Remotes),
            DxForgePanelTab::Remotes,
            active_tab,
            panel,
            cx,
        ))
}

fn forge_tab(
    id: &'static str,
    label: &'static str,
    count: usize,
    tab: DxForgePanelTab,
    active_tab: DxForgePanelTab,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> impl IntoElement {
    let selected = active_tab == tab;
    let panel = panel.clone();

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
        .child(
            Label::new(count.to_string())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .on_click(move |_, _, cx| {
            panel
                .update(cx, |panel, cx| panel.set_active_tab(tab, cx))
                .ok();
        })
}
