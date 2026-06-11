use editor::{Editor, EditorElement, EditorStyle};
use gpui::{AnyElement, Context, Entity, IntoElement, KeyContext, TextStyle};
use settings::Settings;
use theme_settings::ThemeSettings;
use ui::{AiSettingItemStatus, Chip, IconName, prelude::*};

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

pub(super) fn render_catalog_row_labels(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
) -> AnyElement {
    v_flex()
        .min_w_0()
        .gap_0p5()
        .child(
            Label::new(title.into())
                .size(LabelSize::Default)
                .color(Color::Default)
                .truncate(),
        )
        .child(
            Label::new(detail.into())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .truncate(),
        )
        .into_any_element()
}

pub(super) fn render_catalog_status_chip(status: AiSettingItemStatus) -> AnyElement {
    let (label, color, icon) = match status {
        AiSettingItemStatus::Running => ("Ready", Color::Success, IconName::Check),
        AiSettingItemStatus::Error => ("Error", Color::Error, IconName::Warning),
        AiSettingItemStatus::AuthRequired => ("Needs auth", Color::Warning, IconName::LockOutlined),
        AiSettingItemStatus::ClientSecretRequired => {
            ("Needs secret", Color::Warning, IconName::LockOutlined)
        }
        AiSettingItemStatus::Starting | AiSettingItemStatus::Authenticating => {
            ("Pending", Color::Muted, IconName::Clock)
        }
        AiSettingItemStatus::Stopped => ("Inactive", Color::Muted, IconName::FileTextOutlined),
    };

    Chip::new(label)
        .label_color(color)
        .icon(icon)
        .icon_color(color)
        .truncate()
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
