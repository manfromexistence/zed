use agent_settings::AgentLiquidGlassSettings;
use gpui::{AnyElement, IntoElement, div};
use liquid_glass::{
    LiquidGlassStyle, bounded_liquid_glass_layer, load_liquid_glass_backdrop_carrier,
};
use ui::prelude::*;

pub(super) fn render_agent_liquid_glass_chat_input_surface(
    settings: &AgentLiquidGlassSettings,
) -> AnyElement {
    div()
        .id("agent-liquid-glass-chat-input-surface")
        .absolute()
        .inset_0()
        .overflow_hidden()
        .child(bounded_liquid_glass_layer(
            load_liquid_glass_backdrop_carrier(),
            agent_liquid_glass_style_from_settings(settings),
        ))
        .into_any_element()
}

pub(super) fn agent_liquid_glass_style_from_settings(
    settings: &AgentLiquidGlassSettings,
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
