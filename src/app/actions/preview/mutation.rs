use super::super::super::App;
use super::super::shared::bookmark_status_context;
use crate::actions::{JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjFileMutationPlan};
use crate::modes::InteractionMode;

impl App {
    /// Open the describe preview for one prepared describe plan.
    pub fn open_describe_preview(&mut self, describe: JjDescribePlan) {
        let status_context = Some(format!(
            "describe {} from {}",
            describe.target().label(),
            self.view.spec().app_label()
        ));

        let command_label = describe.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            describe.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::DescribePreview { describe, output };
    }

    /// Open the commit preview for the current working-copy commit plan.
    pub fn open_commit_preview(&mut self, commit: JjCommitPlan) {
        let status_context = Some(format!(
            "commit current working-copy change (@) from {}",
            self.view.spec().app_label()
        ));

        let command_label = commit.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            commit.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::CommitPreview { commit, output };
    }

    /// Open the bookmark mutation preview for one prepared bookmark plan.
    pub fn open_bookmark_mutation_preview(&mut self, mutation: JjBookmarkMutationPlan) {
        let status_context = Some(bookmark_status_context(
            &mutation,
            self.view.spec().app_label().as_str(),
        ));

        let command_label = mutation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            mutation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::BookmarkMutationPreview { mutation, output };
    }

    /// Open the file mutation preview for one prepared file plan.
    pub fn open_file_mutation_preview(&mut self, mutation: JjFileMutationPlan) {
        let target = mutation
            .revision()
            .map(|revision| format!("{} at {}", mutation.path(), revision))
            .unwrap_or_else(|| format!("{} at @", mutation.path()));
        let status_context = Some(format!(
            "file {} {} from {}",
            mutation.kind().label(),
            target,
            self.view.spec().app_label()
        ));

        let command_label = mutation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            mutation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::FileMutationPreview { mutation, output };
    }
}
