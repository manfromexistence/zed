use gpui::{AnyElement, App, prelude::*};
use ui::{Color, prelude::*};

use crate::dx_agent_bridge::DxAgentAutomationComposer;

use super::super::super::{metric_row, muted_card};

pub(super) fn dx_agent_automation_composer_contract(
    composer: &DxAgentAutomationComposer,
    cx: &App,
) -> AnyElement {
    let action_state = if composer.save_draft_available || composer.enable_available {
        "available"
    } else {
        "pending runtime"
    };

    let field_summary = composer
        .fields
        .iter()
        .take(4)
        .map(|field| {
            if field.required {
                format!("{}*", field.label)
            } else {
                field.label.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    v_flex()
        .gap_0p5()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .child(metric_row("Composer", composer.status.clone()))
        .child(metric_row("Runtime", action_state.to_string()))
        .child(metric_row("Receipt", composer.receipt_filename.clone()))
        .when(!field_summary.is_empty(), |this| {
            this.child(
                Label::new(format!("Fields: {field_summary}"))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .truncate(),
            )
        })
        .child(muted_card(composer.unavailable_reason.clone(), cx))
        .into_any_element()
}
