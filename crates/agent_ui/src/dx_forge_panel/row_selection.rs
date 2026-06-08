use gpui::{Context, KeyContext, Window};
use menu::{Confirm, SelectFirst, SelectLast, SelectNext, SelectPrevious};

use super::{panel::DxForgePanel, visible_rows::DxForgeRowKey};

impl DxForgePanel {
    pub(super) fn dispatch_context(&self) -> KeyContext {
        let mut dispatch_context = KeyContext::new_with_defaults();
        dispatch_context.add("DxForgePanel");
        dispatch_context.add("menu");
        dispatch_context
    }

    fn active_visible_row_index(&self) -> Option<usize> {
        let active_item = self.active_item.as_ref()?;
        self.visible_rows
            .iter()
            .position(|row| row.item_key() == active_item)
    }

    fn active_visible_row_key(&self) -> Option<DxForgeRowKey> {
        let index = self.active_visible_row_index()?;
        self.visible_rows
            .get(index)
            .map(|row| row.item_key().clone())
    }

    fn set_active_visible_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.visible_rows.get(index) else {
            return;
        };
        let item_key = row.item_key().clone();
        if self.active_item.as_ref() != Some(&item_key) {
            self.active_item = Some(item_key.clone());
            self.scroll_item_into_view(&item_key, window, cx);
            cx.notify();
        }
    }

    fn select_first_visible_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.visible_rows.is_empty() {
            self.set_active_visible_row(0, window, cx);
        }
    }

    fn select_last_visible_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(last_index) = self.visible_rows.len().checked_sub(1) {
            self.set_active_visible_row(last_index, window, cx);
        }
    }

    pub(super) fn select_next(
        &mut self,
        _: &SelectNext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
        let Some(current_index) = self.active_visible_row_index() else {
            self.select_first_visible_row(window, cx);
            return;
        };
        let Some(next_index) = current_index.checked_add(1) else {
            return;
        };

        if next_index < self.visible_rows.len() {
            self.set_active_visible_row(next_index, window, cx);
        }
    }

    pub(super) fn select_previous(
        &mut self,
        _: &SelectPrevious,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
        let Some(current_index) = self.active_visible_row_index() else {
            self.select_last_visible_row(window, cx);
            return;
        };
        let Some(previous_index) = current_index.checked_sub(1) else {
            return;
        };

        self.set_active_visible_row(previous_index, window, cx);
    }

    pub(super) fn select_first(
        &mut self,
        _: &SelectFirst,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
        self.select_first_visible_row(window, cx);
    }

    pub(super) fn select_last(
        &mut self,
        _: &SelectLast,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
        self.select_last_visible_row(window, cx);
    }

    pub(super) fn toggle_active_item_checked(
        &mut self,
        _: &Confirm,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_panel(window, cx);
        let Some(active_item) = self.active_visible_row_key() else {
            return;
        };

        self.toggle_item_checked(active_item, cx);
    }
}
