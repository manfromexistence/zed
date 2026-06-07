use gpui::{
    App, Background, ColorSpace, Hsla, LinearColorStop, linear_color_stop, linear_gradient,
};
use theme::ActiveTheme;

use super::color::Color;

pub const UI_LINEAR_GRADIENT_COLOR_SPACE: ColorSpace = ColorSpace::Oklab;

const GRADIENT_STOP_MIN: f32 = 0.0;
const GRADIENT_STOP_MAX: f32 = 1.0;
const DEFAULT_SURFACE_GRADIENT_ANGLE: f32 = 135.0;
const ACCENT_WASH_START_OPACITY: f32 = 0.16;
const ACCENT_WASH_END_OPACITY: f32 = 0.03;

/// Builds a clamped linear color stop for GPUI surface gradients.
pub fn linear_gradient_stop(color: impl Into<Hsla>, percentage: f32) -> LinearColorStop {
    linear_color_stop(
        color,
        percentage.clamp(GRADIENT_STOP_MIN, GRADIENT_STOP_MAX),
    )
}

/// Builds a two-stop GPUI linear gradient using Oklab interpolation.
pub fn oklab_linear_gradient(angle: f32, from: impl Into<Hsla>, to: impl Into<Hsla>) -> Background {
    oklab_linear_gradient_stops(
        angle,
        linear_gradient_stop(from, GRADIENT_STOP_MIN),
        linear_gradient_stop(to, GRADIENT_STOP_MAX),
    )
}

/// Builds a two-stop GPUI linear gradient from explicit, clamped stops.
pub fn oklab_linear_gradient_stops(
    angle: f32,
    from: LinearColorStop,
    to: LinearColorStop,
) -> Background {
    linear_gradient(
        normalized_linear_gradient_angle(angle),
        clamped_linear_color_stop(from),
        clamped_linear_color_stop(to),
    )
    .color_space(UI_LINEAR_GRADIENT_COLOR_SPACE)
}

/// Builds a semantic theme gradient from Zed UI colors.
pub fn theme_linear_gradient(cx: &App, angle: f32, from: Color, to: Color) -> Background {
    oklab_linear_gradient(angle, from.color(cx), to.color(cx))
}

/// Builds a subtle theme-derived panel gradient without adding render-time work.
pub fn panel_surface_gradient(cx: &App) -> Background {
    let colors = cx.theme().colors();
    oklab_linear_gradient(180.0, colors.panel_background, colors.surface_background)
}

/// Builds a subtle accent wash over the active surface background.
pub fn accent_surface_gradient(cx: &App, accent: Color) -> Background {
    accent_wash_gradient(
        cx.theme().colors().surface_background,
        accent.color(cx),
        DEFAULT_SURFACE_GRADIENT_ANGLE,
    )
}

/// Builds an accent wash from caller-provided theme colors.
pub fn accent_wash_gradient(surface: Hsla, accent: Hsla, angle: f32) -> Background {
    oklab_linear_gradient(
        angle,
        surface.blend(accent.opacity(ACCENT_WASH_START_OPACITY)),
        surface.blend(accent.opacity(ACCENT_WASH_END_OPACITY)),
    )
}

pub fn normalized_linear_gradient_angle(angle: f32) -> f32 {
    if angle.is_finite() {
        angle.rem_euclid(360.0)
    } else {
        0.0
    }
}

fn clamped_linear_color_stop(stop: LinearColorStop) -> LinearColorStop {
    linear_gradient_stop(stop.color, stop.percentage)
}
