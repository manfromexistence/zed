import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const gradientStyles = read("crates/ui/src/styles/gradient.rs");
const gradientFade = read("crates/ui/src/components/gradient_fade.rs");
const skillsIllustration = read("crates/ui/src/components/ai/skills_illustration.rs");
const stylesMod = read("crates/ui/src/styles.rs");
const prelude = read("crates/ui/src/prelude.rs");
const gradientExample = read("crates/gpui/examples/gradient.rs");
const rawCallPattern = (name: string) => new RegExp(`(^|[^A-Za-z0-9_])${name}\\(`);

test("shared GPUI gradients use clamped linear Oklab helpers", () => {
  assert.match(gradientStyles, /UI_LINEAR_GRADIENT_COLOR_SPACE:\s*ColorSpace\s*=\s*ColorSpace::Oklab/);
  assert.match(gradientStyles, /linear_gradient\(/);
  assert.match(gradientStyles, /linear_color_stop\(/);
  assert.match(gradientStyles, /percentage\.clamp\(GRADIENT_STOP_MIN, GRADIENT_STOP_MAX\)/);
  assert.match(gradientStyles, /angle\.rem_euclid\(360\.0\)/);
  assert.match(gradientStyles, /\.color_space\(UI_LINEAR_GRADIENT_COLOR_SPACE\)/);
});

test("theme-aware surface patterns stay cheap and semantic", () => {
  assert.match(gradientStyles, /pub fn theme_linear_gradient\(cx: &App, angle: f32, from: Color, to: Color\)/);
  assert.match(gradientStyles, /from\.color\(cx\)/);
  assert.match(gradientStyles, /to\.color\(cx\)/);
  assert.match(gradientStyles, /pub fn panel_surface_gradient\(cx: &App\)/);
  assert.match(gradientStyles, /colors\.panel_background/);
  assert.match(gradientStyles, /colors\.surface_background/);
  assert.match(gradientStyles, /pub fn accent_surface_gradient\(cx: &App, accent: Color\)/);
  assert.doesNotMatch(gradientStyles, /background_executor|spawn|std::fs|File::open|canvas\(/);
});

test("shared UI surfaces consume gradient helpers instead of raw GPUI gradients", () => {
  assert.match(stylesMod, /mod gradient;/);
  assert.match(stylesMod, /pub use gradient::\*/);
  assert.match(prelude, /oklab_linear_gradient/);
  assert.match(prelude, /theme_linear_gradient/);
  assert.match(gradientFade, /oklab_linear_gradient_stops/);
  assert.match(gradientFade, /linear_gradient_stop/);
  assert.doesNotMatch(gradientFade, rawCallPattern("linear_gradient"));
  assert.doesNotMatch(gradientFade, rawCallPattern("linear_color_stop"));
  assert.match(skillsIllustration, /oklab_linear_gradient/);
  assert.doesNotMatch(skillsIllustration, rawCallPattern("linear_gradient"));
  assert.doesNotMatch(skillsIllustration, rawCallPattern("linear_color_stop"));
});

test("GPUI gradient lane remains linear-only; complex generators stay in Web Preview", () => {
  assert.match(gradientExample, /color_space:\s*ColorSpace::Oklab/);
  assert.doesNotMatch(gradientStyles, /fn\s+\w*(mesh|noise|radial)\w*/i);
  assert.doesNotMatch(gradientFade, /mesh|noise|radial/i);
  assert.doesNotMatch(skillsIllustration, /mesh|noise|radial/i);
});
