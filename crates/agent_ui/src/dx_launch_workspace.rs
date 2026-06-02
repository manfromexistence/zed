use gpui::{
    AnyElement, App, Context, DismissEvent, EventEmitter, FocusHandle, Focusable, Render,
    SharedString, Window, prelude::*,
};
use ui::{IconName, PopoverMenu, Tooltip, prelude::*};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;
use crate::dx_check_score::DxCheckScoreSnapshot;
use crate::dx_deploy_rail::deploy_target_state;
use crate::dx_deploy_targets::DxDeployTargetSnapshot;
use crate::dx_launch_audit::DxLaunchAuditSnapshot;
use crate::dx_launch_binary_cache::DxBinaryCacheSnapshot;
use crate::dx_launch_contracts::DxLaunchContractSnapshot;
use crate::dx_launch_readiness::DxLaunchReadinessSnapshot;
use crate::dx_launch_receipts::DxLaunchReceiptReviewSnapshot;
use crate::dx_launch_source_audit::DxLaunchSourceAuditSnapshot;
use crate::dx_launch_status::DxLaunchStatusSnapshot;
use crate::dx_proof_freshness::DxProofFreshnessSnapshot;
use crate::dx_receipt_history::DxToolHistorySnapshot;
use crate::dx_receipts::DxReceiptSnapshot;
use crate::dx_runtime_proof_status::DxRuntimeProofStatusSnapshot;
use crate::dx_source_sets::DxSourceSetSnapshot;
use crate::dx_style_panel::DxStylePanelSnapshot;
use crate::dx_www_launch_evidence::DxWwwLaunchEvidenceSnapshot;

mod agents;
mod audit;
mod binary_cache;
mod binary_cache_labels;
mod check;
mod check_labels;
mod contracts;
mod launch_receipts;
mod launch_status;
mod launch_status_labels;
mod list_labels;
mod proof;
mod proof_labels;
mod readiness;
mod source_audit;
mod sources;
mod style_panel;
mod tool_history;
mod www_evidence;
use self::list_labels::{bounded_items, yes_no};

#[derive(Clone)]
pub(crate) struct DxLaunchWorkspaceStatus {
    pub active_status: SharedString,
    pub background_task_count: usize,
    pub visible_worktree_count: usize,
    pub agent_bridge: DxAgentBridgeSnapshot,
    pub launch_status: DxLaunchStatusSnapshot,
    pub launch_receipts: DxLaunchReceiptReviewSnapshot,
    pub launch_contracts: DxLaunchContractSnapshot,
    pub launch_readiness: DxLaunchReadinessSnapshot,
    pub launch_audit: DxLaunchAuditSnapshot,
    pub source_audit: DxLaunchSourceAuditSnapshot,
    pub www_evidence: DxWwwLaunchEvidenceSnapshot,
    pub binary_cache: DxBinaryCacheSnapshot,
    pub receipt_snapshot: DxReceiptSnapshot,
    pub source_sets: DxSourceSetSnapshot,
    pub tool_history: DxToolHistorySnapshot,
    pub check_score: DxCheckScoreSnapshot,
    pub deploy_targets: DxDeployTargetSnapshot,
    pub proof_freshness: DxProofFreshnessSnapshot,
    pub runtime_proof_status: DxRuntimeProofStatusSnapshot,
    pub style_panel: DxStylePanelSnapshot,
}

pub(crate) struct DxSourceRowControl {
    pub source_path: String,
    pub element: AnyElement,
}

struct DxLaunchDiagnosticsMenu {
    status: DxLaunchWorkspaceStatus,
    focus_handle: FocusHandle,
}

impl Focusable for DxLaunchDiagnosticsMenu {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for DxLaunchDiagnosticsMenu {}

impl Render for DxLaunchDiagnosticsMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source_summary = self.status.source_sets.attachment_summary();

        v_flex()
            .id("dx-launch-diagnostics-menu")
            .track_focus(&self.focus_handle)
            .w(px(388.0))
            .max_h_64()
            .overflow_y_scroll()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().elevated_surface_background)
            .p_2()
            .child(section_title("Launch Status", IconName::Check))
            .child(launch_status::launch_status_state(
                &self.status.launch_status,
                cx,
            ))
            .child(section_title("Launch Handoff", IconName::ListTodo))
            .child(contracts::launch_contract_state(
                &self.status.launch_contracts,
                cx,
            ))
            .child(section_title("Launch Gate", IconName::TodoProgress))
            .child(readiness::launch_readiness_state(
                &self.status.launch_readiness,
                cx,
            ))
            .child(section_title("Launch Audit", IconName::Sparkle))
            .child(audit::launch_audit_state(&self.status.launch_audit, cx))
            .child(section_title("Source Audit", IconName::Book))
            .child(source_audit::launch_source_audit_state(
                &self.status.source_audit,
                cx,
            ))
            .child(section_title("WWW Evidence", IconName::Public))
            .child(www_evidence::www_launch_evidence_state(
                &self.status.www_evidence,
                cx,
            ))
            .child(section_title("Launch Receipts", IconName::FileTextOutlined))
            .child(launch_receipts::launch_receipt_review_state(
                &self.status.launch_receipts,
                cx,
            ))
            .child(section_title("Binary Cache", IconName::Sliders))
            .child(binary_cache::binary_cache_state(
                &self.status.binary_cache,
                cx,
            ))
            .child(section_title("Agent Social", IconName::Link))
            .child(agents::dx_agent_social_state(&self.status.agent_bridge, cx))
            .child(section_title("Agent Receipts", IconName::FileTextOutlined))
            .child(agents::dx_agent_receipt_state(
                &self.status.agent_bridge,
                cx,
            ))
            .child(section_title("Agent Providers", IconName::Server))
            .child(agents::dx_agent_provider_state(
                &self.status.agent_bridge,
                cx,
            ))
            .child(section_title("Attach", IconName::Paperclip))
            .child(sources::source_attachment_state(&source_summary, cx))
            .child(section_title("Receipts", IconName::FileTextOutlined))
            .child(sources::receipt_source_state(
                &self.status.receipt_snapshot,
                cx,
            ))
            .child(section_title("Tool History", IconName::Archive))
            .child(tool_history::tool_history_state(
                &self.status.tool_history,
                cx,
            ))
    }
}

pub(crate) fn render_workspace_chrome(
    center: AnyElement,
    sidebar_actions: AnyElement,
    source_row_controls: Vec<DxSourceRowControl>,
    source_actions: AnyElement,
    guided_cards: AnyElement,
    show_sources_rail: bool,
    show_progress_rail: bool,
    status: DxLaunchWorkspaceStatus,
    cx: &mut App,
) -> AnyElement {
    div()
        .id("dx-launch-workspace")
        .relative()
        .size_full()
        .min_w_0()
        .overflow_hidden()
        .bg(cx.theme().colors().panel_background)
        .child(div().size_full().min_w_0().overflow_hidden().child(center))
        .when(show_sources_rail, |this| {
            this.child(render_sources_rail(
                sidebar_actions,
                source_row_controls,
                source_actions,
                &status,
                cx,
            ))
        })
        .when(show_progress_rail, |this| {
            this.child(render_right_rail(&status, guided_cards, cx))
        })
        .child(render_response_controller(&status, cx))
        .into_any_element()
}

fn render_response_controller(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let source_summary = status.source_sets.attachment_summary();
    div()
        .id("dx-response-controller-layer")
        .absolute()
        .top_1()
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            h_flex()
                .id("dx-response-controller")
                .gap_1()
                .px_2()
                .py_1()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().colors().border)
                .bg(cx.theme().colors().elevated_surface_background)
                .shadow_sm()
                .occlude()
                .child(response_controller_pill(
                    IconName::Chat,
                    status.active_status.clone(),
                ))
                .child(response_controller_tick(cx))
                .child(response_controller_pill(
                    IconName::Book,
                    format!("{} sources", status.source_sets.total_sources),
                ))
                .child(response_controller_tick(cx))
                .child(response_controller_pill(
                    IconName::Paperclip,
                    format!("{} ready", source_summary.attachable_sources),
                ))
                .child(response_controller_tick(cx))
                .child(response_controller_pill(
                    IconName::Clock,
                    format!("{} tasks", status.background_task_count),
                ))
                .child(response_controller_tick(cx))
                .child(response_controller_pill(
                    IconName::Sliders,
                    status.style_panel.readiness.status.clone(),
                )),
        )
        .into_any_element()
}

fn response_controller_pill(icon: IconName, label: impl Into<SharedString>) -> AnyElement {
    h_flex()
        .min_w_0()
        .gap_1()
        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn response_controller_tick(cx: &App) -> AnyElement {
    div()
        .w_px()
        .h_3()
        .bg(cx.theme().colors().border_variant)
        .into_any_element()
}

fn render_sources_rail(
    sidebar_actions: AnyElement,
    source_row_controls: Vec<DxSourceRowControl>,
    source_actions: AnyElement,
    status: &DxLaunchWorkspaceStatus,
    cx: &mut App,
) -> AnyElement {
    v_flex()
        .id("dx-sources-rail")
        .absolute()
        .left_2()
        .top_2()
        .bottom_2()
        .w(px(246.0))
        .gap_2()
        .p_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().elevated_surface_background)
        .shadow_md()
        .overflow_y_scroll()
        .occlude()
        .child(section_title("Agent", IconName::ZedAgent))
        .child(sidebar_actions)
        .child(section_title("Sources", IconName::Book))
        .child(sources::source_set_stack(
            &status.source_sets,
            source_row_controls,
            cx,
        ))
        .child(section_title("Source Tools", IconName::Paperclip))
        .child(source_actions)
        .child(section_title("Workspace State", IconName::Library))
        .child(workspace_mode_state(status, cx))
        .into_any_element()
}

fn workspace_mode_state(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let source_summary = status.source_sets.attachment_summary();
    let agent_state = if status.agent_bridge.enabled {
        status.agent_bridge.status.clone()
    } else {
        "disabled".to_string()
    };

    v_flex()
        .gap_1()
        .child(workspace_mode_row(
            "Chat",
            IconName::NewThread,
            status.active_status.clone(),
            "Current Agent panel conversation state",
            cx,
        ))
        .child(workspace_mode_row(
            "Tasks",
            IconName::Clock,
            format!("{} retained", status.background_task_count),
            "Background Agent work visible in the right rail",
            cx,
        ))
        .child(workspace_mode_row(
            "Sources",
            IconName::Book,
            format!("{} total", status.source_sets.total_sources),
            format!(
                "{} attach-ready, {} managed receipt(s)",
                source_summary.attachable_sources, source_summary.managed_receipts
            ),
            cx,
        ))
        .child(workspace_mode_row(
            "Agent",
            IconName::ZedAgent,
            agent_state,
            format!(
                "{} automation(s), {} active task(s)",
                status.agent_bridge.automation_count, status.agent_bridge.active_task_count
            ),
            cx,
        ))
        .into_any_element()
}

fn workspace_mode_row(
    label: &'static str,
    icon: IconName,
    state: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    cx: &App,
) -> AnyElement {
    v_flex()
        .min_w_0()
        .gap_0p5()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .child(
            h_flex()
                .justify_between()
                .gap_2()
                .min_w_0()
                .child(
                    h_flex()
                        .gap_1()
                        .min_w_0()
                        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
                        .child(
                            Label::new(label)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
                .child(
                    Label::new(state.into())
                        .size(LabelSize::XSmall)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .child(
            Label::new(detail.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn render_right_rail(
    status: &DxLaunchWorkspaceStatus,
    guided_cards: AnyElement,
    cx: &mut App,
) -> AnyElement {
    v_flex()
        .id("dx-progress-rail")
        .absolute()
        .right_0()
        .top_2()
        .bottom_2()
        .w(px(284.0))
        .gap_2()
        .p_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().elevated_surface_background)
        .shadow_md()
        .overflow_y_scroll()
        .occlude()
        .child(section_title("Progress", IconName::TodoProgress))
        .child(progress_summary(status, cx))
        .child(diagnostics_menu(status.clone()))
        .child(section_title("Guided Actions", IconName::Sparkle))
        .child(guided_cards)
        .child(section_title("Style", IconName::Sliders))
        .child(style_panel::dx_style_panel_state(&status.style_panel, cx))
        .child(section_title("Check", IconName::Check))
        .child(check::check_score_state(&status.check_score, cx))
        .child(section_title("Deploy", IconName::Public))
        .child(deploy_target_state(&status.deploy_targets, cx))
        .when(status.agent_bridge.show_in_agent_rail, |this| {
            this.child(section_title("DX Agents", IconName::ZedAgent))
                .child(agents::dx_agent_bridge_state(&status.agent_bridge, cx))
                .child(section_title("Automations", IconName::ListTodo))
                .child(agents::dx_agent_automation_state(&status.agent_bridge, cx))
        })
        .child(section_title("Proof", IconName::FileTextOutlined))
        .child(proof::proof_freshness_state(&status.proof_freshness, cx))
        .child(proof::runtime_proof_status_state(
            &status.runtime_proof_status,
            cx,
        ))
        .into_any_element()
}

fn diagnostics_menu(status: DxLaunchWorkspaceStatus) -> AnyElement {
    PopoverMenu::new("dx-launch-diagnostics-trigger")
        .trigger_with_tooltip(
            Button::new("dx-launch-diagnostics-button", "Diagnostics")
                .full_width()
                .label_size(LabelSize::Small)
                .color(Color::Muted)
                .start_icon(Icon::new(IconName::Sliders).size(IconSize::Small)),
            Tooltip::text("Open DX diagnostics"),
        )
        .menu(move |_window, cx| {
            let status = status.clone();
            Some(cx.new(|cx| DxLaunchDiagnosticsMenu {
                status,
                focus_handle: cx.focus_handle(),
            }))
        })
        .into_any_element()
}

fn progress_summary(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let source_summary = status.source_sets.attachment_summary();
    v_flex()
        .gap_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .p_2()
        .bg(cx.theme().colors().element_background)
        .child(metric_row("Thread", status.active_status.clone()))
        .child(metric_row(
            "Background",
            format!("{} task(s)", status.background_task_count),
        ))
        .child(metric_row(
            "Sources",
            format!(
                "{} attach-ready / {} receipt(s)",
                source_summary.attachable_sources, source_summary.managed_receipts
            ),
        ))
        .child(metric_row(
            "Worktrees",
            status.visible_worktree_count.to_string(),
        ))
        .into_any_element()
}

fn signal_row(
    id: SharedString,
    icon: IconName,
    color: Color,
    label: impl Into<SharedString>,
) -> AnyElement {
    h_flex()
        .id(id)
        .gap_1()
        .min_w_0()
        .child(Icon::new(icon).size(IconSize::XSmall).color(color))
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(color)
                .truncate(),
        )
        .into_any_element()
}

fn section_title(label: &'static str, icon: IconName) -> AnyElement {
    h_flex()
        .gap_1()
        .items_center()
        .pt_1()
        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(
            Label::new(label)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .into_any_element()
}

fn source_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    cx: &App,
) -> AnyElement {
    h_flex()
        .id(id)
        .gap_1()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .bg(cx.theme().colors().element_background)
        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn metric_row(label: impl Into<SharedString>, value: impl Into<SharedString>) -> AnyElement {
    h_flex()
        .justify_between()
        .gap_2()
        .min_w_0()
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .child(
            Label::new(value.into())
                .size(LabelSize::XSmall)
                .color(Color::Default)
                .truncate(),
        )
        .into_any_element()
}

fn muted_card(label: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .w_full()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .px_2()
        .py_1()
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}
