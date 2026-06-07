use serde_json::Value;

use super::{
    DxAgentPublicCommand, DxAgentRowAction, array_field, bool_field, is_dx_agents_command,
    is_public_dx_agents_command, is_safe_automation_id_arg, is_secret_like_arg,
    public_command_for_runtime, string_field,
};

const AUTOMATION_RUN_COMMAND_PREFIX: &str = "automate run --id";
const AUTOMATION_ENABLE_COMMAND_PREFIX: &str = "automate enable --id";

pub(super) fn automation_composer_actions(value: &Value) -> Vec<DxAgentRowAction> {
    row_actions(
        value,
        |id, automation_id, command, receipt_filename, refresh_command| match id {
            "save_draft" => {
                automation_id.is_none()
                    && receipt_filename == "automate-draft-latest.json"
                    && is_dx_agents_command(command, "automate save-draft --json")
                    && is_composer_refresh_command(refresh_command)
            }
            "enable" => automation_id.is_some_and(|automation_id| {
                receipt_filename == "automate-enable-latest.json"
                    && is_automation_enable_command(command, automation_id)
                    && is_automation_list_refresh_command(refresh_command)
            }),
            "refresh" => {
                automation_id.is_none()
                    && receipt_filename == "automate-composer-latest.json"
                    && is_dx_agents_command(command, "automate composer --json")
                    && is_composer_refresh_command(refresh_command)
            }
            _ => false,
        },
    )
}

pub(super) fn automation_row_actions(
    value: &Value,
    row_automation_id: &str,
) -> Vec<DxAgentRowAction> {
    row_actions(
        value,
        |id, automation_id, command, receipt_filename, refresh_command| match id {
            "enable" => automation_id.is_some_and(|automation_id| {
                automation_id == row_automation_id
                    && receipt_filename == "automate-enable-latest.json"
                    && is_automation_enable_command(command, automation_id)
                    && is_automation_list_refresh_command(refresh_command)
            }),
            "run" => automation_id.is_some_and(|automation_id| {
                automation_id == row_automation_id
                    && receipt_filename == "automate-run-latest.json"
                    && is_automation_run_command(command, automation_id)
                    && is_automation_list_refresh_command(refresh_command)
            }),
            "refresh" => {
                automation_id.is_none()
                    && receipt_filename == "automate-list-latest.json"
                    && is_dx_agents_command(command, "automate list --json")
                    && is_automation_list_refresh_command(refresh_command)
            }
            _ => false,
        },
    )
}

pub(crate) fn automation_public_command_for_action(
    action: &DxAgentRowAction,
) -> Option<DxAgentPublicCommand> {
    match action.id.as_str() {
        "save_draft" => Some(DxAgentPublicCommand::AutomationSaveDraft),
        "enable" => action
            .automation_id
            .clone()
            .map(|automation_id| DxAgentPublicCommand::AutomationEnable { automation_id }),
        "run" => action
            .automation_id
            .clone()
            .map(|automation_id| DxAgentPublicCommand::AutomationRun { automation_id }),
        "refresh" => Some(DxAgentPublicCommand::AutomationsList),
        _ => None,
    }
}

fn row_actions<F>(value: &Value, is_allowed: F) -> Vec<DxAgentRowAction>
where
    F: Fn(&str, Option<&str>, &str, &str, &str) -> bool,
{
    array_field(value, &["actions"])
        .map(|actions| {
            actions
                .iter()
                .take(8)
                .filter_map(|action| row_action(action, &is_allowed))
                .collect()
        })
        .unwrap_or_default()
}

fn row_action<F>(value: &Value, is_allowed: &F) -> Option<DxAgentRowAction>
where
    F: Fn(&str, Option<&str>, &str, &str, &str) -> bool,
{
    let id = string_field(value, &["id"])?;
    let automation_id = string_field(value, &["automation_id"]);
    if automation_id
        .as_deref()
        .is_some_and(|automation_id| !is_safe_automation_id_arg(automation_id))
    {
        return None;
    }
    let command = string_field(value, &["command"])?;
    let public_command = string_field(value, &["public_command"])
        .unwrap_or_else(|| public_command_for_runtime(&command));
    let receipt_filename = string_field(value, &["receipt_filename"])?;
    let refresh_command = string_field(value, &["refresh_command"])?;
    let public_refresh_command = string_field(value, &["public_refresh_command"])
        .unwrap_or_else(|| public_command_for_runtime(&refresh_command));
    let secrets_exposed = bool_field(value, &["secrets_exposed"]).unwrap_or(true);
    let writes_receipt = bool_field(value, &["writes_receipt"]).unwrap_or(false);

    if !writes_receipt
        || secrets_exposed
        || is_secret_like_arg(&command)
        || is_secret_like_arg(&public_command)
        || is_secret_like_arg(&receipt_filename)
        || is_secret_like_arg(&refresh_command)
        || is_secret_like_arg(&public_refresh_command)
        || !is_public_dx_agents_command(&public_command)
        || !is_public_dx_agents_command(&public_refresh_command)
        || !is_allowed(
            &id,
            automation_id.as_deref(),
            &command,
            &receipt_filename,
            &refresh_command,
        )
        || !is_allowed(
            &id,
            automation_id.as_deref(),
            &public_command,
            &receipt_filename,
            &public_refresh_command,
        )
    {
        return None;
    }

    Some(DxAgentRowAction {
        label: string_field(value, &["label"]).unwrap_or_else(|| id.clone()),
        id,
        automation_id,
        command,
        public_command,
        enabled: bool_field(value, &["enabled"]).unwrap_or(false),
        user_action_required: bool_field(value, &["user_action_required"]).unwrap_or(false),
        writes_receipt,
        receipt_filename,
        refresh_command,
        public_refresh_command,
        secrets_exposed,
    })
}

fn is_automation_run_command(command: &str, automation_id: &str) -> bool {
    is_dx_agents_command(
        command,
        &format!("{AUTOMATION_RUN_COMMAND_PREFIX} {automation_id} --json"),
    )
}

fn is_automation_enable_command(command: &str, automation_id: &str) -> bool {
    is_dx_agents_command(
        command,
        &format!("{AUTOMATION_ENABLE_COMMAND_PREFIX} {automation_id} --json"),
    )
}

fn is_automation_list_refresh_command(command: &str) -> bool {
    is_dx_agents_command(command, "automate list --json")
}

fn is_composer_refresh_command(command: &str) -> bool {
    is_dx_agents_command(command, "automate composer --json")
        || is_automation_list_refresh_command(command)
}

#[cfg(test)]
#[path = "automation_actions_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "automation_actions_safety_tests.rs"]
mod safety_tests;
