mod catalog;
mod state;

use gpui::{AnyElement, App, SharedString, px};
use ui::{IconName, Tooltip, prelude::*};

use self::{
    catalog::{ProviderGroup, providers_for},
    state::{RemoteTargetState, remote_target_state},
};
use super::snapshot::DxForgePanelSnapshot;

pub(super) fn remote_target_strip(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
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

    for group in ProviderGroup::ALL {
        strip = strip.child(remote_target_row(group, snapshot, cx));
    }

    strip.into_any_element()
}

fn remote_target_row(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    cx: &App,
) -> AnyElement {
    let state = remote_target_state(group, snapshot);
    let tooltip_title = SharedString::from(group.title());
    let tooltip_meta = remote_target_tooltip(group, &state);

    h_flex()
        .id(SharedString::from(format!(
            "dx-forge-provider-group-{}",
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
        .child(provider_icon_stack(group))
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
                        .child(
                            Label::new(group.providers_label())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                )
                .child(
                    Label::new(state.detail.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .child(
            h_flex()
                .gap_0p5()
                .flex_none()
                .child(
                    Icon::new(state.icon)
                        .size(IconSize::XSmall)
                        .color(state.color),
                )
                .child(
                    Label::new(state.label)
                        .size(LabelSize::XSmall)
                        .color(state.color),
                ),
        )
        .tooltip(move |_, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
        .into_any_element()
}

fn provider_icon_stack(group: ProviderGroup) -> AnyElement {
    let mut icons = h_flex().gap_0p5().flex_none();

    for provider in providers_for(group) {
        icons = icons.child(
            div()
                .id(SharedString::from(format!(
                    "dx-forge-provider-icon-{}",
                    provider.id
                )))
                .child(Icon::new(provider.icon).size(IconSize::Small)),
        );
    }

    icons.into_any_element()
}

fn remote_target_tooltip(group: ProviderGroup, state: &RemoteTargetState) -> String {
    format!(
        "{}\n{}\nProviders: {}",
        state.label,
        state.detail,
        group.provider_sources(),
    )
}
