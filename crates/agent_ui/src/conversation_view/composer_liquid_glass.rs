use std::sync::Arc;

use gpui::{AnyElement, App, Hsla, IntoElement, RenderImage, canvas};
use liquid_glass::{control_surface_liquid_glass_style, paint_liquid_glass_layer};
use ui::{prelude::*, theme_is_transparent, utils::apca_contrast};

const MIN_READABILITY_CONTRAST: f32 = 45.0;
const READABILITY_SAMPLE_EDITOR_ALPHA: f32 = 0.72;
const FALLBACK_BACKGROUND_EDITOR_ALPHA: f32 = 0.86;
const TRANSPARENT_FALLBACK_BACKGROUND_ALPHA: f32 = 0.56;
const OPAQUE_FALLBACK_BACKGROUND_ALPHA: f32 = 0.24;
const TRANSPARENT_READABILITY_OVERLAY_ALPHA: f32 = 0.58;
const OPAQUE_READABILITY_OVERLAY_ALPHA: f32 = 0.42;
const TRANSPARENT_BORDER_ALPHA: f32 = 0.9;
const OPAQUE_BORDER_ALPHA: f32 = 0.68;

pub(super) struct ComposerGlassSurfaceStyle {
    pub(super) background: Hsla,
    pub(super) border: Hsla,
    pub(super) readability_overlay: Option<Hsla>,
}

pub(super) fn composer_glass_surface_style(cx: &mut App) -> ComposerGlassSurfaceStyle {
    let is_transparent = theme_is_transparent(cx);
    let colors = cx.theme().colors();
    let transparent = cx.theme().system().transparent;
    let panel_background = colors.panel_background;
    let editor_background = colors.editor_background;
    let text = colors.text;
    let border = colors.border;

    let readability_base =
        panel_background.blend(editor_background.opacity(READABILITY_SAMPLE_EDITOR_ALPHA));
    let needs_readability_fallback =
        needs_readability_fallback(is_transparent, text, readability_base);

    ComposerGlassSurfaceStyle {
        background: if needs_readability_fallback {
            let base =
                panel_background.blend(editor_background.opacity(FALLBACK_BACKGROUND_EDITOR_ALPHA));

            if is_transparent {
                base.opacity(TRANSPARENT_FALLBACK_BACKGROUND_ALPHA)
            } else {
                base.opacity(OPAQUE_FALLBACK_BACKGROUND_ALPHA)
            }
        } else {
            transparent
        },
        border: if is_transparent {
            border.opacity(TRANSPARENT_BORDER_ALPHA)
        } else {
            border.opacity(OPAQUE_BORDER_ALPHA)
        },
        readability_overlay: needs_readability_fallback.then(|| {
            if is_transparent {
                readability_base.opacity(TRANSPARENT_READABILITY_OVERLAY_ALPHA)
            } else {
                readability_base.opacity(OPAQUE_READABILITY_OVERLAY_ALPHA)
            }
        }),
    }
}

fn needs_readability_fallback(
    is_transparent: bool,
    text: Hsla,
    readability_base: Hsla,
) -> bool {
    is_transparent
        || apca_contrast(text, readability_base).abs() < MIN_READABILITY_CONTRAST
}

pub(super) fn render_composer_liquid_glass_layer(source_image: Arc<RenderImage>) -> AnyElement {
    let style = control_surface_liquid_glass_style();

    canvas(
        move |bounds, _, _| bounds,
        move |bounds, _, window, _cx| {
            paint_liquid_glass_layer(window, bounds, bounds, source_image.clone(), &style);
        },
    )
    .absolute()
    .inset_0()
    .size_full()
    .into_any_element()
}
