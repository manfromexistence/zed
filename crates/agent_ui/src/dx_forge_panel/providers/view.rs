use gpui::{AnyElement, App, MouseButton, SharedString, WeakEntity, rems};
use ui::{
    ButtonStyle, IconButtonShape, IconName, Indicator, ListItem, ListItemSpacing, Tooltip,
    prelude::*,
};
use workspace::Workspace;

use super::{
    catalog::{ForgeProvider, ProviderGroup, providers_for},
    state::{RemoteTargetState, provider_target_state, remote_target_state},
    tooltips::{provider_tooltip_meta, remote_target_tooltip},
};
use crate::dx_forge_panel::{
    controls::{exact_abs_path, open_exact_abs_path},
    panel::DxForgePanel,
    snapshot::DxForgePanelSnapshot,
    visible_rows::remote_target_item_key,
    workflow_rows::{row_scroll_anchor, selection_checkbox},
};

pub(in crate::dx_forge_panel) fn remote_target_strip(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    let mut strip = v_flex()
        .id("dx-forge-remote-targets")
        .w_full()
        .min_w_0()
        .border_b_1()
        .border_color(cx.theme().colors().border);

    for group in ProviderGroup::ALL {
        strip = strip.child(provider_group_controls(
            group, snapshot, workspace, panel, cx,
        ));
    }

    strip.into_any_element()
}

fn provider_target_button(
    provider: &'static ForgeProvider,
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    _cx: &App,
) -> AnyElement {
    debug_assert!(!provider.source_pack.is_empty());
    debug_assert!(!provider.source_slug.is_empty());

    let state = provider_target_state(provider, snapshot);
    let target_path = target_path_for_provider(provider, snapshot).map(String::from);
    let local_path = target_open_path_for_provider(provider, snapshot).and_then(exact_abs_path);
    let enabled = local_path.as_ref().is_some_and(|path| path.exists());
    let title = SharedString::from(provider.label);
    let meta = provider_tooltip_meta(provider, snapshot, &state, target_path.as_deref(), enabled);

    IconButton::new(format!("dx-forge-provider-{}", provider.id), provider.icon)
        .shape(IconButtonShape::Square)
        .icon_size(IconSize::Small)
        .icon_color(Color::Muted)
        .indicator(provider_target_indicator(&state))
        .style(ButtonStyle::Transparent)
        .tab_index(0_isize)
        .disabled(!enabled)
        .tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
        .on_click({
            let workspace = workspace.clone();
            move |_, window, cx| {
                cx.stop_propagation();
                if let Some(path) = local_path.clone().filter(|path| path.exists()) {
                    open_exact_abs_path(workspace.clone(), path, window, cx);
                }
            }
        })
        .into_any_element()
}

fn provider_group_controls(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    let state = remote_target_state(group, snapshot);
    let target_path = target_path_for_group(group, snapshot).map(String::from);
    let local_path = target_open_path_for_group(group, snapshot).and_then(exact_abs_path);
    let enabled = local_path.as_ref().is_some_and(|path| path.exists());
    let tooltip_title = SharedString::from(group.title());
    let tooltip_meta = remote_target_tooltip(group, &state, target_path.as_deref(), enabled);
    let open_title = SharedString::from(format!("Open {}", group.title()));
    let item_key = remote_target_item_key(group.key());
    let scroll_anchor = row_scroll_anchor(panel, &item_key, cx);
    let checked = panel
        .upgrade()
        .is_some_and(|panel| panel.read(cx).item_checked(&item_key));
    let active = panel
        .upgrade()
        .is_some_and(|panel| panel.read(cx).item_active(&item_key));
    let row_key = item_key.clone();
    let panel_for_row = panel.clone();
    let checkbox_id = SharedString::from(format!("remote-{}", group.key()));
    let hover_checkbox_id = SharedString::from(format!("remote-{}-hover", group.key()));
    let checkbox = selection_checkbox(checkbox_id, item_key.clone(), checked, panel);
    let hover_checkbox = selection_checkbox(hover_checkbox_id, item_key, checked, panel);
    let open_button = IconButton::new(
        format!("dx-forge-open-provider-group-{}", group.key()),
        IconName::ArrowUpRight,
    )
    .shape(IconButtonShape::Square)
    .icon_size(IconSize::Small)
    .icon_color(Color::Muted)
    .style(ButtonStyle::Subtle)
    .tab_index(0_isize)
    .disabled(!enabled)
    .tooltip({
        let title = open_title.clone();
        let meta = tooltip_meta.clone();
        move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx)
    })
    .on_click({
        let workspace = workspace.clone();
        move |_, window, cx| {
            cx.stop_propagation();
            if let Some(path) = local_path.clone().filter(|path| path.exists()) {
                open_exact_abs_path(workspace.clone(), path, window, cx);
            }
        }
    });

    ListItem::new(SharedString::from(format!(
        "dx-forge-provider-group-{}",
        group.key()
    )))
    .anchor_scroll(scroll_anchor)
    .inset(true)
    .height(rems(1.75))
    .spacing(ListItemSpacing::Sparse)
    .toggle_state(active)
    .start_slot(
        Icon::new(group_icon(group))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .child(
        h_flex().w_full().min_w_0().gap_1p5().child(
            Label::new(group.title())
                .size(LabelSize::Small)
                .truncate()
                .flex_1(),
        ),
    )
    .end_slot(
        h_flex()
            .flex_none()
            .gap_1()
            .child(Indicator::dot().color(state.color))
            .child(checkbox),
    )
    .end_slot_on_hover(provider_group_actions(
        provider_buttons_for_group(group, snapshot, workspace, cx),
        open_button.into_any_element(),
        hover_checkbox,
        state.color,
    ))
    .tooltip(move |_, cx| Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx))
    .on_click(move |_, window, cx| {
        panel_for_row
            .update(cx, |panel, cx| {
                panel.focus_panel(window, cx);
                panel.activate_item(row_key.clone(), cx)
            })
            .ok();
    })
    .into_any_element()
}

fn provider_buttons_for_group(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    h_flex()
        .id(SharedString::from(format!(
            "dx-forge-provider-targets-{}",
            group.key()
        )))
        .flex_none()
        .gap_0p5()
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .children(
            providers_for(group)
                .map(|provider| provider_target_button(provider, snapshot, workspace, cx)),
        )
        .into_any_element()
}

fn provider_group_actions(
    provider_buttons: AnyElement,
    open_button: AnyElement,
    selection_checkbox: AnyElement,
    status_color: Color,
) -> AnyElement {
    h_flex()
        .flex_none()
        .gap_0p5()
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(provider_buttons)
        .child(Indicator::dot().color(status_color))
        .child(open_button)
        .child(selection_checkbox)
        .into_any_element()
}

fn target_path_for_group<'a>(
    group: ProviderGroup,
    snapshot: &'a DxForgePanelSnapshot,
) -> Option<&'a str> {
    match group {
        ProviderGroup::Code => snapshot.history_root_path.as_deref(),
        ProviderGroup::Storage => snapshot
            .restore_previews
            .first()
            .map(|preview| preview.path.as_str()),
        ProviderGroup::Media => snapshot
            .media_outputs
            .first()
            .map(|output| output.path.as_str()),
    }
}

fn target_open_path_for_group<'a>(
    group: ProviderGroup,
    snapshot: &'a DxForgePanelSnapshot,
) -> Option<&'a str> {
    match group {
        ProviderGroup::Code => snapshot.history_root_path.as_deref(),
        ProviderGroup::Storage => snapshot
            .restore_previews
            .first()
            .map(|preview| preview.open_path.as_str()),
        ProviderGroup::Media => snapshot
            .media_outputs
            .first()
            .map(|output| output.open_path.as_str()),
    }
}

fn target_path_for_provider<'a>(
    provider: &ForgeProvider,
    snapshot: &'a DxForgePanelSnapshot,
) -> Option<&'a str> {
    snapshot
        .remote_provider_for(provider.id)
        .map(|remote| remote.registry_path.as_str())
}

fn target_open_path_for_provider<'a>(
    provider: &ForgeProvider,
    snapshot: &'a DxForgePanelSnapshot,
) -> Option<&'a str> {
    snapshot
        .remote_provider_for(provider.id)
        .map(|remote| remote.registry_open_path.as_str())
}

fn provider_target_indicator(state: &RemoteTargetState) -> Indicator {
    match state.color {
        Color::Success | Color::Warning => Indicator::dot().color(state.color),
        _ => Indicator::dot().color(Color::Muted),
    }
}

fn group_icon(group: ProviderGroup) -> IconName {
    match group {
        ProviderGroup::Code => IconName::GitBranch,
        ProviderGroup::Storage => IconName::CloudDownload,
        ProviderGroup::Media => dx_icon(DxUiIcon::Media),
    }
}
