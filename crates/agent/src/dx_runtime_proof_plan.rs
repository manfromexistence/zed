use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const DX_RUNTIME_PROOF_PLAN_SCHEMA: &str = "zed.dx.runtime_proof.plan.v1";
pub(crate) const DX_RUNTIME_PROOF_PLAN_RECEIPT_SCHEMA: &str =
    "zed.dx.runtime_proof.plan_receipt.v1";

#[derive(Clone, Debug)]
pub(crate) struct DxRuntimeProofPlanRequest {
    pub expected_final_command: Option<String>,
    pub require_clean_git: bool,
    pub require_diff_check: bool,
    pub require_runtime_visual_evidence: bool,
    pub require_runtime_proof_import: bool,
    pub require_profile_backend_proofs: bool,
    pub operator_notes: Vec<String>,
    pub root_mode: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlan {
    pub schema: &'static str,
    pub generated_at_ms: u128,
    pub request: DxRuntimeProofPlanRequestSummary,
    pub status: DxRuntimeProofPlanStatus,
    pub checklist: Vec<DxRuntimeProofPlanStep>,
    pub profile_backend_lanes: Vec<DxRuntimeProofProfileBackendLane>,
    pub evidence_contract: DxRuntimeProofEvidenceContract,
    pub runtime_proof_plan_receipt: Option<DxRuntimeProofPlanReceipt>,
    pub safety: DxRuntimeProofPlanSafety,
    pub next_action: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlanRequestSummary {
    pub expected_final_command: String,
    pub require_clean_git: bool,
    pub require_diff_check: bool,
    pub require_runtime_visual_evidence: bool,
    pub require_runtime_proof_import: bool,
    pub require_profile_backend_proofs: bool,
    pub operator_notes: Vec<String>,
    pub root_mode: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlanStatus {
    pub status: &'static str,
    pub manual_runtime_window_required: bool,
    pub runtime_green_claim_ready: bool,
    pub checklist_step_count: usize,
    pub required_step_count: usize,
    pub blocker_count: usize,
    pub blockers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlanStep {
    pub step_id: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub operator_action: String,
    pub evidence_required: &'static str,
    pub tool_after_evidence: Option<&'static str>,
    pub status: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct DxRuntimeProofProfileBackendLane {
    pub lane_id: &'static str,
    pub profile: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub operator_action: &'static str,
    pub evidence_required: &'static str,
    pub tool_after_evidence: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofEvidenceContract {
    pub final_command: String,
    pub import_tool: &'static str,
    pub import_operator_status: &'static str,
    pub minimum_evidence_lines_for_pass: usize,
    pub accepted_evidence_examples: Vec<&'static str>,
    pub managed_import_root: &'static str,
    pub managed_status_root: &'static str,
    pub runtime_green_claim_requires_import_receipt: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlanSafety {
    pub writes_plan_receipt_only: bool,
    pub runs_just_run: bool,
    pub runs_cargo: bool,
    pub starts_local_servers: bool,
    pub dispatches_browser_input: bool,
    pub runs_external_processes: bool,
    pub mutates_sources: bool,
    pub deploys: bool,
    pub restores_to_target: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DxRuntimeProofPlanReceipt {
    pub schema: &'static str,
    pub status: &'static str,
    pub root_mode: String,
    pub receipt_dir: String,
    pub latest_path: String,
    pub archive_path: String,
    pub written_bytes: usize,
    pub plan_schema: &'static str,
    pub checklist_step_count: usize,
    pub next_action: String,
}

pub(crate) const DX_RUNTIME_PROOF_PROFILE_BACKEND_LANES: &[DxRuntimeProofProfileBackendLane] = &[
    DxRuntimeProofProfileBackendLane {
        lane_id: "dx-metasearch-live-proof",
        profile: "Search",
        label: "Live DX MetaSearch proof",
        required: true,
        operator_action: "Inspect DX MetaSearch service readiness and capture ready or unavailable status before Search claims live backend proof.",
        evidence_required: "inspect_dx_metasearch status schema, base URL, service status, engine count, and warnings or blockers.",
        tool_after_evidence: Some("inspect_dx_metasearch"),
    },
    DxRuntimeProofProfileBackendLane {
        lane_id: "study-source-workspace-execution",
        profile: "Study",
        label: "Study source workspace execution",
        required: true,
        operator_action: "Prepare Study source attachments, context receipts, serializer/RLM plans, runner gates, reduced context, or execution previews from the active workspace source set before claiming a Study execution path.",
        evidence_required: "prepare_dx_source_attachment, prepare_dx_metasearch_context, plan_dx_serializer_rlm_execution, gate_dx_serializer_rlm_runner, write_dx_serializer_rlm_reduced_context, or preview_dx_serializer_rlm_reducer_execution receipt path with source counts and blockers; no external execution unless explicitly approved.",
        tool_after_evidence: Some("preview_dx_serializer_rlm_reducer_execution"),
    },
    DxRuntimeProofProfileBackendLane {
        lane_id: "media-provider-readiness-proof",
        profile: "Media",
        label: "Media provider readiness proof",
        required: true,
        operator_action: "Plan and gate provider or runner readiness, then retain receipts; do not claim generated media unless produced files exist on disk.",
        evidence_required: "plan_dx_media_tool or gate_dx_media_tool_runner receipt path, provider/runner readiness, and produced-file receipt state.",
        tool_after_evidence: Some("gate_dx_media_tool_runner"),
    },
    DxRuntimeProofProfileBackendLane {
        lane_id: "web-preview-runtime-proof",
        profile: "Web Preview",
        label: "Web Preview runtime proof",
        required: true,
        operator_action: "Capture Web Preview runtime evidence as supporting proof, then import canonical DX runtime proof only after governed validation evidence exists.",
        evidence_required: "Web Preview final validation evidence plus canonical import_dx_runtime_proof import/status receipt paths.",
        tool_after_evidence: Some("import_dx_runtime_proof"),
    },
];

pub(crate) fn build_runtime_proof_plan(request: DxRuntimeProofPlanRequest) -> DxRuntimeProofPlan {
    let final_command = clean_optional_text(request.expected_final_command)
        .unwrap_or_else(|| "just run".to_string());
    let operator_notes = clean_lines(request.operator_notes, 12);
    let profile_backend_lanes = if request.require_profile_backend_proofs {
        DX_RUNTIME_PROOF_PROFILE_BACKEND_LANES.to_vec()
    } else {
        Vec::new()
    };
    let mut checklist = vec![DxRuntimeProofPlanStep {
        step_id: "governed-window",
        label: "Open governed runtime validation window",
        required: true,
        operator_action:
            "Confirm the user explicitly allowed runtime validation before any final command runs."
                .to_string(),
        evidence_required: "Operator confirmation that this is the governed validation window.",
        tool_after_evidence: None,
        status: "manual_required",
    }];

    if request.require_clean_git {
        checklist.push(DxRuntimeProofPlanStep {
            step_id: "git-status",
            label: "Confirm tracked git state",
            required: true,
            operator_action: "Run git status --short --branch and capture the tracked-tree state."
                .to_string(),
            evidence_required: "Branch, ahead/behind count, and whether tracked files are clean.",
            tool_after_evidence: None,
            status: "manual_required",
        });
    }
    if request.require_diff_check {
        checklist.push(DxRuntimeProofPlanStep {
            step_id: "diff-check",
            label: "Run lightweight source hygiene",
            required: true,
            operator_action: "Run git diff --check and git diff --cached --check before runtime."
                .to_string(),
            evidence_required: "Exit status and any whitespace/conflict-marker findings.",
            tool_after_evidence: None,
            status: "manual_required",
        });
    }

    for lane in &profile_backend_lanes {
        checklist.push(DxRuntimeProofPlanStep {
            step_id: lane.lane_id,
            label: lane.label,
            required: lane.required,
            operator_action: lane.operator_action.to_string(),
            evidence_required: lane.evidence_required,
            tool_after_evidence: lane.tool_after_evidence,
            status: "manual_required",
        });
    }

    checklist.push(DxRuntimeProofPlanStep {
        step_id: "final-command",
        label: "Run final manual runtime command",
        required: true,
        operator_action: format!(
            "Run `{final_command}` only inside the governed validation window."
        ),
        evidence_required: "Exit status, relevant terminal summary, and visible app/window result.",
        tool_after_evidence: None,
        status: "manual_required",
    });

    if request.require_runtime_visual_evidence {
        checklist.push(DxRuntimeProofPlanStep {
            step_id: "visual-proof",
            label: "Capture visible runtime proof",
            required: true,
            operator_action:
                "Record the visible Zed/DX window state and the Agent panel route exercised."
                    .to_string(),
            evidence_required: "Window title, panel route, screenshot path, or concise visual proof note.",
            tool_after_evidence: None,
            status: "manual_required",
        });
    }
    if request.require_runtime_proof_import {
        checklist.push(DxRuntimeProofPlanStep {
            step_id: "import-proof",
            label: "Import runtime proof receipt",
            required: true,
            operator_action:
                "Call import_dx_runtime_proof with operator_status=passed only after evidence exists."
                    .to_string(),
            evidence_required:
                "Managed runtime proof import/status receipt paths under tools/dx-runtime-proof.",
            tool_after_evidence: Some("import_dx_runtime_proof"),
            status: "manual_required",
        });
    }

    let blockers = vec![
        "Runtime proof cannot be claim-ready until the governed manual validation window runs."
            .to_string(),
        "Runtime-green status requires imported evidence, not this plan receipt alone.".to_string(),
        if request.require_profile_backend_proofs {
            "Profile backend proof lanes require operator evidence for Search, Study, Media, and Web Preview."
                .to_string()
        } else {
            "Profile backend proof lanes were not requested for this plan.".to_string()
        },
    ];
    let required_step_count = checklist.iter().filter(|step| step.required).count();

    DxRuntimeProofPlan {
        schema: DX_RUNTIME_PROOF_PLAN_SCHEMA,
        generated_at_ms: current_epoch_millis(),
        request: DxRuntimeProofPlanRequestSummary {
            expected_final_command: final_command.clone(),
            require_clean_git: request.require_clean_git,
            require_diff_check: request.require_diff_check,
            require_runtime_visual_evidence: request.require_runtime_visual_evidence,
            require_runtime_proof_import: request.require_runtime_proof_import,
            require_profile_backend_proofs: request.require_profile_backend_proofs,
            operator_notes,
            root_mode: request.root_mode,
        },
        status: DxRuntimeProofPlanStatus {
            status: "manual_runtime_proof_required",
            manual_runtime_window_required: true,
            runtime_green_claim_ready: false,
            checklist_step_count: checklist.len(),
            required_step_count,
            blocker_count: blockers.len(),
            blockers,
        },
        checklist,
        profile_backend_lanes,
        evidence_contract: DxRuntimeProofEvidenceContract {
            final_command,
            import_tool: "import_dx_runtime_proof",
            import_operator_status: "passed",
            minimum_evidence_lines_for_pass: if request.require_profile_backend_proofs {
                DX_RUNTIME_PROOF_PROFILE_BACKEND_LANES.len()
            } else {
                1
            },
            accepted_evidence_examples: vec![
                "final command exit status",
                "visible Zed/DX window title",
                "Agent panel route or action exercised",
                "managed receipt path",
                "screenshot or runtime proof artifact path",
                "DX MetaSearch status receipt with ready or unavailable service state",
                "Study source attachment, metasearch context, serializer/RLM plan, gate, reduced-context, or execution-preview receipt path",
                "Media provider plan, runner gate, or produced-file receipt path",
                "Web Preview final validation evidence plus canonical runtime proof import",
            ],
            managed_import_root: "tools/dx-runtime-proof/imports",
            managed_status_root: "tools/dx-runtime-proof/status",
            runtime_green_claim_requires_import_receipt: true,
        },
        runtime_proof_plan_receipt: None,
        safety: DxRuntimeProofPlanSafety {
            writes_plan_receipt_only: true,
            runs_just_run: false,
            runs_cargo: false,
            starts_local_servers: false,
            dispatches_browser_input: false,
            runs_external_processes: false,
            mutates_sources: false,
            deploys: false,
            restores_to_target: false,
        },
        next_action:
            "Use this plan to collect the governed backend and runtime proof lanes, then import the evidence with import_dx_runtime_proof."
                .to_string(),
    }
}

fn clean_lines(values: Vec<String>, limit: usize) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| clean_optional_text(Some(value)))
        .take(limit)
        .collect()
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn current_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
