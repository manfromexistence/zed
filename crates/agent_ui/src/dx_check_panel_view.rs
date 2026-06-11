use std::{collections::HashSet, path::PathBuf};

use gpui::{
    AnyElement, App, AppContext as _, AsyncWindowContext, Context, Entity, EntityId, EventEmitter,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    ScrollHandle, SharedString, Styled, TaskExt, WeakEntity, Window, div, px,
};
use theme::ActiveTheme;
use ui::{
    ButtonStyle, IconButtonShape, Indicator, ListItem, ListItemSpacing, Tab, Tooltip,
    WithScrollbar, prelude::*,
};
use workspace::{
    OpenOptions, Workspace,
    dock::{DockPosition, Panel, PanelEvent, side_panel_header_controls},
};

use crate::dx_check_panel::{
    DxCheckPanelSnapshot, dx_check_panel_snapshot, invalidate_dx_check_panel_snapshot_cache,
};
use crate::dx_check_panel_view::view_rows::{
    adapter_plan_row, config_label, count_label, detail_row, duration_label, empty_row, notice_row,
    notice_title, outcome_label, overflow_row, quick_fix_row, section, section_row, status_color,
    web_audit_row,
};

mod tabs;
mod view_rows;

use tabs::{DxCheckPanelTab, render_tab_bar};

const DX_CHECK_PANEL_KEY: &str = "DxCheckPanel";
const MAX_SECTION_ROWS: usize = 8;
const MAX_NOTICE_ROWS: usize = 4;
const MAX_QUICK_FIX_ROWS: usize = 4;
const MAX_ADAPTER_PLAN_ROWS: usize = 4;
const MAX_WEB_AUDIT_ROWS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DxCheckPanelSectionKind {
    Run,
    Receipt,
    Sections,
    WebAudit,
    AdapterPlans,
    Notices,
    QuickFixes,
    Commands,
}

impl DxCheckPanelSectionKind {
    fn id(self) -> &'static str {
        match self {
            Self::Run => "dx-check-section-run",
            Self::Receipt => "dx-check-section-receipt",
            Self::Sections => "dx-check-section-sections",
            Self::WebAudit => "dx-check-section-web-audit",
            Self::AdapterPlans => "dx-check-section-adapter-plans",
            Self::Notices => "dx-check-section-notices",
            Self::QuickFixes => "dx-check-section-quick-fixes",
            Self::Commands => "dx-check-section-commands",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Run => "Run",
            Self::Receipt => "Receipt",
            Self::Sections => "Sections",
            Self::WebAudit => "Web Audit",
            Self::AdapterPlans => "Adapter Plans",
            Self::Notices => "Notices",
            Self::QuickFixes => "Quick Fixes",
            Self::Commands => "Commands",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Run => IconName::PlayOutlined,
            Self::Receipt => IconName::FileTextOutlined,
            Self::Sections => dx_icon(DxUiIcon::Check),
            Self::WebAudit => dx_icon(DxUiIcon::Evidence),
            Self::AdapterPlans => IconName::Terminal,
            Self::Notices => IconName::Warning,
            Self::QuickFixes => IconName::Sparkle,
            Self::Commands => IconName::Terminal,
        }
    }
}

fn section_count_label(
    section_kind: DxCheckPanelSectionKind,
    snapshot: &DxCheckPanelSnapshot,
) -> Option<SharedString> {
    let count = match section_kind {
        DxCheckPanelSectionKind::Sections => snapshot.sections.len(),
        DxCheckPanelSectionKind::WebAudit => snapshot.web_audits.len(),
        DxCheckPanelSectionKind::AdapterPlans => snapshot.adapter_plans.len(),
        DxCheckPanelSectionKind::Notices => snapshot.blockers.len() + snapshot.warnings.len(),
        DxCheckPanelSectionKind::QuickFixes => snapshot.quick_fixes.len(),
        DxCheckPanelSectionKind::Commands => {
            2 + usize::from(snapshot.detail_command.as_ref().is_some())
        }
        DxCheckPanelSectionKind::Run | DxCheckPanelSectionKind::Receipt => return None,
    };

    Some(SharedString::from(count.to_string()))
}

pub struct DxCheckPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    active_tab: DxCheckPanelTab,
    collapsed_sections: HashSet<DxCheckPanelSectionKind>,
}

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(
            |workspace, _: &zed_actions::dx_check_panel::ToggleFocus, window, cx| {
                workspace.toggle_panel_focus::<DxCheckPanel>(window, cx);
            },
        );
        workspace.register_action(
            |workspace, _: &zed_actions::dx_check_panel::Toggle, window, cx| {
                if !workspace.toggle_panel_focus::<DxCheckPanel>(window, cx) {
                    workspace.close_panel::<DxCheckPanel>(window, cx);
                }
            },
        );
    })
    .detach();
}

impl DxCheckPanel {
    pub async fn load(
        workspace: WeakEntity<Workspace>,
        mut cx: AsyncWindowContext,
    ) -> anyhow::Result<Entity<Self>> {
        workspace.update_in(&mut cx, |workspace, _window, cx| Self::new(workspace, cx))
    }

    fn new(_workspace: &mut Workspace, cx: &mut Context<Workspace>) -> Entity<Self> {
        let workspace = cx.entity().downgrade();

        cx.new(|cx| Self {
            workspace,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            active_tab: DxCheckPanelTab::Overview,
            collapsed_sections: [
                DxCheckPanelSectionKind::Receipt,
                DxCheckPanelSectionKind::Commands,
                DxCheckPanelSectionKind::AdapterPlans,
            ]
            .into_iter()
            .collect(),
        })
    }

    fn snapshot(&self, cx: &App) -> DxCheckPanelSnapshot {
        dx_check_panel_snapshot(&self.workspace_roots(cx))
    }

    fn workspace_roots(&self, cx: &App) -> Vec<String> {
        self.workspace
            .upgrade()
            .map(|workspace| {
                workspace
                    .read(cx)
                    .root_paths(cx)
                    .into_iter()
                    .map(|path| path.display().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        invalidate_dx_check_panel_snapshot_cache();
        cx.notify();
    }

    fn set_active_tab(&mut self, tab: DxCheckPanelTab, cx: &mut Context<Self>) {
        if self.active_tab != tab {
            self.active_tab = tab;
            cx.notify();
        }
    }

    fn section_is_open(&self, section: DxCheckPanelSectionKind) -> bool {
        !self.collapsed_sections.contains(&section)
    }

    fn toggle_section(&mut self, section: DxCheckPanelSectionKind, cx: &mut Context<Self>) {
        if !self.collapsed_sections.insert(section) {
            self.collapsed_sections.remove(&section);
        }
        cx.notify();
    }

    fn render_section_shell(
        &self,
        section_kind: DxCheckPanelSectionKind,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> gpui::Stateful<gpui::Div> {
        let is_open = self.section_is_open(section_kind);
        section(
            section_kind.id(),
            section_kind.title(),
            section_kind.icon(),
            section_count_label(section_kind, snapshot),
            is_open,
            move |_, _, cx| {
                panel
                    .update(cx, |panel, cx| panel.toggle_section(section_kind, cx))
                    .ok();
            },
            cx,
        )
    }

    fn render_header(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel_id: EntityId,
        cx: &App,
    ) -> AnyElement {
        h_flex()
            .id("dx-check-panel-header")
            .h(Tab::container_height(cx))
            .w_full()
            .min_w_0()
            .items_center()
            .justify_between()
            .gap_2()
            .px_2()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .min_w_0()
                    .items_center()
                    .gap_1()
                    .child(
                        Icon::new(IconName::Check)
                            .size(IconSize::Small)
                            .color(status_color(snapshot)),
                    )
                    .child(
                        Label::new("Check")
                            .size(LabelSize::Small)
                            .color(Color::Default)
                            .truncate(),
                    ),
            )
            .child(side_panel_header_controls(
                "dx-check-panel",
                self.workspace.clone(),
                panel_id,
                cx,
            ))
            .into_any_element()
    }

    fn render_status_strip(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let focus_handle = self.focus_handle(cx);
        let color = status_color(snapshot);
        let outcome = outcome_label(
            snapshot.pass_count,
            snapshot.fail_count,
            snapshot.warn_count,
            snapshot.skipped_count,
        );
        let tooltip = format!("{}\n{outcome}", snapshot.status);
        let receipt_path = snapshot.receipt_path.clone();
        let receipt_enabled = snapshot.receipt_present && receipt_path.exists();

        ListItem::new("dx-check-status")
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .selectable(false)
            .start_slot(Indicator::dot().color(color))
            .child(
                h_flex()
                    .min_w_0()
                    .gap_2()
                    .justify_between()
                    .child(
                        Label::new(snapshot.score_label())
                            .size(LabelSize::Small)
                            .color(color)
                            .truncate(),
                    )
                    .child(
                        Label::new(snapshot.status.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .end_slot(
                h_flex()
                    .id("dx-check-status-actions")
                    .flex_none()
                    .gap_0p5()
                    .occlude()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_mouse_up(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        IconButton::new("dx-check-open-receipt", IconName::FileTextOutlined)
                            .shape(IconButtonShape::Square)
                            .icon_size(IconSize::Small)
                            .icon_color(Color::Muted)
                            .style(ButtonStyle::Subtle)
                            .tab_index(0_isize)
                            .track_focus(&focus_handle)
                            .disabled(!receipt_enabled)
                            .tooltip(Tooltip::text(if receipt_enabled {
                                "Open latest Check receipt"
                            } else {
                                "Latest Check receipt is not available"
                            }))
                            .on_click({
                                let workspace = self.workspace.clone();
                                move |_, window, cx| {
                                    if receipt_path.exists() {
                                        open_workspace_path(
                                            workspace.clone(),
                                            receipt_path.clone(),
                                            window,
                                            cx,
                                        );
                                    }
                                }
                            }),
                    )
                    .child(
                        IconButton::new("dx-check-refresh", IconName::RotateCw)
                            .shape(IconButtonShape::Square)
                            .icon_size(IconSize::Small)
                            .icon_color(Color::Muted)
                            .style(ButtonStyle::Subtle)
                            .tab_index(0_isize)
                            .track_focus(&focus_handle)
                            .tooltip(Tooltip::text("Refresh Check panel"))
                            .on_click(move |_, _, cx| {
                                panel.update(cx, |panel, cx| panel.refresh(cx)).ok();
                            }),
                    ),
            )
            .tooltip(Tooltip::text(tooltip))
            .into_any_element()
    }

    fn render_summary(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::Run;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            stack = stack
                .child(detail_row("Last run", snapshot.last_run_label.clone()))
                .child(detail_row("Profile", snapshot.weight_profile.clone()))
                .child(detail_row(
                    "Outcome",
                    outcome_label(
                        snapshot.pass_count,
                        snapshot.fail_count,
                        snapshot.warn_count,
                        snapshot.skipped_count,
                    ),
                ))
                .child(detail_row("Duration", duration_label(snapshot.duration_ms)))
                .child(detail_row(
                    "Checked",
                    count_label(snapshot.checked_paths.len(), "path"),
                ))
                .child(detail_row(
                    "Skipped",
                    count_label(snapshot.skipped_expensive_checks.len(), "check"),
                ))
                .child(detail_row(
                    "Config",
                    config_label(
                        &snapshot.scoring_config_status,
                        snapshot.scoring_config_applies_to_score,
                    ),
                ))
                .child(detail_row(
                    "Scoring",
                    snapshot.scoring_config_summary.clone(),
                ));
        }
        stack.into_any_element()
    }

    fn render_receipt(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let receipt_status = if snapshot.receipt_present {
            "present"
        } else {
            "missing"
        };
        let section_kind = DxCheckPanelSectionKind::Receipt;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            stack = stack
                .child(detail_row("State", receipt_status))
                .child(detail_row(
                    "Source",
                    snapshot.receipt_path.display().to_string(),
                ))
                .child(detail_row("Schema", snapshot.source_schema.clone()));

            if let Some(error) = snapshot.receipt_error.as_ref() {
                stack = stack.child(notice_row(
                    "dx-check-receipt-error",
                    IconName::Warning,
                    Color::Warning,
                    error,
                    None,
                ));
            }
        }

        stack.into_any_element()
    }

    fn render_sections(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::Sections;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            if snapshot.sections.is_empty() {
                stack = stack.child(empty_row("No section scores in the latest receipt."));
            } else {
                for (index, section) in snapshot.sections.iter().take(MAX_SECTION_ROWS).enumerate()
                {
                    stack = stack.child(section_row(index, section));
                }
                if snapshot.sections.len() > MAX_SECTION_ROWS {
                    stack = stack.child(overflow_row(
                        "dx-check-section-overflow",
                        snapshot.sections.len() - MAX_SECTION_ROWS,
                        "section scores",
                    ));
                }
            }
        }
        stack.into_any_element()
    }

    fn render_adapter_plans(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::AdapterPlans;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            if snapshot.adapter_plans.is_empty() {
                stack = stack.child(empty_row("No adapter plans in the latest receipt."));
            } else {
                for (index, plan) in snapshot
                    .adapter_plans
                    .iter()
                    .take(MAX_ADAPTER_PLAN_ROWS)
                    .enumerate()
                {
                    stack = stack.child(adapter_plan_row(index, plan));
                }
                if snapshot.adapter_plans.len() > MAX_ADAPTER_PLAN_ROWS {
                    stack = stack.child(overflow_row(
                        "dx-check-adapter-plan-overflow",
                        snapshot.adapter_plans.len() - MAX_ADAPTER_PLAN_ROWS,
                        "adapter plans",
                    ));
                }
            }
        }
        stack.into_any_element()
    }

    fn render_notices(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::Notices;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            if snapshot.blockers.is_empty() && snapshot.warnings.is_empty() {
                stack = stack.child(empty_row("No blockers or warnings in the latest receipt."));
            }
            for (index, blocker) in snapshot.blockers.iter().take(MAX_NOTICE_ROWS).enumerate() {
                stack = stack.child(notice_row(
                    format!("dx-check-blocker-{index}"),
                    IconName::Warning,
                    Color::Error,
                    &notice_title(blocker),
                    blocker.next_action.as_deref(),
                ));
            }
            if snapshot.blockers.len() > MAX_NOTICE_ROWS {
                stack = stack.child(overflow_row(
                    "dx-check-blocker-overflow",
                    snapshot.blockers.len() - MAX_NOTICE_ROWS,
                    "blockers",
                ));
            }
            for (index, warning) in snapshot.warnings.iter().take(MAX_NOTICE_ROWS).enumerate() {
                stack = stack.child(notice_row(
                    format!("dx-check-warning-{index}"),
                    IconName::Warning,
                    Color::Warning,
                    &notice_title(warning),
                    warning.next_action.as_deref(),
                ));
            }
            if snapshot.warnings.len() > MAX_NOTICE_ROWS {
                stack = stack.child(overflow_row(
                    "dx-check-warning-overflow",
                    snapshot.warnings.len() - MAX_NOTICE_ROWS,
                    "warnings",
                ));
            }
        }
        stack.into_any_element()
    }

    fn render_quick_fixes(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::QuickFixes;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            if snapshot.quick_fixes.is_empty() {
                stack = stack.child(empty_row("No quick fixes in the latest receipt."));
            } else {
                for (index, fix) in snapshot
                    .quick_fixes
                    .iter()
                    .take(MAX_QUICK_FIX_ROWS)
                    .enumerate()
                {
                    stack = stack.child(quick_fix_row(index, fix));
                }
                if snapshot.quick_fixes.len() > MAX_QUICK_FIX_ROWS {
                    stack = stack.child(overflow_row(
                        "dx-check-quick-fix-overflow",
                        snapshot.quick_fixes.len() - MAX_QUICK_FIX_ROWS,
                        "quick fixes",
                    ));
                }
            }
        }
        stack.into_any_element()
    }

    fn render_web_audits(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::WebAudit;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            if snapshot.web_audits.is_empty() {
                stack = stack.child(empty_row("No web-audit results in the latest receipt."));
            } else {
                for (index, audit) in snapshot
                    .web_audits
                    .iter()
                    .take(MAX_WEB_AUDIT_ROWS)
                    .enumerate()
                {
                    stack = stack.child(web_audit_row(index, audit, cx));
                }
                if snapshot.web_audits.len() > MAX_WEB_AUDIT_ROWS {
                    stack = stack.child(overflow_row(
                        "dx-check-web-audit-overflow",
                        snapshot.web_audits.len() - MAX_WEB_AUDIT_ROWS,
                        "web-audit rows",
                    ));
                }
            }
        }
        stack.into_any_element()
    }

    fn render_commands(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> AnyElement {
        let section_kind = DxCheckPanelSectionKind::Commands;
        let mut stack = self.render_section_shell(section_kind, snapshot, panel, cx);
        if self.section_is_open(section_kind) {
            stack = stack
                .child(detail_row("Refresh", snapshot.refresh_command.clone()))
                .child(detail_row("Next", snapshot.next_action.clone()));
            if let Some(detail_command) = snapshot.detail_command.as_ref() {
                stack = stack.child(detail_row("Details", detail_command.clone()));
            }
        }
        stack.into_any_element()
    }
}

impl Panel for DxCheckPanel {
    fn persistent_name() -> &'static str {
        "Check"
    }

    fn panel_key() -> &'static str {
        DX_CHECK_PANEL_KEY
    }

    fn position(&self, _: &Window, _: &App) -> DockPosition {
        DockPosition::Right
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        position == DockPosition::Right
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _: &Window, _: &App) -> Pixels {
        px(340.)
    }

    fn min_size(&self, _: &Window, _: &App) -> Option<Pixels> {
        Some(px(260.))
    }

    fn icon(&self, _: &Window, _: &App) -> Option<IconName> {
        Some(IconName::Check)
    }

    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some("Check")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(zed_actions::dx_check_panel::ToggleFocus)
    }

    fn activation_priority(&self) -> u32 {
        9
    }
}

impl Focusable for DxCheckPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for DxCheckPanel {}

impl Render for DxCheckPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot(cx);
        let panel = cx.entity().downgrade();
        let panel_id = cx.entity().entity_id();

        v_flex()
            .id("dx-check-panel")
            .key_context("DxCheckPanel")
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(&snapshot, panel_id, cx))
            .child(self.render_status_strip(&snapshot, panel.clone(), cx))
            .child(render_tab_bar(
                &snapshot,
                self.active_tab,
                panel.clone(),
                cx,
            ))
            .child(
                div()
                    .id("dx-check-panel-scroll-host")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .id("dx-check-panel-content")
                            .track_scroll(&self.scroll_handle)
                            .size_full()
                            .min_h_0()
                            .min_w_0()
                            .gap_1()
                            .px_1()
                            .py_1()
                            .overflow_y_scroll()
                            .children(self.render_active_tab_sections(&snapshot, panel, cx)),
                    )
                    .vertical_scrollbar_for(&self.scroll_handle, window, cx),
            )
    }
}

impl DxCheckPanel {
    fn render_active_tab_sections(
        &self,
        snapshot: &DxCheckPanelSnapshot,
        panel: WeakEntity<DxCheckPanel>,
        cx: &App,
    ) -> Vec<AnyElement> {
        match self.active_tab {
            DxCheckPanelTab::Overview => vec![
                self.render_summary(snapshot, panel.clone(), cx),
                self.render_sections(snapshot, panel, cx),
            ],
            DxCheckPanelTab::Findings => vec![
                self.render_notices(snapshot, panel.clone(), cx),
                self.render_quick_fixes(snapshot, panel.clone(), cx),
                self.render_web_audits(snapshot, panel.clone(), cx),
                self.render_adapter_plans(snapshot, panel, cx),
            ],
            DxCheckPanelTab::Receipt => vec![
                self.render_receipt(snapshot, panel.clone(), cx),
                self.render_commands(snapshot, panel, cx),
            ],
        }
    }
}

fn open_workspace_path(
    workspace: WeakEntity<Workspace>,
    path: PathBuf,
    window: &mut Window,
    cx: &mut App,
) {
    if !path.exists() {
        return;
    }

    workspace
        .update(cx, |workspace, cx| {
            workspace
                .open_abs_path(
                    path,
                    OpenOptions {
                        focus: Some(true),
                        ..Default::default()
                    },
                    window,
                    cx,
                )
                .detach_and_log_err(cx);
        })
        .ok();
}
