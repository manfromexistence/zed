use gpui::{AnyElement, App, IntoElement};
use ui::{IconName, prelude::*};

use super::{DxLaunchWorkspaceStatus, agents, metric_row, muted_card, section_title};

pub(crate) fn render_automation_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    cx: &mut App,
) -> AnyElement {
    let body = if let Some(status) = status {
        v_flex()
            .gap_2()
            .child(section_title("Runtime", IconName::Server))
            .child(
                v_flex()
                    .gap_1()
                    .child(metric_row(
                        "Bridge",
                        if status.agent_bridge.enabled {
                            status.agent_bridge.status.clone()
                        } else {
                            "disabled".to_string()
                        },
                    ))
                    .child(metric_row(
                        "Scheduled execution",
                        "pending DX Agents runtime".to_string(),
                    ))
                    .child(metric_row(
                        "Automation source",
                        "dx agents automate list --json".to_string(),
                    ))
                    .child(metric_row(
                        "Receipts",
                        status.agent_bridge.receipts.len().to_string(),
                    )),
            )
            .child(section_title("Composer", dx_icon(DxUiIcon::Automations)))
            .child(agents::dx_agent_automation_state(&status.agent_bridge, cx))
            .child(section_title("Receipts", IconName::FileTextOutlined))
            .child(agents::dx_agent_receipt_state(&status.agent_bridge, cx))
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
                                        "Receipt-backed composer, schedule contract, history, and handoff state.",
                                    )
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                                ),
                        ),
                )
                .child(body),
        )
        .into_any_element()
}
