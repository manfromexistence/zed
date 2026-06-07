use crate::dx_agent_bridge::DxAgentRowAction;

pub(super) fn dx_agent_action_line(actions: &[DxAgentRowAction]) -> Option<String> {
    if actions.is_empty() {
        return None;
    }

    let enabled = actions.iter().filter(|action| action.enabled).count();
    let user_actions = actions
        .iter()
        .filter(|action| action.user_action_required)
        .count();
    let public_bridges = actions
        .iter()
        .filter(|action| action.public_command.starts_with("dx agents "))
        .count();
    let receipts = actions
        .iter()
        .take(2)
        .map(|action| action.receipt_filename.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        "{enabled}/{} action(s) advertised, {public_bridges} public bridge(s), {user_actions} user action(s), receipts {receipts}",
        actions.len()
    ))
}
