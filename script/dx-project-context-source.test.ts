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
  const checkCandidates = functionBody(source, "check_receipt_candidates");
  const sourceScopedRoots = functionBody(source, "source_scoped_receipt_roots");
  const receiptRoot = functionBody(source, "receipt_root");
  const receiptRootFor = functionBody(source, "receipt_root_for");
  const workspaceReceiptRoots = functionBody(source, "workspace_receipt_roots");
  const receiptRootCandidates = functionBody(source, "receipt_root_candidates");
  const normalizeProjectRoot = functionBody(source, "normalize_project_root");
  const projectRootKey = functionBody(source, "project_root_key");
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
  assert.match(receiptRoot, /self\.dx_metadata_root\.join\("receipts"\)\.join\(receipt_kind\)/);
  assert.match(
    receiptRootFor,
    /Self::detect\(root\)\.map\(\|context\| context\.receipt_root\(receipt_kind\)\)/,
  );
  assert.match(workspaceReceiptRoots, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assert.match(workspaceReceiptRoots, /\.filter_map\(\|root\| Self::detect\(root\)\)/);
  assert.match(workspaceReceiptRoots, /context\.receipt_root\(receipt_kind\)/);
  assert.match(receiptRootCandidates, /contexts_for_workspace_roots\(workspace_roots\)/);
  assert.match(receiptRootCandidates, /\.take\(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT\)/);
  assertBefore(
    receiptRootCandidates,
    /context\.receipt_root\(receipt_kind\)/,
    /fallback_workspace_root\.as_ref\(\)/,
    "project-local receipt roots must be listed before shared fallback roots",
  );
  assert.match(receiptRootCandidates, /Self::receipt_root_for/);

  assert.match(normalizeProjectRoot, /for component in path\.components\(\)/);
  assert.match(
    normalizeProjectRoot,
    /Component::Prefix\(_\) \| Component::RootDir \| Component::Normal\(_\)/,
  );
  assert.match(normalizeProjectRoot, /Component::CurDir/);
  assert.match(normalizeProjectRoot, /Component::ParentDir/);
  assert.match(projectRootKey, /deploy_root_key\(&normalized\)/);
  assert.match(pathIsSameOrChild, /path_key == root_key/);
  assert.match(pathIsSameOrChild, /root_key\.ends_with\(MAIN_SEPARATOR\)/);
  assert.match(pathIsSameOrChild, /format!\("\{root_key\}\{MAIN_SEPARATOR\}"\)/);
  assert.match(pathIsSameOrChild, /path_key\.starts_with\(&child_prefix\)/);
});

test("DX project context is wired into Check, Style, Deploy, and Web Preview DX Studio", () => {
  const agentRoot = read("crates/agent_ui/src/agent_ui.rs");
  const agentPanel = read("crates/agent_ui/src/agent_panel.rs");
  const checkReader = read("crates/agent_ui/src/dx_check_panel/reader.rs");
  const styleRoots = read("crates/agent_ui/src/dx_style_panel/receipt_roots.rs");
  const deployRoots = read("crates/agent_ui/src/dx_deploy_receipt_roots.rs");
  const deployCheckRoots = read("crates/agent_ui/src/dx_deploy_check_roots.rs");
  const deployHubRoots = read("crates/agent_ui/src/dx_deploy_hub_roots.rs");
  const launchReceiptRoots = read("crates/agent_ui/src/dx_launch_receipt_roots.rs");
  const dxStudioProject = read("crates/web_preview/src/dx_studio/project.rs");

  assert.match(agentRoot, /pub mod dx_project_context;/);
  assert.match(agentRoot, /^mod dx_launch_receipt_roots;$/m);
  assert.match(checkReader, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(
    checkReader,
    /DxProjectContext::check_receipt_candidates\(workspace_roots, DX_FALLBACK_CHECK_RECEIPT\)/,
  );
  assert.match(styleRoots, /DxProjectContext::source_scoped_receipt_roots/);
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
  assert.match(deployHubRoots, /DxProjectContext::receipt_root_for\(root, "deploy"\)/);
  assert.match(launchReceiptRoots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(launchReceiptRoots, /DxProjectContext::receipt_root_candidates/);
  assert.match(launchReceiptRoots, /DxProjectContext::shared_fallback_root/);
  assert.match(launchReceiptRoots, /\.find\(\|root\| root\.is_dir\(\)\)/);
  assert.match(agentPanel, /launch_status_snapshot_for_roots\(&workspace_roots\)/);
  assert.match(agentPanel, /launch_receipt_review_snapshot_for_roots\(&workspace_roots\)/);
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
