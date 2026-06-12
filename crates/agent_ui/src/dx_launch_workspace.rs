use std::sync::Arc;

use agent_settings::{AgentLiquidGlassSettings, AgentSettings};
use gpui::{
    AnyElement, App, ClickEvent, Context, DismissEvent, EventEmitter, FocusHandle, Focusable,
    Render, SharedString, Window, prelude::*,
};
use liquid_glass::{
    LiquidGlassStyle, bounded_liquid_glass_layer, load_liquid_glass_backdrop_carrier,
};
use settings::Settings as _;
use ui::{
    DxRainbowGlow, IconName, ListHeader, ListItem, ListItemSpacing, PopoverMenu, Tooltip,
    prelude::*,
};

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
mod catalog_chrome;
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
mod screen_chrome;
mod source_audit;
mod sources;
mod style_panel;
mod tool_history;
mod tools_screen;
mod www_evidence;

use self::list_labels::{bounded_items, yes_no};

const DX_RAIL_LIQUID_GLASS_BLUR_RADIUS: f32 = 1.25;
const DX_RAIL_LIQUID_GLASS_BLUR_ITERATIONS: u32 = 3;
const DX_RAIL_LIQUID_GLASS_BLUR_DOWNSCALE: f32 = 0.50;
const DX_RAIL_LIQUID_GLASS_CHROMATIC_ABERRATION: f32 = 0.0015;
const DX_RAIL_LIQUID_GLASS_ABERRATION_SAMPLES: u32 = 3;

pub(crate) use automation_screen::{
    AutomationCatalogFilter, DxAutomationCatalogState, render_automation_catalog_rows,
    render_automation_screen,
};
pub(crate) use connections_screen::{
    ConnectionCatalogFilter, DxConnectionsCatalogState, render_connections_catalog_rows,
    render_connections_screen,
};
pub(crate) use tools_screen::{
    DxPluginsCatalogState, PluginCatalogFilter, render_tools_screen,
    render_workflow_node_catalog_rows,
};

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
    SourceController,
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
    pub source_controller_open: bool,
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
            DxLaunchRailSection::SourceController => self.source_controller_open,
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
        .min_h_0()
        .min_w_0()
        .overflow_hidden()
        .bg(cx.theme().colors().panel_background)
        .child(div().size_full().min_w_0().overflow_hidden().child(center))
        .when(show_sources_rail, |this| {
            this.child(render_sources_rail(
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
        .bg(cx.theme().colors().panel_background.opacity(0.10))
        .shadow_md()
        .relative()
        .overflow_hidden()
        .overflow_y_scroll()
        .occlude()
        .when_some(
            rail_liquid_glass_surface("dx-sources-rail-liquid-glass", cx),
            |this, glass| this.child(glass),
        )
        .child(rail_pin_header(
            "dx-sources-rail-pin",
            "Sources",
            rail_controls.sources_pinned,
            DxLaunchRailSide::Sources,
            rail_controls,
        ))
        .child(rail_rainbow_glow("dx-sources-rail-rainbow-glow", 0.))
        .when(!has_sources_rail_content(status), |this| {
            this.child(fullscreen_empty_rail_state(
                "dx-sources-rail-empty",
                "No source data yet",
                "Open a folder or import DX receipts to populate this rail.",
                cx,
            ))
        })
        .when(status.source_sets.total_sources > 0, |this| {
            this.child(rail_section(
                "dx-sources-controller-section",
                "Source Controller",
                dx_icon(DxUiIcon::Source),
                DxLaunchRailSection::SourceController,
                rail_controls,
                sources::source_controller_state(&status.source_sets, cx),
                true,
                cx,
            ))
        })
        .when(status.source_sets.total_sources > 0, |this| {
            this.child(rail_section(
                "dx-sources-stack-section",
                "Sources",
                IconName::Book,
                DxLaunchRailSection::SourceStack,
                rail_controls,
                sources::source_set_stack(&status.source_sets, source_row_controls, cx),
                true,
                cx,
            ))
        })
        .when(has_source_actions(status), |this| {
            this.child(rail_section(
                "dx-sources-tools-section",
                "Next Actions",
                IconName::ListTodo,
                DxLaunchRailSection::SourceTools,
                rail_controls,
                source_actions,
                false,
                cx,
            ))
        })
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
        .bg(cx.theme().colors().panel_background.opacity(0.10))
        .shadow_md()
        .relative()
        .overflow_hidden()
        .overflow_y_scroll()
        .occlude()
        .when_some(
            rail_liquid_glass_surface("dx-progress-rail-liquid-glass", cx),
            |this, glass| this.child(glass),
        )
        .child(rail_pin_header(
            "dx-progress-rail-pin",
            "Agents",
            rail_controls.progress_pinned,
            DxLaunchRailSide::Progress,
            rail_controls,
        ))
        .child(rail_rainbow_glow("dx-progress-rail-rainbow-glow", 0.18))
        .child(diagnostics_menu(status.clone()))
        .when(!has_progress_rail_content(status), |this| {
            this.child(fullscreen_empty_rail_state(
                "dx-progress-rail-empty",
                "No agent activity yet",
                "Agent progress, sources, subagents, and readiness receipts will appear here.",
                cx,
            ))
        })
        .when(has_agent_progress(status), |this| {
            this.child(rail_section(
                "dx-agent-overview-section",
                "Progress",
                dx_icon(DxUiIcon::Agent),
                DxLaunchRailSection::AgentOverview,
                rail_controls,
                agent_workspace::agent_overview_section(status, guided_cards, cx),
                true,
                cx,
            ))
        })
        .when(has_agent_environment(status), |this| {
            this.child(rail_section(
                "dx-agent-threads-section",
                "Environment",
                dx_icon(DxUiIcon::Project),
                DxLaunchRailSection::AgentThreads,
                rail_controls,
                agent_workspace::agent_environment_section(status, cx),
                true,
                cx,
            ))
        })
        .when(status.source_sets.total_sources > 0, |this| {
            this.child(rail_section(
                "dx-agent-tasks-section",
                "Sources",
                dx_icon(DxUiIcon::Source),
                DxLaunchRailSection::AgentTasks,
                rail_controls,
                agent_workspace::agent_sources_section(status, cx),
                true,
                cx,
            ))
        })
        .when(has_agent_subagents(status), |this| {
            this.child(rail_section(
                "dx-agent-subagents-section",
                "Subagents",
                dx_icon(DxUiIcon::Agent),
                DxLaunchRailSection::AgentSubagents,
                rail_controls,
                agent_workspace::agent_subagents_section(status, cx),
                true,
                cx,
            ))
        })
        .when(has_agent_readiness(status), |this| {
            this.child(rail_section(
                "dx-agent-approvals-section",
                "Readiness",
                dx_icon(DxUiIcon::Permissions),
                DxLaunchRailSection::AgentApprovals,
                rail_controls,
                agent_workspace::agent_approvals_section(status, cx),
                false,
                cx,
            ))
        })
        .into_any_element()
}

fn has_source_actions(status: &DxLaunchWorkspaceStatus) -> bool {
    status
        .source_sets
        .sets
        .iter()
        .flat_map(|set| set.sources.iter())
        .any(|source| {
            !matches!(
                source.kind,
                crate::dx_source_sets::DxSourceKind::WorkspaceRoot
            )
        })
        || !status.deploy_targets.targets.is_empty()
}

pub(crate) fn has_sources_rail_content(status: &DxLaunchWorkspaceStatus) -> bool {
    status.source_sets.total_sources > 0 || has_source_actions(status)
}

pub(crate) fn has_progress_rail_content(status: &DxLaunchWorkspaceStatus) -> bool {
    has_agent_progress(status)
        || has_agent_environment(status)
        || has_agent_subagents(status)
        || has_agent_readiness(status)
}

fn has_agent_progress(status: &DxLaunchWorkspaceStatus) -> bool {
    status.agent_bridge.active_task_count > 0 || status.background_thread_count > 0
}

fn has_agent_environment(status: &DxLaunchWorkspaceStatus) -> bool {
    status.agent_bridge.connected_accounts_summary.connected > 0
        || status.agent_bridge.connected_accounts_summary.needs_auth > 0
        || status.agent_bridge.automation_count > 0
}

fn has_agent_subagents(status: &DxLaunchWorkspaceStatus) -> bool {
    status.agent_bridge.active_task_count > 0 || !status.subagent_rows.is_empty()
}

fn has_agent_readiness(status: &DxLaunchWorkspaceStatus) -> bool {
    status.launch_readiness.warning_count > 0 || status.launch_readiness.failed_count > 0
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

fn rail_liquid_glass_surface(id: &'static str, cx: &App) -> Option<AnyElement> {
    let settings = AgentSettings::get_global(cx).liquid_glass.clone();
    if !settings.enabled {
        return None;
    }

    Some(
        div()
            .id(id)
            .absolute()
            .inset_0()
            .overflow_hidden()
            .child(bounded_liquid_glass_layer(
                load_liquid_glass_backdrop_carrier(),
                rail_liquid_glass_style(&settings),
            ))
            .into_any_element(),
    )
}

fn rail_liquid_glass_style(settings: &AgentLiquidGlassSettings) -> LiquidGlassStyle {
    LiquidGlassStyle {
        power_factor: settings.power_factor,
        a: settings.a,
        b: settings.b,
        c: settings.c,
        d: settings.d,
        f_power: settings.f_power,
        noise: settings.noise,
        glow_weight: settings.glow_weight,
        glow_edge0: settings.glow_edge0,
        glow_edge1: settings.glow_edge1,
        glow_bias: settings.glow_bias,
        chromatic_aberration: DX_RAIL_LIQUID_GLASS_CHROMATIC_ABERRATION,
        aberration_samples: DX_RAIL_LIQUID_GLASS_ABERRATION_SAMPLES,
        blur_radius: DX_RAIL_LIQUID_GLASS_BLUR_RADIUS,
        blur_iterations: DX_RAIL_LIQUID_GLASS_BLUR_ITERATIONS,
        blur_downscale: DX_RAIL_LIQUID_GLASS_BLUR_DOWNSCALE,
    }
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
            ListHeader::new(label)
                .inset(true)
                .toggle(Some(is_open))
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
                .on_toggle(move |event, window, cx| {
                    on_toggle(section, event, window, cx);
                }),
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

fn fullscreen_empty_rail_state(
    id: &'static str,
    title: &'static str,
    detail: &'static str,
    _cx: &App,
) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Info)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            v_flex()
                .min_w_0()
                .gap_0p5()
                .child(
                    Label::new(title)
                        .size(LabelSize::Small)
                        .color(Color::Default)
                        .truncate(),
                )
                .child(
                    Label::new(detail)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .tooltip(Tooltip::text(detail))
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
        return stack.into_any_element();
    }

    for (ix, row) in status.subagent_rows.iter().take(6).enumerate() {
        stack = stack.child(subagent_row(
            SharedString::from(format!("dx-subagent-row-{ix}")),
            row,
            cx,
        ));
    }

    if status.subagent_rows.len() > 6 {
        stack = stack.child(compact_status_row(
            "dx-subagents-more",
            IconName::Ellipsis,
            "More subagents",
            format!(
                "{} more subagent row(s) hidden",
                status.subagent_rows.len().saturating_sub(6)
            ),
            cx,
        ));
    }

    stack.into_any_element()
}

fn compact_status_row(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    value: impl Into<SharedString>,
    _cx: &App,
) -> AnyElement {
    let value = value.into();

    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .tooltip(Tooltip::text(format!("{label}: {value}")))
        .into_any_element()
}

fn subagent_row(id: SharedString, row: &DxSubagentStatusRow, _cx: &App) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(subagent_pixel_icon(row.status))
        .child(
            Label::new(row.label.clone())
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .end_slot(subagent_status_indicator(row.status))
        .tooltip(Tooltip::text(format!(
            "{}: {}",
            row.status.label(),
            row.detail
        )))
        .into_any_element()
}

fn subagent_status_indicator(status: DxSubagentStatus) -> AnyElement {
    Icon::new(subagent_status_icon(status))
        .size(IconSize::Small)
        .color(Color::Custom(subagent_status_color(status)))
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
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            Label::new(label.into())
                .size(LabelSize::Small)
                .color(color)
                .truncate(),
        )
        .into_any_element()
}

fn section_title(label: &'static str, icon: IconName) -> AnyElement {
    div()
        .child(
            ListHeader::new(label)
                .inset(true)
                .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted)),
        )
        .into_any_element()
}

fn source_row(
    id: SharedString,
    icon: IconName,
    label: impl Into<SharedString>,
    _cx: &App,
) -> AnyElement {
    ListItem::new(id)
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(Icon::new(icon).size(IconSize::Small).color(Color::Muted))
        .child(
            Label::new(label.into())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn metric_row(label: impl Into<SharedString>, value: impl Into<SharedString>) -> AnyElement {
    let label = label.into();
    let value = value.into();

    ListItem::new(rail_stable_id("dx-launch-metric", label.as_ref()))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
        .end_slot(
            Label::new(value)
                .size(LabelSize::Small)
                .color(Color::Default)
                .truncate(),
        )
        .into_any_element()
}

fn muted_card(label: impl Into<SharedString>, _cx: &App) -> AnyElement {
    let label = label.into();

    ListItem::new(rail_stable_id("dx-launch-empty", label.as_ref()))
        .inset(true)
        .spacing(ListItemSpacing::Sparse)
        .selectable(false)
        .start_slot(
            Icon::new(IconName::Info)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(
            Label::new(label)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

fn rail_stable_id(prefix: &str, label: &str) -> SharedString {
    let mut id = String::with_capacity(prefix.len() + label.len().min(48) + 1);
    id.push_str(prefix);
    id.push('-');
    for ch in label.chars().take(48) {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        } else if !id.ends_with('-') {
            id.push('-');
        }
    }
    SharedString::from(id.trim_end_matches('-').to_string())
}
