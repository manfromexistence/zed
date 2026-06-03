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
            "Generators",
            format!("{} declared", snapshot.visual_generator_count),
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
                "Open Style Controls",
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
        .child(metric("Apply", apply_gate_state_label(&gate.state)))
        .child(metric("Gate", apply_gate_reason_label(&gate.state)))
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
            Label::new(snapshot.readiness.next_action.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn web_preview_state(snapshot: &DxStylePanelSnapshot) -> String {
    if snapshot.web_preview_bridge_ready {
        "Ready".to_string()
    } else if snapshot.web_preview_host_present {
        "Bridge available".to_string()
    } else {
        "Bridge missing".to_string()
    }
}

fn apply_gate_state_label(state: &str) -> &'static str {
    match state {
        "ready_for_explicit_apply" => "Ready for review",
        "needs_static_style_token" => "Needs class token",
        "needs_active_source_digest" => "Needs source digest",
        "needs_trusted_dry_run_receipt" => "Needs dry-run receipt",
        "needs_matching_active_source_receipt" => "Needs matching receipt",
        "needs_editor_write_bridge" => "Needs editor bridge",
        _ => "Review required",
    }
}

fn apply_gate_reason_label(state: &str) -> &'static str {
    match state {
        "ready_for_explicit_apply" => "Trusted review is ready",
        "needs_static_style_token" => "Place the cursor on a class token.",
        "needs_active_source_digest" => "Source digest is unavailable.",
        "needs_trusted_dry_run_receipt" => "Generate a trusted dry-run receipt.",
        "needs_matching_active_source_receipt" => "Receipt does not match this source span.",
        "needs_editor_write_bridge" => "Editor write bridge is gated.",
        _ => "Review the source apply gate.",
    }
}
