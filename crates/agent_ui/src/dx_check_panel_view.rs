use gpui::{
    AnyElement, App, AppContext as _, AsyncWindowContext, Context, Entity, EventEmitter,
    FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    ScrollHandle, Styled, WeakEntity, Window, div, px,
};
use theme::ActiveTheme;
use ui::prelude::*;
use workspace::{
    Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};

use crate::dx_check_panel::{DxCheckPanelSnapshot, dx_check_panel_snapshot};
use crate::dx_check_panel_view::view_rows::{
    config_label, count_label, detail_row, duration_label, empty_row, notice_row, notice_title,
    outcome_label, quick_fix_row, section, section_row, status_color,
};

mod view_rows;

const DX_CHECK_PANEL_KEY: &str = "DxCheckPanel";
const MAX_SECTION_ROWS: usize = 8;
const MAX_NOTICE_ROWS: usize = 4;
const MAX_QUICK_FIX_ROWS: usize = 4;

pub struct DxCheckPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
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

    fn render_header(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        h_flex()
            .p_3()
            .gap_2()
            .justify_between()
            .items_start()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .min_w_0()
                    .gap_2()
                    .items_center()
                    .child(
                        Icon::new(IconName::Check)
                            .size(IconSize::Small)
                            .color(status_color(snapshot)),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(Label::new("Check").size(LabelSize::Small))
                            .child(
                                Label::new(snapshot.title.clone())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted)
                                    .truncate(),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .items_end()
                    .gap_1()
                    .child(
                        Label::new(snapshot.score_label())
                            .size(LabelSize::XSmall)
                            .color(status_color(snapshot))
                            .truncate(),
                    )
                    .child(
                        Label::new(snapshot.status.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .into_any_element()
    }

    fn render_summary(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        section("Run", cx)
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
            ))
            .into_any_element()
    }

    fn render_receipt(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        let receipt_status = if snapshot.receipt_present {
            "present"
        } else {
            "missing"
        };
        let mut stack = section("Receipt", cx)
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

        stack.into_any_element()
    }

    fn render_sections(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        let mut stack = section("Sections", cx);
        if snapshot.sections.is_empty() {
            stack = stack.child(empty_row("No section scores in the latest receipt."));
        } else {
            for section in snapshot.sections.iter().take(MAX_SECTION_ROWS) {
                stack = stack.child(section_row(section));
            }
        }
        stack.into_any_element()
    }

    fn render_notices(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        let mut stack = section("Notices", cx);
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
        for (index, warning) in snapshot.warnings.iter().take(MAX_NOTICE_ROWS).enumerate() {
            stack = stack.child(notice_row(
                format!("dx-check-warning-{index}"),
                IconName::Warning,
                Color::Warning,
                &notice_title(warning),
                warning.next_action.as_deref(),
            ));
        }
        stack.into_any_element()
    }

    fn render_quick_fixes(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        let mut stack = section("Quick Fixes", cx);
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
        }
        stack.into_any_element()
    }

    fn render_commands(&self, snapshot: &DxCheckPanelSnapshot, cx: &App) -> AnyElement {
        let mut stack = section("Commands", cx)
            .child(detail_row("Refresh", snapshot.refresh_command.clone()))
            .child(detail_row("Next", snapshot.next_action.clone()));
        if let Some(detail_command) = snapshot.detail_command.as_ref() {
            stack = stack.child(detail_row("Details", detail_command.clone()));
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
        None
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

        v_flex()
            .id("dx-check-panel")
            .key_context("DxCheckPanel")
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(&snapshot, cx))
            .child(
                div()
                    .size_full()
                    .child(
                        v_flex()
                            .id("dx-check-panel-content")
                            .track_scroll(&self.scroll_handle)
                            .size_full()
                            .min_w_0()
                            .gap_2()
                            .p_2()
                            .overflow_y_scroll()
                            .child(self.render_summary(&snapshot, cx))
                            .child(self.render_receipt(&snapshot, cx))
                            .child(self.render_sections(&snapshot, cx))
                            .child(self.render_notices(&snapshot, cx))
                            .child(self.render_quick_fixes(&snapshot, cx))
                            .child(self.render_commands(&snapshot, cx)),
                    )
                    .vertical_scrollbar_for(&self.scroll_handle, window, cx),
            )
    }
}
