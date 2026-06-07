use std::{
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::{Context as _, Result, anyhow};

use super::command_args::{dx_agents_args, dx_agents_automation_args, dx_agents_platform_args};
use super::command_receipts::{
    clear_action_error_receipt, failed_command_stderr_display, write_action_error_receipt,
    write_json_receipt,
};
use super::{
    bridge_command_label, clear_snapshot_cache, is_safe_automation_id_arg, is_safe_platform_arg,
    is_secret_like_arg,
};

#[derive(Clone)]
pub(crate) enum DxAgentPublicCommand {
    Contract,
    Status,
    Run,
    ReceiptsList,
    AutomationSaveDraft,
    AutomationEnable { automation_id: String },
    AutomationRun { automation_id: String },
    SocialList,
    SocialConnect { platform: String },
    SocialDisconnect { platform: String },
    AutomationsList,
    ProvidersList,
    ModelsList,
    ProviderCatalogRegenerate,
}

struct DxAgentPublicReceiptCapture {
    receipt_filename: &'static str,
    expected_schema: &'static str,
}

impl DxAgentPublicCommand {
    fn args(&self) -> Vec<String> {
        match self {
            Self::Contract => dx_agents_args(&["contract"]),
            Self::Status => dx_agents_args(&["status"]),
            Self::Run => dx_agents_args(&["run"]),
            Self::ReceiptsList => dx_agents_args(&["receipts", "list"]),
            Self::AutomationSaveDraft => dx_agents_args(&["automate", "save-draft"]),
            Self::AutomationEnable { automation_id } => {
                dx_agents_automation_args("enable", automation_id)
            }
            Self::AutomationRun { automation_id } => {
                dx_agents_automation_args("run", automation_id)
            }
            Self::SocialList => dx_agents_args(&["social", "list"]),
            Self::SocialConnect { platform } => {
                dx_agents_platform_args("connect", platform.as_str())
            }
            Self::SocialDisconnect { platform } => {
                dx_agents_platform_args("disconnect", platform.as_str())
            }
            Self::AutomationsList => dx_agents_args(&["automate", "list"]),
            Self::ProvidersList => dx_agents_args(&["providers", "list"]),
            Self::ModelsList => dx_agents_args(&["models", "list"]),
            Self::ProviderCatalogRegenerate => {
                dx_agents_args(&["providers", "catalog", "regenerate"])
            }
        }
    }

    fn is_safe(&self) -> bool {
        match self {
            Self::AutomationEnable { automation_id } | Self::AutomationRun { automation_id } => {
                is_safe_automation_id_arg(automation_id)
            }
            Self::SocialConnect { platform } | Self::SocialDisconnect { platform } => {
                is_safe_platform_arg(platform)
            }
            _ => true,
        }
    }

    fn receipt_capture(&self) -> Option<DxAgentPublicReceiptCapture> {
        match self {
            Self::ProvidersList => Some(DxAgentPublicReceiptCapture {
                receipt_filename: "providers-list-latest.json",
                expected_schema: "dx.agents.zed.providers_list.v1",
            }),
            Self::ModelsList => Some(DxAgentPublicReceiptCapture {
                receipt_filename: "models-list-latest.json",
                expected_schema: "dx.agents.zed.models_list.v1",
            }),
            Self::ProviderCatalogRegenerate => None,
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DxAgentMetadataCommand {
    ImportSummary,
    ReleaseGate,
    ReceiptsInbox,
}

impl DxAgentMetadataCommand {
    fn args(self) -> Vec<String> {
        match self {
            Self::ImportSummary => dx_agents_args(&["import-summary"]),
            Self::ReleaseGate => dx_agents_args(&["release-gate"]),
            Self::ReceiptsInbox => dx_agents_args(&["receipts"]),
        }
    }

    fn receipt_filename(self) -> &'static str {
        match self {
            Self::ImportSummary => "import-summary-latest.json",
            Self::ReleaseGate => "release-gate-latest.json",
            Self::ReceiptsInbox => "receipts-inbox-latest.json",
        }
    }

    fn expected_schema(self) -> &'static str {
        match self {
            Self::ImportSummary => "dx.agents.zed.import_summary.v1",
            Self::ReleaseGate => "dx.agents.zed.release_gate.v1",
            Self::ReceiptsInbox => "dx.agents.zed.receipts.v1",
        }
    }
}

pub(crate) fn run_dx_agent_public_command(
    command: DxAgentPublicCommand,
    cli_path: String,
    dx_home: Option<PathBuf>,
    receipt_root: PathBuf,
) -> Result<()> {
    if !command.is_safe() {
        return Err(anyhow!("unsupported DX Agents public bridge command"));
    }

    let args = command.args();
    let command_label = bridge_command_label(&cli_path, &args);
    let output = match run_bridge_command(cli_path, args, dx_home) {
        Ok(output) => output,
        Err(error) => {
            let _ = write_action_error_receipt(&receipt_root, &command_label, &error);
            clear_snapshot_cache();
            return Err(error);
        }
    };
    if let Some(capture) = command.receipt_capture() {
        if let Err(error) = write_json_receipt(
            &receipt_root.join(capture.receipt_filename),
            &output.stdout,
            capture.expected_schema,
        ) {
            let _ = write_action_error_receipt(&receipt_root, &command_label, &error);
            clear_snapshot_cache();
            return Err(error);
        }
    }
    clear_action_error_receipt(&receipt_root);
    clear_snapshot_cache();
    Ok(())
}

pub(crate) fn run_dx_agent_metadata_command(
    command: DxAgentMetadataCommand,
    cli_path: String,
    dx_home: Option<PathBuf>,
    receipt_root: PathBuf,
) -> Result<()> {
    let args = command.args();
    let command_label = bridge_command_label(&cli_path, &args);
    let output = match run_bridge_command(cli_path, args, dx_home) {
        Ok(output) => output,
        Err(error) => {
            let _ = write_action_error_receipt(&receipt_root, &command_label, &error);
            clear_snapshot_cache();
            return Err(error);
        }
    };
    if let Err(error) = write_json_receipt(
        &receipt_root.join(command.receipt_filename()),
        &output.stdout,
        command.expected_schema(),
    ) {
        let _ = write_action_error_receipt(&receipt_root, &command_label, &error);
        clear_snapshot_cache();
        return Err(error);
    }
    clear_action_error_receipt(&receipt_root);
    clear_snapshot_cache();
    Ok(())
}

fn run_bridge_command(
    cli_path: String,
    args: Vec<String>,
    dx_home: Option<PathBuf>,
) -> Result<Output> {
    if args.iter().any(|arg| is_secret_like_arg(arg)) {
        return Err(anyhow!(
            "DX Agents bridge commands cannot include secret-like arguments"
        ));
    }

    let mut command = Command::new(&cli_path);
    command.args(&args);
    if let Some(dx_home) = dx_home {
        command.env("DX_HOME", dx_home);
    }

    let output = command
        .output()
        .with_context(|| format!("failed to run `{}`", bridge_command_label(&cli_path, &args)))?;

    if !output.status.success() {
        let stderr = failed_command_stderr_display(&output.stderr);
        return Err(anyhow!(
            "`{}` failed: {}",
            bridge_command_label(&cli_path, &args),
            stderr
        ));
    }

    Ok(output)
}
