use std::sync::Arc;

use gpui::{Action, ClickEvent, FocusHandle, prelude::*};
use ui::{
    Chip, Disclosure, ElevationIndex, KeyBinding, ListItem, ListItemSpacing, Tooltip, prelude::*,
};
use zed_actions::agent::ToggleModelSelector;

use crate::CycleFavoriteModels;

enum ModelIcon {
    Name(IconName),
    Path(SharedString),
}

#[derive(IntoElement)]
pub struct ModelSelectorHeader {
    title: SharedString,
    has_border: bool,
}

impl ModelSelectorHeader {
    pub fn new(title: impl Into<SharedString>, has_border: bool) -> Self {
        Self {
            title: title.into(),
            has_border,
        }
    }
}

impl RenderOnce for ModelSelectorHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .px_2()
            .pb_1()
            .when(self.has_border, |this| {
                this.mt_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().colors().border_variant)
            })
            .child(
                Label::new(self.title)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
    }
}

#[derive(IntoElement)]
pub struct ModelSelectorProviderHeader {
    index: usize,
    title: SharedString,
    model_count: usize,
    is_expanded: bool,
    on_toggle: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl ModelSelectorProviderHeader {
    pub fn new(index: usize, title: impl Into<SharedString>, model_count: usize) -> Self {
        Self {
            index,
            title: title.into(),
            model_count,
            is_expanded: true,
            on_toggle: None,
        }
    }

    pub fn is_expanded(mut self, is_expanded: bool) -> Self {
        self.is_expanded = is_expanded;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ModelSelectorProviderHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let disclosure_toggle = self.on_toggle.clone();

        ListItem::new(("provider-group", self.index))
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .when_some(self.on_toggle.clone(), |this, on_toggle| {
                this.on_click(move |event, window, cx| {
                    on_toggle(event, window, cx);
                })
            })
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .gap_2()
                    .child(
                        div().flex_1().child(
                            Label::new(self.title)
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                Chip::new(self.model_count.to_string()).tooltip(Tooltip::text(
                                    format!("{} models in this provider", self.model_count),
                                )),
                            )
                            .child(
                                Disclosure::new(
                                    ("provider-disclosure", self.index),
                                    self.is_expanded,
                                )
                                .on_toggle_expanded(disclosure_toggle),
                            ),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct ModelSelectorListItem {
    index: usize,
    title: SharedString,
    icon: Option<ModelIcon>,
    is_selected: bool,
    is_focused: bool,
    is_latest: bool,
    is_favorite: bool,
    on_toggle_favorite: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    context_info: Option<SharedString>,
    cost_info: Option<SharedString>,
    badges: Vec<SharedString>,
}

impl ModelSelectorListItem {
    pub fn new(index: usize, title: impl Into<SharedString>) -> Self {
        Self {
            index,
            title: title.into(),
            icon: None,
            is_selected: false,
            is_focused: false,
            is_latest: false,
            is_favorite: false,
            on_toggle_favorite: None,
            context_info: None,
            cost_info: None,
            badges: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(ModelIcon::Name(icon));
        self
    }

    pub fn icon_path(mut self, path: SharedString) -> Self {
        self.icon = Some(ModelIcon::Path(path));
        self
    }

    pub fn is_selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    pub fn is_focused(mut self, is_focused: bool) -> Self {
        self.is_focused = is_focused;
        self
    }

    pub fn is_latest(mut self, is_latest: bool) -> Self {
        self.is_latest = is_latest;
        self
    }

    pub fn is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    pub fn on_toggle_favorite(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_favorite = Some(Box::new(handler));
        self
    }

    pub fn cost_info(mut self, cost_info: Option<SharedString>) -> Self {
        self.cost_info = cost_info;
        self
    }

    pub fn context_info(mut self, context_info: Option<SharedString>) -> Self {
        self.context_info = context_info;
        self
    }

    pub fn badges(mut self, badges: Vec<SharedString>) -> Self {
        self.badges = badges;
        self
    }
}

impl RenderOnce for ModelSelectorListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let model_icon_color = if self.is_selected {
            Color::Accent
        } else {
            Color::Muted
        };

        let is_favorite = self.is_favorite;

        ListItem::new(self.index)
            .inset(true)
            .spacing(ListItemSpacing::Sparse)
            .toggle_state(self.is_focused)
            .child(
                h_flex()
                    .w_full()
                    .gap_1p5()
                    .when_some(self.icon, |this, icon| {
                        this.child(
                            match icon {
                                ModelIcon::Name(icon_name) => Icon::new(icon_name),
                                ModelIcon::Path(icon_path) => Icon::from_external_svg(icon_path),
                            }
                            .color(model_icon_color)
                            .size(IconSize::Small),
                        )
                    })
                    .child(Label::new(self.title).truncate())
                    .when(self.is_latest, |parent| parent.child(Chip::new("Latest")))
                    .when_some(self.cost_info, |this, cost_info| {
                        let tooltip_text = if cost_info.ends_with('×') {
                            format!("Cost Multiplier: {}", cost_info)
                        } else if cost_info.contains('$') {
                            format!("Cost per Million Tokens: {}", cost_info)
                        } else {
                            format!("Cost: {}", cost_info)
                        };

                        this.child(Chip::new(cost_info).tooltip(Tooltip::text(tooltip_text)))
                    })
                    .when_some(self.context_info, |this, context_info| {
                        this.child(
                            Chip::new(context_info).tooltip(Tooltip::text("Model context window")),
                        )
                    })
                    .children(
                        self.badges
                            .into_iter()
                            .map(|badge| Chip::new(badge).into_any_element()),
                    ),
            )
            .end_slot(div().pr_2().when(self.is_selected, |this| {
                this.child(Icon::new(IconName::Check).color(Color::Accent))
            }))
            .end_hover_slot(div().pr_1p5().when_some(self.on_toggle_favorite, {
                |this, handle_click| {
                    let (icon, color, tooltip) = if is_favorite {
                        (IconName::StarFilled, Color::Accent, "Unfavorite Model")
                    } else {
                        (IconName::Star, Color::Default, "Favorite Model")
                    };
                    this.child(
                        IconButton::new(("toggle-favorite", self.index), icon)
                            .layer(ElevationIndex::ElevatedSurface)
                            .icon_color(color)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::text(tooltip))
                            .on_click(move |event, window, cx| (handle_click)(event, window, cx)),
                    )
                }
            }))
    }
}

#[derive(IntoElement)]
pub struct ModelSelectorFooter {
    action: Box<dyn Action>,
    focus_handle: FocusHandle,
    status_text: Option<SharedString>,
}

impl ModelSelectorFooter {
    pub fn new(action: Box<dyn Action>, focus_handle: FocusHandle) -> Self {
        Self {
            action,
            focus_handle,
            status_text: None,
        }
    }

    pub fn status_text(mut self, status_text: Option<SharedString>) -> Self {
        self.status_text = status_text;
        self
    }
}

impl RenderOnce for ModelSelectorFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let action = self.action;
        let focus_handle = self.focus_handle;

        h_flex()
            .w_full()
            .p_1p5()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().colors().border_variant)
            .when_some(self.status_text, |this, status_text| {
                this.child(
                    Label::new(status_text)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
            })
            .child(
                Button::new("configure", "Configure")
                    .full_width()
                    .style(ButtonStyle::Outlined)
                    .key_binding(
                        KeyBinding::for_action_in(action.as_ref(), &focus_handle, cx)
                            .map(|kb| kb.size(rems_from_px(12.))),
                    )
                    .on_click(move |_, window, cx| {
                        window.dispatch_action(action.boxed_clone(), cx);
                    }),
            )
    }
}

#[derive(IntoElement)]
pub struct ModelSelectorTooltip {
    show_cycle_row: bool,
}

impl ModelSelectorTooltip {
    pub fn new() -> Self {
        Self {
            show_cycle_row: true,
        }
    }

    pub fn show_cycle_row(mut self, show: bool) -> Self {
        self.show_cycle_row = show;
        self
    }
}

impl RenderOnce for ModelSelectorTooltip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_2()
                    .justify_between()
                    .child(Label::new("Change Model"))
                    .child(KeyBinding::for_action(&ToggleModelSelector, cx)),
            )
            .when(self.show_cycle_row, |this| {
                this.child(
                    h_flex()
                        .pt_1()
                        .gap_2()
                        .border_t_1()
                        .border_color(cx.theme().colors().border_variant)
                        .justify_between()
                        .child(Label::new("Cycle Favorited Models"))
                        .child(KeyBinding::for_action(&CycleFavoriteModels, cx)),
                )
            })
    }
}
