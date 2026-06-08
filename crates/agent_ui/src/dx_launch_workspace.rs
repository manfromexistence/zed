use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, DismissEvent, EventEmitter, FocusHandle, Focusable,
    Render, SharedString, Window, prelude::*,
};
use ui::{Disclosure, DxRainbowGlow, IconName, PopoverMenu, Tooltip, prelude::*};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;
use crate::dx_check_score::DxCheckScoreSnapshot;
use crate::dx_deploy_targets::DxDeployTargetSnapshot;
use crate::dx_evidence_basket::DxEvidenceBasket;
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

mod agent_workspace;
mod agents;
mod audit;
mod automation_screen;
mod binary_cache;
mod binary_cache_labels;
mod check;
mod check_labels;
mod connections_screen;
mod contracts;
mod evidence_basket;
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
mod tools_screen;
mod www_evidence;

use self::list_labels::{bounded_items, yes_no};

pub(crate) use automation_screen::render_automation_screen;
pub(crate) use connections_screen::render_connections_screen;
pub(crate) use tools_screen::render_tools_screen;

#[derive(Clone)]
pub(crate) struct DxLaunchWorkspaceStatus {
    pub active_status: SharedString,
    pub visible_worktree_count: usize,
    pub background_thread_count: usize,
    pub subagent_rows: Vec<DxSubagentStatusRow>,
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
    pub evidence_basket: DxEvidenceBasket,
    pub check_score: DxCheckScoreSnapshot,
    pub deploy_targets: DxDeployTargetSnapshot,
    pub proof_freshness: DxProofFreshnessSnapshot,
    pub runtime_proof_status: DxRuntimeProofStatusSnapshot,
    pub style_panel: DxStylePanelSnapshot,
}

#[derive(Clone)]
pub(crate) struct DxSubagentStatusRow {
    pub label: SharedString,
    pub status: DxSubagentStatus,
    pub detail: SharedString,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DxSubagentStatus {
    Running,
    Queued,
    Blocked,
    Failed,
    Idle,
}

impl DxSubagentStatus {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DxSubagentStatus::Running => "running",
            DxSubagentStatus::Queued => "queued",
            DxSubagentStatus::Blocked => "blocked",
            DxSubagentStatus::Failed => "failed",
            DxSubagentStatus::Idle => "idle",
        }
    }

    pub(crate) fn rank(self) -> usize {
        match self {
            DxSubagentStatus::Running => 0,
            DxSubagentStatus::Blocked => 1,
            DxSubagentStatus::Queued => 2,
            DxSubagentStatus::Failed => 3,
            DxSubagentStatus::Idle => 4,
        }
    }
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
    AgentOverview,
    AgentThreads,
    AgentTasks,
    AgentSubagents,
    AgentApprovals,
}

#[derive(Clone, Copy)]
pub(crate) struct DxLaunchRailState {
    pub source_commands_open: bool,
    pub source_stack_open: bool,
    pub source_tools_open: bool,
    pub agent_overview_open: bool,
    pub agent_threads_open: bool,
    pub agent_tasks_open: bool,
    pub agent_subagents_open: bool,
    pub agent_approvals_open: bool,
}

impl DxLaunchRailState {
    pub(crate) fn is_open(self, section: DxLaunchRailSection) -> bool {
        match section {
            DxLaunchRailSection::SourceCommands => self.source_commands_open,
            DxLaunchRailSection::SourceStack => self.source_stack_open,
            DxLaunchRailSection::SourceTools => self.source_tools_open,
            DxLaunchRailSection::AgentOverview => self.agent_overview_open,
            DxLaunchRailSection::AgentThreads => self.agent_threads_open,
            DxLaunchRailSection::AgentTasks => self.agent_tasks_open,
            DxLaunchRailSection::AgentSubagents => self.agent_subagents_open,
            DxLaunchRailSection::AgentApprovals => self.agent_approvals_open,
        }
    }
}

#[derive(Clone)]
pub(crate) struct DxLaunchRailControls {
    pub state: DxLaunchRailState,
    pub sources_pinned: bool,
    pub progress_pinned: bool,
    pub on_toggle: Arc<dyn Fn(DxLaunchRailSection, &ClickEvent, &mut Window, &mut App) + 'static>,
    pub on_toggle_pin: Arc<dyn Fn(DxLaunchRailSide, &ClickEvent, &mut Window, &mut App) + 'static>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DxLaunchRailSide {
    Sources,
    Progress,
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
            .child(section_title("Launch Handoff", dx_icon(DxUiIcon::Receipts)))
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
            .child(section_title("Source Audit", dx_icon(DxUiIcon::Source)))
            .child(source_audit::launch_source_audit_state(
                &self.status.source_audit,
                cx,
            ))
            .child(section_title("WWW Evidence", dx_icon(DxUiIcon::Evidence)))
            .child(www_evidence::www_launch_evidence_state(
                &self.status.www_evidence,
                cx,
            ))
            .child(section_title("Launch Receipts", IconName::FileTextOutlined))
            .child(launch_receipts::launch_receipt_review_state(
                &self.status.launch_receipts,
                cx,
            ))
            .child(section_title("Binary Cache", dx_icon(DxUiIcon::Storage)))
            .child(binary_cache::binary_cache_state(
                &self.status.binary_cache,
                cx,
            ))
            .child(section_title(
                "Agent Connections",
                dx_icon(DxUiIcon::Connections),
            ))
            .child(agents::dx_agent_social_state(&self.status.agent_bridge, cx))
            .child(section_title("Agent Receipts", IconName::FileTextOutlined))
            .child(agents::dx_agent_receipt_state(
                &self.status.agent_bridge,
                cx,
            ))
            .child(section_title("Agent Providers", dx_icon(DxUiIcon::Gateway)))
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
            .child(section_title(
                "Evidence Basket",
                dx_icon(DxUiIcon::Evidence),
            ))
            .child(evidence_basket::evidence_basket_state(
                &self.status.evidence_basket,
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
    window: &mut Window,
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
                window,
                cx,
            ))
        })
        .when(show_progress_rail, |this| {
            this.child(render_right_rail(
                &status,
                guided_cards,
                &rail_controls,
                window,
                cx,
            ))
        })
        .into_any_element()
}

fn render_sources_rail(
    sidebar_actions: AnyElement,
    source_row_controls: Vec<DxSourceRowControl>,
    source_actions: AnyElement,
    status: &DxLaunchWorkspaceStatus,
    rail_controls: &DxLaunchRailControls,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    v_flex()
        .id("dx-sources-rail")
        .absolute()
        .left_2()
        .top_2()
        .w(px(300.0))
        .max_h(vh(0.86, window))
        .gap_2()
        .p_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().elevated_surface_background)
        .shadow_md()
        .overflow_y_scroll()
        .occlude()
        .child(rail_pin_header(
            "dx-sources-rail-pin",
            "Sources",
            rail_controls.sources_pinned,
            DxLaunchRailSide::Sources,
            rail_controls,
        ))
        .child(rail_rainbow_glow("dx-sources-rail-rainbow-glow", 0.))
        .child(rail_section(
            "dx-sources-commands-section",
            "Commands",
            IconName::Terminal,
            DxLaunchRailSection::SourceCommands,
            rail_controls,
            sidebar_actions,
            true,
            cx,
        ))
        .child(rail_section(
            "dx-sources-stack-section",
            "Sources",
            IconName::Book,
            DxLaunchRailSection::SourceStack,
            rail_controls,
            sources::source_set_stack(&status.source_sets, source_row_controls, cx),
            true,
            cx,
        ))
        .child(rail_section(
            "dx-sources-tools-section",
            "Source Tools",
            dx_icon(DxUiIcon::Source),
            DxLaunchRailSection::SourceTools,
            rail_controls,
            source_actions,
            false,
            cx,
        ))
        .into_any_element()
}

fn render_right_rail(
    status: &DxLaunchWorkspaceStatus,
    guided_cards: AnyElement,
    rail_controls: &DxLaunchRailControls,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    v_flex()
        .id("dx-progress-rail")
        .absolute()
        .right_2()
        .top_2()
        .w(px(300.0))
        .max_h(vh(0.86, window))
        .gap_2()
        .p_2()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().colors().border)
        .bg(cx.theme().colors().elevated_surface_background)
        .shadow_md()
        .overflow_y_scroll()
        .occlude()
        .child(rail_pin_header(
            "dx-progress-rail-pin",
            "Agents",
            rail_controls.progress_pinned,
            DxLaunchRailSide::Progress,
            rail_controls,
        ))
        .child(rail_rainbow_glow("dx-progress-rail-rainbow-glow", 0.18))
        .child(diagnostics_menu(status.clone()))
        .child(rail_section(
            "dx-agent-overview-section",
            "Overview",
            dx_icon(DxUiIcon::Agent),
            DxLaunchRailSection::AgentOverview,
            rail_controls,
            agent_workspace::agent_overview_section(status, guided_cards, cx),
            true,
            cx,
        ))
        .child(rail_section(
            "dx-agent-threads-section",
            "Threads",
            IconName::HistoryRerun,
            DxLaunchRailSection::AgentThreads,
            rail_controls,
            agent_workspace::agent_threads_section(status, cx),
            true,
            cx,
        ))
        .child(rail_section(
            "dx-agent-tasks-section",
            "Tasks",
            IconName::TodoProgress,
            DxLaunchRailSection::AgentTasks,
            rail_controls,
            agent_workspace::agent_tasks_section(status, cx),
            true,
            cx,
        ))
        .child(rail_section(
            "dx-agent-subagents-section",
            "Subagents",
            dx_icon(DxUiIcon::Agent),
            DxLaunchRailSection::AgentSubagents,
            rail_controls,
            agent_workspace::agent_subagents_section(status, cx),
            true,
            cx,
        ))
        .child(rail_section(
            "dx-agent-approvals-section",
            "Approvals",
            dx_icon(DxUiIcon::Permissions),
            DxLaunchRailSection::AgentApprovals,
            rail_controls,
            agent_workspace::agent_approvals_section(status, cx),
            false,
            cx,
        ))
        .into_any_element()
}

fn rail_rainbow_glow(id: &'static str, phase_offset: f32) -> AnyElement {
    div()
        .h(px(8.0))
        .w_full()
        .flex_none()
        .overflow_hidden()
        .child(
            DxRainbowGlow::new()
                .id(id)
                .height(px(3.0))
                .radius(px(2.0))
                .phase_offset(phase_offset),
        )
        .into_any_element()
}

fn rail_pin_header(
    id: &'static str,
    label: &'static str,
    pinned: bool,
    side: DxLaunchRailSide,
    controls: &DxLaunchRailControls,
) -> AnyElement {
    let on_toggle_pin = controls.on_toggle_pin.clone();

    h_flex()
        .id(id)
        .items_center()
        .gap_1()
        .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
        .child(div().flex_1())
        .child(
            IconButton::new(format!("{id}-pin"), IconName::Pin)
                .icon_size(IconSize::Small)
                .toggle_state(pinned)
                .tooltip(Tooltip::text(if pinned {
                    "Unpin rail"
                } else {
                    "Pin rail"
                }))
                .on_click(move |event, window, cx| {
                    on_toggle_pin(side, event, window, cx);
                }),
        )
        .into_any_element()
}

fn rail_section(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    section: DxLaunchRailSection,
    controls: &DxLaunchRailControls,
    content: AnyElement,
    show_bottom_rule: bool,
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
                .cursor_pointer()
                .on_click(move |event, window, cx| {
                    on_toggle(section, event, window, cx);
                })
                .child(Disclosure::new(format!("{id}-disclosure"), is_open))
                .child(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
                .child(div().flex_1()),
        )
        .when(is_open, |this| this.child(content))
        .when(!is_open && show_bottom_rule, |this| {
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
            IconButton::new("dx-launch-diagnostics-button", dx_icon(DxUiIcon::Settings))
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted),
            Tooltip::text("Open diagnostics"),
        )
        .anchor(gpui::Anchor::TopRight)
        .menu(move |_window, cx| {
            let status = status.clone();
            Some(cx.new(|cx| DxLaunchDiagnosticsMenu {
                status,
                focus_handle: cx.focus_handle(),
            }))
        })
        .into_any_element()
}

fn subagent_summary(status: &DxLaunchWorkspaceStatus, cx: &App) -> AnyElement {
    let mut stack = v_flex().gap_1().child(compact_status_row(
        "dx-subagents-active",
        dx_icon(DxUiIcon::Receipts),
        "Active Tasks",
        status.agent_bridge.active_task_count.to_string(),
        cx,
    ));

    if status.subagent_rows.is_empty() {
        return stack
            .child(muted_card("No live subagent state", cx))
            .into_any_element();
    }

    for (ix, row) in status.subagent_rows.iter().take(6).enumerate() {
        stack = stack.child(subagent_row(
            SharedString::from(format!("dx-subagent-row-{ix}")),
            row,
            cx,
        ));
    }

    if status.subagent_rows.len() > 6 {
        stack = stack.child(
            Label::new(format!(
                "+{} more",
                status.subagent_rows.len().saturating_sub(6)
            ))
            .size(LabelSize::XSmall)
            .color(Color::Muted),
        );
    }

    stack.into_any_element()
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

fn subagent_row(id: SharedString, row: &DxSubagentStatusRow, cx: &App) -> AnyElement {
    h_flex()
        .id(id)
        .items_center()
        .gap_2()
        .min_w_0()
        .rounded_sm()
        .px_1()
        .py_0p5()
        .hover(|this| this.bg(cx.theme().colors().element_hover))
        .tooltip(Tooltip::text(row.detail.clone()))
        .child(subagent_pixel_icon(row.status))
        .child(
            Label::new(row.label.clone())
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .child(div().flex_1())
        .child(subagent_status_badge(row.status))
        .into_any_element()
}

fn subagent_status_badge(status: DxSubagentStatus) -> AnyElement {
    let color = subagent_status_color(status);

    h_flex()
        .items_center()
        .gap_0p5()
        .rounded_sm()
        .border_1()
        .border_color(color.opacity(0.42))
        .bg(color.opacity(0.1))
        .px_1()
        .py_0p5()
        .child(
            Icon::new(subagent_status_icon(status))
                .size(IconSize::Indicator)
                .color(Color::Custom(color)),
        )
        .child(
            Label::new(status.label())
                .size(LabelSize::XSmall)
                .color(Color::Custom(color))
                .truncate(),
        )
        .into_any_element()
}

fn subagent_status_icon(status: DxSubagentStatus) -> IconName {
    match status {
        DxSubagentStatus::Running => dx_icon(DxUiIcon::Loading),
        DxSubagentStatus::Queued => IconName::TodoProgress,
        DxSubagentStatus::Blocked => IconName::Warning,
        DxSubagentStatus::Failed => IconName::Close,
        DxSubagentStatus::Idle => IconName::Circle,
    }
}

fn subagent_status_color(status: DxSubagentStatus) -> gpui::Hsla {
    match status {
        DxSubagentStatus::Running => gpui::hsla(188.0 / 360.0, 0.86, 0.52, 1.0),
        DxSubagentStatus::Queued => gpui::hsla(45.0 / 360.0, 0.9, 0.5, 1.0),
        DxSubagentStatus::Blocked => gpui::hsla(25.0 / 360.0, 0.96, 0.55, 1.0),
        DxSubagentStatus::Failed => gpui::hsla(355.0 / 360.0, 0.88, 0.56, 1.0),
        DxSubagentStatus::Idle => gpui::hsla(210.0 / 360.0, 0.22, 0.58, 1.0),
    }
}

fn subagent_pixel_icon(status: DxSubagentStatus) -> AnyElement {
    let color = subagent_status_color(status);

    div()
        .relative()
        .size(px(20.0))
        .flex_shrink_0()
        .rounded_sm()
        .border_1()
        .border_color(color.opacity(0.56))
        .bg(color.opacity(0.12))
        .child(
            div()
                .absolute()
                .left(px(3.0))
                .top(px(3.0))
                .size(px(5.0))
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .left(px(12.0))
                .top(px(3.0))
                .size(px(5.0))
                .bg(color.opacity(0.8)),
        )
        .child(
            div()
                .absolute()
                .left(px(3.0))
                .top(px(12.0))
                .size(px(5.0))
                .bg(color.opacity(0.8)),
        )
        .child(
            div()
                .absolute()
                .left(px(12.0))
                .top(px(12.0))
                .size(px(5.0))
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
