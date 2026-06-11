use gpui::{AnyElement, Context, IntoElement, Window};
use ui::prelude::*;

use crate::AgentPanel;

use super::DxLaunchWorkspaceStatus;
use super::screen_chrome::{
    screen_empty_state, screen_section, workspace_page_header, workspace_stat,
};

mod catalog;
mod sections;

pub(crate) use catalog::{
    AutomationCatalogFilter, DxAutomationCatalogState, render_automation_catalog_rows,
};

pub(crate) fn render_automation_screen(
    status: Option<&DxLaunchWorkspaceStatus>,
    state: &mut DxAutomationCatalogState,
    window: &mut Window,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let (header_stats, body) = if let Some(status) = status {
        let snapshot = &status.agent_bridge;
        (
            vec![
                workspace_stat(
                    "dx-automation-stat-drafts",
                    "Composer",
                    snapshot.automation_composer.status.clone(),
                    cx,
                ),
                workspace_stat(
                    "dx-automation-stat-rows",
                    "Rows",
                    snapshot.automations.len().to_string(),
                    cx,
                ),
                workspace_stat(
                    "dx-automation-stat-active",
                    "Active tasks",
                    snapshot.active_task_count.to_string(),
                    cx,
                ),
                workspace_stat(
                    "dx-automation-stat-blocked",
                    "Blocked tools",
                    snapshot.trusted_tool_bridge.blocked_tool_count.to_string(),
                    cx,
                ),
            ],
            v_flex()
                .gap_3()
                .child(catalog::render_automation_catalog(
                    snapshot, state, window, cx,
                ))
                .child(screen_section(
                    "dx-automation-drafts",
                    "Drafts",
                    dx_icon(DxUiIcon::Automations),
                    snapshot.automation_composer.field_summary_label(),
                    sections::drafts_state(snapshot, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-automation-schedules",
                    "Schedules",
                    dx_icon(DxUiIcon::Automations),
                    format!("{} schedule rows", snapshot.automations.len()),
                    sections::schedules_state(snapshot, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-automation-runs",
                    "Runs",
                    dx_icon(DxUiIcon::Automations),
                    format!("{} active tasks", snapshot.active_task_count),
                    sections::runs_state(snapshot, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-automation-history",
                    "History",
                    dx_icon(DxUiIcon::Receipts),
                    sections::history_summary(snapshot),
                    sections::history_state(snapshot, cx),
                    cx,
                ))
                .child(screen_section(
                    "dx-automation-failures",
                    "Failures",
                    dx_icon(DxUiIcon::Permissions),
                    sections::failure_summary(snapshot),
                    sections::failures_state(snapshot, cx),
                    cx,
                ))
                .into_any_element(),
        )
    } else {
        (
            vec![workspace_stat(
                "dx-automation-stat-loading",
                "State",
                "Loading",
                cx,
            )],
            screen_empty_state(
                "dx-automation-loading",
                dx_icon(DxUiIcon::Receipts),
                "Loading automation receipts and composer contract",
                cx,
            ),
        )
    };

    div()
        .id("dx-automation-screen")
        .size_full()
        .min_w_0()
        .overflow_y_scroll()
        .bg(cx.theme().colors().panel_background)
        .child(
            v_flex()
                .gap_3()
                .p_4()
                .child(workspace_page_header(
                    dx_icon(DxUiIcon::Automations),
                    "Automations",
                    "Composer receipt state, schedule contracts, history, and handoff evidence.",
                    header_stats,
                    cx,
                ))
                .child(body),
        )
        .into_any_element()
}
