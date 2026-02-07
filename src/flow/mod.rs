mod builders;
mod planner;
mod prompt_kind;

pub use planner::plan_command;
pub use prompt_kind::{PromptKind, PromptRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowAction {
    Execute(Vec<String>),
    Render { lines: Vec<String>, status: String },
    Prompt(PromptRequest),
    Status(String),
    Quit,
}

#[cfg(test)]
mod tests;
