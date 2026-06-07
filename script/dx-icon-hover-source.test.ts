import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = (path: string) => readFileSync(path, "utf8");

const animationExt = read("crates/ui/src/traits/animation_ext.rs");
const icon = read("crates/ui/src/components/icon.rs");
const iconButton = read("crates/ui/src/components/button/icon_button.rs");
const buttonLike = read("crates/ui/src/components/button/button_like.rs");
const svg = read("crates/gpui/src/elements/svg.rs");
const prelude = read("crates/ui/src/prelude.rs");

test("icon hover effects stay keyed, bounded, and reduced-motion aware", () => {
  assert.match(animationExt, /pub struct IconHoverEffect/);
  assert.match(animationExt, /pub fn subtle_scale\(\) -> Self/);
  assert.match(animationExt, /pub fn lift\(\) -> Self/);
  assert.match(animationExt, /pub fn reduced_motion\(mut self, reduced_motion: bool\) -> Self/);
  assert.match(animationExt, /self\.opacity = opacity\.clamp\(0\.0, 1\.0\)/);
  assert.match(animationExt, /self\.duration = duration\.max\(Duration::from_millis\(1\)\)/);
  assert.match(animationExt, /Animation::new\(self\.duration\)\.with_easing\(ease_out_quint\(\)\)/);
  assert.match(animationExt, /base\.then\(/);
  assert.match(svg, /pub fn then\(mut self, transform: Transformation\) -> Self/);
});

test("hover animation element preserves target-driven state across frames", () => {
  assert.match(animationExt, /pub struct HoverAnimationElement<E>/);
  assert.match(animationExt, /target_hovered: bool/);
  assert.match(animationExt, /from_progress: f32/);
  assert.match(animationExt, /started_at: Option<Instant>/);
  assert.match(animationExt, /state\.target_hovered != self\.hovered/);
  assert.match(animationExt, /window\.request_animation_frame\(\)/);
  assert.match(animationExt, /effect\.is_reduced_motion\(\)/);
  assert.doesNotMatch(animationExt, /spawn|background_executor|std::fs|thread::sleep/);
});

test("Icon and IconButton expose hover microinteraction primitives", () => {
  assert.match(icon, /HoverAnimatedIcon\(HoverAnimationElement<Icon>\)/);
  assert.match(icon, /pub fn opacity\(mut self, opacity: f32\) -> Self/);
  assert.match(icon, /pub fn with_hover_effect\(/);
  assert.match(icon, /effect\.is_reduced_motion\(\)/);
  assert.match(icon, /apply_hover_effect/);
  assert.match(icon, /\.opacity\(self\.opacity\)/);

  assert.match(iconButton, /hover_icon: Option<IconName>/);
  assert.match(iconButton, /hovered: bool/);
  assert.match(iconButton, /icon_hover_effect: Option<IconHoverEffect>/);
  assert.match(iconButton, /pub fn hover_icon/);
  assert.match(iconButton, /pub fn hovered/);
  assert.match(iconButton, /pub fn icon_hover_effect/);
  assert.match(iconButton, /pub fn on_hover/);
  assert.match(iconButton, /\.or_else\(\|\| self\.hover_icon\.filter\(\|_\| is_hovered\)\)/);
  assert.match(iconButton, /icon_element\.with_hover_effect\(self\.base\.id\(\)\.clone\(\), effect, is_hovered\)/);
});

test("ButtonLike hover callbacks are disabled with the button", () => {
  assert.match(buttonLike, /on_hover: Option<Box<dyn Fn\(&bool, &mut Window, &mut App\) \+ 'static>>/);
  assert.match(buttonLike, /pub fn on_hover/);
  assert.match(buttonLike, /self\.on_hover = Some\(Box::new\(handler\)\)/);
  assert.match(
    buttonLike,
    /\.when_some\(\s*self\.on_hover\.filter\(\|_\| !self\.disabled\)/,
  );
});

test("hover effect helpers are available from the UI prelude", () => {
  assert.match(prelude, /pub use crate::traits::animation_ext::\{CommonAnimationExt, IconHoverEffect\}/);
});
