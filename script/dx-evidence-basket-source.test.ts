import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");
const lineCount = (path: string) => read(path).split(/\r?\n/).length;

const functionBody = (source: string, name: string) => {
  const start = source.search(new RegExp(`fn\\s+${name}\\b`));
  assert.ok(start >= 0, `expected ${name}`);

  const bodyStart = source.indexOf("{", start);
  assert.ok(bodyStart > start, `expected ${name} body`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(start, index + 1);
      }
    }
  }

  assert.fail(`expected ${name} body to close`);
};

test("DX evidence basket is a typed Agent UI projection over existing snapshots", () => {
  const modulePath = "crates/agent_ui/src/dx_evidence_basket.rs";

  assert.ok(existsSync(modulePath), "missing DX evidence basket module");

  const agentUi = read("crates/agent_ui/src/agent_ui.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const launchWorkspace = read("crates/agent_ui/src/dx_launch_workspace.rs");
  const basket = read(modulePath);

  assert.match(agentUi, /^mod dx_evidence_basket;$/m);
  assert.match(agentPanel, /use crate::dx_evidence_basket::dx_evidence_basket;/);
  assert.match(agentPanel, /let evidence_basket = dx_evidence_basket\(&workspace_roots\);/);
  assert.match(agentPanel, /evidence_basket,/);
  assert.match(
    launchWorkspace,
    /use crate::dx_evidence_basket::DxEvidenceBasket;/,
  );
  assert.match(launchWorkspace, /pub evidence_basket: DxEvidenceBasket/);
  assert.match(launchWorkspace, /section_title\(\s*"Evidence Basket"/);
  assert.match(launchWorkspace, /evidence_basket::evidence_basket_state/);
  assert.match(basket, /pub\(crate\) struct DxEvidenceBasket/);
  assert.match(basket, /pub\(crate\) struct DxEvidenceBasketSource/);
  assert.match(basket, /pub\(crate\) enum DxEvidenceBasketSourceKind/);
  assert.match(basket, /pub\(crate\) struct DxEvidenceBasketFlowReadiness/);
  assert.match(
    basket,
    /pub\(crate\) fn dx_evidence_basket\(workspace_roots: &\[String\]\) -> DxEvidenceBasket/,
  );

  assert.match(
    basket,
    /use crate::dx_source_sets::\{DxSourceKind, DxSourceSetSnapshot, source_set_snapshot\};/,
  );
  assert.match(
    basket,
    /use crate::dx_receipt_history::\{DxToolHistorySnapshot, tool_history_snapshot\};/,
  );
  assert.match(
    basket,
    /use crate::flow_speech_runtime::\{FlowSpeechReadinessSnapshot, FlowSpeechRuntime\};/,
  );
  assert.match(basket, /let source_sets = source_set_snapshot\(workspace_roots\);/);
  assert.match(basket, /let tool_history = tool_history_snapshot\(workspace_roots\);/);
  assert.match(
    basket,
    /let flow = FlowSpeechRuntime::detect\(\)\.readiness_snapshot\(\);/,
  );
  assert.match(basket, /let attachment_summary = source_sets\.attachment_summary\(\);/);
  assert.match(basket, /workspace_roots: attachment_summary\.workspace_roots/);
  assert.match(basket, /managed_receipts: attachment_summary\.managed_receipts/);
  assert.match(basket, /media_outputs: attachment_summary\.produced_files/);
  assert.match(basket, /forge_restore_previews: attachment_summary\.restore_previews/);
  assert.match(basket, /attachable_sources: attachment_summary\.attachable_sources/);
});

test("DX evidence basket classifies evidence without direct filesystem or runtime work", () => {
  const basket = read("crates/agent_ui/src/dx_evidence_basket.rs");

  for (const label of [
    "Metasearch",
    "Reduced Context",
    "Media Outputs",
    "Restore Previews",
  ]) {
    assert.match(basket, new RegExp(`"${label}"`));
  }

  assert.match(basket, /fn source_rows_for_set\(/);
  assert.match(basket, /fn source_kind\(kind: DxSourceKind\) -> DxEvidenceBasketSourceKind/);
  assert.match(basket, /DxSourceKind::MetasearchSourcePack/);
  assert.match(basket, /DxSourceKind::ReducedContextReceipt/);
  assert.match(basket, /DxSourceKind::MediaOutput/);
  assert.match(basket, /DxSourceKind::ForgeRestorePreview/);
  assert.match(basket, /fn receipt_bucket_count\(tool_history: &DxToolHistorySnapshot\)/);
  assert.match(basket, /bucket\.count/);
  assert.match(basket, /source\.receipt_drilldowns\.len\(\)/);
  assert.match(basket, /source\.proofs\.len\(\)/);
  assert.match(basket, /source\.warnings\.len\(\)/);

  assert.doesNotMatch(
    basket,
    /std::fs|fs::read|read_dir|read_to_string|File::open|Command::new|cx\.spawn|\bspawn\(/,
  );
  assert.doesNotMatch(basket, /PathBuf|Path::new/);
  assert.ok(lineCount("crates/agent_ui/src/dx_evidence_basket.rs") < 230);
});

test("Flow runtime exposes source-only readiness facts for the evidence basket", () => {
  const flow = read("crates/agent_ui/src/flow_speech_runtime.rs");

  assert.match(flow, /pub\(crate\) struct FlowSpeechReadinessSnapshot/);
  assert.match(flow, /pub\(crate\) flow_root: String/);
  assert.match(flow, /pub\(crate\) dictate_binary: Option<String>/);
  assert.match(flow, /pub\(crate\) stt_model: String/);
  assert.match(flow, /pub\(crate\) stt_ready: bool/);
  assert.match(flow, /pub\(crate\) stt_detail: String/);
  assert.match(flow, /pub\(crate\) kokoro_ready: bool/);
  assert.match(flow, /pub\(crate\) kokoro_detail: String/);
  assert.match(flow, /pub\(crate\) input_device_detail: String/);
  assert.match(flow, /pub\(crate\) fn readiness_snapshot\(&self\) -> FlowSpeechReadinessSnapshot/);
  assert.match(flow, /fn stt_model_label\(&self\) -> String/);
  assert.match(flow, /fn input_device_readiness_detail\(\) -> String/);
  assert.match(flow, /requested_input_device_name\(\)/);

  const readinessSnapshot = functionBody(flow, "readiness_snapshot");
  assert.doesNotMatch(
    readinessSnapshot,
    /start_recording|resolve_input_device|default_input_device|input_devices|Command::new|cpal::/,
  );
});

test("DX evidence basket guard is registered in handoff docs", () => {
  const registry = read("script/dx-handoff-source-guard-registry.test.ts");
  const dx = read("DX.md");

  assert.match(registry, /script\/dx-evidence-basket-source\.test\.ts/);
  assert.match(dx, /script\/dx-evidence-basket-source\.test\.ts/);
});
