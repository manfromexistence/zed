import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const functionBody = (source: string, name: string) => {
  const start = source.search(new RegExp(`fn\\s+${name}(?:<[^>]+>)?\\s*\\(`));
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

const assertBefore = (
  haystack: string,
  before: string | RegExp,
  after: string | RegExp,
  message: string,
) => {
  const beforeIndex =
    typeof before === "string" ? haystack.indexOf(before) : haystack.match(before)?.index ?? -1;
  const afterIndex =
    typeof after === "string" ? haystack.indexOf(after) : haystack.match(after)?.index ?? -1;
  assert.ok(beforeIndex >= 0, `missing ${before}`);
  assert.ok(afterIndex >= 0, `missing ${after}`);
  assert.ok(beforeIndex < afterIndex, message);
};

test("DX project context centralizes bounded local-first paths", () => {
  const source = read("crates/agent_ui/src/dx_project_context.rs");
  const detect = functionBody(source, "detect");
  const contextsForWorkspaceRoots = functionBody(source, "contexts_for_workspace_roots");
  const checkCandidates = functionBody(source, "check_receipt_candidates");
  const sourceScopedRoots = functionBody(source, "source_scoped_receipt_roots");
  const receiptsRoot = functionBody(source, "receipts_root");
  const receiptsRootFor = functionBody(source, "receipts_root_for");
  const receiptRoot = functionBody(source, "receipt_root");
  const receiptRootFor = functionBody(source, "receipt_root_for");
  const auditRoot = functionBody(source, "audit_root");
  const auditRootFor = functionBody(source, "audit_root_for");
  const workspaceReceiptRoots = functionBody(source, "workspace_receipt_roots");
  const receiptsRootCandidates = functionBody(source, "receipts_root_candidates");
  const auditRootCandidates = functionBody(source, "audit_root_candidates");
  const receiptRootCandidates = functionBody(source, "receipt_root_candidates");
  const sharedLaunchExamplesRoot = functionBody(source, "shared_launch_examples_root");
  const normalizeProjectRoot = functionBody(source, "normalize_project_root");
  const projectRootKey = functionBody(source, "project_root_key");
  const workspaceRootCandidate = functionBody(source, "workspace_root_candidate");
  const workspacePathCandidate = functionBody(source, "workspace_path_candidate");
  const pathIsSameOrChild = functionBody(source, "path_is_same_or_child");

  assert.match(source, /use crate::dx_deploy_root_key::deploy_root_key;/);
  assert.match(source, /pub const DX_PROJECT_CONTEXT_ANCESTOR_LIMIT: usize = 8;/);
  assert.match(source, /pub const DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT: usize = 4;/);
  assert.match(source, /pub const DX_SHARED_FALLBACK_ROOT: &str = r"G:\\Dx";/);
  assert.match(source, /pub struct DxProjectContext/);
  assert.match(source, /pub workspace_root: PathBuf/);
  assert.match(source, /pub dx_config_path: PathBuf/);
  assert.match(source, /pub dx_metadata_root: PathBuf/);
  assert.match(source, /pub diagnostics_receipt_path: PathBuf/);
  assert.match(source, /pub check_receipt_path: PathBuf/);
  assert.match(source, /pub forge_receipt_root: PathBuf/);
  assert.match(source, /pub www_route_manifest_path: PathBuf/);
  assert.match(source, /pub style_receipt_root: PathBuf/);

  assert.match(detect, /normalize_project_root\(root\.as_ref\(\)\)\?/);
  assert.match(detect, /workspace_root\.is_dir\(\)/);
  assert.match(contextsForWorkspaceRoots, /workspace_root_candidate\(root\)/);
  assert.match(contextsForWorkspaceRoots, /push_unique_path_key\(&mut seen, root\)/);
  assert.match(contextsForWorkspaceRoots, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assert.match(contextsForWorkspaceRoots, /\.filter_map\(\|root\| Self::detect\(&root\)\)/);
  assertBefore(
    contextsForWorkspaceRoots,
    /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/,
    /Self::detect\(&root\)/,
    "workspace roots must be capped before detect can touch metadata",
  );
  assertBefore(
    contextsForWorkspaceRoots,
    /push_unique_path_key\(&mut seen, root\)/,
    /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/,
    "duplicate workspace roots should not consume the root-detection cap",
  );
  assert.match(checkCandidates, /contexts_for_workspace_roots\(workspace_roots\)/);
  assertBefore(
    checkCandidates,
    /context\.check_receipt_path/,
    /fallback_receipt\.as_ref\(\)/,
    "project-local check receipts must be listed before the shared fallback",
  );
  assert.match(sourceScopedRoots, /normalize_absolute_project_path\(Path::new\(path\)\)/);
  assert.match(sourceScopedRoots, /path_is_same_or_child\(&source_path, &workspace_root\)/);
  assert.match(sourceScopedRoots, /\.take\(DX_PROJECT_CONTEXT_ANCESTOR_LIMIT\)/);
  assert.match(sourceScopedRoots, /\.join\("receipts"\)[\s\S]*\.join\(receipt_kind\)/);
  assert.doesNotMatch(sourceScopedRoots, /DX_SHARED_FALLBACK_ROOT/);
  assert.match(receiptsRoot, /self\.dx_metadata_root\.join\("receipts"\)/);
  assert.match(
    receiptsRootFor,
    /Self::detect\(root\)\.map\(\|context\| context\.receipts_root\(\)\)/,
  );
  assert.match(receiptRoot, /self\.dx_metadata_root\.join\("receipts"\)\.join\(receipt_kind\)/);
  assert.match(
    receiptRootFor,
    /Self::detect\(root\)\.map\(\|context\| context\.receipt_root\(receipt_kind\)\)/,
  );
  assert.match(auditRoot, /self\.dx_metadata_root\.join\("audit"\)\.join\(audit_kind\)/);
  assert.match(
    auditRootFor,
    /Self::detect\(root\)\.map\(\|context\| context\.audit_root\(audit_kind\)\)/,
  );
  assert.match(workspaceReceiptRoots, /workspace_path_candidate\(root\)/);
  assert.match(workspaceReceiptRoots, /push_unique_path_key\(&mut seen_roots, root\)/);
  assert.match(workspaceReceiptRoots, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assert.match(workspaceReceiptRoots, /\.filter_map\(\|root\| Self::detect\(&root\)\)/);
  assert.match(workspaceReceiptRoots, /context\.receipt_root\(receipt_kind\)/);
  assertBefore(
    workspaceReceiptRoots,
    /push_unique_path_key\(&mut seen_roots, root\)/,
    /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/,
    "duplicate PathBuf workspace roots should not consume the receipt-root cap",
  );
  assertBefore(
    workspaceReceiptRoots,
    /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/,
    /Self::detect\(&root\)/,
    "PathBuf workspace roots must be capped before detect can touch metadata",
  );
  assert.doesNotMatch(
    workspaceReceiptRoots,
    /\.iter\(\)\s*\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)\s*\.filter_map\(\|root\| Self::detect\(root\)\)/,
    "PathBuf workspace roots should normalize, absolute-filter, and dedupe before the cap",
  );
  assert.match(receiptsRootCandidates, /contexts_for_workspace_roots\(workspace_roots\)/);
  assert.match(receiptsRootCandidates, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assertBefore(
    receiptsRootCandidates,
    /context\.receipts_root\(\)/,
    /fallback_workspace_root\.as_ref\(\)/,
    "project-local receipt bucket roots must be listed before shared fallback roots",
  );
  assert.match(receiptsRootCandidates, /Self::receipts_root_for/);
  assert.match(auditRootCandidates, /contexts_for_workspace_roots\(workspace_roots\)/);
  assert.match(auditRootCandidates, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assertBefore(
    auditRootCandidates,
    /context\.audit_root\(audit_kind\)/,
    /fallback_workspace_root\.as_ref\(\)/,
    "project-local audit roots must be listed before shared fallback roots",
  );
  assert.match(auditRootCandidates, /Self::audit_root_for/);
  assert.match(receiptRootCandidates, /contexts_for_workspace_roots\(workspace_roots\)/);
  assert.match(receiptRootCandidates, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assertBefore(
    receiptRootCandidates,
    /context\.receipt_root\(receipt_kind\)/,
    /fallback_workspace_root\.as_ref\(\)/,
    "project-local receipt roots must be listed before shared fallback roots",
  );
  assert.match(receiptRootCandidates, /Self::receipt_root_for/);
  assert.match(
    sharedLaunchExamplesRoot,
    /Self::shared_fallback_root\(\)\s*\.join\("cli"\)\s*\.join\("fixtures"\)\s*\.join\("launch-examples"\)/,
  );

  assert.match(normalizeProjectRoot, /for component in path\.components\(\)/);
  assert.match(
    normalizeProjectRoot,
    /Component::Prefix\(_\) \| Component::RootDir \| Component::Normal\(_\)/,
  );
  assert.match(normalizeProjectRoot, /Component::CurDir/);
  assert.match(normalizeProjectRoot, /Component::ParentDir/);
  assert.match(projectRootKey, /deploy_root_key\(&normalized\)/);
  assert.match(workspaceRootCandidate, /let root = root\.trim\(\)/);
  assert.match(workspaceRootCandidate, /root\.is_empty\(\)/);
  assert.match(
    workspaceRootCandidate,
    /let normalized = normalize_project_root\(Path::new\(root\)\)\?/,
  );
  assert.match(workspaceRootCandidate, /normalized\.is_absolute\(\)\.then_some\(normalized\)/);
  assert.doesNotMatch(workspaceRootCandidate, /is_dir|Self::detect/);
  assert.match(workspacePathCandidate, /let normalized = normalize_project_root\(root\)\?/);
  assert.match(workspacePathCandidate, /normalized\.is_absolute\(\)\.then_some\(normalized\)/);
  assert.doesNotMatch(workspacePathCandidate, /is_dir|Self::detect/);
  assert.match(pathIsSameOrChild, /path_key == root_key/);
  assert.match(pathIsSameOrChild, /root_key\.ends_with\(MAIN_SEPARATOR\)/);
  assert.match(pathIsSameOrChild, /format!\("\{root_key\}\{MAIN_SEPARATOR\}"\)/);
  assert.match(pathIsSameOrChild, /path_key\.starts_with\(&child_prefix\)/);
});

test("DX project context is wired into Check, Style, Deploy, and Web Preview DX Studio", () => {
  const agentRoot = read("crates/agent_ui/src/agent_ui.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const agentBridgePaths = read("crates/agent_ui/src/dx_agent_bridge/paths.rs");
  const checkReader = read("crates/agent_ui/src/dx_check_panel/reader.rs");
  const stylePanel = read("crates/agent_ui/src/dx_style_panel.rs");
  const styleRoots = read("crates/agent_ui/src/dx_style_panel/receipt_roots.rs");
  const styleReadiness = read("crates/agent_ui/src/dx_style_panel/readiness.rs");
  const styleSurfaceFixture = read(
    "crates/web_preview/src/dx_style_generator_surface/fixture.rs",
  );
  const deployRoots = read("crates/agent_ui/src/dx_deploy_receipt_roots.rs");
  const deployCheckRoots = read("crates/agent_ui/src/dx_deploy_check_roots.rs");
  const deployHubRoots = read("crates/agent_ui/src/dx_deploy_hub_roots.rs");
  const receiptBuckets = read("crates/agent_ui/src/dx_receipts.rs");
  const launchReceiptRoots = read("crates/agent_ui/src/dx_launch_receipt_roots.rs");
  const launchReadiness = read("crates/agent_ui/src/dx_launch_readiness.rs");
  const launchContracts = read("crates/agent_ui/src/dx_launch_contracts.rs");
  const launchAudit = read("crates/agent_ui/src/dx_launch_audit.rs");
  const launchSourceAuditPaths = read("crates/agent_ui/src/dx_launch_source_audit/paths.rs");
  const wwwLaunchEvidence = read("crates/agent_ui/src/dx_www_launch_evidence.rs");
  const dxStudioProject = read("crates/web_preview/src/dx_studio/project.rs");

  assert.match(agentRoot, /pub mod dx_project_context;/);
  assert.match(agentBridgePaths, /use crate::dx_project_context::\{DxProjectContext, project_root_key\};/);
  assert.match(agentBridgePaths, /DxProjectContext::receipt_root_candidates/);
  assert.match(agentBridgePaths, /DxProjectContext::shared_fallback_root/);
  assert.match(agentRoot, /^mod dx_launch_receipt_roots;$/m);
  assert.match(checkReader, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(
    checkReader,
    /DxProjectContext::check_receipt_candidates\(workspace_roots, fallback_check_receipt\(\)\)/,
  );
  assert.match(
    checkReader,
    /DxProjectContext::receipt_root_for\(DxProjectContext::shared_fallback_root\(\), "check"\)/,
  );
  assert.match(styleRoots, /DxProjectContext::source_scoped_receipt_roots/);
  assert.match(stylePanel, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(stylePanel, /DxProjectContext::shared_fallback_root\(\)\.join\("style"\)/);
  assert.match(stylePanel, /DxProjectContext::shared_fallback_root\(\)\.join\("zed"\)/);
  assert.match(styleReadiness, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(
    styleReadiness,
    /DxProjectContext::receipt_root_for\(DxProjectContext::shared_fallback_root\(\), "style"\)/,
  );
  assert.match(
    styleSurfaceFixture,
    /use agent_ui::dx_project_context::DxProjectContext;/,
  );
  assert.match(
    styleSurfaceFixture,
    /DxProjectContext::shared_fallback_root\(\)\.join\("style"\)/,
  );
  assert.doesNotMatch(
    styleRoots,
    /flat_map\(Path::ancestors\)|starts_with\(workspace_root\)|replace\('\\\\', "\/"\)/,
    "style receipt root path logic should be delegated to the shared project context",
  );
  assert.match(deployRoots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(
    deployRoots,
    /DxProjectContext::workspace_receipt_roots\(workspace_roots, "deploy"\)/,
  );
  assert.match(deployCheckRoots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(
    deployCheckRoots,
    /DxProjectContext::workspace_receipt_roots\(workspace_roots, "check"\)/,
  );
  assert.match(deployCheckRoots, /DxProjectContext::receipt_root_for\(root\.as_ref\(\), "check"\)/);
  assert.match(deployHubRoots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(deployHubRoots, /DxProjectContext::shared_fallback_root/);
  assert.doesNotMatch(deployHubRoots, /r"G:\\Dx"/);
  assert.match(deployHubRoots, /DxProjectContext::receipt_root_for\(root, "deploy"\)/);
  assert.match(receiptBuckets, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(receiptBuckets, /DxProjectContext::receipts_root_candidates/);
  assert.match(launchReceiptRoots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(launchReceiptRoots, /DxProjectContext::receipt_root_candidates/);
  assert.match(launchReceiptRoots, /DxProjectContext::shared_fallback_root/);
  assert.match(launchReceiptRoots, /\.find\(\|root\| root\.is_dir\(\)\)/);
  for (const launchExampleSurface of [launchReadiness, launchContracts, launchAudit]) {
    assert.match(launchExampleSurface, /use crate::dx_project_context::DxProjectContext;/);
    assert.match(launchExampleSurface, /DxProjectContext::shared_launch_examples_root\(\)/);
    assert.doesNotMatch(
      launchExampleSurface,
      /DX_LAUNCH_EXAMPLES_ROOT|r"G:\\Dx\\cli\\fixtures\\launch-examples"/,
    );
  }
  assert.match(agentPanel, /launch_status_snapshot_for_roots\(&workspace_roots\)/);
  assert.match(agentPanel, /launch_receipt_review_snapshot_for_roots\(&workspace_roots\)/);
  assert.match(agentPanel, /launch_source_audit_snapshot_for_roots\(&workspace_roots\)/);
  assert.match(agentPanel, /receipt_snapshot_for_roots\(&workspace_roots\)/);
  assert.match(
    agentPanel,
    /dx_agent_bridge_snapshot_from_settings_for_roots\(\s*input\.agent_settings,\s*&workspace_roots,\s*\)/,
  );
  assert.match(launchSourceAuditPaths, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(launchSourceAuditPaths, /DxProjectContext::audit_root_candidates/);
  assert.match(launchSourceAuditPaths, /DxProjectContext::shared_fallback_root/);
  assert.match(
    wwwLaunchEvidence,
    /use crate::dx_project_context::\{DxProjectContext, project_root_key\};/,
  );
  assert.match(wwwLaunchEvidence, /DxProjectContext::shared_fallback_root\(\)/);
  assert.match(wwwLaunchEvidence, /seen\.insert\(project_root_key\(path\)\)/);
  assert.match(dxStudioProject, /use agent_ui::dx_project_context::DxProjectContext;/);
  assert.match(dxStudioProject, /let project_context = DxProjectContext::detect\(root\);/);
  assert.match(dxStudioProject, /context\.dx_config_path/);
  assert.match(dxStudioProject, /context\.dx_metadata_root/);
  assert.match(dxStudioProject, /context\.forge_receipt_root/);
  assert.match(dxStudioProject, /context\.www_route_manifest_path/);
});

test("DX project context keeps IO out of render paths", () => {
  const context = read("crates/agent_ui/src/dx_project_context.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const projectPanel = read("crates/project_panel/src/project_panel.rs");

  assert.doesNotMatch(context, /\bread_dir\(|\bFile::open\(|read_to_string|read_to_end\(/);
  assert.doesNotMatch(context, /background_executor|cx\.spawn|cx\.spawn_in|render\(/);

  const agentPanelRender = functionBody(agentPanel, "render");
  assert.doesNotMatch(
    agentPanelRender,
    /DxProjectContext|check_receipt_candidates|source_scoped_receipt_roots|workspace_receipt_roots|receipt_root_candidates/,
  );

  const projectPanelRender = functionBody(projectPanel, "render");
  assert.doesNotMatch(
    projectPanelRender,
    /DxProjectContext|check_receipt_candidates|source_scoped_receipt_roots|workspace_receipt_roots|receipt_root_candidates/,
  );
});
