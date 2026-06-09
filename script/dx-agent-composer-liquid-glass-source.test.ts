import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const cargo = read("crates/agent_ui/Cargo.toml");
const conversationView = read("crates/agent_ui/src/conversation_view.rs");
const threadView = read("crates/agent_ui/src/conversation_view/thread_view.rs");
const messageEditor = read("crates/agent_ui/src/message_editor.rs");
const registry = read("script/dx-handoff-source-guard-registry.test.ts");

const functionBody = (source: string, name: string): string => {
  const signature = new RegExp(
    `\\n    (?:pub\\(crate\\)\\s+|pub\\s+)?fn ${name}\\(`,
  );
  const match = signature.exec(source);
  assert.ok(match?.index !== undefined, `expected function ${name}`);

  const start = match.index + 1;
  const openBrace = source.indexOf("{", start);
  assert.ok(openBrace > start, `expected ${name} to have a body`);

  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
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

test("Agent composer stays on normal GPUI chrome without Liquid Glass", () => {
  const renderMessageEditor = functionBody(threadView, "render_message_editor");
  const messageEditorRender = functionBody(messageEditor, "render");

  assert.doesNotMatch(cargo, /liquid_glass\.workspace = true/);
  assert.doesNotMatch(conversationView, /mod composer_liquid_glass;/);
  assert.doesNotMatch(
    threadView,
    /liquid_glass|load_liquid_glass_backdrop_carrier|composer_glass_surface_style|render_composer_liquid_glass_layer|agent-composer-liquid-glass-source/,
  );
  assert.match(renderMessageEditor, /let colors = cx\.theme\(\)\.colors\(\);/);
  assert.match(
    renderMessageEditor,
    /\.border_color\(colors\.border\)\s*\.bg\(colors\.panel_background\)/,
  );
  assert.match(renderMessageEditor, /self\.message_editor\.clone\(\)/);
  assert.match(renderMessageEditor, /self\.render_add_context_button\(cx\)/);
  assert.match(renderMessageEditor, /self\.render_profile_option_slots\(cx\)/);
  assert.match(renderMessageEditor, /self\.render_voice_controls\(window, cx\)/);
  assert.match(renderMessageEditor, /self\.render_send_button\(cx\)/);
  assert.match(messageEditorRender, /background: cx\.theme\(\)\.system\(\)\.transparent/);
  assert.match(renderMessageEditor, /let expands_editor_area = editor_expanded && has_messages;/);
  assert.match(renderMessageEditor, /else \{\s*this\.flex_1\(\)\.w_full\(\)\s*\}/);
  assert.doesNotMatch(renderMessageEditor, /else \{\s*this\.flex_1\(\)\.size_full\(\)\s*\}/);
});

test("composer non-glass guard remains discoverable", () => {
  assert.match(registry, /"script\/dx-agent-composer-liquid-glass-source\.test\.ts"/);
});
