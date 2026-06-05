use gpui::{AnyElement, App, SharedString, WeakEntity, px};
use ui::{ButtonStyle, IconButtonShape, IconName, TintColor, Tooltip, prelude::*};
use workspace::Workspace;

use super::{
    catalog::{ForgeProvider, PROVIDERS, ProviderGroup, providers_for},
    state::{RemoteTargetState, remote_target_state},
};
use crate::dx_forge_panel::{
    controls::{open_workspace_path, workspace_path},
    snapshot::DxForgePanelSnapshot,
};

pub(super) fn remote_target_strip(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let mut strip = v_flex()
        .id("dx-forge-remote-targets")
        .w_full()
        .min_w_0()
        .gap_1()
        .px_2()
        .py_1()
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(
            h_flex()
                .h(px(22.0))
                .w_full()
                .min_w_0()
                .gap_1()
                .child(Icon::new(IconName::CloudDownload).size(IconSize::XSmall))
                .child(
                    Label::new("Remote targets")
                        .size(LabelSize::Small)
                        .truncate(),
                )
                .child(div().flex_1())
                .child(
                    Label::new(format!("{} lanes", ProviderGroup::ALL.len()))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
        );

    strip = strip.child(provider_target_deck(snapshot, workspace, cx));

    for group in ProviderGroup::ALL {
        strip = strip.child(remote_lane_row(group, snapshot, workspace, cx));
    }

    strip.into_any_element()
}

fn provider_target_deck(
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    h_flex()
        .id("dx-forge-provider-target-deck")
        .w_full()
        .min_w_0()
        .gap_1()
        .flex_wrap()
        .px_2()
        .py_1()
        .children(
            PROVIDERS
                .iter()
                .map(|provider| provider_target_button(provider, snapshot, workspace, cx)),
        )
        .into_any_element()
}

fn provider_target_button(
    provider: &'static ForgeProvider,
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let state = remote_target_state(provider.group, snapshot);
    let target_path = target_path_for_group(provider.group, snapshot).map(String::from);
    let local_path = target_path
        .as_deref()
        .and_then(|path| workspace_path(path, &snapshot.workspace_roots));
    let enabled = local_path.as_ref().is_some_and(|path| path.exists());
    let title = SharedString::from(provider.label);
    let meta = provider_tooltip_meta(provider, &state, target_path.as_deref(), enabled);

    IconButton::new(format!("dx-forge-provider-{}", provider.id), provider.icon)
        .shape(IconButtonShape::Square)
        .icon_size(IconSize::Small)
        .icon_color(provider_icon_color(&state))
        .style(provider_button_style(&state))
        .disabled(!enabled)
        .tooltip(move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx))
        .on_click({
            let workspace = workspace.clone();
            move |_, window, cx| {
                if let Some(path) = local_path.clone().filter(|path| path.exists()) {
                    open_workspace_path(workspace.clone(), path, window, cx);
                }
            }
        })
        .into_any_element()
}

fn remote_lane_row(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    cx: &App,
) -> AnyElement {
    let state = remote_target_state(group, snapshot);
    let target_path = target_path_for_group(group, snapshot).map(String::from);
    let local_path = target_path
        .as_deref()
        .and_then(|path| workspace_path(path, &snapshot.workspace_roots));
    let enabled = local_path.as_ref().is_some_and(|path| path.exists());
    let tooltip_title = SharedString::from(group.title());
    let tooltip_meta = remote_target_tooltip(group, &state, target_path.as_deref(), enabled);

    h_flex()
        .id(SharedString::from(format!(
            "dx-forge-remote-lane-{}",
            group.key()
        )))
        .w_full()
        .min_w_0()
        .gap_2()
        .pl_2()
        .pr_1()
        .py_1()
        .border_1()
        .border_r_2()
        .bg(cx.theme().colors().ghost_element_background)
        .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
        .child(
            Icon::new(group_icon(group))
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_0p5()
                .child(
                    h_flex()
                        .min_w_0()
                        .gap_1()
                        .child(Label::new(group.title()).size(LabelSize::Small).truncate())
                        .children(providers_for(group).map(|provider| {
                            Icon::new(provider.icon)
                                .size(IconSize::XSmall)
                                .color(Color::Muted)
                                .into_any_element()
                        })),
                )
                .child(
                    Label::new(state.detail.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(
            h_flex().gap_0p5().flex_none().child(
                Icon::new(state.icon)
                    .size(IconSize::XSmall)
                    .color(state.color),
            ),
        )
        .child(
            IconButton::new(
                format!("dx-forge-open-lane-{}", group.key()),
                IconName::ArrowUpRight,
            )
            .shape(IconButtonShape::Square)
            .icon_size(IconSize::XSmall)
            .icon_color(Color::Muted)
            .disabled(!enabled)
            .tooltip({
                let title = tooltip_title.clone();
                let meta = tooltip_meta.clone();
                move |_, cx| Tooltip::with_meta(title.clone(), None, meta.clone(), cx)
            })
            .on_click({
                let workspace = workspace.clone();
                move |_, window, cx| {
                    if let Some(path) = local_path.clone().filter(|path| path.exists()) {
                        open_workspace_path(workspace.clone(), path, window, cx);
                    }
                }
            }),
        )
        .tooltip(move |_, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
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

fn provider_tooltip_meta(
    provider: &ForgeProvider,
    state: &RemoteTargetState,
    target_path: Option<&str>,
    enabled: bool,
) -> String {
    let mut lines = vec![
        format!("{} target - {}", provider.group.title(), state.label),
        state.detail.clone(),
    ];

    if let Some(path) = target_path {
        lines.push(format!("Local evidence: {path}"));
    } else {
        lines.push("Local evidence is not available yet".to_string());
    }

    if !enabled {
        lines.push("Open a workspace or generate Forge evidence to enable this target".to_string());
    }

    lines.join("\n")
}

fn remote_target_tooltip(
    group: ProviderGroup,
    state: &RemoteTargetState,
    target_path: Option<&str>,
    enabled: bool,
) -> String {
    let mut lines = vec![
        state.label.to_string(),
        state.detail.clone(),
        format!("Providers: {}", group.provider_labels()),
    ];

    if let Some(path) = target_path {
        lines.push(format!("Local evidence: {path}"));
    }

    if !enabled {
        lines.push("No local evidence path is available for this lane".to_string());
    }

    lines.join("\n")
}

fn provider_button_style(state: &RemoteTargetState) -> ButtonStyle {
    match state.color {
        Color::Success => ButtonStyle::Tinted(TintColor::Success),
        Color::Warning => ButtonStyle::Tinted(TintColor::Warning),
        _ => ButtonStyle::Subtle,
    }
}

fn provider_icon_color(state: &RemoteTargetState) -> Color {
    match state.color {
        Color::Success | Color::Warning => Color::Default,
        _ => Color::Muted,
    }
}

fn group_icon(group: ProviderGroup) -> IconName {
    match group {
        ProviderGroup::Code => IconName::GitBranch,
        ProviderGroup::Storage => IconName::CloudDownload,
        ProviderGroup::Media => IconName::Image,
    }
}
