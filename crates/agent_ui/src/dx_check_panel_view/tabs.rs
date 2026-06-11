use std::cmp::Ordering;

use gpui::{App, EntityId, IntoElement, WeakEntity};
use ui::{IconName, Tab, TabBar, TabPosition, Tooltip, prelude::*};

use super::DxCheckPanel;
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
    panel_id: EntityId,
    panel: WeakEntity<DxCheckPanel>,
    _cx: &App,
) -> impl IntoElement {
    TabBar::new(("dx-check-tab-bar", panel_id))
        .child(check_tab(
            "dx-check-tab-overview",
            "Overview",
            snapshot.sections.len(),
            DxCheckPanelTab::Overview,
            active_tab,
            panel.clone(),
        ))
        .child(check_tab(
            "dx-check-tab-findings",
            "Findings",
            snapshot.blockers.len()
                + snapshot.warnings.len()
                + snapshot.quick_fixes.len()
                + snapshot.web_audits.len()
                + snapshot.adapter_plans.len(),
            DxCheckPanelTab::Findings,
            active_tab,
            panel.clone(),
        ))
        .child(check_tab(
            "dx-check-tab-receipt",
            "Receipt",
            usize::from(snapshot.receipt_present),
            DxCheckPanelTab::Receipt,
            active_tab,
            panel,
        ))
}

fn check_tab(
    id: &'static str,
    label: &'static str,
    count: usize,
    tab: DxCheckPanelTab,
    active_tab: DxCheckPanelTab,
    panel: WeakEntity<DxCheckPanel>,
) -> impl IntoElement {
    let selected = active_tab == tab;
    let title = format!("{label} ({count})");

    let tab = Tab::new(id)
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
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            panel
                .update(cx, |panel, cx| panel.set_active_tab(tab, cx))
                .ok();
        });

    if count > 0 {
        tab.end_slot(
            Label::new(count.to_string())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
    } else {
        tab
    }
}

fn tab_position(tab: DxCheckPanelTab, active_tab: DxCheckPanelTab) -> TabPosition {
    let current_index = tab_index(tab);
    let active_index = tab_index(active_tab);

    match current_index {
        0 => TabPosition::First,
        2 => TabPosition::Last,
        _ if current_index == active_index => TabPosition::Middle(Ordering::Equal),
        _ if current_index < active_index => TabPosition::Middle(Ordering::Less),
        _ => TabPosition::Middle(Ordering::Greater),
    }
}

fn tab_index(tab: DxCheckPanelTab) -> usize {
    match tab {
        DxCheckPanelTab::Overview => 0,
        DxCheckPanelTab::Findings => 1,
        DxCheckPanelTab::Receipt => 2,
    }
}

fn tab_icon(tab: DxCheckPanelTab) -> IconName {
    match tab {
        DxCheckPanelTab::Overview => IconName::Check,
        DxCheckPanelTab::Findings => IconName::Warning,
        DxCheckPanelTab::Receipt => IconName::FileTextOutlined,
    }
}
