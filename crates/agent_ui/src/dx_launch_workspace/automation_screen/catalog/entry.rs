use gpui::AnyElement;
use ui::{AiSettingItemStatus, IconName, prelude::*};

use crate::dx_agent_bridge::{DxAgentAutomation, DxAgentBridgeSnapshot};

use super::AutomationCatalogFilter;
use super::details::{
    automation_details, automation_status, composer_contract_details, failure_details,
    runtime_details,
};

#[derive(Clone, Copy)]
pub(super) enum AutomationCatalogEntry {
    ComposerContract,
    RuntimeSummary,
    Automation(usize),
    FailureSummary,
}

pub(super) fn all_automation_entries(
    snapshot: &DxAgentBridgeSnapshot,
) -> Vec<AutomationCatalogEntry> {
    let mut entries = vec![
        AutomationCatalogEntry::ComposerContract,
        AutomationCatalogEntry::RuntimeSummary,
    ];
    entries.extend(
        snapshot
            .automations
            .iter()
            .enumerate()
            .map(|(index, _)| AutomationCatalogEntry::Automation(index)),
    );
    entries.push(AutomationCatalogEntry::FailureSummary);
    entries
}

impl AutomationCatalogEntry {
    pub(super) fn id(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            AutomationCatalogEntry::ComposerContract => "composer-contract".to_string(),
            AutomationCatalogEntry::RuntimeSummary => "runtime-summary".to_string(),
            AutomationCatalogEntry::FailureSummary => "failure-summary".to_string(),
            AutomationCatalogEntry::Automation(index) => snapshot
                .automations
                .get(index)
                .map(|automation| format!("automation-{}", automation.id))
                .unwrap_or_else(|| format!("automation-{index}")),
        }
    }

    pub(super) fn icon(self) -> IconName {
        match self {
            AutomationCatalogEntry::ComposerContract => dx_icon(DxUiIcon::Automations),
            AutomationCatalogEntry::RuntimeSummary => dx_icon(DxUiIcon::Gateway),
            AutomationCatalogEntry::FailureSummary => dx_icon(DxUiIcon::Permissions),
            AutomationCatalogEntry::Automation(_) => dx_icon(DxUiIcon::Automations),
        }
    }

    pub(super) fn title(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            AutomationCatalogEntry::ComposerContract => "Composer contract".to_string(),
            AutomationCatalogEntry::RuntimeSummary => "Runtime evidence".to_string(),
            AutomationCatalogEntry::FailureSummary => "Failure evidence".to_string(),
            AutomationCatalogEntry::Automation(index) => snapshot
                .automations
                .get(index)
                .map(|automation| automation.name.clone())
                .unwrap_or_else(|| "Automation".to_string()),
        }
    }

    pub(super) fn detail_label(self, snapshot: &DxAgentBridgeSnapshot) -> String {
        match self {
            AutomationCatalogEntry::ComposerContract => snapshot.automation_composer.status.clone(),
            AutomationCatalogEntry::RuntimeSummary => format!(
                "{} active / {} blocked tools",
                snapshot.active_task_count, snapshot.trusted_tool_bridge.blocked_tool_count
            ),
            AutomationCatalogEntry::FailureSummary => format!(
                "{} failed proof rows",
                snapshot
                    .automations
                    .iter()
                    .filter(|automation| automation.has_failed_execution_proof())
                    .count()
            ),
            AutomationCatalogEntry::Automation(index) => snapshot
                .automations
                .get(index)
                .map(|automation| automation.status.state.clone())
                .unwrap_or_else(|| "missing automation".to_string()),
        }
    }

    pub(super) fn status(self, snapshot: &DxAgentBridgeSnapshot) -> AiSettingItemStatus {
        match self {
            AutomationCatalogEntry::ComposerContract => {
                if snapshot.automation_composer.runtime_available {
                    AiSettingItemStatus::Running
                } else if snapshot.automation_composer.receipt_present {
                    AiSettingItemStatus::Stopped
                } else {
                    AiSettingItemStatus::AuthRequired
                }
            }
            AutomationCatalogEntry::RuntimeSummary => {
                if snapshot.active_task_count > 0 {
                    AiSettingItemStatus::Running
                } else {
                    AiSettingItemStatus::Stopped
                }
            }
            AutomationCatalogEntry::FailureSummary => {
                if snapshot
                    .automations
                    .iter()
                    .any(|automation| automation.has_failed_execution_proof())
                {
                    AiSettingItemStatus::Error
                } else {
                    AiSettingItemStatus::Running
                }
            }
            AutomationCatalogEntry::Automation(index) => snapshot
                .automations
                .get(index)
                .map(automation_status)
                .unwrap_or(AiSettingItemStatus::Stopped),
        }
    }

    pub(super) fn detail_body(self, snapshot: &DxAgentBridgeSnapshot) -> AnyElement {
        match self {
            AutomationCatalogEntry::ComposerContract => composer_contract_details(snapshot),
            AutomationCatalogEntry::RuntimeSummary => runtime_details(snapshot),
            AutomationCatalogEntry::FailureSummary => failure_details(snapshot),
            AutomationCatalogEntry::Automation(index) => snapshot
                .automations
                .get(index)
                .map(automation_details)
                .unwrap_or_else(|| v_flex().into_any_element()),
        }
    }

    pub(super) fn matches_filter(
        self,
        snapshot: &DxAgentBridgeSnapshot,
        filter: AutomationCatalogFilter,
    ) -> bool {
        match filter {
            AutomationCatalogFilter::All => true,
            AutomationCatalogFilter::Enabled => self
                .automation(snapshot)
                .is_some_and(|automation| automation.status.enabled),
            AutomationCatalogFilter::NeedsRuntime => match self {
                AutomationCatalogEntry::ComposerContract => {
                    !snapshot.automation_composer.runtime_available
                }
                _ => self
                    .automation(snapshot)
                    .is_some_and(|automation| !automation.status.runtime_available),
            },
            AutomationCatalogFilter::Failed => match self {
                AutomationCatalogEntry::FailureSummary => snapshot
                    .automations
                    .iter()
                    .any(|automation| automation.has_failed_execution_proof()),
                _ => self
                    .automation(snapshot)
                    .is_some_and(|automation| automation.has_failed_execution_proof()),
            },
        }
    }

    pub(super) fn matches_search(self, snapshot: &DxAgentBridgeSnapshot, query: &str) -> bool {
        let mut haystack = vec![self.title(snapshot), self.detail_label(snapshot)];
        if let Some(automation) = self.automation(snapshot) {
            haystack.extend([
                automation.id.clone(),
                automation.prompt.clone(),
                automation.source.clone(),
                automation.schedule.summary.clone(),
                automation.status.state.clone(),
                automation.destination.label.clone(),
                automation.next_action.clone(),
            ]);
        }
        haystack
            .iter()
            .any(|value| value.to_lowercase().contains(query))
    }

    fn automation<'a>(self, snapshot: &'a DxAgentBridgeSnapshot) -> Option<&'a DxAgentAutomation> {
        match self {
            AutomationCatalogEntry::Automation(index) => snapshot.automations.get(index),
            _ => None,
        }
    }
}
