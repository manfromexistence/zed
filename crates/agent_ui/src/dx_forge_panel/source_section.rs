use gpui::{AnyElement, App, SharedString, WeakEntity};
use ui::{IconName, prelude::*};
use workspace::Workspace;

use super::{
    controls::open_exact_abs_path_button,
    panel::DxForgePanel,
    rows::{empty_row, section_header},
    snapshot::{DxForgePanelSnapshot, DxForgeSourceRow},
    workflow_rows::selectable_source_row,
};

pub(super) struct SourceSection {
    pub(super) header_id: &'static str,
    pub(super) title: &'static str,
    pub(super) icon: IconName,
    pub(super) empty_id: &'static str,
    pub(super) workspace_empty: &'static str,
    pub(super) empty: &'static str,
    pub(super) row_id: &'static str,
    pub(super) open_id: &'static str,
    pub(super) open_tooltip: &'static str,
}

pub(super) fn source_section(
    section: SourceSection,
    rows: &[DxForgeSourceRow],
    snapshot: &DxForgePanelSnapshot,
    workspace: &WeakEntity<Workspace>,
    panel: &WeakEntity<DxForgePanel>,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex().w_full().min_w_0().child(section_header(
        section.header_id,
        section.title,
        section.icon,
        rows.len(),
        cx,
    ));

    if snapshot.workspace_roots.is_empty() {
        stack = stack.child(empty_row(section.empty_id, section.workspace_empty, cx));
    } else if rows.is_empty() {
        stack = stack.child(empty_row(section.empty_id, section.empty, cx));
    } else {
        for (ix, row) in rows.iter().enumerate() {
            stack = stack.child(selectable_source_row(
                SharedString::from(format!("{}-{ix}", section.row_id)),
                section.icon,
                row,
                panel,
                Some(open_exact_abs_path_button(
                    format!("{}-{ix}", section.open_id),
                    section.open_tooltip,
                    &row.open_path,
                    workspace,
                )),
                cx,
            ));
        }
    }

    stack.into_any_element()
}
