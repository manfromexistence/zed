import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

test("DX runtime proof plan owns backend proof lanes before runtime-green import", () => {
  const plan = read("crates/agent/src/dx_runtime_proof_plan.rs");
  const planTool = read("crates/agent/src/tools/dx_runtime_proof_plan_tool.rs");
  const importModel = read("crates/agent/src/dx_runtime_proof_import.rs");
  const importTool = read("crates/agent/src/tools/dx_runtime_proof_import_tool.rs");
  const statusModel = read("crates/agent_ui/src/dx_runtime_proof_status.rs");
  const summaries = read("crates/agent_ui/src/dx_runtime_proof_status/summaries.rs");
  const runtimePrompt = read("crates/agent_ui/src/dx_launch_prompts/runtime_proof.rs");

  assert.match(plan, /pub\(crate\) struct DxRuntimeProofProfileBackendLane/);
  assert.match(plan, /pub\(crate\) const DX_RUNTIME_PROOF_PROFILE_BACKEND_LANES/);
  for (const lane of [
    "dx-metasearch-live-proof",
    "study-source-workspace-execution",
    "media-provider-readiness-proof",
    "web-preview-runtime-proof",
  ]) {
    assert.match(plan, new RegExp(`lane_id: "${lane}"`), `${lane} should be serialized as lane metadata`);
  }
  assert.match(plan, /step_id: lane\.lane_id/);

  assert.match(plan, /require_profile_backend_proofs: bool/);
  assert.match(plan, /profile_backend_lanes: Vec<DxRuntimeProofProfileBackendLane>/);
  assert.match(plan, /minimum_evidence_lines_for_pass:\s*if request\.require_profile_backend_proofs/);
  assert.match(plan, /runtime_green_claim_ready: false/);
  assert.match(plan, /runs_just_run: false/);
  assert.match(plan, /runs_cargo: false/);
  assert.match(plan, /starts_local_servers: false/);
  assert.match(plan, /dispatches_browser_input: false/);
  assert.match(plan, /runs_external_processes: false/);

  assert.match(planTool, /pub require_profile_backend_proofs: bool/);
  assert.match(planTool, /require_profile_backend_proofs: true/);
  assert.match(planTool, /format!\(\s*"require_profile_backend_proofs=\{\}"/);

  assert.match(importModel, /pub struct DxRuntimeProofProfileBackendEvidence/);
  assert.match(importModel, /pub enum DxRuntimeProofProfileBackendStatus/);
  assert.match(importModel, /pub profile_backend_evidence:\s*Vec<DxRuntimeProofProfileBackendEvidence>/);
  assert.match(importModel, /validate_profile_backend_evidence\(/);
  assert.match(importModel, /missing_required_lanes/);
  assert.match(importModel, /all_required_lanes_passed/);
  assert.match(importModel, /runtime_green_candidate = request\.operator_status == DxRuntimeProofOperatorStatus::Passed[\s\S]*profile_backend_validation\.all_required_lanes_passed/);

  assert.match(importTool, /pub require_profile_backend_proofs: bool/);
  assert.match(importTool, /pub profile_backend_evidence:\s*Vec<dx_runtime_proof_import::DxRuntimeProofProfileBackendEvidence>/);
  assert.match(importTool, /require_profile_backend_proofs: true/);
  assert.match(importTool, /format!\(\s*"profile_backend_evidence_count=\{\}"/);
  assert.match(importTool, /profile_backend_evidence: input\.profile_backend_evidence/);

  assert.match(statusModel, /pub profile_backend_lane_count: usize/);
  assert.match(statusModel, /pub profile_backend_passed_lane_count: usize/);
  assert.match(statusModel, /pub profile_backend_blocker_count: usize/);
  assert.match(
    statusModel,
    /pub\(crate\) fn runtime_green_candidate\(&self\) -> bool \{\s*self\.claim_state == "Runtime green candidate" && self\.blockers\.is_empty\(\)\s*\}/,
  );
  const claimState = functionBody(statusModel, "claim_state");
  assert.match(
    claimState,
    /if let Some\(receipt\) = latest_import[\s\S]*receipt\.evidence_count == 0[\s\S]*receipt\.profile_backend_blocker_count > 0[\s\S]*missing_profile_backend_lanes/,
  );
  assert.match(
    claimState,
    /if import_ready && status_ready && blockers\.is_empty\(\)[\s\S]*"Runtime green candidate"/,
  );
  assert.ok(
    claimState.indexOf("if let Some(receipt) = latest_import") <
      claimState.indexOf('if import_ready && status_ready && blockers.is_empty()'),
    "runtime-green claim must run after import evidence and backend blockers are audited",
  );
  assert.match(importModel, /Missing profile backend proof lane evidence/);
  assert.match(summaries, /let profile_backend_validation = validation\s*\.get\("profile_backend_validation"\)/);
  assert.match(summaries, /profile_backend_passed_lane_count: usize_at\(\s*profile_backend_validation,\s*"passed_lane_count",?\s*\)/);

  assert.match(runtimePrompt, /profile_backend_proofs/);
  assert.match(runtimePrompt, /profile_backend_evidence/);
  assert.match(runtimePrompt, /dx-metasearch-live-proof/);
  assert.match(runtimePrompt, /study-source-workspace-execution/);
  assert.match(runtimePrompt, /media-provider-readiness-proof/);
  assert.match(runtimePrompt, /web-preview-runtime-proof/);
  assert.match(runtimePrompt, /backend lanes/);
});

test("DX runtime proof status keeps receipt IO and JSON helpers focused", () => {
  const parentPath = "crates/agent_ui/src/dx_runtime_proof_status.rs";
  const receiptsPath = "crates/agent_ui/src/dx_runtime_proof_status/receipts.rs";
  const fieldsPath = "crates/agent_ui/src/dx_runtime_proof_status/fields.rs";
  const summariesPath = "crates/agent_ui/src/dx_runtime_proof_status/summaries.rs";

  assert.ok(existsSync(receiptsPath), "missing focused runtime-proof receipt IO module");
  assert.ok(existsSync(fieldsPath), "missing focused runtime-proof JSON field module");
  assert.ok(existsSync(summariesPath), "missing focused runtime-proof summary parser module");

  const parent = read(parentPath);
  const receipts = read(receiptsPath);
  const fields = read(fieldsPath);
  const summaries = read(summariesPath);

  assert.match(parent, /^mod fields;$/m);
  assert.match(parent, /^mod receipts;$/m);
  assert.match(parent, /^mod summaries;$/m);
  assert.match(parent, /use self::receipts::\{count_receipt_files, latest_receipt_paths\};/);
  assert.match(parent, /use self::summaries::\{parse_import_summary, parse_plan_summary, parse_status_summary\};/);
  assert.doesNotMatch(parent, /fn count_receipt_files\(/);
  assert.doesNotMatch(parent, /fn latest_receipt_paths\(/);
  assert.doesNotMatch(parent, /fn read_json\(/);
  assert.doesNotMatch(parent, /fn string_at\(/);
  assert.doesNotMatch(parent, /fn compact_text\(/);
  assert.doesNotMatch(parent, /fn parse_plan_summary\(/);
  assert.doesNotMatch(parent, /fn parse_import_summary\(/);
  assert.doesNotMatch(parent, /fn parse_status_summary\(/);
  assert.match(receipts, /pub\(super\) fn count_receipt_files/);
  assert.match(receipts, /pub\(super\) fn latest_receipt_paths/);
  assert.match(receipts, /pub\(super\) fn read_json/);
  assert.match(receipts, /const MAX_RECEIPT_BYTES: u64 = 128 \* 1024;/);
  assert.match(receipts, /\.take\(MAX_RECEIPT_BYTES \+ 1\)/);
  assert.match(receipts, /if buffer\.len\(\) > MAX_RECEIPT_BYTES as usize \{\s*return None;\s*\}/);
  assert.doesNotMatch(receipts, /\.take\(MAX_RECEIPT_BYTES\)/);
  assert.match(fields, /pub\(super\) fn string_at/);
  assert.match(fields, /pub\(super\) fn compact_string_array_at/);
  assert.match(fields, /fn compact_text/);
  assert.match(summaries, /use super::fields::\{/);
  assert.match(summaries, /use super::receipts::read_json;/);
  assert.match(summaries, /pub\(super\) fn parse_plan_summary/);
  assert.match(summaries, /pub\(super\) fn parse_import_summary/);
  assert.match(summaries, /pub\(super\) fn parse_status_summary/);
  assert.doesNotMatch(
    summaries,
    /(^|[^\w])string_at\(/,
    "runtime-proof summaries should not use raw string extraction for display and prompt fields",
  );
  assert.match(summaries, /status: compact_string_at\(status, "status"\)/);
  assert.match(
    summaries,
    /expected_final_command: compact_string_at\(request, "expected_final_command"\)/,
  );
  assert.match(summaries, /profile_backend_lane_count: array_len_at\(plan, "profile_backend_lanes"\)/);
  assert.match(summaries, /profile_backend_lanes: profile_backend_lane_labels\(plan\)/);
  assert.match(summaries, /fn profile_backend_lane_labels\(plan: &Value\) -> Vec<String>/);
  assert.match(summaries, /next_action: compact_string_at\(plan, "next_action"\)/);
  assert.match(
    summaries,
    /operator_status: compact_string_at\(request, "operator_status"\)/,
  );
  assert.match(
    summaries,
    /validation_status: compact_string_at\(validation, "status"\)/,
  );
  assert.match(
    summaries,
    /headline: compact_string_at\(operator_status_copy, "headline"\)/,
  );
  assert.match(summaries, /headline: compact_string_at\(status_copy, "headline"\)/);

  assert.ok(lineCount(parentPath) < 280, "dx_runtime_proof_status.rs should stay focused on snapshot assembly");
  assert.ok(lineCount(receiptsPath) < 95, "runtime-proof receipt IO module should stay small");
  assert.ok(lineCount(fieldsPath) < 95, "runtime-proof JSON field module should stay small");
  assert.ok(lineCount(summariesPath) < 170, "runtime-proof summary parser module should stay small");
});

function functionBody(source: string, name: string): string {
  const signature = source.indexOf(`fn ${name}`);
  assert.notEqual(signature, -1, `missing function ${name}`);
  const bodyStart = source.indexOf("{", signature);
  assert.notEqual(bodyStart, -1, `missing function body for ${name}`);
  let depth = 0;
  for (let index = bodyStart; index < source.length; index++) {
    const char = source[index];
    if (char === "{") depth++;
    if (char === "}") {
      depth--;
      if (depth === 0) return source.slice(bodyStart, index + 1);
    }
  }
  assert.fail(`unterminated function body for ${name}`);
}
