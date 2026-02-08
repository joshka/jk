//! Command planning and prompt-token translation.
//!
//! This layer decides whether input executes immediately, opens a prompt, renders a local view, or
//! updates status/quit state.

mod builders;
mod planner;
mod prompt_kind;

pub use planner::plan_command;
pub use prompt_kind::{PromptKind, PromptRequest};

/// Planned runtime action produced by command parsing or prompt submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowAction {
    /// Execute one `jj` command token sequence.
    Execute(Vec<String>),
    /// Render local in-app lines without invoking `jj`.
    Render { lines: Vec<String>, status: String },
    /// Enter prompt mode to collect additional command arguments.
    Prompt(PromptRequest),
    /// Update footer status without changing primary content.
    Status(String),
    /// Request application shutdown.
    Quit,
}

#[cfg(test)]
mod tests;
