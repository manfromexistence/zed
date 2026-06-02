use gpui::{Action, AnyElement, App, prelude::*};
use ui::{IconName, prelude::*};
use zed_actions::dx_style::OpenGeneratorPreviewForContext;

use super::{
    DxStylePanelSnapshot, active_context::ActiveStyleContextSnapshot, panel_metric::metric,
};

pub(super) fn generator_host_card(
    snapshot: &DxStylePanelSnapshot,
    source_context_json: String,
    can_open_generator: bool,
    cx: &App,
) -> AnyElement {
    v_flex()
        .id("dx-style-panel-generator-host")
        .gap_2()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .p_2()
        .bg(cx.theme().colors().element_background)
        .child(metric("Web Preview", web_preview_state(snapshot)))
        .child(metric(
            "Controls",
            format!("{} visual controls", snapshot.visual_generator_count),
        ))
        .child(metric("Readiness", snapshot.readiness.status.clone()))
        .child(
            Label::new(snapshot.readiness.summary.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .child(
            Button::new(
                "dx-style-panel-open-generator-preview",
                "Open Web Preview Controls",
            )
            .full_width()
            .label_size(LabelSize::Small)
            .color(Color::Muted)
            .start_icon(Icon::new(IconName::Sparkle).size(IconSize::Small))
            .disabled(!can_open_generator)
            .on_click(move |_, window, cx| {
                window.dispatch_action(
                    OpenGeneratorPreviewForContext {
                        source_context_json: source_context_json.clone(),
                    }
                    .boxed_clone(),
                    cx,
                );
            }),
        )
        .into_any_element()
}

pub(super) fn style_context_card(
    snapshot: &DxStylePanelSnapshot,
    active_context: &ActiveStyleContextSnapshot,
    cx: &App,
) -> AnyElement {
    let gate = &active_context.apply_gate;
    let active_style_target = active_context
        .css_property
        .clone()
        .or_else(|| active_context.token.clone())
        .or_else(|| active_context.group_context.summary())
        .unwrap_or_else(|| active_context.status.clone());

    v_flex()
        .id("dx-style-panel-context-card")
        .gap_1()
        .rounded_sm()
        .p_2()
        .bg(cx.theme().colors().element_background)
        .child(metric("Status", snapshot.status.clone()))
        .child(metric("Target", active_style_target))
        .when_some(active_context.css_generator.clone(), |this, generator| {
            this.child(metric("Generator", generator))
        })
        .when_some(active_context.span_byte_range(), |this, span| {
            this.child(metric("Span", span))
        })
        .child(metric("Apply", gate.state.clone()))
        .child(metric("Gate", gate.reason.clone()))
        .into_any_element()
}

pub(super) fn readiness_card(snapshot: &DxStylePanelSnapshot, cx: &App) -> AnyElement {
    v_flex()
        .id("dx-style-panel-readiness-card")
        .gap_1()
        .rounded_sm()
        .p_2()
        .bg(cx.theme().colors().element_background)
        .child(metric(
            "Docs",
            format!(
                "{}/{}",
                snapshot.readiness.docs_ready, snapshot.readiness.docs_expected
            ),
        ))
        .child(metric(
            "Contracts",
            format!(
                "{}/{}",
                snapshot.readiness.contracts_ready, snapshot.readiness.contracts_expected
            ),
        ))
        .child(metric(
            "Fixtures",
            format!(
                "{}/{}",
                snapshot.readiness.fixtures_ready, snapshot.readiness.fixtures_expected
            ),
        ))
        .child(metric(
            "Receipts",
            snapshot.readiness.receipt_count.to_string(),
        ))
        .child(
            Label::new(snapshot.next_action.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn web_preview_state(snapshot: &DxStylePanelSnapshot) -> String {
    if snapshot.web_preview_bridge_ready {
        "controls ready".to_string()
    } else if snapshot.web_preview_host_present {
        "host connected".to_string()
    } else {
        "host unavailable".to_string()
    }
}
