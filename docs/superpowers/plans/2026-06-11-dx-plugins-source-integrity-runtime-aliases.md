# DX Plugins Source Integrity And Runtime Aliases

## Goal

Make the next DX Plugins foundation slice production-readable without heavy validation:

- Emit a source-root integrity report for first-party plugin manifests.
- Keep Browser, Computer, and Driven source roots constrained to DX-owned allowlisted roots.
- Add explicit runtime-status aliases for `dx.browser`, `dx.computer`, and `dx.driven`.
- Guard the contract with focused source tests and forbidden-upstream scans.
- Update repo handoff docs, then commit the coherent source-only change.

## Constraints

- Do not run `just run`, Cargo builds, broad Cargo checks, local servers, or browser automation.
- Keep this as a source-verified contract slice.
- Keep the six spawned subagents isolated to audit scopes; do not spawn more.
- Avoid dummy UI, fake ready states, broad filesystem scans, or external plugin source references.

## Implementation Steps

1. Add a typed source-root integrity model to `dx_plugin_manifest.rs`.
2. Build the catalog from discovery plus manifests, then validate manifest root ids and runtime entrypoints against the allowlist.
3. Add the managed Playwright runner source root needed by `dx.computer` so its runtime entrypoint no longer points outside `dxjs_runtime`.
4. Add runtime alias metadata to `inspect_agent_plugin_runtime_status`, mapping DX plugin ids to existing Browser, managed Chrome, PC-use, and Driven readiness lanes.
5. Extend focused Node source guards for the new schemas, aliases, and root-entrypoint boundary.
6. Update `todo.txt`, `changelog.txt`, and `DX.md`, then run lightweight verification and commit.

## Verification

- `node --test script\dx-plugin-catalog-source.test.ts`
- `node --test script\dx-agent-plugin-runtime-status-source.test.ts`
- Focused `rg` forbidden-upstream scan over touched plugin source files.
- Focused conflict-marker scan over touched files.
- `git diff --check`
