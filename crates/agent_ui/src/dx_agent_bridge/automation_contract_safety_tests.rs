use serde_json::json;

use super::automations;

#[test]
fn automation_execution_proof_requires_successful_run_receipt() {
    let receipt = json!({
        "automations": [
            {
                "id": "daily-dx-audit",
                "receipts": [{
                    "kind": "automation_draft",
                    "schema_version": "dx.agents.zed.automation_draft.v1",
                    "status": "passed",
                    "receipt_path": ".dx/receipts/agents/draft-latest.json"
                }],
                "history": [{
                    "run_id": "missing-proof",
                    "status": "passed"
                }]
            },
            {
                "id": "daily-dx-audit",
                "receipts": [{
                    "kind": "automation_run",
                    "schema_version": "dx.agents.zed.automation_run.v1",
                    "status": "passed",
                    "receipt_path": ".dx/receipts/agents/run-latest.json"
                }]
            },
            {
                "id": "weekly-dx-audit",
                "history": [{
                    "run_id": "run-2",
                    "status": "completed",
                    "receipt_path": ".dx/receipts/agents/run-2.json"
                }]
            }
        ]
    });

    let rows = automations(&receipt);

    assert!(!rows[0].has_successful_execution_proof());
    assert!(rows[1].has_successful_execution_proof());
    assert!(rows[2].has_successful_execution_proof());
}

#[test]
fn automation_execution_proof_flags_failed_run_receipt() {
    let receipt = json!({
        "automations": [{
            "id": "daily-dx-audit",
            "receipts": [{
                "kind": "automation_run",
                "schema_version": "dx.agents.zed.automation_run.v1",
                "status": "failed",
                "receipt_path": ".dx/receipts/agents/run-latest.json"
            }],
            "history": [{
                "run_id": "run-1",
                "status": "error",
                "receipt_path": ".dx/receipts/agents/run-1.json"
            }]
        }]
    });

    let rows = automations(&receipt);

    assert!(rows[0].has_failed_execution_proof());
    assert!(!rows[0].has_successful_execution_proof());
}

#[test]
fn automation_text_fields_are_redacted_cleaned_and_bounded() {
    let long_prompt = format!("{}\n{}", "Review secret_token value", "x".repeat(240));
    let receipt = json!({
        "automations": [{
            "id": "daily-dx-audit",
            "name": "Daily\u{0007}DX\u{0008}Audit",
            "prompt": long_prompt,
            "destination": {
                "label": "Workspace\tTarget",
                "target": "sk-test-secret"
            },
            "receipts": [{
                "kind": "automation_run",
                "schema_version": "dx.agents.zed.automation_run.v1",
                "status": "passed",
                "receipt_path": ".dx/receipts/agents/run-latest.json"
            }],
            "history": [{
                "run_id": "run-1",
                "status": "passed",
                "receipt_path": "secret-api-key"
            }]
        }]
    });

    let rows = automations(&receipt);
    let row = &rows[0];

    assert_eq!(row.name, "DailyDXAudit");
    assert_eq!(row.destination.label, "Workspace Target");
    assert_eq!(row.destination.target, "<redacted>");
    assert_eq!(row.history[0].receipt_path, "<redacted>");
    assert_eq!(row.prompt, "<redacted>");
    assert!(row.prompt.chars().count() <= 180);
}
