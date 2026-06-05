pub(crate) mod panel;

mod panel_view;
mod rows;
mod snapshot;

use gpui::actions;

actions!(
    dx_forge,
    [
        /// Toggles the DX Forge dock panel.
        TogglePanel
    ]
);
