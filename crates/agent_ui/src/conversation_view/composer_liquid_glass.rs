use std::sync::Arc;

use gpui::{AnyElement, App, Hsla, RenderImage};
use liquid_glass::{bounded_liquid_glass_layer, control_surface_liquid_glass_style};
use ui::{theme_is_transparent, utils::apca_contrast};

const MIN_TEXT_READABILITY_CONTRAST: f32 = 45.0;
const MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST: f32 = 30.0;
const READABILITY_SAMPLE_EDITOR_ALPHA: f32 = 0.72;
const READABILITY_PLATE_EDITOR_ALPHA: f32 = 0.82;
const READABILITY_PLATE_ALPHA: f32 = 0.62;
const TRANSPARENT_READABILITY_VEIL_ALPHA: f32 = 0.18;
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
    let text_muted = colors.text_muted;
    let text_placeholder = colors.text_placeholder;
    let icon = colors.icon;
    let icon_muted = colors.icon_muted;
    let border = colors.border;

    let readability_base =
        panel_background.blend(editor_background.opacity(READABILITY_SAMPLE_EDITOR_ALPHA));
    let needs_readability_plate = needs_readability_plate(
        text,
        text_muted,
        text_placeholder,
        icon,
        icon_muted,
        readability_base,
    );

    ComposerGlassSurfaceStyle {
        background: transparent,
        border: if is_transparent {
            border.opacity(TRANSPARENT_BORDER_ALPHA)
        } else {
            border.opacity(OPAQUE_BORDER_ALPHA)
        },
        readability_overlay: if needs_readability_plate {
            let plate =
                panel_background.blend(editor_background.opacity(READABILITY_PLATE_EDITOR_ALPHA));

            Some(plate.opacity(READABILITY_PLATE_ALPHA))
        } else {
            is_transparent.then(|| readability_base.opacity(TRANSPARENT_READABILITY_VEIL_ALPHA))
        },
    }
}

fn needs_readability_plate(
    text: Hsla,
    text_muted: Hsla,
    text_placeholder: Hsla,
    icon: Hsla,
    icon_muted: Hsla,
    readability_base: Hsla,
) -> bool {
    apca_contrast(text, readability_base).abs() < MIN_TEXT_READABILITY_CONTRAST
        || apca_contrast(text_muted, readability_base).abs()
            < MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST
        || apca_contrast(text_placeholder, readability_base).abs()
            < MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST
        || apca_contrast(icon, readability_base).abs()
            < MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST
        || apca_contrast(icon_muted, readability_base).abs()
            < MIN_SUPPORTING_CONTENT_READABILITY_CONTRAST
}

pub(super) fn render_composer_liquid_glass_layer(source_image: Arc<RenderImage>) -> AnyElement {
    bounded_liquid_glass_layer(source_image, control_surface_liquid_glass_style())
}
