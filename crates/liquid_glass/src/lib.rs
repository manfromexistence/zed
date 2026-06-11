mod backgrounds;
mod element;
mod ui_state;

use gpui::App;

pub use backgrounds::{
    BackgroundAsset, load_backgrounds, load_glass_surface, load_liquid_glass_backdrop_carrier,
};
pub use element::{
    LiquidGlassStyle, bounded_liquid_glass_layer, liquid_glass_layer, paint_liquid_glass_layer,
};
pub use ui_state::{GLASS_VARIANTS, UiState};

pub fn default_liquid_glass_style() -> LiquidGlassStyle {
    let state = ui_state::UiState::default();

    liquid_glass_style_from_state(&state)
}

pub fn control_surface_liquid_glass_style() -> LiquidGlassStyle {
    LiquidGlassStyle {
        power_factor: 3.0,
        a: 0.7,
        b: 2.3,
        c: 5.2,
        d: 6.9,
        f_power: 1.0,
        noise: 0.06,
        glow_weight: 0.25,
        glow_edge0: 0.5,
        glow_edge1: -0.5,
        glow_bias: 0.0,
        chromatic_aberration: 0.008,
        aberration_samples: 5,
        blur_radius: 2.0,
        blur_iterations: 1,
        blur_downscale: 0.5,
    }
}

pub fn liquid_glass_style_from_settings(
    settings: &agent_settings::AgentLiquidGlassSettings,
) -> LiquidGlassStyle {
    LiquidGlassStyle {
        power_factor: settings.power_factor,
        a: settings.a,
        b: settings.b,
        c: settings.c,
        d: settings.d,
        f_power: settings.f_power,
        noise: settings.noise,
        glow_weight: settings.glow_weight,
        glow_edge0: settings.glow_edge0,
        glow_edge1: settings.glow_edge1,
        glow_bias: settings.glow_bias,
        chromatic_aberration: settings.chromatic_aberration,
        aberration_samples: settings.aberration_samples,
        blur_radius: settings.blur_radius,
        blur_iterations: settings.blur_iterations,
        blur_downscale: settings.blur_downscale,
    }
}

fn liquid_glass_style_from_state(state: &ui_state::UiState) -> LiquidGlassStyle {
    LiquidGlassStyle {
        power_factor: state.power_factor,
        a: state.a,
        b: state.b,
        c: state.c,
        d: state.d,
        f_power: state.f_power,
        noise: state.noise,
        glow_weight: state.glow_weight,
        glow_edge0: state.glow_edge0,
        glow_edge1: state.glow_edge1,
        glow_bias: state.glow_bias,
        chromatic_aberration: state.chromatic_aberration,
        aberration_samples: state.aberration_samples,
        blur_radius: state.blur_radius,
        blur_iterations: state.blur_iterations,
        blur_downscale: state.blur_downscale,
    }
}

pub fn init(_cx: &mut App) {}
