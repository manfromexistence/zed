use anyhow::Result as AnyhowResult;
use gpui::{
    DismissEvent, EventEmitter, FocusHandle, Focusable, Render, ScrollHandle, TaskExt, WeakEntity,
};
use ui::{
    Banner, KeyBinding, ListItem, ListItemSpacing, Modal, ModalFooter, ModalHeader, Section,
    Severity, WithScrollbar, prelude::*,
};
use workspace::ModalView;

use crate::AgentPanel;
use crate::dx_agent_bridge::DxWorkflowNodeSummary;
use crate::workflow_node_icons::workflow_node_icon_asset_for;

mod fields;
mod keychain;

use fields::{DxPluginCredentialInputField, clear_secret_input_fields, credential_input_fields};
pub(crate) use keychain::credential_storage_unavailable_reason;
use keychain::{
    credentials_saved_prompt, dx_plugin_credential_keychain_url, save_plugin_credentials,
};

pub(crate) struct DxPluginCredentialModal {
    node: DxWorkflowNodeSummary,
    panel: WeakEntity<AgentPanel>,
    inputs: Vec<DxPluginCredentialInputField>,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    last_error: Option<SharedString>,
    saving: bool,
}

impl DxPluginCredentialModal {
    pub(crate) fn new(
        node: DxWorkflowNodeSummary,
        panel: WeakEntity<AgentPanel>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            inputs: credential_input_fields(&node, window, cx, dx_plugin_credential_keychain_url),
            node,
            panel,
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            last_error: None,
            saving: false,
        }
    }

    fn confirm(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }

        let task = save_plugin_credentials(&self.node, &self.inputs, cx);
        self.saving = true;
        self.last_error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            this.update_in(cx, |this, window, cx| {
                this.saving = false;
                match result {
                    Ok(handles) => {
                        let prompt = credentials_saved_prompt(&this.node, &handles);
                        this.clear_secret_inputs(window, cx);
                        if let Some(panel) = this.panel.upgrade() {
                            panel.update(cx, |panel, cx| {
                                panel.draft_dx_plugin_credentials_saved_prompt(
                                    prompt.clone(),
                                    window,
                                    cx,
                                );
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                    Err(error) => {
                        this.last_error = Some(error);
                        cx.notify();
                    }
                }
            })?;
            AnyhowResult::<()>::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn cancel(&mut self, _: &menu::Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_secret_inputs(window, cx);
        cx.emit(DismissEvent);
    }

    fn clear_secret_inputs(&self, window: &mut Window, cx: &mut Context<Self>) {
        clear_secret_input_fields(&self.inputs, window, cx);
    }

    fn can_submit(&self, cx: &App) -> bool {
        !self.saving
            && self
                .inputs
                .iter()
                .any(|field| !field.input.read(cx).is_empty(cx))
    }

    fn render_input_section(&self, _cx: &mut Context<Self>) -> AnyElement {
        if self.inputs.is_empty() {
            return ListItem::new("dx-plugin-credential-modal-no-inputs")
                .spacing(ListItemSpacing::Dense)
                .selectable(false)
                .start_slot(
                    Icon::new(dx_icon(DxUiIcon::Credentials))
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    Label::new("No credential inputs were declared for this plugin.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .into_any_element();
        }

        v_flex()
            .gap_2()
            .children(self.inputs.iter().map(|field| field.input.clone()))
            .into_any_element()
    }

    fn on_tab(&mut self, _: &menu::SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(
        &mut self,
        _: &menu::SelectPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus_prev(cx);
    }
}

impl EventEmitter<DismissEvent> for DxPluginCredentialModal {}

impl Focusable for DxPluginCredentialModal {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for DxPluginCredentialModal {}

impl Render for DxPluginCredentialModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);
        let icon = workflow_node_icon_asset_for(
            self.node.icon.as_deref(),
            Some(self.node.category.as_str()),
            self.node.display_name.as_str(),
        );

        v_flex()
            .id("dx-plugin-credential-modal")
            .key_context("DxPluginCredentialModal")
            .w(rems(36.))
            .elevation_3(cx)
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .capture_any_mouse_down(cx.listener(|this, _, window, cx| {
                this.focus_handle(cx).focus(window, cx);
            }))
            .child(
                Modal::new("dx-plugin-credential-modal", None::<ScrollHandle>)
                    .header(
                        ModalHeader::new()
                            .headline("Configure Plugin Credentials")
                            .description("Values are saved to the OS keychain; Agent receives a save summary only."),
                    )
                    .section(Section::new().child(
                        h_flex()
                            .gap_2()
                            .child(icon.render(IconSize::Medium, Color::Default))
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(
                                        Label::new(self.node.display_name.clone())
                                            .size(LabelSize::Small)
                                            .color(Color::Default),
                                    )
                                    .child(
                                        Label::new(format!(
                                            "{} / {}",
                                            self.node.source_package, self.node.source_path
                                        ))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted)
                                        .truncate(),
                                    ),
                            ),
                    ))
                    .section(Section::new().child(
                        Banner::new().severity(Severity::Info).child(
                            div()
                                .text_xs()
                                .child("Stored values and keychain item names are never written to settings, receipts, logs, or prompts."),
                        ),
                    ))
                    .when_some(self.last_error.clone(), |this, error| {
                        this.section(Section::new().child(
                            Banner::new()
                                .severity(Severity::Warning)
                                .child(div().text_xs().child(error)),
                        ))
                    })
                    .child(
                        div()
                            .size_full()
                            .vertical_scrollbar_for(&self.scroll_handle, window, cx)
                            .child(
                                v_flex()
                                    .id("dx-plugin-credential-modal-content")
                                    .size_full()
                                    .tab_group()
                                    .max_h(rems_from_px(380.))
                                    .pl_3()
                                    .pr_4()
                                    .pb_2()
                                    .gap_2()
                                    .overflow_y_scroll()
                                    .track_scroll(&self.scroll_handle)
                                    .child(self.render_input_section(cx)),
                            ),
                    )
                    .footer(ModalFooter::new().end_slot(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("dx-plugin-credential-cancel", "Cancel")
                                    .key_binding(
                                        KeyBinding::for_action_in(
                                            &menu::Cancel,
                                            &focus_handle,
                                            cx,
                                        )
                                        .map(|kb| kb.size(rems_from_px(12.))),
                                    )
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.cancel(&menu::Cancel, window, cx)
                                    })),
                            )
                            .child(
                                Button::new("dx-plugin-credential-save", "Save Credentials")
                                    .key_binding(
                                        KeyBinding::for_action_in(
                                            &menu::Confirm,
                                            &focus_handle,
                                            cx,
                                        )
                                        .map(|kb| kb.size(rems_from_px(12.))),
                                    )
                                    .disabled(!self.can_submit(cx))
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.confirm(&menu::Confirm, window, cx)
                                    })),
                            ),
                    )),
            )
    }
}
