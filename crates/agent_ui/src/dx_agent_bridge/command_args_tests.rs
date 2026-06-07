use super::dx_agents_automation_args;

#[test]
fn automation_run_args_match_dx_agents_contract() {
    assert_eq!(
        dx_agents_automation_args("run", "daily-dx-audit"),
        strings(&[
            "agents",
            "automate",
            "run",
            "--id",
            "daily-dx-audit",
            "--json"
        ])
    );
}

#[test]
fn automation_enable_args_match_dx_agents_contract() {
    assert_eq!(
        dx_agents_automation_args("enable", "daily-dx-audit"),
        strings(&[
            "agents",
            "automate",
            "enable",
            "--id",
            "daily-dx-audit",
            "--json"
        ])
    );
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_string()).collect()
}
