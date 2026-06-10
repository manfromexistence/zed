use serde_json::json;

use super::{automation_composer, automations};

#[test]
fn automation_rows_parse_composer_ready_contract_fields() {
    let receipt = json!({
        "automations": [{
            "id": "daily-dx-audit",
            "name": "Daily DX Audit",
            "prompt": "Review DX receipts and summarize blockers.",
            "source": "dx_agents",
            "schedule": {
                "kind": "recurring",
                "summary": "Every day at 09:00",
                "rrule": "FREQ=DAILY;BYHOUR=9",
                "timezone": "Asia/Dhaka"
            },
            "status": "scheduled",
            "enabled": true,
            "runtime": {
                "available": false,
                "unavailable_reason": "scheduler adapter pending"
            },
            "destination": {
                "kind": "workspace",
                "label": "G:\\Dx\\code",
                "target": "G:\\Dx\\code"
            },
            "last_run_at": "2026-06-06T09:00:00Z",
            "next_run_at": "2026-06-07T09:00:00Z",
            "receipts": [{
                "kind": "automation_run",
                "schema_version": "dx.agents.zed.automation_run.v1",
                "status": "passed",
                "receipt_path": ".dx/receipts/agents/run-latest.json"
            }],
            "history": [{
                "run_id": "run-1",
                "status": "passed",
                "started_at": "2026-06-06T09:00:00Z",
                "finished_at": "2026-06-06T09:01:00Z",
                "receipt_path": ".dx/receipts/agents/run-latest.json"
            }],
            "actions": [{
                "id": "refresh",
                "label": "Refresh",
                "command": "dx agents automate list --json",
                "public_command": "dx agents automate list --json",
                "receipt_filename": "automate-list-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            }]
        }]
    });

    let rows = automations(&receipt);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "Daily DX Audit");
    assert_eq!(rows[0].prompt, "Review DX receipts and summarize blockers.");
    assert_eq!(rows[0].schedule.kind, "recurring");
    assert_eq!(rows[0].schedule.rrule, "FREQ=DAILY;BYHOUR=9");
    assert_eq!(rows[0].schedule.timezone, "Asia/Dhaka");
    assert_eq!(rows[0].status.state, "scheduled");
    assert!(rows[0].status.enabled);
    assert!(!rows[0].status.runtime_available);
    assert_eq!(rows[0].destination.kind, "workspace");
    assert_eq!(rows[0].last_run, "2026-06-06T09:00:00Z");
    assert_eq!(rows[0].next_run, "2026-06-07T09:00:00Z");
    assert_eq!(
        rows[0].receipts[0].schema_version,
        "dx.agents.zed.automation_run.v1"
    );
    assert_eq!(rows[0].history[0].run_id, "run-1");
    assert_eq!(rows[0].actions.len(), 1);
}

#[test]
fn automation_composer_falls_back_to_pending_backend_contract() {
    let composer = automation_composer(None, true);

    assert_eq!(composer.status, "waiting_for_automation_composer_contract");
    assert!(!composer.receipt_present);
    assert!(!composer.runtime_available);
    assert!(!composer.save_draft_available);
    assert!(!composer.enable_available);
    assert_eq!(composer.receipt_filename, "automate-composer-latest.json");
    assert!(!composer.fields_receipt_backed);
    assert_eq!(
        composer.field_summary_label(),
        "Field template pending receipt"
    );
    assert_eq!(composer.fields.len(), 4);
    assert!(
        composer
            .fields
            .iter()
            .any(|field| field.id == "prompt" && field.required)
    );
}

#[test]
fn automation_composer_marks_receipt_fields_as_receipt_backed() {
    let receipt = json!({
        "status": "ready",
        "runtime_available": true,
        "receipt_filename": "automate-composer-latest.json",
        "fields": [{
            "id": "prompt",
            "label": "Prompt",
            "kind": "textarea",
            "required": true,
            "placeholder": "What should DX Agents run?",
            "status": "ready"
        }]
    });

    let composer = automation_composer(Some(&receipt), true);

    assert!(composer.receipt_present);
    assert!(composer.fields_receipt_backed);
    assert_eq!(composer.field_summary_label(), "Receipt fields");
    assert_eq!(composer.fields.len(), 1);
    assert_eq!(composer.fields[0].id, "prompt");
}

#[test]
fn automation_actions_reject_secret_or_non_receipt_commands() {
    let receipt = json!({
        "automations": [{
            "id": "unsafe",
            "enabled": true,
            "actions": [
                {
                    "id": "run",
                    "command": "dx agents run --json --token secret",
                    "public_command": "dx agents run --json --token secret",
                    "receipt_filename": "run-latest.json",
                    "refresh_command": "dx agents automate list --json",
                    "public_refresh_command": "dx agents automate list --json",
                    "writes_receipt": true,
                    "enabled": true,
                    "secrets_exposed": false
                },
                {
                    "id": "refresh",
                    "command": "dx agents automate list --json",
                    "public_command": "dx agents automate list --json",
                    "receipt_filename": "automate-list-latest.json",
                    "refresh_command": "dx agents automate list --json",
                    "public_refresh_command": "dx agents automate list --json",
                    "writes_receipt": false,
                    "enabled": true,
                    "secrets_exposed": false
                }
            ]
        }]
    });

    let rows = automations(&receipt);

    assert!(rows[0].actions.is_empty());
}
