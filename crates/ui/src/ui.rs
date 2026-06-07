//! # UI – Zed UI Primitives & Components
//!
//! This crate provides a set of UI primitives and components that are used to build all of the elements in Zed's UI.
//!
//! ## Related Crates:
//!
//! - [`ui_macros`] - proc_macros support for this crate
//! - `ui_input` - the single line input component

pub mod component_prelude;
mod components;
mod dx_icon_paths;
mod dx_icons;
pub mod prelude;
mod styles;
mod traits;
pub mod utils;

pub use components::*;
pub use dx_icon_paths::*;
pub use dx_icons::*;
pub use prelude::*;
pub use styles::*;
pub use traits::animation_ext::*;
