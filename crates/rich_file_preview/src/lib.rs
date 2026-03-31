mod asset_item;
mod browser_extensions;
mod preview_view;
mod registry;
mod web_inspector;
mod webview_dev_session;
mod webview_panel;

use gpui::{App, AppContext as _, Context, Window};
use workspace::Workspace;

pub use asset_item::PreviewAssetItem;
pub use browser_extensions::{BrowserProfile, DetectedExtension, PreviewExtensionSupport};
pub use preview_view::UniversalPreviewView;
pub use registry::{PreviewHandler, PreviewKind, preview_handler_for_extension};
pub use webview_panel::EmbeddedWebPreviewPanel;

pub fn init(cx: &mut App) {
    workspace::register_project_item::<UniversalPreviewView>(cx);

    cx.observe_new(
        |workspace: &mut Workspace, window: Option<&mut Window>, cx: &mut Context<Workspace>| {
            let Some(window) = window else {
                return;
            };
            EmbeddedWebPreviewPanel::register(workspace);

            if workspace.panel::<EmbeddedWebPreviewPanel>(cx).is_none() {
                let panel =
                    cx.new(|cx| EmbeddedWebPreviewPanel::new(workspace.weak_handle(), window, cx));
                workspace.add_panel(panel, window, cx);
            }
        },
    )
    .detach();
}
