use std::cmp::Ordering;

use gpui::{App, EntityId, IntoElement, WeakEntity};
use ui::{DxUiIcon, IconName, Tab, TabBar, TabPosition, Tooltip, dx_icon, prelude::*};

use super::{
    panel::{DxForgePanel, DxForgePanelTab},
    snapshot::DxForgePanelSnapshot,
    visible_rows::visible_row_count_for_tab,
};

pub(super) fn render_tab_bar(
    snapshot: &DxForgePanelSnapshot,
    active_tab: DxForgePanelTab,
    panel_id: EntityId,
    panel: &WeakEntity<DxForgePanel>,
    _cx: &App,
) -> impl IntoElement {
    TabBar::new(("dx-forge-tab-bar", panel_id))
        .child(forge_tab(
            "dx-forge-tab-repository",
            "Repository",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Repository),
            DxForgePanelTab::Repository,
            active_tab,
            panel,
        ))
        .child(forge_tab(
            "dx-forge-tab-packages",
            "Packages",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Packages),
            DxForgePanelTab::Packages,
            active_tab,
            panel,
        ))
        .child(forge_tab(
            "dx-forge-tab-media",
            "Media",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Media),
            DxForgePanelTab::Media,
            active_tab,
            panel,
        ))
        .child(forge_tab(
            "dx-forge-tab-remotes",
            "Remotes",
            visible_row_count_for_tab(snapshot, DxForgePanelTab::Remotes),
            DxForgePanelTab::Remotes,
            active_tab,
            panel,
        ))
}

fn forge_tab(
    id: &'static str,
    label: &'static str,
    count: usize,
    tab: DxForgePanelTab,
    active_tab: DxForgePanelTab,
    panel: &WeakEntity<DxForgePanel>,
) -> impl IntoElement {
    let selected = active_tab == tab;
    let panel = panel.clone();
    let row_noun = if count == 1 { "row" } else { "rows" };
    let title = format!("{label}: {count} {row_noun}");

    Tab::new(id)
        .fill_available_width()
        .position(tab_position(tab, active_tab))
        .toggle_state(selected)
        .selected_bottom_border(true)
        .start_slot(
            Icon::new(tab_icon(tab))
                .size(IconSize::Small)
                .color(if selected {
                    Color::Default
                } else {
                    Color::Muted
                }),
        )
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .when(!selected, |this| this.color(Color::Muted))
                .truncate(),
        )
        .tooltip(Tooltip::text(title))
        .on_click(move |_, _window, cx| {
            cx.stop_propagation();
            panel
                .update(cx, |panel, cx| {
                    panel.set_active_tab(tab, cx);
                })
                .ok();
        })
}

fn tab_position(tab: DxForgePanelTab, active_tab: DxForgePanelTab) -> TabPosition {
    let current_index = tab_index(tab);
    let active_index = tab_index(active_tab);

    match current_index {
        0 => TabPosition::First,
        3 => TabPosition::Last,
        _ if current_index == active_index => TabPosition::Middle(Ordering::Equal),
        _ if current_index < active_index => TabPosition::Middle(Ordering::Less),
        _ => TabPosition::Middle(Ordering::Greater),
    }
}

fn tab_index(tab: DxForgePanelTab) -> usize {
    match tab {
        DxForgePanelTab::Repository => 0,
        DxForgePanelTab::Packages => 1,
        DxForgePanelTab::Media => 2,
        DxForgePanelTab::Remotes => 3,
    }
}

fn tab_icon(tab: DxForgePanelTab) -> IconName {
    match tab {
        DxForgePanelTab::Repository => IconName::GitBranch,
        DxForgePanelTab::Packages => IconName::Box,
        DxForgePanelTab::Media => dx_icon(DxUiIcon::Media),
        DxForgePanelTab::Remotes => IconName::CloudDownload,
    }
}
