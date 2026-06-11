# Voice And Forge Stability Plan

## Goal

Fix the Dx Agent voice status experience and the Forge tab crash without broad runtime validation or heavy builds.

## Scope

- Show readable animated STT/TTS phase labels instead of collapsed ellipsis-only voice status text.
- Reuse the existing Kokoro JSON-lines server mode for repeated read-aloud requests, with the current one-shot command kept as fallback.
- Stop Forge tab pointer events from bubbling into panel focus/window paths, and keep tab IDs stable through typed GPUI IDs.
- Update focused source guards, `todo.txt`, and `changelog.txt`.

## Verification

- Focused Node source guards for voice and Forge.
- `git diff --check`.
- No `just run`, Cargo build, or broad checks unless separately authorized.
