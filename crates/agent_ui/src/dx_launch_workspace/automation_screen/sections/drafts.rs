use gpui::{AnyElement, App, prelude::*};
use ui::{AiSettingItem, AiSettingItemSource, AiSettingItemStatus, IconName, prelude::*};

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use super::super::super::screen_chrome::{screen_detail_row, screen_detail_stack};

pub(crate) fn drafts_state(snapshot: &DxAgentBridgeSnapshot, _cx: &App) -> AnyElement {
    let composer = &snapshot.automation_composer;
    let status = if composer.save_draft_available {
        AiSettingItemStatus::Running
    } else if composer.runtime_available {
        AiSettingItemStatus::Starting
    } else {
        AiSettingItemStatus::Stopped
    };
    let field_summary = composer.field_summary(5);

    AiSettingItem::new(
        "dx-automation-drafts-composer",
        "Draft composer",
        status,
        AiSettingItemSource::Custom,
    )
    .icon(
        Icon::new(dx_icon(DxUiIcon::Automations))
            .size(IconSize::Small)
            .color(Color::Muted),
    )
    .detail_label(match status {
        AiSettingItemStatus::Running => "Ready",
        AiSettingItemStatus::Starting => "Runtime ready",
        _ => "Unavailable",
    })
    .details(screen_detail_stack(vec![
        screen_detail_row(
            "dx-automation-drafts-status".into(),
            IconName::Server,
            "Status",
            composer.status.clone(),
        ),
        screen_detail_row(
            "dx-automation-drafts-fields".into(),
            IconName::TextSnippet,
            composer.field_summary_label(),
            if field_summary.is_empty() {
                composer.empty_field_summary_label().to_string()
            } else {
                field_summary
            },
        ),
        screen_detail_row(
            "dx-automation-drafts-receipt".into(),
            dx_icon(DxUiIcon::Receipts),
            "Receipt",
            composer.receipt_filename.clone(),
        ),
        screen_detail_row(
            "dx-automation-drafts-action".into(),
            IconName::FileTextOutlined,
            "Next action",
            composer.next_action.clone(),
        ),
    ]))
    .into_any_element()
}
