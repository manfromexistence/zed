import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const registeredGuardScripts = [
  "script/dx-handoff-source-guard-registry.test.ts",
  "script/dx-windows-reliability-source.test.ts",
  "script/zed-builtin-schema-source.test.ts",
  "script/zed-platform-source.test.ts",
  "script/zed-startup-reliability-source.test.ts",
  "script/dx-project-agent-registry-source.test.ts",
  "script/dx-agent-registry-ui-source.test.ts",
  "script/dx-completion-provider-source.test.ts",
  "script/dx-project-search-source.test.ts",
  "script/dx-prompt-store-source.test.ts",
  "script/dx-task-inventory-source.test.ts",
  "script/dx-lsp-aggregation-source.test.ts",
  "script/dx-language-model-selector-source.test.ts",
  "script/dx-model-selector-source.test.ts",
  "script/dx-agent-ui-selector-source.test.ts",
  "script/zed-open-listener-source.test.ts",
  "script/dx-context-server-store-source.test.ts",
  "script/dx-worktree-event-source.test.ts",
  "script/dx-entry-view-state-source.test.ts",
  "script/dx-profile-selector-source.test.ts",
  "script/dx-terminal-thread-metadata-source.test.ts",
  "script/dx-terminal-display-source.test.ts",
  "script/dx-editor-git-source.test.ts",
  "script/dx-editor-search-source.test.ts",
  "script/dx-editor-items-source.test.ts",
  "script/dx-editor-context-menu-source.test.ts",
  "script/dx-editor-lsp-ui-source.test.ts",
  "script/dx-editor-element-source.test.ts",
  "script/dx-editor-line-ops-source.test.ts",
  "script/dx-language-registry-source.test.ts",
  "script/zed-app-fanout-source.test.ts",
  "script/dx-terminal-view-source.test.ts",
  "script/dx-file-finder-source.test.ts",
  "script/dx-agent-diff-source.test.ts",
  "script/dx-editor-navigation-source.test.ts",
  "script/dx-editor-inlay-semantic-source.test.ts",
  "script/dx-editor-input-source.test.ts",
  "script/dx-editor-power-rainbow-source.test.ts",
  "script/dx-workspace-ui-state-source.test.ts",
  "script/dx-title-bar-source.test.ts",
  "script/dx-project-panel-source.test.ts",
  "script/dx-diagnostics-ui-source.test.ts",
  "script/dx-git-ui-source.test.ts",
  "script/dx-settings-keymap-source.test.ts",
  "script/dx-settings-ui-source.test.ts",
  "script/dx-extensions-ui-source.test.ts",
  "script/dx-outline-theme-source.test.ts",
  "script/dx-command-palette-source.test.ts",
  "script/dx-picker-core-source.test.ts",
  "script/dx-search-ui-source.test.ts",
  "script/dx-debugger-ui-source.test.ts",
  "script/dx-debugger-panel-source.test.ts",
  "script/dx-debugger-run-terminal-source.test.ts",
  "script/dx-debugger-stack-frame-source.test.ts",
  "script/dx-debugger-module-list-source.test.ts",
  "script/dx-collab-panel-source.test.ts",
  "script/dx-collab-notifications-source.test.ts",
  "script/dx-markdown-preview-source.test.ts",
  "script/dx-activity-indicator-source.test.ts",
  "script/dx-project-symbols-source.test.ts",
  "script/dx-snippets-source.test.ts",
  "script/dx-repl-ui-source.test.ts",
  "script/dx-preview-surfaces-source.test.ts",
  "script/dx-component-preview-source.test.ts",
  "script/dx-acp-tools-source.test.ts",
  "script/dx-breadcrumbs-source.test.ts",
  "script/dx-icon-picker-source.test.ts",
  "script/dx-icon-system-source.test.ts",
  "script/dx-icon-theme-source.test.ts",
  "script/dx-gpui-gradient-source.test.ts",
  "script/dx-language-selector-source.test.ts",
  "script/dx-toolchain-selector-source.test.ts",
  "script/dx-dev-container-source.test.ts",
  "script/dx-recent-projects-ui-source.test.ts",
  "script/dx-notifications-source.test.ts",
  "script/dx-encoding-selector-source.test.ts",
  "script/dx-line-ending-selector-source.test.ts",
  "script/dx-settings-profile-selector-source.test.ts",
  "script/dx-tasks-modal-source.test.ts",
  "script/dx-feedback-source.test.ts",
  "script/dx-font-panel-source.test.ts",
  "script/dx-media-panel-source.test.ts",
  "script/dx-ui-panel-source.test.ts",
  "script/dx-agent-config-options-source.test.ts",
  "script/dx-agent-tool-picker-source.test.ts",
  "script/dx-debugger-attach-modal-source.test.ts",
  "script/dx-debugger-new-process-source.test.ts",
  "script/dx-open-path-prompt-source.test.ts",
  "script/dx-tab-switcher-source.test.ts",
  "script/dx-agent-server-store-source.test.ts",
  "script/dx-workspace-reentrant-source.test.ts",
  "script/dx-workspace-persistence-source.test.ts",
  "script/dx-workspace-enumeration-source.test.ts",
  "script/dx-source-quality.test.ts",
  "script/dx-style-panel-source.test.ts",
  "script/dx-forge-panel-source.test.ts",
  "script/dx-project-context-source.test.ts",
  "script/dx-studio-project-source.test.ts",
  "script/dx-buffer-codegen-source.test.ts",
  "script/dx-inline-prompt-source.test.ts",
  "script/dx-agent-panel-clipboard-source.test.ts",
  "script/dx-agent-panel-input-source.test.ts",
  "script/dx-agent-composer-liquid-glass-source.test.ts",
  "script/dx-agent-voice-controls-source.test.ts",
  "script/dx-agent-thread-view-source.test.ts",
  "script/dx-agent-configuration-source.test.ts",
  "script/dx-agent-configuration-modal-source.test.ts",
  "script/dx-agent-persisted-state-source.test.ts",
  "script/dx-mention-set-source.test.ts",
  "script/dx-message-editor-source.test.ts",
  "script/dx-thread-metadata-source.test.ts",
  "script/dx-thread-archive-source.test.ts",
  "script/dx-deploy-panel-source.test.ts",
  "script/dx-deploy-receipts-source.test.ts",
  "script/dx-deploy-launch-gate-source.test.ts",
  "script/dx-deploy-launch-evidence-source.test.ts",
  "script/dx-check-panel-source.test.ts",
  "script/dx-launch-workspace-source.test.ts",
  "script/dx-launch-audit-source.test.ts",
  "script/dx-launch-audit-fixtures.test.ts",
  "script/dx-launch-binary-cache-source.test.ts",
  "script/dx-launch-contracts-source.test.ts",
  "script/dx-launch-contracts-fixtures.test.ts",
  "script/dx-launch-prompts-source.test.ts",
  "script/dx-launch-readiness-source.test.ts",
  "script/dx-launch-readiness-fixtures.test.ts",
  "script/dx-launch-receipts-source.test.ts",
  "script/dx-launch-source-audit-source.test.ts",
  "script/dx-launch-status-source.test.ts",
  "script/dx-receipt-history-source.test.ts",
  "script/dx-receipt-directory-source.test.ts",
  "script/dx-runtime-proof-status-source.test.ts",
  "script/dx-source-sets-source.test.ts",
  "script/dx-ai-search-profile-source.test.ts",
  "script/dx-evidence-basket-source.test.ts",
  "script/dx-agent-bridge-source.test.ts",
  "script/dx-agent-connections-ui-source.test.ts",
  "script/dx-agent-workspaces-source.test.ts",
  "script/dx-automations-source.test.ts",
  "script/dx-plugin-catalog-source.test.ts",
  "script/dx-plugin-credentials-source.test.ts",
  "script/dx-icon-hover-source.test.ts",
  "script/web-preview-payload-source.test.ts",
  "script/dx-www-launch-evidence-source.test.ts",
  "script/web-preview-platform-lifecycle.test.ts",
];

test("DX.md exposes the lightweight source guard registry", () => {
  const dx = read("DX.md");

  assert.match(dx, /## Lightweight Source Guard Registry/);
  assert.match(dx, /These guards are source-contract checks only\./);
  assert.match(dx, /do not prove native runtime behavior/);
  assert.match(dx, /Run the narrowest guard that matches the owned lane/);

  for (const guard of registeredGuardScripts) {
    assert.ok(existsSync(guard), `registered guard should exist: ${guard}`);
    assert.ok(dx.includes(guard), `DX.md should list ${guard}`);
  }

  assert.match(
    dx,
    /script\/dx-style-panel-source\.test\.ts` - DX Style .*source-apply session/,
  );
  assert.match(
    dx,
    /script\/dx-project-panel-source\.test\.ts` - project panel .*compact Project header icon\/side-panel controls\/selection toolbar.*file-browser operation toolbar.*cached storage roots.*compact storage-root strip spacing with max-width storage shortcuts.*storage-root capacity\/status label ownership.*folder storage overview.*dense storage drilldown rows.*storage ranking.*compact bounded entry metadata chip spacing.*media shelf\/gallery lower-bound count labels/,
  );
  assert.match(
    dx,
    /script\/dx-project-panel-source\.test\.ts` - project panel .*tooltip-only largest-file drilldown detail.*entry info badge tooltips/,
  );
  assert.match(
    dx,
    /script\/dx-media-panel-source\.test\.ts` - media panel .*local\/remote-result `ListItem` rows.*URL insertion `ListItem` row.*tooltip-backed square icon URL\/local\/remote asset actions.*disabled no-op filter pager arrows.*subtle square header\/filter controls.*square icon history-management actions.*hover action slots.*recent\/pinned `ListHeader` history-section chrome boundaries/,
  );
  assert.match(
    dx,
    /script\/dx-ui-panel-source\.test\.ts` - UI panel status-row.*catalog-result `ListItem` rows.*install-plan sparse `ListItem` guidance.*primary catalog action ownership.*disabled no-op filter pager arrows.*subtle square header\/filter controls.*square icon secondary catalog actions.*recent\/pinned `ListHeader` history-section.*square icon history-management actions.*hover action slots/,
  );
  assert.match(
    dx,
    /script\/dx-forge-panel-source\.test\.ts` - DX Forge .*workflow tab and source-open tooltip wording.*compact status\/action row spacing.*tabbable subtle source\/provider open controls.*empty-row tooltip preservation.*provider-group hover readiness dots.*provider-state compact-dot ownership without dead icon plumbing/,
  );
  assert.match(
    dx,
    /script\/dx-check-panel-source\.test\.ts` - DX Check .*keyboard-reachable status-row receipt\/refresh actions.*compact section count labels.*compact notice\/quick-fix\/adapter-plan\/web-audit `ListItem` rows.*capped web audit rows.*collapsed Adapter Plans default/,
  );
});

test("handoff docs keep source-only proof separate from runtime readiness", () => {
  const dx = read("DX.md");
  const agents = read("AGENTS.md");

  assert.match(
    dx,
    /Existing `100\/100`, "complete", and "production" notes in older handoffs mean source\/code-complete/,
  );
  assert.match(
    dx,
    /Do not claim runtime-green, production-ready, or launch-ready from these docs alone\./,
  );
  assert.match(dx, /no-`just run` and no-Cargo by direct instruction/);
  assert.match(
    agents,
    /current user prompt explicitly opens the final validation window/,
  );
  assert.match(agents, /source-only or release-hygiene passes/);
  assert.match(
    agents,
    /\*\*NEVER\*\* when the current user prompt or handoff lane forbids it/,
  );
});

test("current handoff names the no-runtime-proof production-readiness boundary", () => {
  const dx = read("DX.md");
  const todo = read("todo.txt");
  const currentVerificationLane =
    dx.match(/## Current Verification Lane[\s\S]*?(?=\n## Lightweight Source Guard Registry)/)?.[0] ?? "";
  const currentLogoIconLane =
    currentVerificationLane.match(/- Current DX logo and icon-theme source slice:[^\n]+/)?.[0] ?? "";

  assert.ok(currentLogoIconLane, "expected current DX logo and icon-theme source lane");
  assert.match(currentLogoIconLane, /Agent chat input toolbar now centers/);
  assert.match(
    currentLogoIconLane,
    /Design, Graphics, Presentations, Spreadsheets, Video, Music, Whiteboard, 3D, Shader, and DX Web/,
  );
  assert.match(currentLogoIconLane, /theme-aware light\/dark variants/);
  assert.match(currentLogoIconLane, /real Add Context `\+` trigger stays on the left control cluster/);
  assert.match(currentLogoIconLane, /Project Panel to the vendored `DX Icon` theme/);
  assert.match(currentLogoIconLane, /zed-extensions\/material-icon-theme` v1\.3\.1/);
  assert.match(currentLogoIconLane, /Focused Node source guards/);
  assert.match(currentLogoIconLane, /null-byte scan/);
  assert.match(currentLogoIconLane, /Cargo, build, `just run`, native launch, local servers, browser automation, and visual runtime proof remain deferred by instruction/);
  assert.doesNotMatch(currentLogoIconLane, /runtime-green|launch-ready|production-ready/i);
  assert.match(todo, /Agent chat input and Project Panel icon theme branding are source-verified/);
  assert.match(dx, /Current production readiness is source-audited only/i);
  assert.match(
    dx,
    /Skipped by direct instruction: Cargo build\/check\/test\/clippy, `just run`, local servers, browser automation, and live editor runtime proof\./,
  );
  assert.match(todo, /Skipped by direct instruction: Cargo build\/check\/test\/clippy, `just run`, local servers, browser automation, and live editor runtime proof\./);
});

test("current Project Panel handoff names the storage drilldown ListItem contract", () => {
  const dx = read("DX.md");
  const currentProjectPanelLane =
    dx.match(/Current Web Preview completion[\s\S]*?visual proof remain deferred\./)?.[0] ?? "";

  assert.ok(currentProjectPanelLane, "expected current Web Preview and Project Panel handoff lane");
  assert.match(
    currentProjectPanelLane,
    /storage drilldown rows use focusable shared `ListItem` chrome/,
  );
  assert.doesNotMatch(
    currentProjectPanelLane,
    /storage drilldown rows use focusable shared `ButtonLike` chrome/,
  );
});
