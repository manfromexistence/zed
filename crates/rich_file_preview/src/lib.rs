mod asset_item;
mod browser_extensions;
mod preview_view;
mod registry;
mod web_inspector;
mod webview_dev_session;
mod webview_panel;

use gpui::{App, Context, Window};
use workspace::Workspace;

pub use asset_item::PreviewAssetItem;
pub use browser_extensions::{BrowserProfile, DetectedExtension, PreviewExtensionSupport};
pub use preview_view::UniversalPreviewView;
pub use registry::{PreviewHandler, PreviewKind, preview_handler_for_extension};
pub use webview_panel::{EmbeddedWebPreviewPanel, ToggleEmbeddedWebPreview};

pub fn init(cx: &mut App) {
    workspace::register_project_item::<UniversalPreviewView>(cx);

    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, _cx: &mut Context<Workspace>| {
            let Some(_window) = window else {
                return;
            };
            EmbeddedWebPreviewPanel::register(workspace);
        },
    )
    .detach();
}
