import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

const agentUi = readFileSync("crates/agent_ui/src/agent_ui.rs", "utf8");
const moduleRoot = readFileSync("crates/agent_ui/src/dx_forge_panel.rs", "utf8");
const controls = readFileSync("crates/agent_ui/src/dx_forge_panel/controls.rs", "utf8");
const panel = readFileSync("crates/agent_ui/src/dx_forge_panel/panel.rs", "utf8");
const machineCachePath = "crates/agent_ui/src/dx_forge_panel/machine_cache.rs";
const machineCache = existsSync(machineCachePath)
  ? readFileSync(machineCachePath, "utf8")
  : "";
const packageStatusPath = "crates/agent_ui/src/dx_forge_panel/package_status.rs";
const packageStatus = existsSync(packageStatusPath)
  ? readFileSync(packageStatusPath, "utf8")
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
const snapshot = readFileSync("crates/agent_ui/src/dx_forge_panel/snapshot.rs", "utf8");
const snapshotStatePath = "crates/agent_ui/src/dx_forge_panel/snapshot_state.rs";
const snapshotState = existsSync(snapshotStatePath)
  ? readFileSync(snapshotStatePath, "utf8")
  : "";
const panelView = readFileSync("crates/agent_ui/src/dx_forge_panel/panel_view.rs", "utf8");
const rows = readFileSync("crates/agent_ui/src/dx_forge_panel/rows.rs", "utf8");
const icons = readFileSync("crates/icons/src/icons.rs", "utf8");
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
  machineCache,
  packageStatus,
  remoteRegistrySources,
  panel,
  providers,
  sourceSection,
  snapshot,
  snapshotState,
  panelView,
  rows,
].join("\n");
const forgeReaderSources = [
  machineCache,
  packageStatus,
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
    /\bgit::|GitStore|git_store|Repository|GitRepository/,
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
  assert.match(panel, /Some\(IconName::Forgejo\)/);
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

test("Forge panel reads package-status without runtime overclaims", () => {
  assert.ok(
    existsSync(packageStatusPath),
    "Forge package-status reader must live in a focused module",
  );
  assert.match(moduleRoot, /mod package_status;/);
  assert.match(snapshot, /package_status_rows\(workspace_roots\)/);
  assert.match(snapshot, /pub\(super\) package_statuses: Vec<DxForgeSourceRow>/);
  assert.match(snapshot, /visible_package_status_warning_count/);
  assert.match(snapshot, /PACKAGE_STATUS_LABEL/);
  assert.match(panelView, /fn package_status_section/);
  assert.ok(
    panelView.indexOf("package_status_section(snapshot, workspace, cx)") <
      panelView.indexOf("receipt_section(snapshot, workspace, cx)"),
    "package status should be visible before raw receipts",
  );
  assert.match(panelView, /"Package Status"/);
  assert.match(panelView, /No Forge package status found/);
  assert.match(packageStatus, /const MAX_PACKAGE_STATUS_BYTES: u64 = 1024 \* 1024;/);
  assert.match(packageStatus, /join\("\.dx"\)[\s\S]*\.join\("forge"\)[\s\S]*\.join\("package-status\.json"\)/);
  assert.match(packageStatus, /join\("\.forge"\)[\s\S]*\.join\("receipts"\)[\s\S]*\.join\("package-status\.json"\)/);
  assert.ok(
    packageStatus.indexOf('join(".forge")') < packageStatus.indexOf('join(".dx")'),
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
    panelView.indexOf("package_status_section(snapshot, workspace, cx)") <
      panelView.indexOf("machine_cache_section(snapshot, workspace, cx)") &&
      panelView.indexOf("machine_cache_section(snapshot, workspace, cx)") <
        panelView.indexOf("receipt_section(snapshot, workspace, cx)"),
    "machine cache evidence should sit between package status and raw receipts",
  );
  assert.match(panelView, /"Machine Caches"/);
  assert.match(panelView, /No Forge machine caches found/);
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
  assert.match(machineCache, /let dx_root = Path::new\(root\)\.join\("\.dx"\);/);
  assert.match(machineCache, /join\("\.dx"\)/);
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
  assert.match(panelView, /Open a workspace to read Forge machine caches/);
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
  assert.match(panelView, /"Remote Registry"/);
  assert.match(panelView, /No Forge remote registry found/);
  assert.ok(
    panelView.indexOf("remote_registry_section(snapshot, workspace, cx)") <
      panelView.indexOf("package_status_section(snapshot, workspace, cx)"),
    "remote registry should be visible before package/cache evidence",
  );

  assert.match(remoteRegistry, /const REMOTE_REGISTRY_CACHE_TTL: Duration = Duration::from_secs\(5\);/);
  assert.match(remoteRegistry, /const MAX_REMOTE_REGISTRY_BYTES: u64 = 256 \* 1024;/);
  assert.match(remoteRegistry, /const MAX_WORKSPACE_ROOTS: usize = 4;/);
  assert.match(remoteRegistry, /join\("\.forge"\)[\s\S]*\.join\("remotes\.json"\)/);
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
  assert.match(remoteRegistrySources, /registry file only; live remote health unchecked/);
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
  assert.match(providersView, /provider_target_state\(provider, snapshot\)/);
  assert.match(providersView, /target_path_for_provider\(provider, snapshot\)/);
  assert.match(providersView, /target_open_path_for_provider\(provider, snapshot\)/);
  assert.match(providersView, /remote\.registry_open_path\.as_str\(\)/);
  assert.match(providers, /Configured remote:/);
  assert.match(providers, /Registry:/);
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
  assert.match(panelView, /No Forge receipts found/);
  assert.match(
    panelView,
    /Forge receipts found, but no known summaries were readable/,
  );
  assert.match(panelView, /No restore previews found/);
  assert.match(panelView, /No media outputs found/);
  assert.match(panelView, /side_panel_header_controls/);
  assert.match(rows, /DxForgePanelState::NoWorkspace/);
  assert.match(rows, /\.min_w_0\(\)/);
  assert.match(rows, /Tooltip::with_meta/);
  assert.match(rows, /truncate_start\(\)/);
});

test("Forge panel uses Git-style controls instead of metric cards", () => {
  const rowShellBody =
    rows.match(/fn row_shell\([\s\S]*?\r?\n}\r?\n\r?\nfn receipt_tooltip/)?.[0] ?? "";
  const emptyRowBody =
    rows.match(/pub\(super\) fn empty_row\([\s\S]*?\r?\n}\r?\n\r?\npub\(super\) fn state_presentation/)?.[0] ?? "";
  const evidenceRowBodies = `${rowShellBody}\n${emptyRowBody}`;

  assert.match(moduleRoot, /mod controls;/);
  assert.match(panelView, /toolbar\(snapshot, workspace, panel, cx\)/);
  assert.match(panelView, /section_header\(/);
  assert.match(rows, /pub\(super\) fn section_header/);
  assert.match(rows, /ListItem/);
  assert.match(rows, /ListItemSpacing/);
  assert.match(rowShellBody, /\)\s*->\s*ListItem\s*\{/);
  assert.match(rowShellBody, /ListItem::new\(id\)/);
  assert.match(rowShellBody, /\.inset\(true\)/);
  assert.match(rowShellBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(rowShellBody, /\.start_slot\(/);
  assert.match(rowShellBody, /\.end_slot\(open_button\)/);
  assert.match(emptyRowBody, /ListItem::new\(id\)/);
  assert.match(emptyRowBody, /\.inset\(true\)/);
  assert.match(emptyRowBody, /\.spacing\(ListItemSpacing::Sparse\)/);
  assert.match(emptyRowBody, /\.selectable\(false\)/);
  assert.doesNotMatch(
    evidenceRowBodies,
    /\bh_flex\(\)|Stateful<Div>|\bDiv\b|\.border_1\(\)|ghost_element_(?:background|hover|active)/,
  );
  assert.match(rows, /ghost_element_hover/);
  assert.match(controls, /IconButton::new\("dx-forge-open-history", IconName::FolderOpen\)/);
  assert.match(controls, /IconButton::new\("dx-forge-refresh", IconName::RotateCw\)/);
  assert.match(controls, /IconButton::new\(id, IconName::ArrowUpRight\)/);
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

test("Forge panel renders DX icon provider targets with snapshot-driven readiness", () => {
  assert.match(moduleRoot, /mod providers;/);
  assert.match(panelView, /remote_target_strip\(snapshot, workspace, cx\)/);
  assert.ok(existsSync(providersRootPath), "Forge provider module boundary must live in providers/mod.rs");
  assert.ok(existsSync(providersCatalogPath), "Forge provider metadata must live in catalog.rs");
  assert.ok(existsSync(providersStatePath), "Forge provider state must live in state.rs");
  assert.ok(existsSync(providersViewPath), "Forge provider GPUI rendering must live in view.rs");
  assert.ok(existsSync(providersTooltipsPath), "Forge provider tooltip copy must live in tooltips.rs");
  assert.match(providersRoot, /mod view;/);
  assert.match(providersRoot, /mod tooltips;/);
  assert.match(providersRoot, /pub\(super\) use self::view::remote_target_strip;/);

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
  assert.match(providersView, /fn provider_buttons_for_group/);
  assert.match(providersView, /fn remote_lane_row/);
  assert.match(providersView, /fn target_path_for_group/);
  assert.match(providers, /fn provider_tooltip_meta/);
  assert.match(providersView, /IconButton::new\(format!\("dx-forge-provider-\{\}", provider\.id\), provider\.icon\)/);
  assert.match(providersView, /provider_buttons_for_group\(group, snapshot, workspace, cx\)/);
  assert.match(providersView, /IconButtonShape::Square/);
  assert.match(providersView, /ButtonStyle::Subtle/);
  assert.match(providersView, /ButtonStyle::Tinted\(TintColor::Warning\)/);
  assert.match(providersView, /ButtonStyle::Tinted\(TintColor::Success\)/);
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
  assert.match(providers, /Icon::new\(state\.icon\)/);
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
  assert.match(receiptFiles, /for entry in entries\.flatten\(\) \{/);
  assert.doesNotMatch(receiptFiles, /entries\s*\.flatten\(\)\s*\.take\(RECEIPT_HISTORY_LATEST_ROOT_ENTRY_LIMIT\)/);
  assert.doesNotMatch(receiptFiles, /children\s*\.flatten\(\)\s*\.take\(RECEIPT_HISTORY_LATEST_NESTED_ENTRY_LIMIT\)/);
  assert.match(sourceSetReceipts, /const LATEST_RECEIPT_CANDIDATE_LIMIT: usize = 64;/);
  assert.match(sourceSetReceipts, /push_latest_receipt_candidate/);
  assert.doesNotMatch(sourceSetReceipts, /entries\.flatten\(\)\.take\(128\)/);
  assert.match(receiptFields, /array_len_field\(value, &\["restore_execution", "restore", "blockers"\]\)/);
  assert.match(receiptFields, /array_len_field\(value, &\["runner_gate", "validation", "blockers"\]\)/);
});

test("Forge panel files stay small and professionally named", () => {
  const lineCounts = new Map([
    ["dx_forge_panel.rs", moduleRoot],
    ["controls.rs", controls],
    ["machine_cache.rs", machineCache],
    ["package_status.rs", packageStatus],
    ["remote_registry.rs", remoteRegistry],
    ["remote_registry/providers.rs", remoteRegistryProviders],
    ["panel.rs", panel],
    ["providers/mod.rs", providersRoot],
    ["providers/catalog.rs", providersCatalog],
    ["providers/state.rs", providersState],
    ["providers/tooltips.rs", providersTooltips],
    ["providers/view.rs", providersView],
    ["source_section.rs", sourceSection],
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

  assert.match(allForgePanelSources, /source-only receipt evidence/);
  assert.match(allForgePanelSources, /receipt file only; live checks not executed/);
  assert.match(allForgePanelSources, /live remote health unchecked/);
});

test("Forge panel opens exact source-owned paths in multi-root workspaces", () => {
  const openPathButtonBody =
    controls.match(/pub\(super\) fn open_exact_abs_path_button\([\s\S]*?\n}\n\npub\(super\) fn exact_abs_path/)?.[0] ?? "";

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

  assert.doesNotMatch(openPathButtonBody, /\bworkspace_roots\b|workspace_path\(/);
  assert.doesNotMatch(
    controls,
    /for root in workspace_roots[\s\S]*PathBuf::from\(root\)\.join|workspace_roots\s*\.\s*first\(\)[\s\S]*join/,
  );
});
