use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, DismissEvent, EventEmitter, FocusHandle, Focusable,
    Render, SharedString, Window, prelude::*,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DxLaunchRailSection {
    SourceCommands,
    SourceStack,
    SourceTools,
    WorkspaceState,
    Progress,
    Environment,
    Subagents,
    SourceSummary,
    Readiness,
}

#[derive(Clone, Copy)]
pub(crate) struct DxLaunchRailState {
    pub source_commands_open: bool,
    pub source_stack_open: bool,
    pub source_tools_open: bool,
    pub workspace_state_open: bool,
    pub progress_open: bool,
    pub environment_open: bool,
    pub subagents_open: bool,
    pub source_summary_open: bool,
    pub readiness_open: bool,
}

impl DxLaunchRailState {
    pub(crate) fn is_open(self, section: DxLaunchRailSection) -> bool {
        match section {
            DxLaunchRailSection::SourceCommands => self.source_commands_open,
            DxLaunchRailSection::SourceStack => self.source_stack_open,
            DxLaunchRailSection::SourceTools => self.source_tools_open,
            DxLaunchRailSection::WorkspaceState => self.workspace_state_open,
            DxLaunchRailSection::Progress => self.progress_open,
            DxLaunchRailSection::Environment => self.environment_open,
            DxLaunchRailSection::Subagents => self.subagents_open,
            DxLaunchRailSection::SourceSummary => self.source_summary_open,
            DxLaunchRailSection::Readiness => self.readiness_open,
        }
    }
}

#[derive(Clone)]
pub(crate) struct DxLaunchRailControls {
    pub state: DxLaunchRailState,
    pub on_toggle: Arc<dyn Fn(DxLaunchRailSection, &ClickEvent, &mut Window, &mut App) + 'static>,
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
    rail_controls: DxLaunchRailControls,
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
                &rail_controls,
                cx,
            ))
        })
        .when(show_progress_rail, |this| {
            this.child(render_right_rail(&status, guided_cards, &rail_controls, cx))
        })
        .into_any_element()
}

fn render_sources_rail(
    sidebar_actions: AnyElement,
    source_row_controls: Vec<DxSourceRowControl>,
    source_actions: AnyElement,
    status: &DxLaunchWorkspaceStatus,
    rail_controls: &DxLaunchRailControls,
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
        .child(rail_section(
            "dx-sources-commands-section",
            "Agent",
            IconName::ZedAgent,
            DxLaunchRailSection::SourceCommands,
            rail_controls,
            sidebar_actions,
            cx,
        ))
        .child(rail_section(
            "dx-sources-stack-section",
            "Sources",
            IconName::Book,
            DxLaunchRailSection::SourceStack,
            rail_controls,
            sources::source_set_stack(&status.source_sets, source_row_controls, cx),
            cx,
        ))
        .child(rail_section(
            "dx-sources-tools-section",
            "Tools",
            IconName::Paperclip,
            DxLaunchRailSection::SourceTools,
            rail_controls,
            source_actions,
            cx,
        ))
        .child(rail_section(
            "dx-workspace-state-section",
            "Workspace",
            IconName::Library,
            DxLaunchRailSection::WorkspaceState,
            rail_controls,
            workspace_mode_state(status, cx),
            cx,
        ))
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
            "Current thread",
            cx,
        ))
        .child(workspace_mode_row(
            "Tasks",
            IconName::Clock,
            format!("{} retained", status.background_task_count),
            "Retained threads",
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
    rail_controls: &DxLaunchRailControls,
    cx: &mut App,
) -> AnyElement {
    v_flex()
        .id("dx-progress-rail")
        .absolute()
        .right_2()
        .top_2()
        .bottom_2()
        .w(px(300.0))
        .gap_2()
        .p_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().elevated_surface_background)
        .shadow_md()
        .overflow_y_scroll()
        .occlude()
        .child(diagnostics_menu(status.clone()))
        .child(rail_section(
            "dx-progress-summary-section",
            "Progress",
            IconName::TodoProgress,
            DxLaunchRailSection::Progress,
            rail_controls,
            progress_summary(status, cx),
            cx,
        ))
        .child(rail_section(
            "dx-environment-section",
            "Environment",
            IconName::Settings,
            DxLaunchRailSection::Environment,
            rail_controls,
            environment_summary(status, cx),
            cx,
        ))
        .child(rail_section(
            "dx-subagents-section",
            "Subagents",
            IconName::ZedAgent,
            DxLaunchRailSection::Subagents,
            rail_controls,
            subagent_summary(status, cx),
            cx,
        ))
        .child(rail_section(
            "dx-source-summary-section",
            "Sources",
            IconName::Book,
            DxLaunchRailSection::SourceSummary,
            rail_controls,
            source_summary(status, cx),
            cx,
        ))
        .child(rail_section(
            "dx-readiness-section",
            "Readiness",
            IconName::Check,
            DxLaunchRailSection::Readiness,
            rail_controls,
            readiness_summary(status, guided_cards, cx),
            cx,
        ))
        .into_any_element()
}

fn rail_section(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    section: DxLaunchRailSection,
    controls: &DxLaunchRailControls,
    content: AnyElement,
    cx: &App,
) -> AnyElement {
    let is_open = controls.state.is_open(section);
    let on_toggle = controls.on_toggle.clone();

    v_flex()
        .id(id)
        .gap_1()
        .child(
            h_flex()
                .id(format!("{id}-header"))
                .items_center()
                .gap_1()
                .py_0p5()
                .child(
                    Disclosure::new(format!("{id}-disclosure"), is_open).on_click(
                        move |event, window, cx| {
                            on_toggle(section, event, window, cx);
                        },
                    ),
                )
                .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
                .child(div().flex_1()),
        )
        .when(is_open, |this| this.child(content))
        .when(!is_open, |this| {
            this.child(
                div()
                    .h(px(1.0))
                    .w_full()
                    .bg(cx.theme().colors().border_variant),
            )
        })
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
        .child(progress_step_row(
            "dx-progress-thread",
            status.active_status.as_ref() != "Idle",
            "Thread",
            status.active_status.clone(),
            cx,
        ))
        .child(progress_step_row(
            "dx-progress-sources",
            source_summary.attachable_sources > 0,
            "Sources",
            format!("{} attach-ready", source_summary.attachable_sources),
            cx,
        ))
        .child(progress_step_row(
            "dx-progress-style",
            status.style_panel.web_preview_bridge_ready,
            "Style",
            if status.style_panel.web_preview_bridge_ready {
                "host ready"
            } else {
                "host pending"
            },
            cx,
        ))
        .child(progress_step_row(
            "dx-progress-check",
            status.check_score.score >= 80,
            "Check",
            format!(
                "{}/100 {}",
                status.check_score.score, status.check_score.state
            ),
            cx,
        ))
        .child(progress_step_row(
            "dx-progress-runtime",
            status.runtime_proof_status.runtime_green_candidate(),
            "Runtime",
            status.runtime_proof_status.claim_state.clone(),
            cx,
        ))
        .into_any_element()
}

fn environment_summary(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-env-workspace",
            IconName::Library,
            "Workspace",
            format!("{} root(s)", status.visible_worktree_count),
            cx,
        ))
        .child(compact_status_row(
            "dx-env-local",
            IconName::Terminal,
            "Local",
            yes_no(status.receipt_snapshot.root_exists),
            cx,
        ))
        .child(compact_status_row(
            "dx-env-agent",
            IconName::ZedAgent,
            "Agent",
            if status.agent_bridge.enabled {
                status.agent_bridge.status.clone()
            } else {
                "disabled".to_string()
            },
            cx,
        ))
        .child(compact_status_row(
            "dx-env-commit",
            IconName::GitBranch,
            "Proof",
            format!("{} fresh", status.proof_freshness.fresh_receipt_count()),
            cx,
        ))
        .into_any_element()
}

fn subagent_summary(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1().child(compact_status_row(
        "dx-subagents-active",
        IconName::ListTodo,
        "Active",
        format!("{} task(s)", status.agent_bridge.active_task_count),
        cx,
    ));

    if status.agent_bridge.automations.is_empty() {
        return stack
            .child(muted_card("No active subagents", cx))
            .into_any_element();
    }

    for (ix, automation) in status.agent_bridge.automations.iter().take(6).enumerate() {
        let label = if automation.id.is_empty() {
            automation.source.clone()
        } else {
            automation.id.clone()
        };
        stack = stack.child(subagent_row(
            SharedString::from(format!("dx-subagent-row-{ix}")),
            ix,
            label,
            automation.status.clone(),
            automation.enabled,
            cx,
        ));
    }

    if status.agent_bridge.automation_count > 6 {
        stack = stack.child(
            Label::new(format!(
                "Show {} more",
                status.agent_bridge.automation_count.saturating_sub(6)
            ))
            .size(LabelSize::XSmall)
            .color(Color::Muted),
        );
    }

    stack.into_any_element()
}

fn source_summary(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let source_summary = status.source_sets.attachment_summary();
    v_flex()
        .gap_1()
        .child(compact_status_row(
            "dx-source-summary-roots",
            IconName::Folder,
            "Roots",
            source_summary.workspace_roots.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-summary-attach",
            IconName::Attach,
            "Attach",
            source_summary.attachable_sources.to_string(),
            cx,
        ))
        .child(compact_status_row(
            "dx-source-summary-receipts",
            IconName::FileTextOutlined,
            "Receipts",
            source_summary.managed_receipts.to_string(),
            cx,
        ))
        .into_any_element()
}

fn readiness_summary(
    status: &DxLaunchWorkspaceStatus,
    guided_cards: AnyElement,
    cx: &App,
) -> AnyElement {
    v_flex()
        .gap_1()
        .child(guided_cards)
        .child(style_panel::dx_style_panel_state(&status.style_panel, cx))
        .child(check::check_score_state(&status.check_score, cx))
        .child(deploy_target_state(&status.deploy_targets, cx))
        .when(status.agent_bridge.show_in_agent_rail, |this| {
            this.child(agents::dx_agent_bridge_state(&status.agent_bridge, cx))
                .child(agents::dx_agent_automation_state(&status.agent_bridge, cx))
        })
        .child(proof::proof_freshness_state(&status.proof_freshness, cx))
        .child(proof::runtime_proof_status_state(
            &status.runtime_proof_status,
            cx,
        ))
        .into_any_element()
}

fn progress_step_row(
    id: &'static str,
    complete: bool,
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    cx: &App,
) -> AnyElement {
    let icon = if complete {
        IconName::TodoComplete
    } else {
        IconName::Circle
    };
    let color = if complete {
        Color::Success
    } else {
        Color::Muted
    };

    h_flex()
        .id(id)
        .items_start()
        .gap_2()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .child(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            v_flex()
                .min_w_0()
                .gap_0p5()
                .child(
                    Label::new(label.into())
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                )
                .child(
                    Label::new(detail.into())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .into_any_element()
}

fn compact_status_row(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
    cx: &App,
) -> AnyElement {
    h_flex()
        .id(id)
        .items_center()
        .justify_between()
        .gap_2()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .child(
            h_flex()
                .gap_1p5()
                .min_w_0()
                .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                ),
        )
        .child(
            Label::new(value.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn subagent_row(
    id: SharedString,
    index: usize,
    label: impl Into<SharedString>,
    state: impl Into<SharedString>,
    active: bool,
    cx: &App,
) -> AnyElement {
    let state = state.into();
    let activity = if active {
        SharedString::from("is working")
    } else if state.as_ref().is_empty() {
        SharedString::from("idle")
    } else {
        state
    };

    h_flex()
        .id(id)
        .items_center()
        .gap_2()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .child(subagent_pixel_icon(index))
        .child(
            Label::new(label.into())
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .child(
            Label::new(activity)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn subagent_pixel_icon(index: usize) -> AnyElement {
    let colors = [
        gpui::hsla(210.0 / 360.0, 0.92, 0.56, 1.0),
        gpui::hsla(25.0 / 360.0, 0.96, 0.55, 1.0),
        gpui::hsla(355.0 / 360.0, 0.88, 0.56, 1.0),
        gpui::hsla(188.0 / 360.0, 0.86, 0.52, 1.0),
        gpui::hsla(0.0 / 360.0, 0.84, 0.58, 1.0),
        gpui::hsla(18.0 / 360.0, 0.98, 0.5, 1.0),
    ];
    let color = colors[index % colors.len()];

    div()
        .relative()
        .size(px(24.0))
        .flex_shrink_0()
        .rounded_sm()
        .border_1()
        .border_color(color.opacity(0.56))
        .bg(color.opacity(0.12))
        .child(
            div()
                .absolute()
                .left(px(4.0))
                .top(px(4.0))
                .size(px(6.0))
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .left(px(14.0))
                .top(px(4.0))
                .size(px(6.0))
                .bg(color.opacity(0.8)),
        )
        .child(
            div()
                .absolute()
                .left(px(4.0))
                .top(px(14.0))
                .size(px(6.0))
                .bg(color.opacity(0.8)),
        )
        .child(
            div()
                .absolute()
                .left(px(14.0))
                .top(px(14.0))
                .size(px(6.0))
                .bg(color),
        )
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
