use editor::{Editor, EditorElement, EditorStyle};
use gpui::{AnyElement, Context, Entity, IntoElement, KeyContext, TextStyle};
use theme_settings::ThemeSettings;
use ui::{IconName, prelude::*};

use crate::AgentPanel;

pub(super) fn render_catalog_search(
    id: &'static str,
    editor: &Entity<Editor>,
    cx: &mut Context<AgentPanel>,
) -> AnyElement {
    let mut key_context = KeyContext::new_with_defaults();
    key_context.add("BufferSearchBar");

    h_flex()
        .id(id)
        .key_context(key_context)
        .h_8()
        .min_w(rems_from_px(320.))
        .flex_1()
        .pl_1p5()
        .pr_2()
        .gap_2()
        .border_1()
        .border_color(cx.theme().colors().border)
        .rounded_md()
        .child(Icon::new(IconName::MagnifyingGlass).color(Color::Muted))
        .child(render_text_input(editor, cx))
        .into_any_element()
}

fn render_text_input(editor: &Entity<Editor>, cx: &mut Context<AgentPanel>) -> impl IntoElement {
    let settings = ThemeSettings::get_global(cx);
    let text_style = TextStyle {
        color: if editor.read(cx).read_only(cx) {
            cx.theme().colors().text_disabled
        } else {
            cx.theme().colors().text
        },
        font_family: settings.ui_font.family.clone(),
        font_features: settings.ui_font.features.clone(),
        font_fallbacks: settings.ui_font.fallbacks.clone(),
        font_size: rems(0.875).into(),
        font_weight: settings.ui_font.weight,
        line_height: relative(1.3),
        ..Default::default()
    };

    EditorElement::new(
        editor,
        EditorStyle {
            background: cx.theme().colors().editor_background,
            local_player: cx.theme().players().local(),
            text: text_style,
            ..Default::default()
        },
    )
}
