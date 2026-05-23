use super::{InteractionMode, view_menu_options};
use crate::app::status_line::StatusLine;
use crate::command::{Binding, project_help};
use crate::tui::Overlay;
use crate::view_state::ViewState;

impl InteractionMode {
    /// Projects transient prompt state into the status line for the current draw.
    ///
    /// Only prompt-like modes override the stored status. Durable ready/error status belongs to
    /// `status_line`, and accepting, cancelling, or mutating prompt input belongs to app dispatch.
    /// This method should stay side-effect free so lifecycle code can ask what the screen would
    /// show without changing the active mode.
    pub fn status_line(&self, view: &ViewState, stored_status: &StatusLine) -> StatusLine {
        match self {
            Self::SearchPrompt(input) => StatusLine::with_message(view, format!("/{input}")),
            Self::LogRevsetPrompt(input) => {
                StatusLine::with_message(view, format!("revset: {input}"))
            }
            Self::DescribePrompt { target, input } => {
                StatusLine::with_message(view, format!("describe {}: {input}", target.label()))
            }
            Self::CommitPrompt(input) => {
                StatusLine::with_message(view, format!("commit @: {input}"))
            }
            Self::BookmarkNamePrompt {
                kind,
                target,
                input,
            } => StatusLine::with_message(
                view,
                format!("bookmark {} {}: {input}", kind.label(), target.label()),
            ),
            Self::BookmarkRenamePrompt { old_name, input } => {
                StatusLine::with_message(view, format!("bookmark rename {old_name}: {input}"))
            }
            Self::AbandonConfirm { input, .. } => StatusLine::with_message(
                view,
                format!("type exact revision to confirm abandon: {input}"),
            ),
            _ => stored_status.clone(),
        }
    }

    /// Borrows the active screen state as the shared TUI overlay model.
    ///
    /// This is a projection boundary only: the returned `Overlay` borrows menu options, prompts,
    /// and action output owned by `InteractionMode` or static tables. Overlay chrome and layout
    /// belong to `tui`, while key handling, command execution, and mode transitions stay in app
    /// dispatch.
    pub fn overlay<'a>(
        &'a self,
        view: &'a ViewState,
        app_bindings: &'static [Binding],
    ) -> Overlay<'a> {
        match self {
            Self::Help => Overlay::Help {
                sections: project_help(app_bindings, view.bindings(), view.help_context()),
            },
            Self::CopyMenu { options, selected } => Overlay::CopyMenu {
                options,
                selected: *selected,
            },
            Self::ViewMenu { selected } => Overlay::ViewMenu {
                options: view_menu_options(),
                selected: *selected,
            },
            Self::ActionMenu { menu, selected } => Overlay::ActionMenu {
                menu,
                selected: *selected,
            },
            Self::RolePrompt {
                prompt, selected, ..
            } => Overlay::RolePrompt {
                prompt,
                selected: *selected,
            },
            Self::DescribePreview { output, .. } => Overlay::ActionPane {
                title: "Describe",
                output,
            },
            Self::CommitPreview { output, .. } => Overlay::ActionPane {
                title: "Commit",
                output,
            },
            Self::BookmarkMutationPreview { output, .. } => Overlay::ActionPane {
                title: "Bookmark",
                output,
            },
            Self::FileMutationPreview { output, .. } => Overlay::ActionPane {
                title: "File",
                output,
            },
            Self::NewPreview { output, .. } => Overlay::ActionPane {
                title: "New change",
                output,
            },
            Self::DuplicatePreview { output, .. } => Overlay::ActionPane {
                title: "Duplicate",
                output,
            },
            Self::RebasePreview { output, .. } => Overlay::ActionPane {
                title: "Rebase",
                output,
            },
            Self::SplitPreview { output, .. } => Overlay::ActionPane {
                title: "Split",
                output,
            },
            Self::RestorePreview { output, .. } => Overlay::ActionPane {
                title: "Restore",
                output,
            },
            Self::RevertPreview { output, .. } => Overlay::ActionPane {
                title: "Revert",
                output,
            },
            Self::SquashPreview { output, .. } => Overlay::ActionPane {
                title: "Squash",
                output,
            },
            Self::AbsorbPreview { output, .. } => Overlay::ActionPane {
                title: "Absorb",
                output,
            },
            Self::AbandonPreview { output, .. } => Overlay::ActionPane {
                title: "Abandon",
                output,
            },
            Self::AbandonConfirm { input, output, .. } => Overlay::AbandonConfirm { input, output },
            Self::PushRemotePrompt {
                remotes, selected, ..
            } => Overlay::PushRemotePrompt {
                remotes,
                selected: *selected,
            },
            Self::FetchRemotePrompt { remotes, selected } => Overlay::FetchRemotePrompt {
                remotes,
                selected: *selected,
            },
            Self::FetchPreview { output, .. } => Overlay::ActionPane {
                title: "Fetch",
                output,
            },
            Self::PushPreview { output, .. } => Overlay::ActionPane {
                title: "Push",
                output,
            },
            Self::OperationRecoveryPreview { output, .. } => Overlay::ActionPane {
                title: "Operation recovery",
                output,
            },
            Self::OperationTargetPreview { output, .. } => Overlay::ActionPane {
                title: "Operation action",
                output,
            },
            Self::WorkingCopyNavigationPreview { navigation, output } => Overlay::ActionPane {
                title: navigation.overlay_title(),
                output,
            },
            Self::Normal
            | Self::SearchPrompt(_)
            | Self::LogRevsetPrompt(_)
            | Self::DescribePrompt { .. }
            | Self::CommitPrompt(_)
            | Self::BookmarkNamePrompt { .. }
            | Self::BookmarkRenamePrompt { .. } => Overlay::None,
        }
    }
}
