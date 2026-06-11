use gpui::{Action, AnyElement, App, SharedString, prelude::*};
use ui::{IconName, Tooltip, prelude::*};
use zed_actions::dx_style::OpenGeneratorPreview;

use crate::dx_style_panel::{DxStylePanelRow, DxStylePanelSnapshot};

use super::bounded_items;

mod rows;

use rows::{style_detail_row, style_note_row, style_path_row, style_section};

pub(super) fn dx_style_panel_state(snapshot: &DxStylePanelSnapshot, _cx: &App) -> AnyElement {
    let web_preview_state = if snapshot.web_preview_bridge_ready {
        "Bridge ready"
    } else if snapshot.web_preview_host_present {
        "Bridge available"
    } else {
        "Bridge missing"
    };

    let mut stack = v_flex()
        .gap_1()
        .child(style_section(
            "dx-style-overview-section",
            "Style",
            dx_icon(DxUiIcon::Style),
            snapshot.status.clone(),
        ))
        .child(style_detail_row(
            "dx-style-next-action",
            "Next Action",
            snapshot.next_action.clone(),
            IconName::ListTodo,
            Color::Muted,
        ))
        .child(style_detail_row(
            "dx-style-root",
            "Root",
            if snapshot.root_exists {
                snapshot.root.display().to_string()
            } else {
                "missing".to_string()
            },
            IconName::Folder,
            if snapshot.root_exists {
                Color::Muted
            } else {
                Color::Warning
            },
        ))
        .child(style_detail_row(
            "dx-style-generators",
            "Generators",
            format!("{} declared", snapshot.visual_generator_count),
            IconName::Sparkle,
            Color::Muted,
        ))
        .child(style_detail_row(
            "dx-style-web-preview",
            "Web Preview",
            web_preview_state,
            IconName::Public,
            if snapshot.web_preview_bridge_ready {
                Color::Accent
            } else {
                Color::Muted
            },
        ))
        .child(style_detail_row(
            "dx-style-readiness",
            "Readiness",
            snapshot.readiness.status.clone(),
            IconName::ListTodo,
            Color::Muted,
        ))
        .child(style_note_row(
            "dx-style-readiness-summary",
            IconName::Info,
            Color::Muted,
            snapshot.readiness.summary.clone(),
        ))
        .child(style_detail_row(
            "dx-style-readiness-files",
            "Readiness Files",
            format!(
                "docs {} / {}, contracts {} / {}, fixtures {} / {}, artifacts {} / {}",
                snapshot.readiness.docs_ready,
                snapshot.readiness.docs_expected,
                snapshot.readiness.contracts_ready,
                snapshot.readiness.contracts_expected,
                snapshot.readiness.fixtures_ready,
                snapshot.readiness.fixtures_expected,
                snapshot.readiness.artifacts_ready,
                snapshot.readiness.artifacts_expected
            ),
            IconName::FileCode,
            Color::Muted,
        ))
        .child(
            Button::new("dx-style-open-generator-preview", "Open Style Controls")
                .full_width()
                .label_size(LabelSize::Small)
                .color(Color::Muted)
                .start_icon(Icon::new(dx_icon(DxUiIcon::Style)).size(IconSize::Small))
                .disabled(!snapshot.web_preview_bridge_ready)
                .tooltip(Tooltip::text(if snapshot.web_preview_bridge_ready {
                    "Open the DX Style Web Preview controls"
                } else {
                    "DX Style Web Preview bridge is not ready"
                }))
                .on_click(|_, window, cx| {
                    window.dispatch_action(OpenGeneratorPreview.boxed_clone(), cx);
                }),
        );

    if !snapshot.root_exists {
        stack = stack.child(style_note_row(
            "dx-style-root-missing",
            IconName::Warning,
            Color::Warning,
            format!("Missing dx-style root: {}", snapshot.root.display()),
        ));
    } else if !snapshot.grouped_contract_ready {
        stack = stack.child(style_note_row(
            "dx-style-contract-warning",
            IconName::Warning,
            Color::Warning,
            "Grouped-class contract is not available for editor writes",
        ));
    }

    stack = stack
        .child(style_section(
            "dx-style-contract-section",
            "Contracts",
            IconName::FileTextOutlined,
            format!(
                "{} / {}",
                snapshot.readiness.contracts_ready, snapshot.readiness.contracts_expected
            ),
        ))
        .child(style_contract_row(
            "Plan Contract",
            snapshot.plan_present,
            &snapshot.plan_path,
        ))
        .child(style_contract_row(
            "Grouping Contract",
            snapshot.grouped_contract_present,
            &snapshot.grouped_contract_path,
        ))
        .child(style_contract_row(
            "Control Catalog",
            snapshot.generator_catalog_present,
            &snapshot.generator_catalog_path,
        ))
        .child(style_contract_row(
            "Editor Contract",
            snapshot.editor_contract_present,
            &snapshot.editor_contract_path,
        ))
        .child(style_contract_row(
            "Preview Bridge",
            snapshot.web_preview_host_present,
            &snapshot.web_preview_host_path,
        ));

    stack = stack.child(style_section(
        "dx-style-source-rows-section",
        "Source Rows",
        IconName::ListTree,
        format!("{} rows", snapshot.rows.len()),
    ));
    for (ix, row) in snapshot.rows.iter().take(7).enumerate() {
        stack = stack.child(dx_style_row(
            SharedString::from(format!("dx-style-row-{ix}")),
            row,
        ));
    }
    if snapshot.rows.len() > 7 {
        stack = stack.child(style_note_row(
            "dx-style-source-rows-overflow",
            IconName::Ellipsis,
            Color::Muted,
            format!("{} more source rows not shown", snapshot.rows.len() - 7),
        ));
    }

    stack = stack
        .child(style_section(
            "dx-style-readiness-section",
            "Readiness",
            IconName::Archive,
            format!("{} receipts", snapshot.readiness.receipt_count),
        ))
        .child(style_detail_row(
            "dx-style-contract-rows",
            "Contracts",
            bounded_items(
                &snapshot.readiness.contract_rows,
                3,
                "No DX Style contract rows",
            ),
            IconName::FileTextOutlined,
            Color::Muted,
        ))
        .child(style_detail_row(
            "dx-style-fixture-rows",
            "Fixtures",
            bounded_items(
                &snapshot.readiness.fixture_rows,
                3,
                "No DX Style fixture rows",
            ),
            IconName::FileCode,
            Color::Muted,
        ))
        .child(style_detail_row(
            "dx-style-receipt-roots",
            "Receipt Roots",
            bounded_items(
                &snapshot.readiness.receipt_rows,
                2,
                "No DX Style receipt roots",
            ),
            IconName::Folder,
            Color::Muted,
        ))
        .child(style_detail_row(
            "dx-style-readiness-next",
            "Readiness Next",
            snapshot.readiness.next_action.clone(),
            IconName::ListTodo,
            Color::Muted,
        ));

    if snapshot.readiness.receipt_count == 0 {
        stack = stack.child(style_note_row(
            "dx-style-receipts-empty",
            IconName::Info,
            Color::Muted,
            "No DX Style build/check receipt has been read by Zed.",
        ));
    }

    if !snapshot.readiness.missing_rows.is_empty() {
        stack = stack.child(style_detail_row(
            "dx-style-missing-readiness",
            "Missing",
            bounded_items(
                &snapshot.readiness.missing_rows,
                3,
                "No missing DX Style readiness files",
            ),
            IconName::Warning,
            Color::Warning,
        ));
    }

    for (ix, warning) in snapshot.warnings.iter().take(2).enumerate() {
        stack = stack.child(style_note_row(
            SharedString::from(format!("dx-style-warning-{ix}")),
            IconName::Info,
            Color::Muted,
            warning.clone(),
        ));
    }

    stack.into_any_element()
}

fn style_contract_row(label: &'static str, present: bool, path: &std::path::Path) -> AnyElement {
    style_path_row(
        SharedString::from(format!("dx-style-contract-{}", label.replace(' ', "-"))),
        label,
        present,
        path,
    )
}

fn dx_style_row(id: SharedString, row: &DxStylePanelRow) -> AnyElement {
    style_detail_row(
        id,
        row.label.clone(),
        format!("{} - {}", row.state, row.detail),
        IconName::FileTextOutlined,
        Color::Muted,
    )
}
