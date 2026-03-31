use fs::Fs;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString,
    Window,
};
use settings::{SettingsStore, update_settings_file};
use theme::{ThemeRegistry, try_parse_color};
use theme_settings::{ThemeColorsContent, ThemeStyleContent};
use ui::{Button, Color, prelude::*};
use ui_input::InputField;
use workspace::{ModalView, with_active_or_new_workspace};

pub(crate) fn open_dx_theme_editor(window: &mut Window, cx: &mut App) {
    with_active_or_new_workspace(cx, |workspace, window, cx| {
        workspace.toggle_modal(window, cx, |window, cx| DxThemeEditor::new(window, cx));
    });
}

pub(crate) struct DxThemeEditor {
    focus_handle: FocusHandle,
    target_theme_name: SharedString,
    background: Entity<InputField>,
    surface: Entity<InputField>,
    editor: Entity<InputField>,
    element: Entity<InputField>,
    accent: Entity<InputField>,
    border: Entity<InputField>,
    text: Entity<InputField>,
    text_muted: Entity<InputField>,
    status_message: Option<SharedString>,
    status_is_error: bool,
}

impl DxThemeEditor {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let colors = cx.theme().colors();
        let background = new_color_field(
            "App Background",
            color_to_hex(colors.background),
            window,
            cx,
        );
        let surface = new_color_field(
            "Surface",
            color_to_hex(colors.surface_background),
            window,
            cx,
        );
        let editor = new_color_field(
            "Editor Background",
            color_to_hex(colors.editor_background),
            window,
            cx,
        );
        let element = new_color_field(
            "Element Background",
            color_to_hex(colors.element_background),
            window,
            cx,
        );
        let accent = new_color_field("Accent", color_to_hex(colors.text_accent), window, cx);
        let border = new_color_field("Border", color_to_hex(colors.border), window, cx);
        let text = new_color_field("Text", color_to_hex(colors.text), window, cx);
        let text_muted = new_color_field("Muted Text", color_to_hex(colors.text_muted), window, cx);

        Self {
            focus_handle: background.focus_handle(cx),
            target_theme_name: cx.theme().name.clone(),
            background,
            surface,
            editor,
            element,
            accent,
            border,
            text,
            text_muted,
            status_message: Some("Live changes update the active theme immediately.".into()),
            status_is_error: false,
        }
    }

    fn apply_overrides(
        &mut self,
        _: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.build_overrides(cx) {
            Ok(style) => {
                let theme_name = self.target_theme_name.to_string();
                SettingsStore::update_global(cx, |store, cx| {
                    store.update_user_settings(cx, |settings| {
                        settings.theme.experimental_theme_overrides = None;
                        settings
                            .theme
                            .theme_overrides
                            .insert(theme_name.clone(), style.clone());
                    });
                });

                let fs = <dyn Fs>::global(cx);
                let theme_name = self.target_theme_name.to_string();
                update_settings_file(fs, cx, move |settings, _| {
                    settings.theme.experimental_theme_overrides = None;
                    settings.theme.theme_overrides.insert(theme_name, style);
                });

                self.status_message =
                    Some(format!("Updated {} instantly.", self.target_theme_name).into());
                self.status_is_error = false;
            }
            Err(error) => {
                self.status_message = Some(error.to_string().into());
                self.status_is_error = true;
            }
        }

        cx.notify();
    }

    fn reset_overrides(
        &mut self,
        _: &gpui::ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let theme_name = self.target_theme_name.to_string();
        SettingsStore::update_global(cx, |store, cx| {
            store.update_user_settings(cx, |settings| {
                settings.theme.experimental_theme_overrides = None;
                settings.theme.theme_overrides.remove(theme_name.as_str());
            });
        });

        let fs = <dyn Fs>::global(cx);
        let theme_name = self.target_theme_name.to_string();
        update_settings_file(fs, cx, move |settings, _| {
            settings.theme.experimental_theme_overrides = None;
            settings.theme.theme_overrides.remove(theme_name.as_str());
        });

        let colors = ThemeRegistry::global(cx)
            .get(self.target_theme_name.as_ref())
            .map(|theme| theme.styles.colors.clone())
            .unwrap_or_else(|_| cx.theme().colors().clone());
        set_field_text(
            &self.background,
            color_to_hex(colors.background),
            window,
            cx,
        );
        set_field_text(
            &self.surface,
            color_to_hex(colors.surface_background),
            window,
            cx,
        );
        set_field_text(
            &self.editor,
            color_to_hex(colors.editor_background),
            window,
            cx,
        );
        set_field_text(
            &self.element,
            color_to_hex(colors.element_background),
            window,
            cx,
        );
        set_field_text(&self.accent, color_to_hex(colors.text_accent), window, cx);
        set_field_text(&self.border, color_to_hex(colors.border), window, cx);
        set_field_text(&self.text, color_to_hex(colors.text), window, cx);
        set_field_text(
            &self.text_muted,
            color_to_hex(colors.text_muted),
            window,
            cx,
        );

        self.status_message =
            Some(format!("Reset overrides for {}.", self.target_theme_name).into());
        self.status_is_error = false;
        cx.notify();
    }

    fn dismiss(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn build_overrides(&self, cx: &App) -> anyhow::Result<ThemeStyleContent> {
        let background = validated_hex(self.background.read(cx).text(cx), "App Background")?;
        let surface = validated_hex(self.surface.read(cx).text(cx), "Surface")?;
        let editor = validated_hex(self.editor.read(cx).text(cx), "Editor Background")?;
        let element = validated_hex(self.element.read(cx).text(cx), "Element Background")?;
        let accent = validated_hex(self.accent.read(cx).text(cx), "Accent")?;
        let border = validated_hex(self.border.read(cx).text(cx), "Border")?;
        let text = validated_hex(self.text.read(cx).text(cx), "Text")?;
        let text_muted = validated_hex(self.text_muted.read(cx).text(cx), "Muted Text")?;

        let accent_soft = with_alpha_hex(&accent, 0.24)?;
        let accent_soft_strong = with_alpha_hex(&accent, 0.38)?;
        let border_soft = with_alpha_hex(&border, 0.45)?;

        Ok(ThemeStyleContent {
            colors: ThemeColorsContent {
                background: Some(background.clone()),
                surface_background: Some(surface.clone()),
                elevated_surface_background: Some(surface.clone()),
                element_background: Some(element.clone()),
                element_hover: Some(accent_soft.clone()),
                element_active: Some(accent_soft_strong.clone()),
                element_selected: Some(accent_soft.clone()),
                drop_target_background: Some(accent_soft.clone()),
                drop_target_border: Some(accent.clone()),
                ghost_element_hover: Some(accent_soft.clone()),
                ghost_element_active: Some(accent_soft_strong.clone()),
                ghost_element_selected: Some(accent_soft.clone()),
                text: Some(text.clone()),
                text_muted: Some(text_muted.clone()),
                text_accent: Some(accent.clone()),
                icon: Some(text.clone()),
                icon_muted: Some(text_muted.clone()),
                icon_accent: Some(accent.clone()),
                border: Some(border.clone()),
                border_variant: Some(border_soft.clone()),
                border_focused: Some(accent.clone()),
                border_selected: Some(accent.clone()),
                status_bar_background: Some(surface.clone()),
                title_bar_background: Some(surface.clone()),
                title_bar_inactive_background: Some(surface.clone()),
                toolbar_background: Some(surface.clone()),
                tab_bar_background: Some(surface.clone()),
                tab_inactive_background: Some(surface.clone()),
                tab_active_background: Some(background.clone()),
                panel_background: Some(surface.clone()),
                panel_focused_border: Some(accent.clone()),
                pane_focused_border: Some(accent.clone()),
                editor_foreground: Some(text.clone()),
                editor_background: Some(editor.clone()),
                editor_gutter_background: Some(editor.clone()),
                editor_subheader_background: Some(surface.clone()),
                editor_active_line_background: Some(accent_soft.clone()),
                editor_highlighted_line_background: Some(accent_soft.clone()),
                editor_line_number: Some(text_muted.clone()),
                editor_active_line_number: Some(text.clone()),
                editor_hover_line_number: Some(accent.clone()),
                editor_invisible: Some(text_muted.clone()),
                editor_wrap_guide: Some(border_soft.clone()),
                editor_active_wrap_guide: Some(border.clone()),
                editor_document_highlight_read_background: Some(accent_soft.clone()),
                editor_document_highlight_write_background: Some(accent_soft_strong.clone()),
                scrollbar_thumb_background: Some(border_soft.clone()),
                scrollbar_thumb_hover_background: Some(accent_soft_strong.clone()),
                scrollbar_thumb_border: Some(border_soft.clone()),
                scrollbar_track_border: Some(border_soft.clone()),
                search_match_background: Some(accent_soft.clone()),
                search_active_match_background: Some(accent_soft_strong.clone()),
                link_text_hover: Some(accent),
                ..ThemeColorsContent::default()
            },
            ..ThemeStyleContent::default()
        })
    }
}

impl Focusable for DxThemeEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for DxThemeEditor {}
impl ModalView for DxThemeEditor {}

impl Render for DxThemeEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = format!("{} Theme Editor", self.target_theme_name);
        let message_color = if self.status_is_error {
            Color::Error
        } else {
            Color::Muted
        };

        v_flex()
            .w(rems(34.))
            .gap_3()
            .p_4()
            .child(
                h_flex()
                    .justify_between()
                    .items_start()
                    .child(
                        v_flex().gap_1().child(Headline::new(title)).child(
                            Label::new(
                                "Tune the active Dx palette and watch the editor update live.",
                            )
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                        ),
                    )
                    .child(
                        Button::new("close-theme-editor", "Close")
                            .on_click(cx.listener(Self::dismiss)),
                    ),
            )
            .child(render_color_field(&self.background, cx))
            .child(render_color_field(&self.surface, cx))
            .child(render_color_field(&self.editor, cx))
            .child(render_color_field(&self.element, cx))
            .child(render_color_field(&self.accent, cx))
            .child(render_color_field(&self.border, cx))
            .child(render_color_field(&self.text, cx))
            .child(render_color_field(&self.text_muted, cx))
            .when_some(self.status_message.clone(), |this, status| {
                this.child(
                    Label::new(status)
                        .color(message_color)
                        .size(LabelSize::Small),
                )
            })
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("reset-dx-theme", "Reset")
                            .on_click(cx.listener(Self::reset_overrides)),
                    )
                    .child(
                        Button::new("apply-dx-theme", "Update Theme")
                            .on_click(cx.listener(Self::apply_overrides)),
                    ),
            )
    }
}

fn new_color_field(
    label: &'static str,
    value: String,
    window: &mut Window,
    cx: &mut Context<DxThemeEditor>,
) -> Entity<InputField> {
    let field = cx.new(|cx| InputField::new(window, cx, "#RRGGBBAA").label(label));
    field.update(cx, |field, cx| {
        field.set_text(&value, window, cx);
    });
    field
}

fn set_field_text(
    field: &Entity<InputField>,
    value: String,
    window: &mut Window,
    cx: &mut Context<DxThemeEditor>,
) {
    field.update(cx, |field, cx| {
        field.set_text(&value, window, cx);
    });
}

fn render_color_field(field: &Entity<InputField>, cx: &App) -> impl IntoElement {
    let swatch_color = field.read(cx).text(cx).trim().to_string();
    let swatch_color =
        try_parse_color(&swatch_color).unwrap_or_else(|_| cx.theme().colors().border);

    h_flex()
        .gap_2()
        .items_center()
        .child(
            div()
                .size_6()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().colors().border_variant)
                .bg(swatch_color),
        )
        .child(v_flex().w_full().child(field.clone()))
}

fn validated_hex(value: String, label: &str) -> anyhow::Result<String> {
    let value = value.trim().to_string();
    try_parse_color(&value)
        .map(|_| value)
        .map_err(|error| anyhow::anyhow!("{label} expects a valid hex color: {error}"))
}

fn with_alpha_hex(color: &str, alpha: f32) -> anyhow::Result<String> {
    let mut rgba = gpui::Rgba::try_from(color)?;
    rgba.a = alpha.clamp(0.0, 1.0);
    Ok(rgba_to_hex(rgba))
}

fn color_to_hex(color: gpui::Hsla) -> String {
    rgba_to_hex(color.into())
}

fn rgba_to_hex(color: gpui::Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        (color.a * 255.0).round() as u8
    )
}
