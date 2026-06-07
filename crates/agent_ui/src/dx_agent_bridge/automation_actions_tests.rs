use serde_json::json;

use super::{automation_composer_actions, automation_row_actions};

#[test]
fn automation_run_action_requires_typed_backend_target() {
    let receipt = json!({
        "id": "daily-dx-audit",
        "actions": [
            {
                "id": "run",
                "command": "dx agents run --json",
                "public_command": "dx agents run --json",
                "receipt_filename": "automate-run-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            },
            {
                "id": "run",
                "automation_id": "daily-dx-audit",
                "command": "dx agents automate run --id daily-dx-audit --json",
                "public_command": "dx agents automate run --id daily-dx-audit --json",
                "receipt_filename": "automate-run-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            }
        ]
    });

    let actions = automation_row_actions(&receipt, "daily-dx-audit");

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].id, "run");
    assert_eq!(actions[0].automation_id.as_deref(), Some("daily-dx-audit"));
}

#[test]
fn automation_composer_actions_parse_backend_contract() {
    let receipt = json!({
        "actions": [
            {
                "id": "save_draft",
                "command": "dx agents automate save-draft --json",
                "public_command": "dx agents automate save-draft --json",
                "receipt_filename": "automate-draft-latest.json",
                "refresh_command": "dx agents automate composer --json",
                "public_refresh_command": "dx agents automate composer --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            },
            {
                "id": "enable",
                "automation_id": "daily-dx-audit",
                "command": "dx agents automate enable --id daily-dx-audit --json",
                "public_command": "dx agents automate enable --id daily-dx-audit --json",
                "receipt_filename": "automate-enable-latest.json",
                "refresh_command": "dx agents automate list --json",
                "public_refresh_command": "dx agents automate list --json",
                "writes_receipt": true,
                "enabled": true,
                "secrets_exposed": false
            }
        ]
    });

    let actions = automation_composer_actions(&receipt);

    assert_eq!(actions.len(), 2);
    assert!(actions.iter().any(|action| action.id == "save_draft"));
    assert!(actions.iter().any(|action| action.id == "enable"));
}
