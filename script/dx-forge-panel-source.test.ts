import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

const agentUi = readFileSync("crates/agent_ui/src/agent_ui.rs", "utf8");
const moduleRoot = readFileSync("crates/agent_ui/src/dx_forge_panel.rs", "utf8");
const controls = readFileSync("crates/agent_ui/src/dx_forge_panel/controls.rs", "utf8");
const panel = readFileSync("crates/agent_ui/src/dx_forge_panel/panel.rs", "utf8");
const rowSelectionPath = "crates/agent_ui/src/dx_forge_panel/row_selection.rs";
const rowSelection = existsSync(rowSelectionPath)
  ? readFileSync(rowSelectionPath, "utf8")
  : "";
const rootsPath = "crates/agent_ui/src/dx_forge_panel/roots.rs";
const roots = existsSync(rootsPath)
  ? readFileSync(rootsPath, "utf8")
  : "";
const machineCachePath = "crates/agent_ui/src/dx_forge_panel/machine_cache.rs";
const machineCache = existsSync(machineCachePath)
  ? readFileSync(machineCachePath, "utf8")
  : "";
const packageStatusPath = "crates/agent_ui/src/dx_forge_panel/package_status.rs";
const packageStatus = existsSync(packageStatusPath)
  ? readFileSync(packageStatusPath, "utf8")
  : "";
const packageStatusCachePath =
  "crates/agent_ui/src/dx_forge_panel/package_status_cache.rs";
const packageStatusCache = existsSync(packageStatusCachePath)
  ? readFileSync(packageStatusCachePath, "utf8")
  : "";
const remoteRegistryPath = "crates/agent_ui/src/dx_forge_panel/remote_registry.rs";
const remoteRegistry = existsSync(remoteRegistryPath)
  ? readFileSync(remoteRegistryPath, "utf8")
  : "";
const remoteRegistryProvidersPath =
  "crates/agent_ui/src/dx_forge_panel/remote_registry/providers.rs";
const remoteRegistryProviders = existsSync(remoteRegistryProvidersPath)
  ? readFileSync(remoteRegistryProvidersPath, "utf8")
  : "";
const sourceSectionPath = "crates/agent_ui/src/dx_forge_panel/source_section.rs";
const sourceSection = existsSync(sourceSectionPath)
  ? readFileSync(sourceSectionPath, "utf8")
  : "";
const workflowRowsPath = "crates/agent_ui/src/dx_forge_panel/workflow_rows.rs";
const workflowRows = existsSync(workflowRowsPath)
  ? readFileSync(workflowRowsPath, "utf8")
  : "";
const visibleRowsPath = "crates/agent_ui/src/dx_forge_panel/visible_rows.rs";
const visibleRows = existsSync(visibleRowsPath)
  ? readFileSync(visibleRowsPath, "utf8")
  : "";
const providersRootPath = "crates/agent_ui/src/dx_forge_panel/providers/mod.rs";
const providersCatalogPath = "crates/agent_ui/src/dx_forge_panel/providers/catalog.rs";
const providersStatePath = "crates/agent_ui/src/dx_forge_panel/providers/state.rs";
const providersViewPath = "crates/agent_ui/src/dx_forge_panel/providers/view.rs";
const providersTooltipsPath = "crates/agent_ui/src/dx_forge_panel/providers/tooltips.rs";
const providersRoot = existsSync(providersRootPath)
  ? readFileSync(providersRootPath, "utf8")
  : "";
const providersCatalog = existsSync(providersCatalogPath)
  ? readFileSync(providersCatalogPath, "utf8")
  : "";
const providersState = existsSync(providersStatePath)
  ? readFileSync(providersStatePath, "utf8")
  : "";
const providersView = existsSync(providersViewPath)
  ? readFileSync(providersViewPath, "utf8")
  : "";
const providersTooltips = existsSync(providersTooltipsPath)
  ? readFileSync(providersTooltipsPath, "utf8")
  : "";

const normalizedPath = (path: string) => path.replaceAll("\\", "/");

const collectRustFiles = (root: string): string[] =>
  readdirSync(root, { withFileTypes: true })
    .flatMap((entry) => {
      const child = join(root, entry.name);
      if (entry.isDirectory()) return collectRustFiles(child);
      return entry.name.endsWith(".rs") ? [normalizedPath(child)] : [];
    })
    .sort();

const extractRustMethod = (source: string, name: string): string => {
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const signature = new RegExp(
    `\\n    (?:pub\\(super\\)\\s+)?fn\\s+${escapedName}\\b`,
  );
  const match = signature.exec(source);
  assert.ok(match, `missing Rust method ${name}`);

  const bodyStart = source.indexOf("{", match.index);
  assert.notEqual(bodyStart, -1, `missing Rust method body for ${name}`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index++) {
    const character = source[index];
    if (character === "{") depth++;
    if (character === "}") depth--;
    if (depth === 0) return source.slice(match.index, index + 1);
  }

  assert.fail(`unterminated Rust method body for ${name}`);
};

const extractRustFunction = (source: string, name: string): string => {
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const signature = new RegExp(
    `(?:^|\\n)(?:pub(?:\\([^)]*\\))?\\s+)?fn\\s+${escapedName}\\b`,
  );
  const match = signature.exec(source);
  assert.ok(match, `missing Rust function ${name}`);

  const bodyStart = source.indexOf("{", match.index);
  assert.notEqual(bodyStart, -1, `missing Rust function body for ${name}`);

  let depth = 0;
  for (let index = bodyStart; index < source.length; index++) {
    const character = source[index];
    if (character === "{") depth++;
    if (character === "}") depth--;
    if (depth === 0) return source.slice(match.index, index + 1);
  }

  assert.fail(`unterminated Rust function body for ${name}`);
};
const escapedTextPattern = (text: string): RegExp =>
  new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
const snapshot = readFileSync("crates/agent_ui/src/dx_forge_panel/snapshot.rs", "utf8");
const snapshotStatePath = "crates/agent_ui/src/dx_forge_panel/snapshot_state.rs";
const snapshotState = existsSync(snapshotStatePath)
  ? readFileSync(snapshotStatePath, "utf8")
  : "";
const panelView = readFileSync("crates/agent_ui/src/dx_forge_panel/panel_view.rs", "utf8");
const rows = readFileSync("crates/agent_ui/src/dx_forge_panel/rows.rs", "utf8");
const tabs = readFileSync("crates/agent_ui/src/dx_forge_panel/tabs.rs", "utf8");
const icons = readFileSync("crates/icons/src/icons.rs", "utf8");
const dxIcons = readFileSync("crates/ui/src/dx_icons.rs", "utf8");
const receiptHistoryRoot = readFileSync("crates/agent_ui/src/dx_receipt_history.rs", "utf8");
const sourceSetsRoot = readFileSync("crates/agent_ui/src/dx_source_sets.rs", "utf8");
const sourceSetCache = readFileSync("crates/agent_ui/src/dx_source_sets/cache.rs", "utf8");
const zedActions = readFileSync("crates/zed_actions/src/lib.rs", "utf8");
const receiptBuckets = readFileSync("crates/agent_ui/src/dx_receipt_history/buckets.rs", "utf8");
const forgeHistory = readFileSync("crates/agent_ui/src/dx_receipt_history/forge_history.rs", "utf8");
const receiptFiles = readFileSync("crates/agent_ui/src/dx_receipt_history/receipt_files.rs", "utf8");
const receiptFields = readFileSync(
  "crates/agent_ui/src/dx_receipt_history/forge_receipt_fields.rs",
  "utf8",
);
const sourceSets = readFileSync("crates/agent_ui/src/dx_source_sets.rs", "utf8");
const sourceSetReceipts = readFileSync("crates/agent_ui/src/dx_source_sets/receipts.rs", "utf8");
const providers = [
  providersRoot,
  providersCatalog,
  providersState,
  providersView,
  providersTooltips,
].join("\n");
const remoteRegistrySources = [remoteRegistry, remoteRegistryProviders].join("\n");
const forgeSources = [
  moduleRoot,
  controls,
  rowSelection,
  machineCache,
  packageStatus,
  packageStatusCache,
  remoteRegistrySources,
  panel,
  roots,
  providers,
  sourceSection,
  visibleRows,
  workflowRows,
  snapshot,
  snapshotState,
  panelView,
  rows,
].join("\n");
const forgeReaderSources = [
  machineCache,
  packageStatus,
  packageStatusCache,
  remoteRegistrySources,
  receiptBuckets,
  receiptFiles,
  receiptFields,
  sourceSets,
  sourceSetReceipts,
].join("\n");

test("Forge panel is wired through agent UI without touching Git panel ownership", () => {
  assert.match(agentUi, /\r?\nmod dx_forge_panel;\r?\n/);
  assert.match(agentUi, /dx_forge_panel::panel::init\(cx\);/);
  assert.ok(
    agentUi.indexOf("agent_panel::init(cx);") <
      agentUi.indexOf("dx_forge_panel::panel::init(cx);"),
    "Forge panel should be registered beside the existing agent panels",
  );
  assert.doesNotMatch(forgeSources, /git_panel|GitPanel|git_ui::/);
  assert.doesNotMatch(
    forgeSources,
    /\bgit::|GitStore|git_store|GitRepository/,
  );
});

test("Forge panel owns a stable local action and dock identity", () => {
  assert.match(zedActions, /pub mod dx_forge/);
  assert.match(zedActions, /actions!\(\s*dx_forge,/);
  assert.doesNotMatch(moduleRoot, /actions!\(/);
  assert.match(panel, /use zed_actions::dx_forge::TogglePanel;/);
  assert.match(panel, /workspace\.register_action\(\|workspace, _:\s*&TogglePanel/);
  assert.ok(
    panel.indexOf("workspace.register_action") <
      panel.indexOf("let Some(window) = window else"),
    "Forge action should be registered even when observe_new has no window yet",
  );
  assert.match(panel, /const DX_FORGE_PANEL_KEY: &str = "dx_forge_panel";/);
  assert.match(panel, /fn persistent_name\(\) -> &'static str \{\s*"Forge"/);
  assert.match(panel, /DockPosition::Left/);
  assert.match(panel, /position == DockPosition::Left/);
  assert.match(panel, /use ui::\{DxUiIcon, IconName, dx_icon\};/);
  assert.match(panel, /use ui::prelude::\*/);
  assert.match(panel, /Some\(dx_icon\(DxUiIcon::Forge\)\)/);
  assert.match(panel, /fn activation_priority\(&self\) -> u32 \{\s*4\s*\}/);
  assert.match(panel, /fn starts_open\(&self, _:\s*&Window, _:\s*&App\) -> bool \{\s*false/);
  assert.doesNotMatch(panel, /DockPosition::Right|DockPosition::Bottom/);
});

test("Forge snapshot reuses existing bounded DX readers", () => {
  assert.match(snapshot, /tool_history_snapshot\(workspace_roots\)/);
  assert.match(snapshot, /source_set_snapshot\(workspace_roots\)/);
  assert.match(snapshot, /"Forge History"/);
  assert.match(snapshot, /"Restore Previews"/);
  assert.match(snapshot, /"Media Outputs"/);
  assert.match(moduleRoot, /mod snapshot_state;/);
  assert.match(snapshotState, /const MAX_WORKSPACE_ROOTS: usize = 4;/);
  assert.match(snapshot, /workspace_scope\(workspace_roots\)/);
  assert.match(snapshot, /configured_forge_root_count\(workspace_roots\)/);
  assert.match(receiptBuckets, /Path::new\("tools"\)\.join\("dx-forge"\)/);
  assert.match(sourceSets, /join\("dx-forge"\)\.join\("restores"\)/);
  assert.match(sourceSets, /join\("dx-media"\)\.join\("executions"\)/);
  assert.match(snapshot, /target_path: summary\.target_path\.clone\(\)/);
  assert.match(snapshot, /source_path: summary\.source_path\.clone\(\)/);
  assert.match(snapshot, /restore_destination_root: summary\.restore_destination_root\.clone\(\)/);
  assert.match(snapshot, /blocker_count: summary\.blocker_count/);
  assert.match(receiptHistoryRoot, /pub source_path: String/);
  assert.match(forgeHistory, /source_path: path\.display\(\)\.to_string\(\)/);
  assert.match(snapshot, /warnings: source\.warnings\.clone\(\)/);
  assert.match(snapshot, /receipt_drilldowns/);
  assert.doesNotMatch(
    `${forgeSources}\n${forgeReaderSources}`,
    /std::process|Command::new|powershell|cmd\.exe|\bshell\b|spawn/i,
  );
});

test("Forge filesystem readers share DxProjectContext-derived roots", () => {
  assert.ok(existsSync(rootsPath), "Forge root derivation must live in a focused module");
  assert.match(moduleRoot, /mod roots;/);
  assert.match(roots, /use crate::dx_project_context::DxProjectContext;/);
  assert.match(roots, /pub\(super\) struct ForgeRootContext/);
  assert.match(roots, /pub\(super\) fn forge_root_contexts\(workspace_roots: &\[String\]\)/);
  assert.match(roots, /DxProjectContext::contexts_for_workspace_roots\(workspace_roots\)/);
  assert.match(roots, /context\.dx_metadata_root/);
  assert.match(roots, /context\.workspace_root/);
  assert.match(roots, /pub\(super\) fn forge_package_status_candidates/);
  assert.match(roots, /pub\(super\) fn forge_remote_registry_path/);
  assert.match(roots, /pub\(super\) fn forge_machine_cache_root/);
  assert.match(roots, /pub\(super\) fn configured_forge_root_count/);
  assert.match(roots, /canonical_forge_root/);
  assert.match(roots, /legacy_forge_root/);
  assert.match(roots, /shared_fallback_forge_root/);
  assert.match(roots, /shared_fallback_metadata_forge_root/);
  assert.match(roots, /shared_fallback: false/);
  assert.match(roots, /shared_fallback: true/);
  assert.match(roots, /fn is_workspace_configured/);
  assert.match(roots, /!self\.shared_fallback && self\.is_configured\(\)/);
  assert.match(roots, /filter\(ForgeRootContext::is_workspace_configured\)/);
  assert.match(roots, /PathBuf/);

  assert.match(snapshot, /use crate::dx_forge_panel::roots::configured_forge_root_count;/);
  assert.doesNotMatch(snapshotState, /Path::new\(root\)[\s\S]*join\("\.dx"\)\.join\("forge"\)/);
  assert.doesNotMatch(snapshotState, /Path::new\(root\)[\s\S]*join\("tools"\)\.join\("dx-forge"\)/);
  assert.match(remoteRegistry, /use super::roots::forge_root_contexts;/);
  assert.match(remoteRegistry, /for context in forge_root_contexts\(workspace_roots\)/);
  assert.match(remoteRegistry, /context\.forge_remote_registry_path\(\)/);
  assert.match(packageStatus, /use super::roots::forge_root_contexts;/);
  assert.match(packageStatus, /for context in forge_root_contexts\(workspace_roots\)/);
  assert.match(packageStatus, /context\s*\.forge_package_status_candidates\(\)/);
  assert.match(machineCache, /use super::roots::forge_root_contexts;/);
  assert.match(machineCache, /for context in forge_root_contexts\(workspace_roots\)/);
  assert.match(machineCache, /context\.forge_machine_cache_root\(\)/);

  for (const [name, source] of [
    ["remote_registry.rs", remoteRegistry],
    ["package_status.rs", packageStatus],
    ["machine_cache.rs", machineCache],
    ["snapshot_state.rs", snapshotState],
  ] as const) {
    assert.doesNotMatch(
      source,
      /Path::new\(root\)\.join\("\.dx"\)|Path::new\(root\)\.join\("\.forge"\)|Path::new\(root\)\.join\("tools"\)\.join\("dx-forge"\)/,
      `${name} should use dx_forge_panel::roots instead of rebuilding Forge roots`,
    );
  }
});

test("Forge panel reads package-status without runtime overclaims", () => {
  assert.ok(
    existsSync(packageStatusPath),
    "Forge package-status reader must live in a focused module",
  );
  assert.match(moduleRoot, /mod package_status;/);
  assert.match(moduleRoot, /mod package_status_cache;/);
  assert.match(snapshot, /package_status_rows\(workspace_roots\)/);
  assert.match(snapshot, /pub\(super\) package_statuses: Vec<DxForgeSourceRow>/);
  assert.match(snapshot, /visible_package_status_warning_count/);
  assert.match(snapshot, /PACKAGE_STATUS_LABEL/);
  assert.match(panelView, /fn package_status_section/);
  assert.ok(
    panelView.indexOf("package_status_section(snapshot, workspace, panel, cx)") <
      panelView.indexOf("machine_cache_section(snapshot, workspace, panel, cx)"),
    "package status should be visible before machine cache evidence",
  );
  assert.match(panelView, /title: PACKAGE_STATUS_LABEL/);
  assert.match(panelView, /No package status found/);
  assert.match(packageStatusCache, /const PACKAGE_STATUS_CACHE_TTL: Duration = Duration::from_secs\(5\);/);
  assert.match(packageStatus, /const MAX_PACKAGE_STATUS_BYTES: u64 = 1024 \* 1024;/);
  assert.match(packageStatusCache, /OnceLock<[\s\S]*Mutex<Option<\(Instant, Vec<String>, Vec<DxForgeSourceRow>\)>>/);
  assert.match(packageStatus, /pub\(super\) fn invalidate_package_status_snapshot_cache/);
  assert.match(packageStatus, /package_status_cache::invalidate_package_status_snapshot_cache\(\)/);
  assert.match(packageStatus, /pub\(super\) fn scan_package_status_rows\(workspace_roots: &\[String\]\)/);
  assert.match(packageStatusCache, /let rows = scan_package_status_rows\(workspace_roots\)/);
  assert.match(packageStatusCache, /now\.duration_since\(\*cached_at\) <= PACKAGE_STATUS_CACHE_TTL/);
  assert.match(roots, /join\("\.dx"\)[\s\S]*\.join\("forge"\)[\s\S]*\.join\("package-status\.json"\)/);
  assert.match(roots, /join\("\.forge"\)[\s\S]*\.join\("receipts"\)[\s\S]*\.join\("package-status\.json"\)/);
  assert.ok(
    roots.indexOf('join(".forge")') < roots.indexOf('join(".dx")'),
    "canonical .forge/receipts package-status should be checked before legacy .dx/forge package-status",
  );
  assert.match(packageStatus, /package_status_candidate_rows/);
  assert.match(packageStatus, /\.find_map\(\|path\|/);
  assert.doesNotMatch(packageStatus, /\.filter_map\(\|path\|/);
  assert.match(packageStatus, /file\.by_ref\(\)\s*\.take\(MAX_PACKAGE_STATUS_BYTES \+ 1\)/);
  assert.match(packageStatus, /serde_json::from_slice/);
  assert.match(packageStatus, /forge\.package_status_receipt/);
  assert.match(packageStatus, /forge_package_status_row/);
  assert.match(packageStatus, /unreadable_package_status_row/);
  assert.match(packageStatus, /path\.is_file\(\)/);
  assert.match(packageStatus, /package status could not be read within/);
  assert.match(packageStatus, /forge_summary_missing_count/);
  assert.match(packageStatus, /summary field\(s\) missing/);
  assert.match(packageStatus, /package_lock_present/);
  assert.match(packageStatus, /integrity_state/);
  assert.match(packageStatus, /valid_packages/);
  assert.match(packageStatus, /missing_packages/);
  assert.match(packageStatus, /mismatched_packages/);
  assert.match(packageStatus, /unsafe_remote_count/);
  assert.match(packageStatus, /tracked_media_assets/);
  assert.match(packageStatus, /receipt file only; live checks not executed/);
  assert.match(packageStatus, /package_lane_visibility/);
  assert.match(packageStatus, /receipt_hash_refresh/);
  assert.match(packageStatus, /no_node_modules_required/);
  assert.match(packageStatus, /runtime_execution/);
  assert.match(packageStatus, /browser_proof/);
  assert.match(packageStatus, /live_provider_proof/);
  assert.match(packageStatus, /source-only receipt evidence/);
  assert.doesNotMatch(packageStatus, /runtime\/provider proof pending/);
  assert.match(packageStatus, /package_status_evidence_detail\(/);
  assert.match(packageStatus, /status_detail\(/);
  const statusDetailBody =
    packageStatus.match(/fn status_detail\([\s\S]*?\r?\n}\r?\n\r?\nfn package_status_label/)?.[0] ?? "";
  assert.match(
    statusDetailBody,
    /\{package_count\} packages · \{status\} · \{current_receipts\} receipt hashes current/,
  );
  assert.doesNotMatch(
    statusDetailBody,
    /node_modules|proof|evidence|source-only|runtime\/provider|live checks not executed/i,
  );
  assert.match(
    packageStatus,
    /detail:\s*format!\("\{status\} package-status; \{node_modules\}; \{evidence_detail\}"\)/,
  );
  assert.match(packageStatus, /warning_count\(/);
  assert.doesNotMatch(packageStatus, /\.machine/);
  assert.doesNotMatch(
    packageStatus,
    /std::process|Command::new|powershell|cmd\.exe|\bshell\b|spawn/i,
  );
  assert.doesNotMatch(
    `${packageStatus}\n${panelView}`,
    /connected|synced live|runtime proven|provider proven|browser verified/i,
  );
  assert.match(panel, /use crate::dx_forge_panel::package_status::invalidate_package_status_snapshot_cache;/);
  assert.match(panel, /invalidate_package_status_snapshot_cache\(\)/);
  assert.ok(
    panel.indexOf("invalidate_machine_cache_snapshot_cache()") <
      panel.indexOf("invalidate_package_status_snapshot_cache()") &&
      panel.indexOf("invalidate_package_status_snapshot_cache()") <
        panel.indexOf("invalidate_remote_registry_snapshot_cache()"),
    "Forge refresh should invalidate all cached filesystem-backed readers together",
  );
});

test("Forge panel surfaces bounded machine-cache evidence without freshness overclaims", () => {
  assert.ok(
    existsSync(machineCachePath),
    "Forge machine-cache reader must live in a focused module",
  );
  assert.match(moduleRoot, /mod machine_cache;/);
  assert.match(snapshot, /machine_cache_rows\(workspace_roots\)/);
  assert.match(snapshot, /pub\(super\) machine_caches: Vec<DxForgeSourceRow>/);
  assert.match(snapshot, /visible_machine_cache_warning_count/);
  assert.match(snapshot, /MACHINE_CACHES_LABEL/);
  assert.match(panelView, /fn machine_cache_section/);
  assert.ok(
    panelView.indexOf("package_status_section(snapshot, workspace, panel, cx)") <
      panelView.indexOf("machine_cache_section(snapshot, workspace, panel, cx)") &&
      panelView.indexOf("media_section(snapshot, workspace, panel, cx)") <
        panelView.indexOf("restore_section(snapshot, workspace, panel, cx)"),
    "package and media workflow sections should keep their source-backed order",
  );
  assert.match(panelView, /title: MACHINE_CACHES_LABEL/);
  assert.match(panelView, /No machine caches found/);
  assert.match(machineCache, /const MACHINE_CACHE_CACHE_TTL: Duration = Duration::from_secs\(5\);/);
  assert.match(machineCache, /const MAX_MACHINE_CACHE_ROOTS: usize = 4;/);
  assert.match(machineCache, /const MAX_MACHINE_CACHE_FILES: usize = 64;/);
  assert.match(machineCache, /const MAX_MACHINE_CACHE_DIRECTORIES: usize = 512;/);
  assert.match(machineCache, /const MAX_MACHINE_CACHE_ENTRIES_PER_DIRECTORY: usize = 256;/);
  assert.match(machineCache, /const MAX_MACHINE_CACHE_TOTAL_ENTRIES: usize = 4096;/);
  assert.match(machineCache, /const MACHINE_HEADER_BYTES: usize = 8;/);
  assert.match(machineCache, /OnceLock<Mutex<Option<\(Instant, Vec<String>, Vec<DxForgeSourceRow>\)>>/);
  assert.match(machineCache, /pub\(super\) fn invalidate_machine_cache_snapshot_cache/);
  assert.match(machineCache, /VecDeque::from/);
  assert.match(machineCache, /summary\.scanned_directories >= MAX_MACHINE_CACHE_DIRECTORIES/);
  assert.match(machineCache, /total_entries >= MAX_MACHINE_CACHE_TOTAL_ENTRIES/);
  assert.match(machineCache, /directory_entries > MAX_MACHINE_CACHE_ENTRIES_PER_DIRECTORY/);
  assert.match(machineCache, /let dx_root = context\.forge_machine_cache_root\(\);/);
  assert.match(roots, /join\("\.dx"\)/);
  assert.match(machineCache, /eq_ignore_ascii_case\("machine"\)/);
  assert.match(machineCache, /File::open\(path\)\.ok\(\)\?/);
  assert.match(machineCache, /file\.read\(&mut bytes\)\.ok\(\)\?/);
  assert.match(machineCache, /DXM1/);
  assert.match(machineCache, /DXMCACH1/);
  assert.match(machineCache, /metadata_path_for/);
  assert.match(machineCache, /\.machine\.meta\.json/);
  assert.match(machineCache, /freshness unchecked/);
  assert.match(machineCache, /metadata sidecar/);
  assert.match(machineCache, /Machine family/);
  assert.match(machineCache, /serializer document cache/);
  assert.match(machineCache, /typed cache/);
  assert.match(machineCache, /unknown machine cache/);
  assert.match(machineCache, /std::fs::read_dir/);
  assert.match(machineCache, /entry\.file_type\(\)/);
  assert.doesNotMatch(machineCache, /std::fs::read\(/);
  assert.doesNotMatch(machineCache, /read_to_end|read_to_string|std::fs::read_to_string/);
  assert.match(snapshot, /let machine_caches = machine_cache_rows\(workspace_roots\);/);
  assert.match(snapshot, /machine_cache_count: machine_caches\.len\(\)/);
  assert.match(snapshot, /machine_caches_label: MACHINE_CACHES_LABEL/);
  assert.match(snapshot, /machine_caches,/);
  assert.match(snapshotState, /machine_cache_count: usize/);
  assert.match(snapshotState, /machine_caches_label: &'static str/);
  assert.match(snapshotState, /visible_machine_cache_warning_count: usize/);
  assert.match(snapshotState, /visible machine cache warning\(s\) need review/);
  assert.match(snapshotState, /Missing Forge receipt, remote-registry, package-status, or machine-cache root/);
  assert.match(snapshotState, /input\.machine_cache_count > 0/);
  assert.match(panelView, /&snapshot\.machine_caches/);
  assert.match(panelView, /Open a workspace to inspect machine caches/);
  assert.match(panelView, /row_id: "dx-forge-machine-cache"/);
  assert.match(panelView, /open_id: "dx-forge-open-machine-cache-root"/);
  assert.match(panelView, /open_tooltip: "Open machine cache root"/);
  assert.doesNotMatch(
    machineCache,
    /std::process|Command::new|powershell|cmd\.exe|\bshell\b|spawn/i,
  );
  assert.doesNotMatch(
    `${machineCache}\n${panelView}`,
    /source hash matches|receipt-backed machine|provider proof|browser proof|runtime proof|live Forge status|fresh machine|verified machine|hash verified|metadata verified|freshness verified|cache verified|runtime-backed machine|browser-backed machine|provider-backed machine/i,
  );
});

test("Forge panel reads Forge remote registry and makes provider targets concrete", () => {
  assert.ok(
    existsSync(remoteRegistryPath),
    "Forge remote registry reader must live in a focused module",
  );
  assert.match(moduleRoot, /mod remote_registry;/);
  assert.match(snapshot, /remote_registry_snapshot\(workspace_roots\)/);
  assert.match(snapshot, /pub\(super\) remote_registries: Vec<DxForgeSourceRow>/);
  assert.match(snapshot, /pub\(super\) remote_providers: Vec<DxForgeRemoteProvider>/);
  assert.match(snapshot, /pub\(super\) open_path: String/);
  assert.match(snapshot, /pub\(super\) registry_open_path: String/);
  assert.match(snapshot, /visible_remote_registry_warning_count/);
  assert.match(snapshot, /REMOTE_REGISTRY_LABEL/);
  assert.match(panel, /invalidate_remote_registry_snapshot_cache\(\)/);
  assert.match(panelView, /fn remote_registry_section/);
  assert.match(panelView, /title: REMOTE_REGISTRY_LABEL/);
  assert.match(panelView, /No remote registry found/);
  const remotesBranch =
    panelView.match(/DxForgePanelTab::Remotes => vec!\[[\s\S]*?\],/)?.[0] ?? "";
  assert.match(remotesBranch, /remote_target_strip\(snapshot, workspace, panel, cx\)/);
  assert.match(remotesBranch, /remote_registry_section\(snapshot, workspace, panel, cx\)/);
  assert.ok(
    remotesBranch.indexOf("remote_target_strip(snapshot, workspace, panel, cx)") <
      remotesBranch.indexOf("remote_registry_section(snapshot, workspace, panel, cx)"),
    "remote targets should be visible before remote registry evidence",
  );

  assert.match(remoteRegistry, /const REMOTE_REGISTRY_CACHE_TTL: Duration = Duration::from_secs\(5\);/);
  assert.match(remoteRegistry, /const MAX_REMOTE_REGISTRY_BYTES: u64 = 256 \* 1024;/);
  assert.match(remoteRegistry, /const MAX_WORKSPACE_ROOTS: usize = 4;/);
  assert.match(roots, /join\("\.forge"\)[\s\S]*\.join\("remotes\.json"\)/);
  assert.match(remoteRegistry, /File::open\(path\)\.ok\(\)\?/);
  assert.match(remoteRegistry, /file\.by_ref\(\)\s*\.take\(MAX_REMOTE_REGISTRY_BYTES \+ 1\)/);
  assert.match(remoteRegistry, /serde_json::from_slice/);
  assert.match(remoteRegistry, /get\("primary"\)/);
  assert.match(remoteRegistry, /get\("remotes"\)/);
  assert.match(remoteRegistrySources, /kind_counts/);
  assert.match(remoteRegistrySources, /enabled_count/);
  assert.match(remoteRegistrySources, /disabled_count/);
  assert.match(remoteRegistrySources, /branch_mapping_count/);
  assert.match(remoteRegistrySources, /auth_backend_count/);
  assert.match(remoteRegistrySources, /registry file only; health unchecked/);
  assert.match(remoteRegistry, /let registry_open_path = path\.display\(\)\.to_string\(\)/);
  assert.match(remoteRegistry, /open_path: registry_open_path\.clone\(\)/);
  assert.match(
    remoteRegistry,
    /remote_providers\([\s\S]*&remotes,[\s\S]*primary\.as_deref\(\),[\s\S]*&path_label,[\s\S]*&registry_open_path,[\s\S]*MAX_REMOTE_ROWS,[\s\S]*\)/,
  );
  assert.match(remoteRegistryProviders, /registry_open_path: &str/);
  assert.match(remoteRegistryProviders, /registry_open_path: registry_open_path\.to_string\(\)/);
  assert.match(sourceSection, /&row\.open_path/);

  for (const kind of [
    "GitHub",
    "GitLab",
    "Bitbucket",
    "GoogleDrive",
    "Dropbox",
    "YouTube",
    "SoundCloud",
    "SoundBox",
  ]) {
    assert.match(remoteRegistrySources, new RegExp(kind));
  }

  assert.match(
    remoteRegistryProviders,
    /"soundcloud" => Some\(\("soundcloud", "media", "SoundCloud"\)\)/,
  );
  assert.match(
    remoteRegistryProviders,
    /"soundbox" => Some\(\("soundbox", "media", "SoundBox"\)\)/,
  );
  assert.match(
    remoteRegistryProviders,
    /"soundcloud" => "SoundCloud"/,
  );
  assert.match(
    remoteRegistryProviders,
    /"soundbox" => "SoundBox"/,
  );

  assert.match(snapshot, /remote_provider_for\(&self, provider_id: &str\)/);
  assert.match(snapshot, /fn remote_provider_rank/);
  assert.doesNotMatch(snapshot, /\.find\(\|provider\| provider\.provider_id == provider_id\)/);
  assert.match(snapshot, /remote_registry_count_for_group\(&self, group_key: &str\)/);
  assert.match(snapshot, /configured_provider_count_for_group\(&self, group_key: &str\)/);
  assert.match(providersState, /fn provider_target_state/);
  assert.match(providersState, /snapshot\.remote_provider_for\(provider\.id\)/);
  assert.match(providersState, /configured_provider_count_for_group\(group\.key\(\)\)/);
  assert.match(providersState, /"Not configured"/);
  assert.match(providersState, /"Configured"/);
  assert.match(
    providersState,
    /fn target_state\(label: &'static str, detail: impl Into<String>, color: Color\)/,
  );
  assert.doesNotMatch(providersState, /IconName::|_icon: IconName/);
  assert.doesNotMatch(
    providersState,
    /health unchecked[\s\S]{0,260}Color::Success|Color::Success[\s\S]{0,260}health unchecked/,
  );
  assert.match(providersView, /provider_target_state\(provider, snapshot\)/);
  assert.match(providersView, /target_path_for_provider\(provider, snapshot\)/);
  assert.match(providersView, /target_open_path_for_provider\(provider, snapshot\)/);
  assert.match(providersView, /remote\.registry_open_path\.as_str\(\)/);
  assert.match(providers, /Configured remote:/);
  assert.match(providers, /Registry:/);
  assert.match(providersTooltips, /Registry path is unavailable/);
  assert.match(providersTooltips, /Open a workspace with remotes\.json/);
  assert.match(providersTooltips, /Path unavailable/);
  assert.match(providersTooltips, /No path found/);
  assert.match(
    providersTooltips,
    /if target_path\.is_some\(\) \{[\s\S]*Registry path is unavailable[\s\S]*\} else \{[\s\S]*Open a workspace with remotes\.json/,
  );
  assert.match(
    providersTooltips,
    /if target_path\.is_some\(\) \{[\s\S]*Path unavailable[\s\S]*\} else \{[\s\S]*No path found/,
  );
  assert.match(remoteRegistrySources, /catalog_provider_info/);
  assert.match(remoteRegistrySources, /remote kind\(s\) not in provider icon catalog/);
  for (const unsupportedButton of ["forge", "r2", "mega", "pinterest", "sketchfab"]) {
    assert.doesNotMatch(
      remoteRegistryProviders,
      new RegExp(`Some\\(\\("${unsupportedButton}",`),
    );
  }
  assert.doesNotMatch(
    remoteRegistrySources,
    /std::process|Command::new|powershell|cmd\.exe|\bshell\b|spawn|reqwest|ureq|hyper|TcpStream/i,
  );
  assert.doesNotMatch(
    `${remoteRegistrySources}\n${providers}`,
    /connected remote|synced live|remote verified|provider verified|live health|network checked/i,
  );
});

test("Forge panel renders real receipt, restore, and media states", () => {
  assert.match(panelView, /fn receipt_section/);
  assert.match(panelView, /fn restore_section/);
  assert.match(panelView, /fn media_section/);
  assert.match(panelView, /WithScrollbar/);
  assert.match(panelView, /vertical_scrollbar_for\(scroll_handle, window, cx\)/);
  assert.match(panelView, /No receipts found/);
  assert.match(
    panelView,
    /Receipt summaries unavailable/,
  );
  assert.match(panelView, /No restore previews found/);
  assert.match(panelView, /No media outputs found/);
  assert.match(rows, /side_panel_header_controls/);
  assert.match(rows, /DxForgePanelState::NoWorkspace/);
  assert.match(rows, /\.min_w_0\(\)/);
  assert.match(workflowRows, /Tooltip::with_meta/);
  assert.match(workflowRows, /fn receipt_tooltip/);
  assert.match(workflowRows, /fn source_tooltip/);
});

test("Forge panel uses Git-style controls instead of metric cards", () => {
  const selectableRowBody = extractRustFunction(workflowRows, "selectable_row");
  const selectableRowActionsBody = extractRustFunction(workflowRows, "selectable_row_actions");
  const statusStripBody = extractRustFunction(rows, "status_strip");
  const sectionHeaderBody = extractRustFunction(rows, "section_header");
  const emptyRowBody = extractRustFunction(rows, "empty_row");
  const evidenceRowBodies = `${statusStripBody}\n${sectionHeaderBody}\n${selectableRowBody}\n${emptyRowBody}`;

  assert.match(moduleRoot, /mod controls;/);
  assert.match(panelView, /panel_header\(workspace, panel_id, cx\)/);
  assert.match(panelView, /status_strip\(/);
  assert.match(panelView, /section_header\(/);
  assert.match(rows, /pub\(super\) fn section_header/);
  assert.match(statusStripBody, /ListItem::new\("dx-forge-status"\)/);
  assert.match(statusStripBody, /\.inset\(true\)/);
  assert.match(statusStripBody, /\.selectable\(false\)/);
  assert.match(statusStripBody, /\.height\(rems\(1\.75\)\)/);
  assert.match(statusStripBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(statusStripBody, /\.start_slot\(/);
  assert.match(statusStripBody, /\.end_slot\(/);
  assert.match(statusStripBody, /Tooltip::with_meta/);
  assert.match(rows, /ListHeader/);
  assert.match(sectionHeaderBody, /ListHeader::new\(title\)/);
  assert.match(sectionHeaderBody, /div\(\)[\s\S]*\.id\(id\)/);
  assert.match(sectionHeaderBody, /\.inset\(true\)/);
  assert.match(sectionHeaderBody, /\.start_slot\(/);
  assert.match(sectionHeaderBody, /Tooltip::text\(count_tooltip\)/);
  assert.match(sectionHeaderBody, /\.end_slot\([\s\S]*Label::new\(count\.to_string\(\)\)/);
  assert.match(sectionHeaderBody, /\.color\(Color::Muted\)/);
  assert.doesNotMatch(
    `${statusStripBody}\n${sectionHeaderBody}`,
    /\.h\(px\((?:28|32)\.0\)\)|\.border_1\(\)|\.border_y_1\(\)|\.border_r_2\(\)|ghost_element_hover/,
  );
  assert.match(workflowRows, /ListItem/);
  assert.match(rows, /ListItemSpacing/);
  assert.match(selectableRowBody, /\)\s*->\s*ListItem\s*\{/);
  assert.match(selectableRowBody, /ListItem::new\(id\)/);
  assert.match(selectableRowBody, /\.inset\(true\)/);
  assert.match(selectableRowBody, /\.height\(rems\(1\.75\)\)/);
  assert.match(selectableRowBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.doesNotMatch(selectableRowBody, /\.height\(px\(52\.0\)\)/);
  assert.doesNotMatch(selectableRowBody, /\.spacing\(ListItemSpacing::Dense\)/);
  assert.doesNotMatch(selectableRowBody, /\bpath: String\b/);
  assert.match(selectableRowBody, /\.start_slot\(Icon::new\(icon\)/);
  assert.match(selectableRowBody, /let hover_checkbox_id = SharedString::from\(format!\("\{id\}-hover"\)\)/);
  assert.match(selectableRowBody, /let checkbox = selection_checkbox\(id\.clone\(\), item_key\.clone\(\), checked, panel\)/);
  assert.match(selectableRowBody, /let hover_checkbox = selection_checkbox\(hover_checkbox_id, item_key, checked, panel\)/);
  assert.match(selectableRowBody, /selectable_row_actions\(open_button, hover_checkbox\)/);
  assert.match(selectableRowBody, /\.end_slot\(checkbox\)/);
  assert.match(selectableRowBody, /\.end_slot_on_hover\(row_actions\)/);
  assert.doesNotMatch(selectableRowBody, /\.start_slot\(selection_checkbox\)/);
  assert.match(workflowRows, /fn selectable_row_actions/);
  assert.match(selectableRowActionsBody, /\.on_mouse_down\(MouseButton::Left/);
  assert.match(selectableRowActionsBody, /\.gap_0p5\(\)/);
  assert.match(selectableRowActionsBody, /cx\.stop_propagation\(\);/);
  assert.match(selectableRowActionsBody, /actions = actions\.child\(open_button\)/);
  assert.match(selectableRowActionsBody, /actions\.child\(selection_checkbox\)\.into_any_element\(\)/);
  assert.match(emptyRowBody, /ListItem::new\(id\)/);
  assert.match(emptyRowBody, /\.inset\(true\)/);
  assert.match(emptyRowBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(emptyRowBody, /\.selectable\(false\)/);
  assert.match(emptyRowBody, /\.tooltip\(Tooltip::text\(label\)\)/);
  assert.doesNotMatch(
    evidenceRowBodies,
    /Stateful<Div>|\bDiv\b|\.border_1\(\)|ghost_element_(?:background|hover|active)/,
  );
  assert.match(controls, /IconButton::new\("dx-forge-open-history", IconName::FolderOpen\)/);
  assert.match(controls, /IconButton::new\("dx-forge-refresh", IconName::RotateCw\)/);
  assert.match(
    controls,
    /IconButton::new\("dx-forge-open-history", IconName::FolderOpen\)[\s\S]*\.tab_index\(0_isize\)/,
  );
  assert.match(
    controls,
    /IconButton::new\("dx-forge-open-history", IconName::FolderOpen\)[\s\S]*move \|_, window, cx\| \{[\s\S]*cx\.stop_propagation\(\);[\s\S]*open_exact_abs_path/,
  );
  assert.match(
    controls,
    /IconButton::new\("dx-forge-refresh", IconName::RotateCw\)[\s\S]*\.tab_index\(0_isize\)/,
  );
  assert.match(
    controls,
    /IconButton::new\("dx-forge-refresh", IconName::RotateCw\)[\s\S]*move \|_, _, cx\| \{[\s\S]*cx\.stop_propagation\(\);[\s\S]*panel\.update/,
  );
  assert.match(controls, /IconButton::new\(id, IconName::ArrowUpRight\)/);
  assert.match(
    controls,
    /move \|_, window, cx\| \{[\s\S]*cx\.stop_propagation\(\);[\s\S]*open_exact_abs_path/,
  );
  assert.match(controls, /open_abs_path\(/);
  assert.match(controls, /OpenOptions/);
  assert.match(panel, /pub\(super\) fn refresh/);
  assert.match(panel, /invalidate_machine_cache_snapshot_cache\(\)/);
  assert.match(panel, /invalidate_tool_history_snapshot_cache\(\)/);
  assert.match(panel, /invalidate_source_set_snapshot_cache\(\)/);
  assert.match(receiptHistoryRoot, /pub\(crate\) fn invalidate_tool_history_snapshot_cache/);
  assert.match(sourceSetsRoot, /pub\(crate\) use self::cache::invalidate_source_set_snapshot_cache/);
  assert.match(sourceSetCache, /pub\(crate\) fn invalidate_source_set_snapshot_cache/);
  assert.doesNotMatch(rows, /pub\(super\) fn metric_row/);
  assert.doesNotMatch(rows, /pub\(super\) fn section\(/);
  assert.doesNotMatch(panelView, /Forge Proof|Workspace scope|Visible blockers|Visible restore warnings/);
  assert.doesNotMatch(panelView, /\.p_2\(\)[\s\S]*receipt_section/);
});

test("Forge panel uses workflow tabs with Git-style selectable rows", () => {
  assert.ok(
    existsSync(workflowRowsPath),
    "Forge workflow rows should live in a focused module",
  );
  assert.ok(
    existsSync(visibleRowsPath),
    "Forge visible row state should live in a focused module",
  );
  assert.match(moduleRoot, /mod workflow_rows;/);
  assert.match(moduleRoot, /mod visible_rows;/);
  assert.match(panel, /active_tab: DxForgePanelTab::Repository/);
  for (const tab of ["Repository", "Packages", "Media", "Remotes"]) {
    assert.match(panel, new RegExp(`\\b${tab},`));
    assert.match(tabs, new RegExp(`DxForgePanelTab::${tab}`));
    assert.match(tabs, new RegExp(`"${tab}"`));
  }
  assert.match(tabs, /TabBar/);
  assert.match(tabs, /TabPosition/);
  assert.match(tabs, /TabBar::new\(\("dx-forge-tab-bar", panel_id\)\)/);
  const forgeTabBody = extractRustFunction(tabs, "forge_tab");
  assert.match(forgeTabBody, /Tab::new\(id\)/);
  assert.match(forgeTabBody, /\.fill_available_width\(\)/);
  assert.match(forgeTabBody, /\.position\(tab_position\(tab, active_tab\)\)/);
  assert.match(forgeTabBody, /\.toggle_state\(selected\)/);
  assert.match(forgeTabBody, /\.selected_bottom_border\(true\)/);
  assert.match(forgeTabBody, /\.start_slot\(\s*Icon::new\(tab_icon\(tab\)\)/);
  assert.match(forgeTabBody, /let row_noun = if count == 1 \{ "row" \} else \{ "rows" \};/);
  assert.match(forgeTabBody, /let title = format!\("\{label\}: \{count\} \{row_noun\}"\)/);
  assert.doesNotMatch(forgeTabBody, /\.end_slot\(count_/);
  assert.doesNotMatch(forgeTabBody, /focus_panel\(window, cx\)/);
  assert.doesNotMatch(forgeTabBody, /\.on_mouse_down\(MouseButton::Left/);
  assert.doesNotMatch(forgeTabBody, /\.on_mouse_up\(MouseButton::Left/);
  assert.match(forgeTabBody, /cx\.stop_propagation\(\);/);
  assert.match(forgeTabBody, /panel\.set_active_tab\(tab, cx\)/);
  const tabPositionBody = extractRustFunction(tabs, "tab_position");
  assert.match(tabPositionBody, /TabPosition::First/);
  assert.match(tabPositionBody, /TabPosition::Last/);
  assert.match(tabPositionBody, /TabPosition::Middle/);
  assert.match(tabs, /fn tab_icon/);
  assert.match(panelView, /\.track_scroll\(scroll_handle\)/);
  assert.match(panelView, /\.overflow_y_scroll\(\)/);
  assert.match(panelView, /\.vertical_scrollbar_for\(scroll_handle, window, cx\)/);
  assert.doesNotMatch(
    forgeTabBody,
    /h_flex\(\)|\.border_b_1\(\)|ghost_element|editor_background/,
  );
  for (const tab of ["Repository", "Packages", "Media", "Remotes"]) {
    assert.match(
      tabs,
      new RegExp(
        `visible_row_count_for_tab\\(snapshot, DxForgePanelTab::${tab}\\)`,
      ),
    );
  }
  assert.doesNotMatch(
    tabs,
    /snapshot\.(latest_receipts|package_statuses|machine_caches|media_outputs|restore_previews|remote_registries|remote_providers)\.len\(\)/,
  );
  for (const oldTab of ["Targets", "Sources", "Receipts"]) {
    assert.doesNotMatch(panel, new RegExp(`DxForgePanelTab::${oldTab}`));
    assert.doesNotMatch(tabs, new RegExp(`"${oldTab}"`));
  }

  assert.match(panel, /active_item: Option<DxForgeRowKey>/);
  assert.match(panel, /checked_items: HashSet<DxForgeRowKey>/);
  assert.match(panel, /visible_rows: Vec<DxForgeVisibleRow>/);
  assert.match(panel, /row_scroll_anchors: HashMap<DxForgeRowKey, ScrollAnchor>/);
  const clearActiveItemBody = extractRustMethod(panel, "clear_active_item");
  const syncVisibleRowsBody = extractRustMethod(panel, "sync_visible_rows");
  const refreshBody = extractRustMethod(panel, "refresh");
  const setActiveTabBody = extractRustMethod(panel, "set_active_tab");
  const checkedItemReset =
    /(?:self\.checked_items\s*(?:\.clear\(\)|\.drain\(\)|=\s*(?:HashSet::default\(\)|HashSet::new\(\)|Default::default\(\)))|std::mem::take\(&mut self\.checked_items\))/;
  assert.match(clearActiveItemBody, /self\.active_item\s*=\s*None;/);
  assert.doesNotMatch(clearActiveItemBody, /checked_items/);
  assert.match(syncVisibleRowsBody, /let visible_keys = visible_rows[\s\S]*\.collect::<HashSet<_>>\(\);/);
  assert.match(syncVisibleRowsBody, /visible_keys\.contains\(item_key\)/);
  assert.match(syncVisibleRowsBody, /self\.row_scroll_anchors\s*\.retain\(\|item_key, _\| visible_keys\.contains\(item_key\)\)/);
  assert.match(syncVisibleRowsBody, /ScrollAnchor::for_handle\(self\.scroll_handle\.clone\(\)\)/);
  assert.match(syncVisibleRowsBody, /self\.visible_rows\s*=\s*visible_rows;/);
  assert.doesNotMatch(syncVisibleRowsBody, checkedItemReset);
  assert.match(refreshBody, /self\.clear_active_item\(\);[\s\S]*cx\.notify\(\);/);
  assert.doesNotMatch(refreshBody, checkedItemReset);
  assert.match(
    setActiveTabBody,
    /if\s+self\.active_tab\s*!=\s*tab\s*\{[\s\S]*self\.active_tab\s*=\s*tab;[\s\S]*self\.clear_active_item\(\);[\s\S]*cx\.notify\(\);/,
  );
  assert.match(setActiveTabBody, /self\.scroll_handle\.set_offset\(point\(px\(0\.\), px\(0\.\)\)\);/);
  assert.doesNotMatch(setActiveTabBody, checkedItemReset);
  assert.match(panel, /activate_item/);
  assert.match(panel, /row_scroll_anchor/);
  assert.match(panel, /scroll_item_into_view/);
  assert.match(panel, /item_active/);
  assert.match(panel, /visible_rows_for_tab\(&snapshot, self\.active_tab\)/);
  assert.match(panel, /self\.sync_visible_rows\(visible_rows\)/);
  assert.match(panel, /toggle_item_checked/);
  assert.match(panel, /item_checked/);
  assert.doesNotMatch(
    panel,
    /active_item: Option<String>|checked_items: HashSet<String>|item_key: String|item_key: &str/,
  );
  assert.doesNotMatch(panel, /\bselected_items\b|toggle_item_selection|item_selected/);
  assert.match(visibleRows, /pub\(super\) struct DxForgeVisibleRow/);
  assert.match(visibleRows, /pub\(super\) enum DxForgeVisibleRowKind/);
  assert.match(visibleRows, /pub\(super\) enum DxForgeRowKey/);
  assert.match(visibleRows, /#\[derive\(Clone, Debug, PartialEq, Eq, Hash\)\]/);
  assert.match(visibleRows, /Receipt\(String\)/);
  assert.match(visibleRows, /Source\(String\)/);
  assert.match(visibleRows, /RemoteGroup\(String\)/);
  assert.match(visibleRows, /pub\(super\) fn receipt_item_key\(receipt: &DxForgeReceiptRow\) -> DxForgeRowKey/);
  assert.match(visibleRows, /pub\(super\) fn source_item_key\(source: &DxForgeSourceRow\) -> DxForgeRowKey/);
  assert.match(visibleRows, /pub\(super\) fn remote_target_item_key\(group_key: &str\) -> DxForgeRowKey/);
  assert.match(visibleRows, /pub\(super\) fn visible_row_count_for_tab/);
  assert.match(visibleRows, /pub\(super\) fn visible_rows_for_tab/);
  assert.match(visibleRows, /DxForgePanelTab::Repository[\s\S]*snapshot\.latest_receipts/);
  assert.match(visibleRows, /DxForgePanelTab::Packages[\s\S]*snapshot\.package_statuses[\s\S]*snapshot\.machine_caches/);
  assert.match(visibleRows, /DxForgePanelTab::Media[\s\S]*snapshot\.media_outputs[\s\S]*snapshot\.restore_previews/);
  assert.match(visibleRows, /DxForgePanelTab::Remotes[\s\S]*REMOTE_TARGET_GROUP_KEYS[\s\S]*snapshot\.remote_registries/);
  assert.doesNotMatch(
    visibleRows,
    /gpui::|ui::|ListItem|Label::|Checkbox|IconButton|h_flex|v_flex|Tooltip/,
  );
  assert.match(workflowRows, /pub\(super\) fn selectable_source_row/);
  assert.match(workflowRows, /pub\(super\) fn selectable_receipt_row/);
  assert.match(workflowRows, /receipt_item_key\(receipt\)/);
  assert.match(workflowRows, /source_item_key\(source\)/);
  assert.match(providersView, /remote_target_item_key\(group\.key\(\)\)/);
  assert.doesNotMatch(
    `${workflowRows}\n${providersView}`,
    /format!\("(?:source|receipt|remote):/,
  );
  assert.match(workflowRows, /Checkbox::new/);
  assert.match(workflowRows, /ToggleState::Selected/);
  assert.match(workflowRows, /ToggleState::Unselected/);
  assert.match(workflowRows, /ElevationIndex::Surface/);
  assert.match(workflowRows, /ListItem::new\(id/);
  assert.match(workflowRows, /\.height\(rems\(1\.75\)\)/);
  assert.match(workflowRows, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(workflowRows, /\.start_slot\(Icon::new\(icon\)/);
  assert.match(workflowRows, /fn selectable_row_actions/);
  assert.match(workflowRows, /\.child\(selection_checkbox\)/);
  assert.match(workflowRows, /let checked = item_checked\(panel, &item_key, cx\)/);
  assert.match(workflowRows, /let active = item_active\(panel, &item_key, cx\)/);
  assert.match(workflowRows, /selection_checkbox\(id\.clone\(\), item_key\.clone\(\), checked, panel\)/);
  assert.match(workflowRows, /selection_checkbox\(hover_checkbox_id, item_key, checked, panel\)/);
  assert.match(workflowRows, /row_scroll_anchor\(panel, &row_key, cx\)/);
  assert.match(workflowRows, /\.anchor_scroll\(scroll_anchor\)/);
  assert.match(workflowRows, /\.toggle_state\(active\)/);
  const selectionCheckboxBody = workflowRows.match(
    /pub\(super\) fn selection_checkbox\([\s\S]*?\r?\n}\r?\n\r?\nfn selectable_row/,
  )?.[0] ?? "";
  assert.match(selectionCheckboxBody, /let checkbox_key = item_key\.clone\(\)/);
  assert.match(selectionCheckboxBody, /panel\.focus_panel\(window, cx\)/);
  assert.match(selectionCheckboxBody, /panel\.activate_item\(checkbox_key\.clone\(\), cx\)/);
  assert.match(selectionCheckboxBody, /panel\.toggle_item_checked\(checkbox_key\.clone\(\), cx\)/);
  assert.match(
    workflowRows,
    /panel\s*\.update\(cx, \|panel, cx\|[\s\S]*panel\.focus_panel\(window, cx\)[\s\S]*panel\.activate_item\(row_key\.clone\(\), cx\)/,
  );
  assert.doesNotMatch(
    workflowRows.match(/fn selectable_row\([\s\S]*?\r?\n}\r?\n\r?\nfn selectable_row_actions/)?.[0] ?? "",
    /panel\.toggle_item_(?:selection|checked)\(row_key\.clone\(\), cx\)/,
  );
  assert.match(selectionCheckboxBody, /\.on_click\(move \|_, window, cx\| \{[\s\S]*cx\.stop_propagation\(\);[\s\S]*\}\)/);
  assert.match(workflowRows, /cx\.stop_propagation\(\)/);
  assert.match(workflowRows, /fn item_active/);
  assert.match(panelView, /DxForgePanelTab::Repository/);
  assert.match(panelView, /DxForgePanelTab::Packages/);
  assert.match(panelView, /DxForgePanelTab::Media/);
  assert.match(panelView, /DxForgePanelTab::Remotes/);
  assert.match(panelView, /repository_section\(snapshot, workspace, panel, cx\)/);
  assert.match(panelView, /package_status_section\(snapshot, workspace, panel, cx\)/);
  assert.match(panelView, /media_section\(snapshot, workspace, panel, cx\)/);
  assert.match(panelView, /remote_target_strip\(snapshot, workspace, panel, cx\)/);
  assert.doesNotMatch(
    workflowRows,
    /\b(?:Badge|Chip|Pill|Tag|StatusBadge|BadgeCluster)\b/i,
  );
});

test("Forge panel wires visible-row navigation to menu actions", () => {
  assert.ok(
    existsSync(rowSelectionPath),
    "Forge visible-row selection should live in a focused module",
  );
  assert.match(moduleRoot, /mod row_selection;/);
  assert.match(rowSelection, /use menu::\{Confirm, SelectFirst, SelectLast, SelectNext, SelectPrevious\};/);
  assert.match(rowSelection, /KeyContext/);

  const dispatchContextBody = extractRustMethod(rowSelection, "dispatch_context");
  assert.match(dispatchContextBody, /KeyContext::new_with_defaults\(\)/);
  assert.match(dispatchContextBody, /dispatch_context\.add\("DxForgePanel"\)/);
  assert.match(dispatchContextBody, /dispatch_context\.add\("menu"\)/);

  const renderBody = extractRustMethod(panel, "render");
  const actionRootChain =
    renderBody.match(/v_flex\(\)[\s\S]*?\.child\(panel_view::render_panel/)?.[0] ?? "";
  assert.ok(
    actionRootChain,
    "Forge panel should bind focus and menu ownership on the root element",
  );
  assert.match(panel, /MouseButton/);
  assert.match(panel, /MouseDownEvent/);
  assert.match(actionRootChain, /\.id\("dx-forge-panel-action-root"\)/);
  assert.match(actionRootChain, /\.key_context\(self\.dispatch_context\(\)\)/);
  assert.match(actionRootChain, /\.track_focus\(&self\.focus_handle\)/);
  assert.match(
    actionRootChain,
    /\.on_mouse_down\(\s*MouseButton::Left,\s*cx\.listener\(Self::focus_panel_on_mouse_down\),?\s*\)/,
  );
  for (const method of [
    "select_next",
    "select_previous",
    "select_first",
    "select_last",
    "toggle_active_item_checked",
  ]) {
    assert.match(
      actionRootChain,
      new RegExp(`\\.on_action\\(cx\\.listener\\(Self::${method}\\)\\)`),
    );
  }

  const focusPanelBody = extractRustMethod(panel, "focus_panel");
  assert.match(focusPanelBody, /window: &mut Window/);
  assert.match(focusPanelBody, /self\.focus_handle\.focus\(window, cx\)/);
  assert.doesNotMatch(
    focusPanelBody,
    /stop_propagation|toggle_panel_focus|ensure_panel|set_active_tab|activate_item|toggle_item_checked|refresh|sync_visible_rows|cx\.notify\(\)/,
  );
  const focusPanelOnMouseDownBody = extractRustMethod(panel, "focus_panel_on_mouse_down");
  assert.match(focusPanelOnMouseDownBody, /_: &MouseDownEvent/);
  assert.match(focusPanelOnMouseDownBody, /self\.focus_panel\(window, cx\)/);
  assert.doesNotMatch(
    focusPanelOnMouseDownBody,
    /stop_propagation|toggle_panel_focus|ensure_panel|set_active_tab|activate_item|toggle_item_checked|refresh|sync_visible_rows|cx\.notify\(\)/,
  );

  const activeVisibleRowIndexBody = extractRustMethod(rowSelection, "active_visible_row_index");
  assert.match(activeVisibleRowIndexBody, /self\.active_item\.as_ref\(\)\?/);
  assert.match(activeVisibleRowIndexBody, /self\.visible_rows\s*\.iter\(\)\s*\.position\(\|row\| row\.item_key\(\) == active_item\)/);

  const activeVisibleRowKeyBody = extractRustMethod(rowSelection, "active_visible_row_key");
  assert.match(activeVisibleRowKeyBody, /self\.active_visible_row_index\(\)\?/);
  assert.match(activeVisibleRowKeyBody, /self\.visible_rows\s*\.get\(index\)/);
  assert.match(activeVisibleRowKeyBody, /row\.item_key\(\)\.clone\(\)/);

  assert.match(panel, /row_scroll_anchors:\s*HashMap<DxForgeRowKey, ScrollAnchor>/);
  const syncVisibleRowsBody = extractRustMethod(panel, "sync_visible_rows");
  assert.match(syncVisibleRowsBody, /visible_keys[\s\S]*collect::<HashSet<_>>\(\)/);
  assert.match(syncVisibleRowsBody, /row_scroll_anchors[\s\S]*retain\(\|item_key, _\| visible_keys\.contains\(item_key\)\)/);
  assert.match(syncVisibleRowsBody, /ScrollAnchor::for_handle\(self\.scroll_handle\.clone\(\)\)/);
  const rowScrollAnchorBody = extractRustMethod(panel, "row_scroll_anchor");
  assert.match(rowScrollAnchorBody, /self\.row_scroll_anchors\.get\(item_key\)\.cloned\(\)/);
  const scrollItemIntoViewBody = extractRustMethod(panel, "scroll_item_into_view");
  assert.match(scrollItemIntoViewBody, /self\.row_scroll_anchor\(item_key\)/);
  assert.match(scrollItemIntoViewBody, /scroll_anchor\.scroll_to\(window, cx\)/);
  assert.match(workflowRows, /let scroll_anchor = row_scroll_anchor\(panel, &row_key, cx\);/);
  assert.match(workflowRows, /\.anchor_scroll\(scroll_anchor\)/);
  assert.match(providersView, /let scroll_anchor = row_scroll_anchor\(panel, &item_key, cx\);/);
  assert.match(providersView, /\.anchor_scroll\(scroll_anchor\)/);

  const setActiveVisibleRowBody = extractRustMethod(rowSelection, "set_active_visible_row");
  assert.match(setActiveVisibleRowBody, /self\.visible_rows\.get\(index\)/);
  assert.match(setActiveVisibleRowBody, /row\.item_key\(\)\.clone\(\)/);
  assert.match(setActiveVisibleRowBody, /self\.scroll_item_into_view\(&item_key, window, cx\)/);
  assert.match(setActiveVisibleRowBody, /cx\.notify\(\)/);

  const selectFirstVisibleRowBody = extractRustMethod(rowSelection, "select_first_visible_row");
  assert.match(selectFirstVisibleRowBody, /if !self\.visible_rows\.is_empty\(\)/);
  assert.match(selectFirstVisibleRowBody, /self\.set_active_visible_row\(0, window, cx\)/);

  const selectLastVisibleRowBody = extractRustMethod(rowSelection, "select_last_visible_row");
  assert.match(selectLastVisibleRowBody, /self\.visible_rows\.len\(\)\.checked_sub\(1\)/);
  assert.match(selectLastVisibleRowBody, /self\.set_active_visible_row\(last_index, window, cx\)/);

  const selectNextBody = extractRustMethod(rowSelection, "select_next");
  assert.match(selectNextBody, /_: &SelectNext/);
  assert.match(selectNextBody, /self\.focus_panel\(window, cx\)/);
  assert.match(selectNextBody, /self\.active_visible_row_index\(\)/);
  assert.match(selectNextBody, /self\.select_first_visible_row\(window, cx\)/);
  assert.match(selectNextBody, /checked_add\(1\)/);
  assert.match(selectNextBody, /next_index < self\.visible_rows\.len\(\)/);
  assert.match(selectNextBody, /self\.set_active_visible_row\(next_index, window, cx\)/);
  assert.doesNotMatch(selectNextBody, /self\.active_item\s*=/);
  assert.doesNotMatch(selectNextBody, /% self\.visible_rows\.len\(\)/);

  const selectPreviousBody = extractRustMethod(rowSelection, "select_previous");
  assert.match(selectPreviousBody, /_: &SelectPrevious/);
  assert.match(selectPreviousBody, /window: &mut Window/);
  assert.match(selectPreviousBody, /self\.focus_panel\(window, cx\)/);
  assert.match(selectPreviousBody, /self\.active_visible_row_index\(\)/);
  assert.match(selectPreviousBody, /self\.select_last_visible_row\(window, cx\)/);
  assert.match(selectPreviousBody, /checked_sub\(1\)/);
  assert.match(selectPreviousBody, /self\.set_active_visible_row\(previous_index, window, cx\)/);
  assert.doesNotMatch(selectPreviousBody, /self\.active_item\s*=/);
  assert.doesNotMatch(selectPreviousBody, /% self\.visible_rows\.len\(\)/);

  const toggleActiveBody = extractRustMethod(rowSelection, "toggle_active_item_checked");
  assert.match(toggleActiveBody, /_: &Confirm/);
  assert.match(toggleActiveBody, /window: &mut Window/);
  assert.match(toggleActiveBody, /self\.focus_panel\(window, cx\)/);
  assert.match(toggleActiveBody, /let Some\(active_item\) = self\.active_visible_row_key\(\) else/);
  assert.match(toggleActiveBody, /self\.toggle_item_checked\(active_item, cx\)/);
  assert.doesNotMatch(toggleActiveBody, /self\.active_item\.clone\(\)/);
  assert.doesNotMatch(toggleActiveBody, /checked_items\.(insert|remove|clear)/);
  assert.doesNotMatch(toggleActiveBody, /self\.visible_rows\.first\(\)/);

  const rowSelectionBodies = [
    selectNextBody,
    selectPreviousBody,
    toggleActiveBody,
  ].join("\n");
  assert.doesNotMatch(
    rowSelectionBodies,
    /snapshot\.|latest_receipts|package_statuses|machine_caches|media_outputs|restore_previews|remote_registries/,
    "row selection should only use the rendered visible_rows model",
  );
});

test("Forge panel renders DX icon provider targets with snapshot-driven readiness", () => {
  assert.match(moduleRoot, /mod providers;/);
  assert.match(panelView, /remote_target_strip\(snapshot, workspace, panel, cx\)/);
  assert.ok(existsSync(providersRootPath), "Forge provider module boundary must live in providers/mod.rs");
  assert.ok(existsSync(providersCatalogPath), "Forge provider metadata must live in catalog.rs");
  assert.ok(existsSync(providersStatePath), "Forge provider state must live in state.rs");
  assert.ok(existsSync(providersViewPath), "Forge provider GPUI rendering must live in view.rs");
  assert.ok(existsSync(providersTooltipsPath), "Forge provider tooltip copy must live in tooltips.rs");
  assert.match(providersRoot, /mod view;/);
  assert.match(providersRoot, /mod tooltips;/);
  assert.match(providersRoot, /pub\(super\) use self::view::remote_target_strip;/);
  const providerGroupControlsBody =
    providersView.match(
      /fn provider_group_controls\([\s\S]*?\r?\n}\r?\n\r?\nfn provider_buttons_for_group/,
    )?.[0] ?? "";
  assert.match(providerGroupControlsBody, /row_scroll_anchor\(panel, &item_key, cx\)/);
  assert.match(providerGroupControlsBody, /\.anchor_scroll\(scroll_anchor\)/);
  assert.match(
    providerGroupControlsBody,
    /\.on_click\(move \|_, window, cx\| \{[\s\S]*panel\.focus_panel\(window, cx\)[\s\S]*panel\.activate_item\(row_key\.clone\(\), cx\)/,
  );

  const providerTargets = [
    ["GitHub", "DxForgeProviderGithub", "dx_forge_provider_github", "ProviderGroup::Code", "svgl", "github_dark"],
    ["GitLab", "DxForgeProviderGitlab", "dx_forge_provider_gitlab", "ProviderGroup::Code", "svgl", "gitlab"],
    [
      "Bitbucket",
      "DxForgeProviderBitbucket",
      "dx_forge_provider_bitbucket",
      "ProviderGroup::Code",
      "simple-icons",
      "bitbucket",
    ],
    [
      "Google Drive",
      "DxForgeProviderDrive",
      "dx_forge_provider_drive",
      "ProviderGroup::Storage",
      "svgl",
      "drive",
    ],
    [
      "Dropbox",
      "DxForgeProviderDropbox",
      "dx_forge_provider_dropbox",
      "ProviderGroup::Storage",
      "svgl",
      "dropbox",
    ],
    [
      "YouTube",
      "DxForgeProviderYoutube",
      "dx_forge_provider_youtube",
      "ProviderGroup::Media",
      "svgl",
      "youtube",
    ],
    [
      "SoundBox",
      "DxForgeProviderSoundbox",
      "dx_forge_provider_soundbox",
      "ProviderGroup::Media",
      "svgl",
      "soundcloud-logo",
    ],
    [
      "SoundCloud",
      "DxForgeProviderSoundcloud",
      "dx_forge_provider_soundcloud",
      "ProviderGroup::Media",
      "svgl",
      "soundcloud-logo",
    ],
  ];

  const providerBlock = (label: string) => {
    const start = providers.indexOf(`label: "${label}"`);
    assert.ok(start >= 0, `missing provider ${label}`);
    const end = providers.indexOf("ForgeProvider {", start + 1);
    return providers.slice(start, end >= 0 ? end : undefined);
  };

  for (const [label, iconName, fileName, group, sourcePack, sourceSlug] of providerTargets) {
    const block = providerBlock(label);
    assert.match(icons, new RegExp(`\\b${iconName}\\b`));
    assert.match(block, new RegExp(`icon:\\s*IconName::${iconName}\\b`));
    assert.match(block, new RegExp(`group:\\s*${group}\\b`));
    assert.match(block, new RegExp(`source_pack:\\s*"${sourcePack}"`));
    assert.match(block, new RegExp(`source_slug:\\s*"${sourceSlug}"`));

    const iconPath = `assets/icons/${fileName}.svg`;
    assert.ok(
      existsSync(iconPath),
      `${fileName}.svg must be a tracked DX provider icon asset`,
    );
    const svg = readFileSync(iconPath, "utf8");
    assert.match(svg, /^<svg\b[^>]*viewBox=/);
    assert.match(svg, /<(path|circle|rect|polygon|g)\b/);
    assert.match(svg, /currentColor/);
    assert.doesNotMatch(svg, /<script|<foreignObject|on[a-z]+\s*=|javascript:|data:|xlink:href|href=/i);
    assert.doesNotMatch(svg, /<defs|linearGradient|url\(|stop-color|#[0-9a-f]{3,8}|fill=["']red["']/i);
    assert.doesNotMatch(svg, /placeholder|sample|todo/i);
  }

  assert.equal((providersCatalog.match(/^\s*ForgeProvider\s*\{/gm) ?? []).length, providerTargets.length);
  assert.match(providers, /ProviderGroup::Code/);
  assert.match(providers, /ProviderGroup::Storage/);
  assert.match(providers, /ProviderGroup::Media/);
  assert.match(providersView, /providers_for\(group\)/);
  assert.match(providersView, /fn provider_target_button/);
  assert.match(providersView, /fn provider_group_controls/);
  assert.match(providersView, /fn provider_buttons_for_group/);
  assert.match(providersView, /fn provider_group_actions/);
  assert.match(providersView, /fn target_path_for_group/);
  assert.match(providers, /fn provider_tooltip_meta/);
  assert.match(providersView, /IconButton::new\(format!\("dx-forge-provider-\{\}", provider\.id\), provider\.icon\)/);
  assert.match(providersView, /\.indicator\(provider_target_indicator\(&state\)\)/);
  assert.match(providersView, /fn provider_target_indicator/);
  assert.match(providersView, /Indicator::dot\(\)\.color\(state\.color\)/);
  assert.doesNotMatch(providersView, /fn provider_button_style/);
  assert.doesNotMatch(providersView, /fn provider_icon_color/);
  assert.match(providersView, /provider_buttons_for_group\(group, snapshot, workspace, cx\)/);
  const remoteTargetStripBody =
    providersView.match(
      /pub\(in crate::dx_forge_panel\) fn remote_target_strip\([\s\S]*?\r?\n}\r?\n\r?\nfn provider_target_button/,
    )?.[0] ?? "";
  const providerButtonsBody =
    providersView.match(
      /fn provider_buttons_for_group\([\s\S]*?\r?\n}\r?\n\r?\nfn provider_group_actions/,
    )?.[0] ?? "";
  const providerTargetButtonBody =
    providersView.match(
      /fn provider_target_button\([\s\S]*?\r?\n}\r?\n\r?\nfn provider_group_controls/,
    )?.[0] ?? "";
  const providerGroupActionsBody =
    providersView.match(
      /fn provider_group_actions\([\s\S]*?\r?\n}\r?\n\r?\nfn target_path_for_group/,
    )?.[0] ?? "";
  assert.ok(remoteTargetStripBody, "remote_target_strip body should remain source-guarded");
  assert.ok(providerGroupControlsBody, "provider_group_controls body should remain source-guarded");
  assert.ok(providerTargetButtonBody, "provider_target_button body should remain source-guarded");
  assert.ok(providerButtonsBody, "provider_buttons_for_group body should remain source-guarded");
  assert.match(remoteTargetStripBody, /v_flex\(\)/);
  assert.doesNotMatch(remoteTargetStripBody, /\bRemote targets\b|\blanes\b|ProviderGroup::ALL\.len\(\)/i);
  assert.match(providerGroupControlsBody, /ListItem::new/);
  assert.match(providerGroupControlsBody, /\.inset\(true\)/);
  assert.match(providerGroupControlsBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(providerGroupControlsBody, /\.start_slot\(/);
  assert.match(providerGroupControlsBody, /\.end_slot\(/);
  assert.match(providerGroupControlsBody, /\.child\(checkbox\)/);
  assert.match(
    providerGroupControlsBody,
    /\.end_slot_on_hover\(provider_group_actions\([\s\S]*open_button\.into_any_element\(\),[\s\S]*hover_checkbox,[\s\S]*state\.color,[\s\S]*\)\)/,
  );
  assert.match(providerGroupControlsBody, /let checked = panel[\s\S]*item_checked\(&item_key\)/);
  assert.match(providerGroupControlsBody, /let active = panel[\s\S]*item_active\(&item_key\)/);
  assert.match(providerGroupControlsBody, /selection_checkbox\([\s\S]*checked,[\s\S]*panel,/);
  assert.match(providerGroupControlsBody, /\.toggle_state\(active\)/);
  assert.match(providerGroupControlsBody, /let row_key = item_key\.clone\(\)/);
  assert.match(providerGroupControlsBody, /\.on_click\(move \|_, window, cx\|/);
  assert.match(
    providerGroupControlsBody,
    /panel_for_row\s*\.update\(cx, \|panel, cx\|[\s\S]*panel\.focus_panel\(window, cx\)[\s\S]*panel\.activate_item\(row_key\.clone\(\), cx\)/,
  );
  assert.doesNotMatch(
    providerGroupControlsBody,
    /panel\.toggle_item_(?:selection|checked)\(row_key\.clone\(\), cx\)/,
  );
  assert.match(providerGroupControlsBody, /cx\.stop_propagation\(\);[\s\S]*open_exact_abs_path/);
  assert.doesNotMatch(
    providerGroupControlsBody,
    /\.border_1\(\)|\.border_r_\d+\(\)|ghost_element_(?:background|hover|active)/,
  );
  assert.match(providerGroupControlsBody, /provider_buttons_for_group\(group, snapshot, workspace, cx\)/);
  assert.match(providerGroupControlsBody, /Label::new\(group\.title\(\)\)/);
  assert.doesNotMatch(providerGroupControlsBody, /Label::new\(state\.detail\.clone\(\)\)/);
  assert.match(providerGroupControlsBody, /remote_target_tooltip\(group, &state, target_path\.as_deref\(\), enabled\)/);
  assert.match(providerGroupControlsBody, /Indicator::dot\(\)\.color\(state\.color\)/);
  assert.equal(
    (providerGroupControlsBody.match(/Indicator::dot\(\)\.color\(state\.color\)/g) ?? []).length,
    1,
  );
  assert.match(providerGroupControlsBody, /format!\("Open \{\}", group\.title\(\)\)/);
  assert.match(providerGroupControlsBody, /IconButton::new\([\s\S]*"dx-forge-open-provider-group-\{\}"[\s\S]*\.style\(ButtonStyle::Subtle\)[\s\S]*\.tab_index\(0_isize\)[\s\S]*\.disabled\(!enabled\)/);
  assert.match(providerTargetButtonBody, /\.style\(ButtonStyle::Transparent\)[\s\S]*\.tab_index\(0_isize\)[\s\S]*\.disabled\(!enabled\)/);
  assert.match(providerGroupActionsBody, /status_color: Color/);
  assert.match(providerGroupActionsBody, /\.gap_0p5\(\)/);
  assert.match(
    providerButtonsBody,
    /providers_for\(group\)[\s\S]*provider_target_button\(provider, snapshot, workspace, cx\)/,
  );
  assert.match(providerButtonsBody, /\.on_mouse_down\(MouseButton::Left/);
  assert.match(providerButtonsBody, /\.on_mouse_up\(MouseButton::Left/);
  assert.match(providerButtonsBody, /cx\.stop_propagation\(\);/);
  assert.match(providerGroupActionsBody, /\.on_mouse_down\(MouseButton::Left/);
  assert.match(providerGroupActionsBody, /\.on_mouse_up\(MouseButton::Left/);
  assert.match(providerGroupActionsBody, /Indicator::dot\(\)\.color\(status_color\)/);
  assert.equal(
    (providerGroupActionsBody.match(/Indicator::dot\(\)\.color\(status_color\)/g) ?? []).length,
    1,
  );
  assert.match(providerGroupActionsBody, /\.child\(open_button\)/);
  assert.match(providerGroupActionsBody, /cx\.stop_propagation\(\);/);
  assert.match(providersView, /IconButtonShape::Square/);
  assert.match(providersView, /ButtonStyle::Transparent/);
  assert.match(providersView, /Indicator/);
  assert.doesNotMatch(providersView, /ButtonStyle::Tinted|TintColor/);
  assert.match(providersView, /open_exact_abs_path\(/);
  assert.match(providersState, /fn code_target_state/);
  assert.match(providersState, /fn storage_target_state/);
  assert.match(providersState, /fn media_target_state/);
  assert.match(providers, /source_pack: "svgl"/);
  assert.match(providers, /source_pack: "simple-icons"/);
  assert.match(providers, /source_slug:/);
  assert.match(providers, /ProviderGroup::ALL/);
  assert.match(providers, /for group in ProviderGroup::ALL/);
  assert.match(providers, /fn remote_target_state/);
  assert.match(providers, /history_root_exists/);
  assert.match(providers, /receipt_count/);
  assert.match(providers, /summarized_receipt_count/);
  assert.match(providers, /visible_blocker_count/);
  assert.match(providers, /restore_previews\.len\(\)/);
  assert.match(providers, /visible_restore_warning_count/);
  assert.match(providers, /media_outputs\.len\(\)/);
  assert.match(providers, /Tooltip::with_meta/);
  assert.doesNotMatch(providers, /Icon::new\(state\.icon\)/);
  assert.doesNotMatch(providersView, /Label::new\(state\.label\)/);
  assert.match(providers, /source_slug/);
  assert.doesNotMatch(providers, /fn provider_icon_stack/);
  assert.doesNotMatch(providers, /fn provider_target_deck/);
  assert.doesNotMatch(providers, /Label::new\(group\.providers_label\(\)\)/);
  assert.doesNotMatch(providersView, /Icon source/);
  assert.doesNotMatch(
    providers,
    /ProviderStatus|ready_count|provider_status|provider_tooltip\(provider|Status:/,
  );
});

test("Forge readers keep latest receipt edge cases visible", () => {
  assert.match(receiptFiles, /const RECEIPT_HISTORY_LATEST_ROOT_ENTRY_LIMIT: usize = 64;/);
  assert.match(receiptFiles, /const RECEIPT_HISTORY_LATEST_NESTED_ENTRY_LIMIT: usize = 64;/);
  assert.match(receiptFiles, /entries\s*\.flatten\(\)\s*\.take\(RECEIPT_HISTORY_LATEST_ROOT_ENTRY_LIMIT\)/);
  assert.match(receiptFiles, /children\s*\.flatten\(\)\s*\.take\(RECEIPT_HISTORY_LATEST_NESTED_ENTRY_LIMIT\)/);
  assert.doesNotMatch(receiptFiles, /for entry in entries\.flatten\(\) \{/);
  assert.doesNotMatch(receiptFiles, /for child in children\.flatten\(\) \{/);
  assert.match(sourceSetReceipts, /const LATEST_RECEIPT_ROOT_ENTRY_LIMIT: usize = 128;/);
  assert.match(sourceSetReceipts, /const LATEST_RECEIPT_CANDIDATE_LIMIT: usize = 64;/);
  assert.match(sourceSetReceipts, /entries\.flatten\(\)\.take\(LATEST_RECEIPT_ROOT_ENTRY_LIMIT\)/);
  assert.match(sourceSetReceipts, /push_latest_receipt_candidate/);
  assert.match(receiptFields, /array_len_field\(value, &\["restore_execution", "restore", "blockers"\]\)/);
  assert.match(receiptFields, /array_len_field\(value, &\["runner_gate", "validation", "blockers"\]\)/);
});

test("Forge panel files stay small and professionally named", () => {
  const lineCounts = new Map([
    ["dx_forge_panel.rs", moduleRoot],
    ["controls.rs", controls],
    ["row_selection.rs", rowSelection],
    ["roots.rs", roots],
    ["machine_cache.rs", machineCache],
    ["package_status.rs", packageStatus],
    ["package_status_cache.rs", packageStatusCache],
    ["remote_registry.rs", remoteRegistry],
    ["remote_registry/providers.rs", remoteRegistryProviders],
    ["panel.rs", panel],
    ["providers/mod.rs", providersRoot],
    ["providers/catalog.rs", providersCatalog],
    ["providers/state.rs", providersState],
    ["providers/tooltips.rs", providersTooltips],
    ["providers/view.rs", providersView],
    ["source_section.rs", sourceSection],
    ["visible_rows.rs", visibleRows],
    ["snapshot.rs", snapshot],
    ["snapshot_state.rs", snapshotState],
    ["panel_view.rs", panelView],
    ["rows.rs", rows],
  ]);

  for (const [name, source] of lineCounts) {
    assert.ok(
      source.split("\n").length <= 300,
      `${name} should stay small enough to review quickly`,
    );
  }

  for (const term of [
    "v1",
    "demo",
    "prototype",
    "skeleton",
    "mock",
    "fake",
    "sample",
    "temp",
    "tmp",
    "placeholder",
  ]) {
    assert.doesNotMatch(forgeSources.toLowerCase(), new RegExp(`\\b${term}\\b`));
  }
});

test("Forge panel source surface is closed against UI slop and proof overclaims", () => {
  const forgePanelSourcePaths = [
    "crates/agent_ui/src/dx_forge_panel.rs",
    ...collectRustFiles("crates/agent_ui/src/dx_forge_panel"),
  ];
  const allForgePanelSources = forgePanelSourcePaths
    .map((path) => readFileSync(path, "utf8"))
    .join("\n");

  for (const path of forgePanelSourcePaths) {
    assert.ok(
      readFileSync(path, "utf8").split(/\r?\n/).length <= 300,
      `${path} should stay small enough to review quickly`,
    );
  }

  assert.doesNotMatch(
    allForgePanelSources,
    /\b(?:Badge|Chip|Pill|Tag|StatusBadge|BadgeCluster)\b|fn\s+\w*(?:badge|chip|pill|tag|cluster)\w*\s*\(|\b(?:badge|chip|pill)_cluster\b/i,
  );
  assert.doesNotMatch(
    allForgePanelSources,
    /\b(?:connected remote|synced live|runtime\s+(?:proven|verified|ready|green)|provider\s+(?:proven|verified|ready|green)|browser\s+(?:proven|verified|ready|green)|live\s+(?:remote\s+)?health\s+(?:checked|verified)|source\s+hash\s+matches|hash\s+verified|metadata\s+verified|freshness\s+verified|cache\s+verified|runtime-backed|browser-backed|provider-backed)\b/i,
  );
  assert.match(snapshot, /Evidence,/);
  assert.match(snapshotState, /DxForgePanelState::Evidence/);
  assert.match(rows, /DxForgePanelState::Evidence => \(IconName::FileTextOutlined, Color::Muted, "Evidence"\)/);

  assert.match(allForgePanelSources, /source-only receipt evidence/);
  assert.match(allForgePanelSources, /receipt file only; live checks not executed/);
  assert.match(allForgePanelSources, /health unchecked/);
});

test("Forge panel copy stays concise and honors the DX cog icon contract", () => {
  assert.match(dxIcons, /DxUiIcon::Settings => IconName::DxCog/);
  assert.doesNotMatch(forgeSources, /IconName::Settings/);

  for (const phrase of [
    "Open a workspace to read",
    "no known summaries were readable",
    "no known receipt summaries were readable",
    "Forge receipt root is missing",
    "No Forge remote registry found",
    "No Forge package status found",
    "No Forge machine caches found",
    "No Forge receipts found",
    "live remote health unchecked",
    "Forge is configured, but no receipts were found",
    "Forge root configured; no receipts yet",
    "Missing tools/dx-forge receipt root",
    "is available in the DX icon catalog; no local remote is registered",
  ]) {
    assert.doesNotMatch(forgeSources, escapedTextPattern(phrase));
  }

  for (const phrase of [
    "Open a workspace to inspect Forge history",
    "Receipt summaries unavailable",
    "Receipt history is unavailable",
    "No remote registry found",
    "No package status found",
    "No machine caches found",
    "No receipts found",
    "History configured; no receipts yet",
    "health unchecked",
  ]) {
    assert.match(forgeSources, escapedTextPattern(phrase));
  }
});

test("Forge panel opens exact source-owned paths in multi-root workspaces", () => {
  const openPathButtonBody = extractRustFunction(
    controls,
    "open_exact_abs_path_button",
  );

  assert.match(snapshot, /pub\(super\) open_path: String/);
  assert.match(sourceSets, /pub open_path: String/);
  assert.match(snapshot, /open_path:\s*source\.open_path\.clone\(\)/);
  assert.doesNotMatch(snapshot, /open_path:\s*source\.path\.clone\(\)/);

  assert.match(sourceSection, /open_exact_abs_path_button\([\s\S]*&row\.open_path/);
  assert.doesNotMatch(sourceSection, /open_exact_abs_path_button\([\s\S]*&row\.path/);
  assert.doesNotMatch(sourceSection, /open_exact_abs_path_button\([\s\S]*&snapshot\.workspace_roots/);

  assert.equal(
    (packageStatus.match(/open_path:\s*path\.display\(\)\.to_string\(\)/g) ?? []).length,
    3,
  );
  assert.match(machineCache, /open_path:\s*dx_root\.display\(\)\.to_string\(\)/);
  assert.match(remoteRegistry, /open_path:\s*registry_open_path\.clone\(\)/);
  assert.match(remoteRegistryProviders, /registry_open_path:\s*registry_open_path\.to_string\(\)/);

  assert.match(providersView, /target_open_path_for_provider\(provider, snapshot\)/);
  assert.match(providersView, /target_open_path_for_group\(group, snapshot\)/);
  assert.match(providersView, /remote\.registry_open_path\.as_str\(\)/);
  assert.doesNotMatch(providersView, /workspace_path\([\s\S]*snapshot\.workspace_roots/);
  assert.match(controls, /pub\(super\) fn exact_abs_path/);
  assert.match(controls, /pub\(super\) fn open_exact_abs_path/);
  assert.doesNotMatch(controls, /pub\(super\) fn open_workspace_path/);

  assert.match(openPathButtonBody, /let tooltip_text = if enabled \{/);
  assert.match(openPathButtonBody, /\.style\(ButtonStyle::Subtle\)[\s\S]*\.tab_index\(0_isize\)[\s\S]*\.disabled\(!enabled\)/);
  assert.match(openPathButtonBody, /format!\("\{tooltip\} unavailable"\)/);
  assert.match(openPathButtonBody, /Tooltip::text\(tooltip_text\)/);
  assert.doesNotMatch(openPathButtonBody, /"Source unavailable"/);
  assert.doesNotMatch(openPathButtonBody, /\bworkspace_roots\b|workspace_path\(/);
  assert.doesNotMatch(
    controls,
    /for root in workspace_roots[\s\S]*PathBuf::from\(root\)\.join|workspace_roots\s*\.\s*first\(\)[\s\S]*join/,
  );
});
