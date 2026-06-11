use gpui::{AnyElement, App, SharedString, prelude::*};
use ui::prelude::*;

use crate::dx_agent_bridge::DxAgentBridgeSnapshot;

use self::rows::{dx_agent_model_row, dx_agent_provider_row};
use self::summary::dx_agent_provider_summary_rows;
use super::super::screen_chrome::{screen_detail_row, screen_empty_state};

mod rows;
mod summary;

pub(super) const VISIBLE_PROVIDER_ROW_LIMIT: usize = 3;
pub(super) const VISIBLE_MODEL_ROW_LIMIT: usize = 2;

pub(in super::super) fn dx_agent_provider_state(
    snapshot: &DxAgentBridgeSnapshot,
    cx: &App,
) -> AnyElement {
    let mut stack = v_flex()
        .gap_1()
        .children(dx_agent_provider_summary_rows(snapshot));

    if !snapshot.show_managed_providers {
        return stack
            .child(screen_empty_state(
                "dx-agent-providers-hidden",
                dx_icon(DxUiIcon::Gateway),
                "Managed provider rows hidden by settings",
                cx,
            ))
            .into_any_element();
    }

    if let Some(source_hash) = snapshot.catalog.source_hash.as_ref() {
        stack = stack.child(screen_detail_row(
            "dx-agent-provider-source-hash".into(),
            dx_icon(DxUiIcon::Receipts),
            "Source hash",
            source_hash.clone(),
        ));
    }
    if let Some(error) = snapshot.catalog.error.as_ref() {
        stack = stack.child(screen_detail_row(
            "dx-agent-provider-catalog-error".into(),
            IconName::Warning,
            "Catalog error",
            error.clone(),
        ));
    }

    if snapshot.providers.is_empty() {
        stack = stack.child(screen_empty_state(
            "dx-agent-providers-empty",
            dx_icon(DxUiIcon::Gateway),
            format!("Run {}", snapshot.catalog.safe_regeneration_command),
            cx,
        ));
    } else {
        for (ix, provider) in snapshot
            .providers
            .iter()
            .take(VISIBLE_PROVIDER_ROW_LIMIT)
            .enumerate()
        {
            stack = stack.child(dx_agent_provider_row(
                SharedString::from(format!("dx-agent-provider-{ix}")),
                provider,
                cx,
            ));
        }
    }

    for (ix, model) in snapshot
        .models
        .iter()
        .take(VISIBLE_MODEL_ROW_LIMIT)
        .enumerate()
    {
        stack = stack.child(dx_agent_model_row(
            SharedString::from(format!("dx-agent-model-{ix}")),
            model,
            cx,
        ));
    }

    stack.into_any_element()
}
