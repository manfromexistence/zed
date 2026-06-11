# DX WWW Turbo Web Preview Integration

## Goal

Connect Zed Web Preview to running DX WWW localhost apps through source-owned runtime contracts, without starting dev servers, running builds, or adopting external framework internals.

## Implementation Plan

1. Reuse the existing Web Preview and DX Studio IPC bridge.
2. Detect only loopback DX WWW pages before probing runtime endpoints.
3. Read `/_dx/hot-reload/version` with the DX WWW `resource` query contract and no cached responses.
4. Read `/_dx/devtools/session` as an optional runtime capability signal.
5. Store a compact native snapshot in `WebPreviewView` and expose it through `browser_session_snapshot`.
6. Keep the bridge one-shot with short TTL caching; do not poll, start servers, or mutate files.
7. Guard the contract with a focused Node source test.

## Boundaries

- No `just run`, Cargo build/check/test/clippy, or server startup in this lane.
- No edits to `G:\Dx\www` while parallel DX WWW worker output is dirty.
- No raw hot-reload token exposure in native summaries.
- No fake Turbo runtime claims: Zed consumes DX WWW receipts and status surfaces only.

## Verification

- Focused source guard: `node --test script\dx-www-turbo-web-preview-source.test.ts`
- Touched-file formatting: `rustfmt --edition 2024 --check crates\web_preview\src\web_preview_view.rs`
- Diff hygiene: `git diff --check`
