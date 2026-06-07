pub(super) fn dx_agents_args(args: &[&str]) -> Vec<String> {
    let mut command = Vec::with_capacity(args.len() + 2);
    command.push("agents".to_string());
    command.extend(args.iter().map(|arg| (*arg).to_string()));
    command.push("--json".to_string());
    command
}

pub(super) fn dx_agents_platform_args(action: &str, platform: &str) -> Vec<String> {
    vec![
        "agents".to_string(),
        "social".to_string(),
        action.to_string(),
        "--platform".to_string(),
        platform.to_string(),
        "--json".to_string(),
    ]
}

pub(super) fn dx_agents_automation_args(action: &str, automation_id: &str) -> Vec<String> {
    vec![
        "agents".to_string(),
        "automate".to_string(),
        action.to_string(),
        "--id".to_string(),
        automation_id.to_string(),
        "--json".to_string(),
    ]
}

#[cfg(test)]
#[path = "command_args_tests.rs"]
mod tests;
