use crate::{AgentTool, ToolCallEventStream, ToolInput, dx_metasearch_agent_bridge};
use agent_client_protocol::schema as acp;
use anyhow::Result;
use futures::FutureExt as _;
use gpui::{App, AppContext, Entity, SharedString, Task};
use http_client::HttpClientWithUrl;
use paths::data_dir;
use project::Project;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use util::markdown::MarkdownInlineCode;

const DX_METASEARCH_STATUS_LATEST_FILE_NAME: &str = "latest-dx-metasearch-status-receipt.json";

/// Inspect the local DX metasearch service, runtime health, and engine catalog.
///
/// Use this before targeted metasearch calls when the agent needs to know which engines are
/// available, unhealthy, disabled, or appropriate for exact-engine source searches.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DxMetasearchStatusToolInput {
    /// Optional service base URL. Defaults to DX_METASEARCH_BASE_URL or http://127.0.0.1:8888.
    pub base_url: Option<String>,
    /// Include the compact engine catalog from /api/v1/engines.
    pub include_engines: bool,
    /// Maximum engines to include in the compact response. Defaults to 40 and caps at 200.
    pub engine_limit: Option<usize>,
    /// Persist the inspected status to a managed receipt file after explicit authorization.
    pub write_status_receipt: bool,
    /// Prefer workspace-local receipts under `<workspace>/tools`; falls back to Zed data.
    pub receipt_root_mode: DxMetasearchStatusReceiptRootMode,
}

impl Default for DxMetasearchStatusToolInput {
    fn default() -> Self {
        Self {
            base_url: None,
            include_engines: true,
            engine_limit: Some(40),
            write_status_receipt: false,
            receipt_root_mode: DxMetasearchStatusReceiptRootMode::Workspace,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DxMetasearchStatusReceiptRootMode {
    #[default]
    Workspace,
    ZedData,
}

impl From<DxMetasearchStatusToolInput> for dx_metasearch_agent_bridge::DxMetasearchStatusRequest {
    fn from(input: DxMetasearchStatusToolInput) -> Self {
        Self {
            base_url: input.base_url,
            include_engines: input.include_engines,
            engine_limit: input.engine_limit,
        }
    }
}

pub struct DxMetasearchStatusTool {
    project: Entity<Project>,
    http_client: Arc<HttpClientWithUrl>,
}

impl DxMetasearchStatusTool {
    pub fn new(project: Entity<Project>, http_client: Arc<HttpClientWithUrl>) -> Self {
        Self {
            project,
            http_client,
        }
    }
}

impl AgentTool for DxMetasearchStatusTool {
    type Input = DxMetasearchStatusToolInput;
    type Output = String;

    const NAME: &'static str = "inspect_dx_metasearch";

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Fetch
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        if let Ok(input) = input {
            if let Some(base_url) = input.base_url.as_deref() {
                format!("Inspect DX metasearch at {}", MarkdownInlineCode(base_url)).into()
            } else {
                "Inspect DX metasearch".into()
            }
        } else {
            "Inspect DX metasearch".into()
        }
    }

    fn run(
        self: Arc<Self>,
        input: ToolInput<Self::Input>,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output, Self::Output>> {
        let http_client = self.http_client.clone();
        let project = self.project.clone();

        cx.spawn(async move |cx| {
            let input = input.recv().await.map_err(|error| error.to_string())?;
            let status_receipt_target = input.write_status_receipt.then(|| {
                let project_root = cx.update(|cx| workspace_root_for_project(&project, cx));
                DxMetasearchStatusReceiptTarget::new(project_root, input.receipt_root_mode)
            });
            let base_permission_value = input
                .base_url
                .clone()
                .unwrap_or_else(|| "DX_METASEARCH_BASE_URL/default".to_string());
            let authorize = cx.update(|cx| {
                let mut permission_values = vec![
                    base_permission_value,
                    format!("include_engines={}", input.include_engines),
                    format!("write_status_receipt={}", input.write_status_receipt),
                ];
                if let Some(receipt_target) = &status_receipt_target {
                    permission_values.push(path_string(&receipt_target.latest_path));
                    permission_values.push(path_string(&receipt_target.archive_path));
                }
                let context = crate::ToolPermissionContext::new(Self::NAME, permission_values);
                event_stream.authorize(self.initial_title(Ok(input.clone()), cx), context, cx)
            });
            let status_request: dx_metasearch_agent_bridge::DxMetasearchStatusRequest =
                input.clone().into();
            let unavailable_status_request = status_request.clone();

            let inspect_task = cx.background_spawn({
                let http_client = http_client.clone();
                async move {
                    authorize.await.map_err(|error| error.to_string())?;
                    dx_metasearch_agent_bridge::inspect_metasearch_status(
                        http_client,
                        status_request,
                    )
                    .await
                }
            });

            let inspection_result = futures::select! {
                result = inspect_task.fuse() => {
                    result
                }
                _ = event_stream.cancelled_by_user().fuse() => {
                    return Err("DX metasearch inspection cancelled by user".to_string());
                }
            };
            let mut response = match inspection_result {
                Ok(response) => response,
                Err(error) if status_receipt_target.is_some() => {
                    dx_metasearch_agent_bridge::unavailable_metasearch_status(
                        unavailable_status_request,
                        error,
                    )
                }
                Err(error) => return Err(error),
            };

            if let Some(receipt_target) = status_receipt_target {
                response.status_receipt =
                    Some(receipt_target.write_receipt(&response).map_err(|error| {
                        format!("Failed to write DX metasearch status receipt: {error}")
                    })?);
            }

            event_stream.update_fields(acp::ToolCallUpdateFields::new().title(format!(
                "Inspected DX metasearch: {} engine(s), status {}",
                response.engine_summary.catalog_count, response.service.status
            )));

            serde_json::to_string_pretty(&response).map_err(|error| {
                format!("Failed to serialize DX metasearch status response: {error}")
            })
        })
    }
}

struct DxMetasearchStatusReceiptTarget {
    root_mode: DxMetasearchStatusReceiptRootMode,
    project_root: Option<PathBuf>,
    allowed_root: PathBuf,
    receipt_dir: PathBuf,
    latest_path: PathBuf,
    archive_path: PathBuf,
}

impl DxMetasearchStatusReceiptTarget {
    fn new(project_root: Option<PathBuf>, root_mode: DxMetasearchStatusReceiptRootMode) -> Self {
        let use_workspace = matches!(root_mode, DxMetasearchStatusReceiptRootMode::Workspace)
            && project_root.is_some();
        let allowed_root = if use_workspace {
            project_root
                .as_ref()
                .expect("workspace root checked above")
                .join("tools")
        } else {
            data_dir().join("dx-metasearch")
        };
        let receipt_dir = if use_workspace {
            allowed_root.join("dx-metasearch").join("status")
        } else {
            allowed_root.join("status")
        };
        let latest_path = receipt_dir.join(DX_METASEARCH_STATUS_LATEST_FILE_NAME);
        let archive_path = receipt_dir.join(format!(
            "dx-metasearch-status-{}.json",
            current_epoch_millis()
        ));

        Self {
            root_mode,
            project_root,
            allowed_root,
            receipt_dir,
            latest_path,
            archive_path,
        }
    }

    fn write_receipt(
        &self,
        response: &dx_metasearch_agent_bridge::DxMetasearchStatusResponse,
    ) -> Result<dx_metasearch_agent_bridge::DxMetasearchStatusReceipt, String> {
        self.validate()?;
        let receipt = serde_json::json!({
            "schema": dx_metasearch_agent_bridge::DX_METASEARCH_STATUS_RECEIPT_SCHEMA,
            "written_at_ms": current_epoch_millis(),
            "source_tool": DxMetasearchStatusTool::NAME,
            "root_mode": self.root_mode_label(),
            "project_root": self.project_root.as_ref().map(path_string),
            "receipt_dir": path_string(&self.receipt_dir),
            "latest_path": path_string(&self.latest_path),
            "archive_path": path_string(&self.archive_path),
            "metasearch_status": {
                "schema": response.schema,
                "request": &response.request,
                "source": &response.source,
                "service": &response.service,
                "engine_summary": &response.engine_summary,
            },
            "safety": {
                "written_after_authorization": true,
                "writes_under_managed_root": true,
                "starts_metasearch_server": false,
                "deep_fetches_pages": false,
                "runs_serializer_or_rlm": false,
                "dispatches_browser_input": false,
            },
            "next_action": "Use this status receipt as Search backend proof, or preserve its unavailable/blocker state until the DX MetaSearch service is live."
        });
        let receipt_json = serde_json::to_vec_pretty(&receipt)
            .map_err(|error| format!("Failed to serialize metasearch status receipt: {error}"))?;

        fs::create_dir_all(&self.receipt_dir).map_err(|error| {
            format!(
                "Failed to prepare DX metasearch status directory {}: {error}",
                self.receipt_dir.display()
            )
        })?;
        fs::write(&self.latest_path, &receipt_json).map_err(|error| {
            format!(
                "Failed to write DX metasearch latest status receipt {}: {error}",
                self.latest_path.display()
            )
        })?;
        fs::write(&self.archive_path, &receipt_json).map_err(|error| {
            format!(
                "Failed to archive DX metasearch status receipt {}: {error}",
                self.archive_path.display()
            )
        })?;

        Ok(dx_metasearch_agent_bridge::DxMetasearchStatusReceipt {
            schema: dx_metasearch_agent_bridge::DX_METASEARCH_STATUS_RECEIPT_SCHEMA,
            status: "written",
            root_mode: self.root_mode_label().to_string(),
            receipt_dir: path_string(&self.receipt_dir),
            latest_path: path_string(&self.latest_path),
            archive_path: path_string(&self.archive_path),
            written_bytes: receipt_json.len(),
            service_status: response.service.status.clone(),
            engine_count: response.engine_summary.catalog_count,
            warning_count: response.service.warning_count,
            next_action:
                "Use this status receipt as Search backend proof, preserving unavailable states as blockers when present."
                    .to_string(),
        })
    }

    fn validate(&self) -> Result<(), String> {
        for path in [&self.receipt_dir, &self.latest_path, &self.archive_path] {
            if !path.starts_with(&self.allowed_root) {
                return Err(format!(
                    "Refusing to write DX metasearch status receipt at unmanaged path {} outside {}",
                    path.display(),
                    self.allowed_root.display()
                ));
            }
        }

        Ok(())
    }

    fn root_mode_label(&self) -> &'static str {
        match self.root_mode {
            DxMetasearchStatusReceiptRootMode::Workspace if self.project_root.is_some() => {
                "workspace"
            }
            DxMetasearchStatusReceiptRootMode::Workspace => "zed_data_fallback",
            DxMetasearchStatusReceiptRootMode::ZedData => "zed_data",
        }
    }
}

fn workspace_root_for_project(project: &Entity<Project>, cx: &App) -> Option<PathBuf> {
    project
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).abs_path().as_ref().to_path_buf())
}

fn path_string(path: impl AsRef<Path>) -> String {
    path.as_ref().display().to_string()
}

fn current_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}
