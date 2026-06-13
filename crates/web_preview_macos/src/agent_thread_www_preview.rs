use agent_ui::agent_thread_www_preview::{
    AgentThreadWwwPreviewHooks, register_agent_thread_www_preview_hooks,
};
use gpui::{AnyElement, AnyEntity, App, WeakEntity, Window};
use ui::prelude::*;
use workspace::{Item, Workspace};

use crate::web_preview_view::WebPreviewView;

pub fn register_hooks() {
    register_agent_thread_www_preview_hooks(AgentThreadWwwPreviewHooks {
        open_url: open_agent_thread_www_preview,
        deactivate: deactivate_agent_thread_www_preview,
        render: render_agent_thread_www_preview,
    });
}

fn open_agent_thread_www_preview(
    workspace: WeakEntity<Workspace>,
    url: String,
    existing: Option<AnyEntity>,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyEntity> {
    if let Some(existing) = existing {
        if let Ok(preview) = existing.clone().downcast::<WebPreviewView>() {
            preview.update(cx, |preview, cx| {
                preview.load_agent_thread_url(&url, window, cx);
            });
            return Some(existing);
        }
    }

    let preview = cx.new(|cx| WebPreviewView::new_for_agent_thread(workspace, url, window, cx));
    Some(preview.into())
}

fn deactivate_agent_thread_www_preview(preview: &AnyEntity, window: &mut Window, cx: &mut App) {
    if let Ok(preview) = preview.clone().downcast::<WebPreviewView>() {
        preview.update(cx, |preview, cx| preview.deactivated(window, cx));
    }
}

fn render_agent_thread_www_preview(
    preview: &AnyEntity,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    preview
        .clone()
        .downcast::<WebPreviewView>()
        .map(|preview| div().size_full().child(preview).into_any_element())
        .unwrap_or_else(|_| div().size_full().into_any_element())
}
