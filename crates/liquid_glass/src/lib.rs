mod backgrounds;
mod element;
mod liquid_glass_view;
mod ui_state;

use gpui::App;
use workspace::Workspace;

pub use backgrounds::load_glass_surface;
pub use element::{LiquidGlassStyle, liquid_glass_layer, paint_liquid_glass_layer};
pub use liquid_glass_view::LiquidGlassView;

pub fn default_liquid_glass_style() -> LiquidGlassStyle {
    let state = ui_state::UiState::default();

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

pub fn init(cx: &mut App) {
    workspace::register_serializable_item::<LiquidGlassView>(cx);

    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };

        LiquidGlassView::register(workspace, window, cx);
    })
    .detach();
}
