import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const agentUi = readFileSync("crates/agent_ui/src/agent_ui.rs", "utf8");
const moduleRoot = readFileSync("crates/agent_ui/src/dx_forge_panel.rs", "utf8");
const controls = readFileSync("crates/agent_ui/src/dx_forge_panel/controls.rs", "utf8");
const panel = readFileSync("crates/agent_ui/src/dx_forge_panel/panel.rs", "utf8");
const providersRootPath = "crates/agent_ui/src/dx_forge_panel/providers/mod.rs";
const providersCatalogPath = "crates/agent_ui/src/dx_forge_panel/providers/catalog.rs";
const providersStatePath = "crates/agent_ui/src/dx_forge_panel/providers/state.rs";
const providersRoot = existsSync(providersRootPath)
  ? readFileSync(providersRootPath, "utf8")
  : "";
const providersCatalog = existsSync(providersCatalogPath)
  ? readFileSync(providersCatalogPath, "utf8")
  : "";
const providersState = existsSync(providersStatePath)
  ? readFileSync(providersStatePath, "utf8")
  : "";
const snapshot = readFileSync("crates/agent_ui/src/dx_forge_panel/snapshot.rs", "utf8");
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
const providers = [providersRoot, providersCatalog, providersState].join("\n");
const forgeSources = [moduleRoot, controls, panel, providers, snapshot, panelView, rows].join("\n");
const forgeReaderSources = [
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
  assert.match(panel, /Some\(IconName::Archive\)/);
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
  assert.match(snapshot, /const MAX_WORKSPACE_ROOTS: usize = 4;/);
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
  assert.match(moduleRoot, /mod controls;/);
  assert.match(panelView, /toolbar\(snapshot, workspace, panel, cx\)/);
  assert.match(panelView, /section_header\(/);
  assert.match(rows, /pub\(super\) fn section_header/);
  assert.match(rows, /ghost_element_background/);
  assert.match(rows, /ghost_element_hover/);
  assert.match(rows, /ghost_element_active/);
  assert.match(controls, /Button::new\("dx-forge-open-history", "History"\)/);
  assert.match(controls, /IconButton::new\("dx-forge-refresh", IconName::RotateCw\)/);
  assert.match(controls, /IconButton::new\(id, IconName::ArrowUpRight\)/);
  assert.match(controls, /open_abs_path\(/);
  assert.match(controls, /OpenOptions/);
  assert.match(panel, /pub\(super\) fn refresh/);
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
  assert.match(panelView, /remote_target_strip\(snapshot, cx\)/);
  assert.ok(existsSync(providersRootPath), "Forge provider view must live in providers/mod.rs");
  assert.ok(existsSync(providersCatalogPath), "Forge provider metadata must live in catalog.rs");
  assert.ok(existsSync(providersStatePath), "Forge provider state must live in state.rs");

  const providerTargets = [
    ["GitHub", "DxForgeProviderGithub", "dx_forge_provider_github", "ProviderGroup::Code", "svgl"],
    ["GitLab", "DxForgeProviderGitlab", "dx_forge_provider_gitlab", "ProviderGroup::Code", "svgl"],
    [
      "Bitbucket",
      "DxForgeProviderBitbucket",
      "dx_forge_provider_bitbucket",
      "ProviderGroup::Code",
      "material-icon-theme",
    ],
    [
      "Google Drive",
      "DxForgeProviderDrive",
      "dx_forge_provider_drive",
      "ProviderGroup::Storage",
      "svgl",
    ],
    [
      "Dropbox",
      "DxForgeProviderDropbox",
      "dx_forge_provider_dropbox",
      "ProviderGroup::Storage",
      "svgl",
    ],
    [
      "YouTube",
      "DxForgeProviderYoutube",
      "dx_forge_provider_youtube",
      "ProviderGroup::Media",
      "svgl",
    ],
    [
      "SoundCloud",
      "DxForgeProviderSoundcloud",
      "dx_forge_provider_soundcloud",
      "ProviderGroup::Media",
      "svgl",
    ],
  ];

  const providerBlock = (label: string) => {
    const start = providers.indexOf(`label: "${label}"`);
    assert.ok(start >= 0, `missing provider ${label}`);
    const end = providers.indexOf("ForgeProvider {", start + 1);
    return providers.slice(start, end >= 0 ? end : undefined);
  };

  for (const [label, iconName, fileName, group, sourcePack] of providerTargets) {
    const block = providerBlock(label);
    assert.match(icons, new RegExp(`\\b${iconName}\\b`));
    assert.match(block, new RegExp(`icon:\\s*IconName::${iconName}\\b`));
    assert.match(block, new RegExp(`group:\\s*${group}\\b`));
    assert.match(block, new RegExp(`source_pack:\\s*"${sourcePack}"`));

    const iconPath = `assets/icons/${fileName}.svg`;
    assert.ok(
      existsSync(iconPath),
      `${fileName}.svg must be exported from the DX icon CLI`,
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
  assert.match(providersRoot, /providers_for\(group\)/);
  assert.match(providersRoot, /Icon::new\(provider\.icon\)/);
  assert.match(providersState, /fn code_target_state/);
  assert.match(providersState, /fn storage_target_state/);
  assert.match(providersState, /fn media_target_state/);
  assert.match(providers, /source_pack: "svgl"/);
  assert.match(providers, /source_pack: "material-icon-theme"/);
  assert.match(providers, /ProviderGroup::ALL/);
  assert.match(providers, /for group in ProviderGroup::ALL/);
  assert.match(providers, /fn remote_target_state/);
  assert.match(providers, /fn provider_icon_stack/);
  assert.match(providers, /history_root_exists/);
  assert.match(providers, /receipt_count/);
  assert.match(providers, /summarized_receipt_count/);
  assert.match(providers, /visible_blocker_count/);
  assert.match(providers, /restore_previews\.len\(\)/);
  assert.match(providers, /visible_restore_warning_count/);
  assert.match(providers, /media_outputs\.len\(\)/);
  assert.match(providers, /Tooltip::with_meta/);
  assert.match(providers, /Icon::new\(state\.icon\)/);
  assert.match(providers, /Label::new\(state\.label\)/);
  assert.match(providers, /Label::new\(state\.detail\.clone\(\)\)/);
  assert.match(providers, /h_flex\(\)[\s\S]*\.gap_0p5\(\)[\s\S]*\.flex_none\(\)[\s\S]*Icon::new\(state\.icon\)/);
  assert.doesNotMatch(providers, /Button::new|IconButton::new/);
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
    ["panel.rs", panel],
    ["providers/mod.rs", providersRoot],
    ["providers/catalog.rs", providersCatalog],
    ["providers/state.rs", providersState],
    ["snapshot.rs", snapshot],
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
