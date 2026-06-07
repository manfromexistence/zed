use serde_json::json;

use super::automation_row_actions;

#[test]
fn automation_row_actions_reject_mismatched_row_targets() {
    let receipt = json!({
        "id": "daily-dx-audit",
        "actions": [{
            "id": "run",
            "automation_id": "weekly-dx-audit",
            "command": "dx agents automate run --id weekly-dx-audit --json",
            "public_command": "dx agents automate run --id weekly-dx-audit --json",
            "receipt_filename": "automate-run-latest.json",
            "refresh_command": "dx agents automate list --json",
            "public_refresh_command": "dx agents automate list --json",
            "writes_receipt": true,
            "enabled": true,
            "secrets_exposed": false
        }]
    });

    let actions = automation_row_actions(&receipt, "daily-dx-audit");

    assert!(actions.is_empty());
}

#[test]
fn automation_row_actions_reject_generic_or_mismatched_public_commands() {
    let receipt = json!({
        "id": "daily-dx-audit",
        "actions": [
            {
                "id": "run",
                "automation_id": "daily-dx-audit",
                "command": "dx agents automate run --id daily-dx-audit --json",
                "public_command": "dx agents run --json",
                "receipt_filename": "automate-run-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            },
            {
                "id": "enable",
                "automation_id": "daily-dx-audit",
                "command": "dx agents automate enable --id daily-dx-audit --json",
                "public_command": "dx agents automate enable --id weekly-dx-audit --json",
                "receipt_filename": "automate-enable-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            }
        ]
    });

    let actions = automation_row_actions(&receipt, "daily-dx-audit");

    assert!(actions.is_empty());
}
