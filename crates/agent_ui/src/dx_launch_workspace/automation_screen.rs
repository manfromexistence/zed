use gpui::{AnyElement, App, IntoElement};
use ui::{IconName, prelude::*};

use super::{DxLaunchWorkspaceStatus, muted_card, section_title};

mod sections;

pub(crate) fn render_automation_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    cx: &mut App,
) -> AnyElement {
    let body = if let Some(status) = status {
        v_flex()
            .gap_2()
            .child(section_title("Drafts", dx_icon(DxUiIcon::Automations)))
            .child(sections::drafts_state(&status.agent_bridge, cx))
            .child(section_title("Schedules", IconName::Clock))
            .child(sections::schedules_state(&status.agent_bridge, cx))
            .child(section_title("Runs", IconName::TodoProgress))
            .child(sections::runs_state(&status.agent_bridge, cx))
            .child(section_title("History", IconName::HistoryRerun))
            .child(sections::history_state(&status.agent_bridge, cx))
            .child(section_title("Failures", IconName::Warning))
            .child(sections::failures_state(&status.agent_bridge, cx))
            .into_any_element()
    } else {
        muted_card("Loading automation receipts and composer contract", cx)
    };

    div()
        .id("dx-automation-screen")
        .size_full()
        .min_w_0()
        .overflow_y_scroll()
        .bg(cx.theme().colors().panel_background)
        .child(
            v_flex()
                .gap_3()
                .p_4()
                .child(
                    h_flex()
                        .items_start()
                        .gap_2()
                        .child(
                            Icon::new(dx_icon(DxUiIcon::Automations))
                                .size(IconSize::Medium)
                                .color(Color::Muted),
                        )
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    Label::new("Automations")
                                        .size(LabelSize::Default)
                                        .color(Color::Default),
                                )
                                .child(
                                    Label::new(
                                        "Composer receipt state, schedule contracts, history, and handoff evidence.",
                                    )
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                                ),
                        ),
                )
                .child(body),
        )
        .into_any_element()
}
