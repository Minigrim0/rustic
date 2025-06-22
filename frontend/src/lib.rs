//! Rustic Frontend library
//!
//! This module exports the components needed for the Rustic frontend application.

pub mod mapping;
pub mod tabs;
pub mod widgets;

pub use tabs::Tab;
pub use widgets::prelude::*;

/// Re-export of common modules used by the frontend
pub mod prelude {
    pub use super::mapping::*;
    pub use super::tabs::*;
    pub use super::widgets::prelude::*;
}
