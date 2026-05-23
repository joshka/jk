//! Generated help projection policy.
//!
//! This module decides which commands are shown in each view's help overlay and
//! how those commands are grouped. Key binding matching stays in `command.rs`;
//! help projection only consumes the binding vocabulary.

mod metadata;
mod model;
mod projection;

pub use model::{HelpContext, HelpRow, HelpSection, HelpSectionKind};
pub use projection::{command_is_visible_in_help, project_help};

#[cfg(test)]
mod tests;
