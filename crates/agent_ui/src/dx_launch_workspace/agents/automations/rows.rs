use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::{Color, prelude::*};

use crate::dx_agent_bridge::DxAgentAutomation;

use super::super::super::metric_row;
use super::super::actions::dx_agent_action_line;
use super::labels::{
    automation_destination_label, automation_history_label, automation_receipt_label,
    automation_schedule_label,
};

pub(super) fn dx_agent_automation_row(
    id: SharedString,
    automation: &DxAgentAutomation,
    cx: &App,
) -> AnyElement {
    let state = if automation.status.enabled && automation.status.runtime_available {
        automation.status.state.clone()
    } else if automation.status.enabled {
        "runtime pending".to_string()
    } else {
        "paused".to_string()
    };
    let schedule = automation_schedule_label(automation);
    let destination = automation_destination_label(automation);
    let next_label = if automation.status.runtime_available {
        "next"
    } else {
        "requested next"
    };
    let run_window = format!(
        "last {} / {next_label} {}",
        automation.last_run, automation.next_run
    );
    let receipt_summary = automation_receipt_label(automation);
    let history_summary = automation_history_label(automation);

    v_flex()
        .id(id)
        .gap_0p5()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .child(metric_row(automation.name.clone(), state))
        .child(
            Label::new(format!("{schedule} -> {destination}"))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .when(!automation.prompt.is_empty(), |this| {
            this.child(
                Label::new(format!("Prompt: {}", automation.prompt))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .child(
            Label::new(run_window)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .when(!automation.status.runtime_available, |this| {
            this.child(
                Label::new(format!("Runtime: {}", automation.status.unavailable_reason))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when(!receipt_summary.is_empty(), |this| {
            this.child(
                Label::new(receipt_summary)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when(!history_summary.is_empty(), |this| {
            this.child(
                Label::new(history_summary)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when(!automation.next_action.is_empty(), |this| {
            this.child(
                Label::new(automation.next_action.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .when_some(
            dx_agent_action_line(&automation.actions),
            |this, action_line| {
                this.child(
                    Label::new(action_line)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                )
            },
        )
        .into_any_element()
}
