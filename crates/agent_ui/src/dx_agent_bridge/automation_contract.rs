use serde_json::Value;

use super::automation_actions::{automation_composer_actions, automation_row_actions};
use super::{DxAgentRowAction, array_field, bool_field, safe_string_field};

const MAX_AUTOMATION_TEXT_CHARS: usize = 180;
const MAX_AUTOMATION_LIST_ITEMS: usize = 6;

#[derive(Clone)]
pub(crate) struct DxAgentAutomation {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub source: String,
    pub schedule: DxAgentAutomationSchedule,
    pub status: DxAgentAutomationStatus,
    pub destination: DxAgentAutomationDestination,
    pub last_run: String,
    pub next_run: String,
    pub receipts: Vec<DxAgentAutomationReceiptRef>,
    pub history: Vec<DxAgentAutomationHistoryEntry>,
    pub actions: Vec<DxAgentRowAction>,
    pub next_action: String,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationComposer {
    pub schema_version: String,
    pub status: String,
    pub receipt_present: bool,
    pub runtime_available: bool,
    pub save_draft_available: bool,
    pub enable_available: bool,
    pub receipt_filename: String,
    pub command: String,
    pub next_action: String,
    pub unavailable_reason: String,
    pub actions: Vec<DxAgentRowAction>,
    pub fields_receipt_backed: bool,
    pub fields: Vec<DxAgentAutomationComposerField>,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationComposerField {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub required: bool,
    pub value: String,
    pub placeholder: String,
    pub status: String,
}

impl DxAgentAutomationComposer {
    pub(crate) fn field_summary(&self, limit: usize) -> String {
        self.fields
            .iter()
            .take(limit)
            .map(|field| {
                if field.required {
                    format!("{}*", field.label)
                } else {
                    field.label.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn field_summary_label(&self) -> &'static str {
        if self.fields_receipt_backed {
            "Receipt fields"
        } else if self.receipt_present {
            "Field template"
        } else {
            "Field template pending receipt"
        }
    }

    pub(crate) fn empty_field_summary_label(&self) -> &'static str {
        if self.fields_receipt_backed {
            "No composer fields in receipt"
        } else {
            "No composer field template until receipt"
        }
    }
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationSchedule {
    pub kind: String,
    pub summary: String,
    pub rrule: String,
    pub timezone: String,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationStatus {
    pub state: String,
    pub enabled: bool,
    pub runtime_available: bool,
    pub unavailable_reason: String,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationDestination {
    pub kind: String,
    pub label: String,
    pub target: String,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationReceiptRef {
    pub kind: String,
    pub schema_version: String,
    pub status: String,
    pub path: String,
}

#[derive(Clone)]
pub(crate) struct DxAgentAutomationHistoryEntry {
    pub run_id: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub receipt_path: String,
}

impl DxAgentAutomation {
    pub(crate) fn has_successful_execution_proof(&self) -> bool {
        self.receipts
            .iter()
            .any(|receipt| receipt.has_automation_run_status(is_success_status))
            || self
                .history
                .iter()
                .any(|entry| entry.has_receipted_status(is_success_status))
    }

    pub(crate) fn has_failed_execution_proof(&self) -> bool {
        self.receipts
            .iter()
            .any(|receipt| receipt.has_automation_run_status(is_failed_status))
            || self
                .history
                .iter()
                .any(|entry| entry.has_receipted_status(is_failed_status))
    }
}

impl DxAgentAutomationReceiptRef {
    fn has_automation_run_status(&self, predicate: fn(&str) -> bool) -> bool {
        !self.path.is_empty() && self.is_automation_run_receipt() && predicate(&self.status)
    }

    fn is_automation_run_receipt(&self) -> bool {
        self.kind.eq_ignore_ascii_case("automation_run")
            || self
                .schema_version
                .to_ascii_lowercase()
                .starts_with("dx.agents.zed.automation_run.")
    }
}

impl DxAgentAutomationHistoryEntry {
    fn has_receipted_status(&self, predicate: fn(&str) -> bool) -> bool {
        !self.receipt_path.is_empty() && predicate(&self.status)
    }
}

fn is_success_status(status: &str) -> bool {
    matches!(
        normalized_status(status).as_str(),
        "passed" | "pass" | "success" | "succeeded" | "complete" | "completed" | "ok"
    )
}

fn is_failed_status(status: &str) -> bool {
    matches!(
        normalized_status(status).as_str(),
        "failed" | "failure" | "error" | "errored" | "blocked" | "cancelled" | "canceled"
    )
}

fn normalized_status(status: &str) -> String {
    status.trim().to_ascii_lowercase()
}

pub(super) fn automation_composer(
    value: Option<&Value>,
    root_exists: bool,
) -> DxAgentAutomationComposer {
    let receipt_present = value.is_some();
    let runtime_available = value
        .and_then(|value| bool_field(value, &["runtime_available"]))
        .or_else(|| {
            value
                .and_then(|value| value.get("runtime"))
                .and_then(|runtime| bool_field(runtime, &["available"]))
        })
        .unwrap_or(false);
    let actions = value.map(automation_composer_actions).unwrap_or_default();
    let save_draft_available = runtime_available
        && actions
            .iter()
            .any(|action| action.id == "save_draft" && action.enabled);
    let enable_available = runtime_available
        && actions
            .iter()
            .any(|action| action.id == "enable" && action.enabled);
    let fields = value
        .and_then(|value| array_field(value, &["fields"]))
        .map(|fields| fields.iter().take(12).filter_map(composer_field).collect());
    let fields_receipt_backed = fields.is_some();

    DxAgentAutomationComposer {
        schema_version: value
            .and_then(|value| automation_text_field(value, &["schema_version"]))
            .unwrap_or_else(|| "dx.agents.zed.automation_composer.v1".to_string()),
        status: value
            .and_then(|value| automation_text_field(value, &["status"]))
            .unwrap_or_else(|| {
                if root_exists {
                    "waiting_for_automation_composer_contract".to_string()
                } else {
                    "missing_receipt_root".to_string()
                }
            }),
        receipt_present,
        runtime_available,
        save_draft_available,
        enable_available,
        receipt_filename: value
            .and_then(|value| automation_text_field(value, &["receipt_filename"]))
            .unwrap_or_else(|| "automate-composer-latest.json".to_string()),
        command: value
            .and_then(|value| automation_text_field(value, &["command"]))
            .unwrap_or_else(|| "dx agents automate composer --json".to_string()),
        next_action: value
            .and_then(|value| automation_text_field(value, &["next_action"]))
            .unwrap_or_else(|| {
                "Wait for DX Agents to emit an automation composer contract receipt.".to_string()
            }),
        unavailable_reason: value
            .and_then(|value| automation_text_field(value, &["unavailable_reason"]))
            .or_else(|| {
                value
                    .and_then(|value| value.get("runtime"))
                    .and_then(|runtime| automation_text_field(runtime, &["unavailable_reason"]))
            })
            .unwrap_or_else(|| {
                if root_exists {
                    "DX Agents has not exposed a typed automation create/update runtime yet."
                        .to_string()
                } else {
                    "DX Agents receipt root is missing.".to_string()
                }
            }),
        actions,
        fields_receipt_backed,
        fields: fields.unwrap_or_else(default_composer_fields),
    }
}

pub(super) fn automations(value: &Value) -> Vec<DxAgentAutomation> {
    array_field(value, &["automations"])
        .map(|automations| automations.iter().take(12).map(automation_row).collect())
        .unwrap_or_default()
}

fn automation_row(automation: &Value) -> DxAgentAutomation {
    let id = automation_text_field(automation, &["id"]).unwrap_or_else(|| "automation".to_string());
    let name = automation_text_field(automation, &["name"])
        .or_else(|| automation_text_field(automation, &["display_name"]))
        .unwrap_or_else(|| id.clone());
    let source =
        automation_text_field(automation, &["source"]).unwrap_or_else(|| "unknown".to_string());
    let schedule = automation_schedule(automation);
    let status = automation_status(automation);
    let destination = automation_destination(automation);
    let actions = automation_row_actions(automation, &id);

    DxAgentAutomation {
        id,
        name,
        prompt: automation_text_field(automation, &["prompt"])
            .or_else(|| automation_text_field(automation, &["prompt_preview"]))
            .unwrap_or_default(),
        source,
        schedule,
        status,
        destination,
        last_run: automation_text_field(automation, &["last_run"])
            .or_else(|| automation_text_field(automation, &["last_run_at"]))
            .unwrap_or_else(|| "never".to_string()),
        next_run: automation_text_field(automation, &["next_run"])
            .or_else(|| automation_text_field(automation, &["next_run_at"]))
            .unwrap_or_else(|| "pending runtime".to_string()),
        receipts: automation_receipts(automation),
        history: automation_history(automation),
        actions,
        next_action: automation_text_field(automation, &["next_action"]).unwrap_or_default(),
    }
}

fn automation_schedule(automation: &Value) -> DxAgentAutomationSchedule {
    let schedule = automation.get("schedule");
    let kind = schedule
        .and_then(|schedule| automation_text_field(schedule, &["kind"]))
        .or_else(|| automation_text_field(automation, &["schedule_kind"]))
        .unwrap_or_else(|| "unknown".to_string());

    DxAgentAutomationSchedule {
        summary: schedule
            .and_then(|schedule| automation_text_field(schedule, &["summary"]))
            .or_else(|| automation_text_field(automation, &["schedule"]))
            .or_else(|| automation_text_field(automation, &["schedule_label"]))
            .unwrap_or_else(|| kind.clone()),
        rrule: schedule
            .and_then(|schedule| automation_text_field(schedule, &["rrule"]))
            .unwrap_or_default(),
        timezone: schedule
            .and_then(|schedule| automation_text_field(schedule, &["timezone"]))
            .or_else(|| automation_text_field(automation, &["timezone"]))
            .unwrap_or_default(),
        kind,
    }
}

fn automation_status(automation: &Value) -> DxAgentAutomationStatus {
    let runtime = automation.get("runtime");
    let enabled = bool_field(automation, &["enabled"]).unwrap_or(false);
    let runtime_available = runtime
        .and_then(|runtime| bool_field(runtime, &["available"]))
        .or_else(|| bool_field(automation, &["runtime_available"]))
        .unwrap_or(false);

    DxAgentAutomationStatus {
        state: automation_text_field(automation, &["status"])
            .unwrap_or_else(|| "unknown".to_string()),
        enabled,
        runtime_available,
        unavailable_reason: runtime
            .and_then(|runtime| automation_text_field(runtime, &["unavailable_reason"]))
            .or_else(|| automation_text_field(automation, &["unavailable_reason"]))
            .unwrap_or_else(|| {
                if runtime_available {
                    String::new()
                } else if enabled {
                    "Runtime execution contract pending".to_string()
                } else {
                    "Automation is paused or unavailable".to_string()
                }
            }),
    }
}

fn automation_destination(automation: &Value) -> DxAgentAutomationDestination {
    let destination = automation.get("destination");

    DxAgentAutomationDestination {
        kind: destination
            .and_then(|destination| automation_text_field(destination, &["kind"]))
            .or_else(|| automation_text_field(automation, &["destination_kind"]))
            .unwrap_or_else(|| "workspace".to_string()),
        label: destination
            .and_then(|destination| automation_text_field(destination, &["label"]))
            .or_else(|| automation_text_field(automation, &["destination_label"]))
            .or_else(|| automation_text_field(automation, &["destination"]))
            .unwrap_or_else(|| "Current workspace".to_string()),
        target: destination
            .and_then(|destination| automation_text_field(destination, &["target"]))
            .or_else(|| automation_text_field(automation, &["destination_target"]))
            .unwrap_or_default(),
    }
}

fn automation_receipts(automation: &Value) -> Vec<DxAgentAutomationReceiptRef> {
    array_field(automation, &["receipts"])
        .map(|receipts| {
            receipts
                .iter()
                .take(MAX_AUTOMATION_LIST_ITEMS)
                .map(|receipt| DxAgentAutomationReceiptRef {
                    kind: automation_text_field(receipt, &["kind"])
                        .unwrap_or_else(|| "automation".to_string()),
                    schema_version: automation_text_field(receipt, &["schema_version"])
                        .unwrap_or_default(),
                    status: automation_text_field(receipt, &["status"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    path: automation_text_field(receipt, &["path"])
                        .or_else(|| automation_text_field(receipt, &["receipt_path"]))
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn automation_history(automation: &Value) -> Vec<DxAgentAutomationHistoryEntry> {
    array_field(automation, &["history"])
        .map(|history| {
            history
                .iter()
                .take(MAX_AUTOMATION_LIST_ITEMS)
                .map(|entry| DxAgentAutomationHistoryEntry {
                    run_id: automation_text_field(entry, &["run_id"]).unwrap_or_default(),
                    status: automation_text_field(entry, &["status"])
                        .unwrap_or_else(|| "unknown".to_string()),
                    started_at: automation_text_field(entry, &["started_at"]).unwrap_or_default(),
                    finished_at: automation_text_field(entry, &["finished_at"]).unwrap_or_default(),
                    receipt_path: automation_text_field(entry, &["receipt_path"])
                        .or_else(|| automation_text_field(entry, &["path"]))
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn composer_field(field: &Value) -> Option<DxAgentAutomationComposerField> {
    Some(DxAgentAutomationComposerField {
        id: automation_text_field(field, &["id"])?,
        label: automation_text_field(field, &["label"]).unwrap_or_else(|| "Field".to_string()),
        kind: automation_text_field(field, &["kind"]).unwrap_or_else(|| "text".to_string()),
        required: bool_field(field, &["required"]).unwrap_or(false),
        value: automation_text_field(field, &["value"]).unwrap_or_default(),
        placeholder: automation_text_field(field, &["placeholder"]).unwrap_or_default(),
        status: automation_text_field(field, &["status"]).unwrap_or_else(|| "pending".to_string()),
    })
}

fn default_composer_fields() -> Vec<DxAgentAutomationComposerField> {
    [
        ("name", "Name", "text", "Status check"),
        ("prompt", "Prompt", "textarea", "What should DX Agents run?"),
        (
            "schedule",
            "Schedule",
            "schedule",
            "Manual until runtime is available",
        ),
        (
            "destination",
            "Destination",
            "destination",
            "Current workspace",
        ),
    ]
    .into_iter()
    .map(
        |(id, label, kind, placeholder)| DxAgentAutomationComposerField {
            id: id.to_string(),
            label: label.to_string(),
            kind: kind.to_string(),
            required: matches!(id, "name" | "prompt"),
            value: String::new(),
            placeholder: placeholder.to_string(),
            status: "pending_backend_contract".to_string(),
        },
    )
    .collect()
}

fn automation_text_field(value: &Value, path: &[&str]) -> Option<String> {
    safe_string_field(value, path).and_then(bound_automation_text)
}

fn bound_automation_text(value: String) -> Option<String> {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact = compact
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();

    if compact.is_empty() {
        return None;
    }
    if compact.chars().count() <= MAX_AUTOMATION_TEXT_CHARS {
        return Some(compact);
    }

    let mut bounded = compact
        .chars()
        .take(MAX_AUTOMATION_TEXT_CHARS.saturating_sub(3))
        .collect::<String>();
    bounded.push_str("...");
    Some(bounded)
}

#[cfg(test)]
#[path = "automation_contract_tests.rs"]
mod automation_contract_tests;

#[cfg(test)]
#[path = "automation_contract_safety_tests.rs"]
mod automation_contract_safety_tests;
