use crate::actions::{
    JjDuplicatePlan, JjNewPlan, JjSplitPlan, JjWorkingCopyNavigationKind,
    JjWorkingCopyNavigationPlan,
};
use crate::app::actions::shared::short_id;
use crate::app::status_line::StatusLine;
use crate::app::{App, clamp_view_to_current_viewport};
use crate::jj::LogViewMode;
use crate::modes::InteractionMode;

impl App {
    /// Translate a log-local edit/next/prev command into a working-copy navigation preview.
    pub fn open_log_working_copy_navigation_preview(&mut self, kind: JjWorkingCopyNavigationKind) {
        let navigation = match kind {
            JjWorkingCopyNavigationKind::Edit => {
                let Some(revision) = self.graph_selected_revision() else {
                    self.status = StatusLine::error(
                        &self.view,
                        "edit from log requires a selected row with an exact revision".to_owned(),
                    );
                    return;
                };
                JjWorkingCopyNavigationPlan::edit(revision)
            }
            JjWorkingCopyNavigationKind::Next => JjWorkingCopyNavigationPlan::next(),
            JjWorkingCopyNavigationKind::Prev => JjWorkingCopyNavigationPlan::prev(),
        };

        self.open_working_copy_navigation_preview(navigation);
    }

    /// Open the preview for one working-copy navigation plan.
    pub fn open_working_copy_navigation_preview(
        &mut self,
        navigation: JjWorkingCopyNavigationPlan,
    ) {
        let status_context = Some(match navigation.kind() {
            JjWorkingCopyNavigationKind::Edit => format!(
                "edit exact log revision {} from {}",
                navigation
                    .target_change_id()
                    .expect("edit preview requires exact change id"),
                self.view.spec().app_label()
            ),
            JjWorkingCopyNavigationKind::Next => format!(
                "move @ with jj next --edit from {}",
                self.view.spec().app_label()
            ),
            JjWorkingCopyNavigationKind::Prev => format!(
                "move @ with jj prev --edit from {}",
                self.view.spec().app_label()
            ),
        });

        let command_label = navigation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            navigation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::WorkingCopyNavigationPreview { navigation, output };
    }

    /// Open the new-change preview for one prepared parent list.
    pub fn open_new_preview(&mut self, new_change: JjNewPlan) {
        let parent_labels = new_change
            .parents()
            .iter()
            .map(|parent| short_id(parent))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = Some(format!(
            "new from {} parent(s) from {} | parent(s): {}",
            new_change.parents().len(),
            self.view.spec().app_label(),
            parent_labels
        ));

        let command_label = new_change.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            new_change.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::NewPreview { new_change, output };
    }

    /// Open the duplicate preview for one exact source revision.
    pub fn open_duplicate_preview(&mut self, duplicate: JjDuplicatePlan) {
        let status_context = Some(format!(
            "duplicate exact source {} from {}",
            duplicate.source(),
            self.view.spec().app_label()
        ));

        let command_label = duplicate.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            duplicate.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::DuplicatePreview { duplicate, output };
    }

    /// Run `jj new` from trunk and update the active app view to reveal the new working copy.
    pub fn run_new_trunk(&mut self, _viewport_height: u16) {
        if let Err(error) = self.services.resolve_revision("trunk()") {
            self.status = StatusLine::error(&self.view, error.to_string());
            return;
        }

        match self.services.run_new_trunk() {
            Ok(_) => {
                let new_change_id = match self.services.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        return;
                    }
                };
                self.finish_new_trunk_success(&new_change_id);
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    /// Refresh and reveal the new trunk child in the current log-oriented view.
    fn finish_new_trunk_success(&mut self, new_change_id: &str) {
        match self.refresh_view_state() {
            Ok(()) => {
                clamp_view_to_current_viewport(&mut self.view);
                let revealed_in_recent =
                    match self.reveal_log_change(new_change_id, LogViewMode::Recent) {
                        Ok(switched_modes) => {
                            clamp_view_to_current_viewport(&mut self.view);
                            switched_modes
                        }
                        Err(error) => {
                            self.status = StatusLine::error(&self.view, error.to_string());
                            return;
                        }
                    };
                let message = if revealed_in_recent {
                    "created new change from trunk | showing recent work | jj undo"
                } else {
                    "created new change from trunk | jj undo"
                };
                self.status = StatusLine::with_message(&self.view, message);
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    /// Open the split preview for one exact or working-copy split plan.
    pub fn open_split_preview(&mut self, split: JjSplitPlan) {
        let status_context = Some(format!(
            "{} from {}",
            split.status_context(),
            self.view.spec().app_label()
        ));

        let command_label = split.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            split.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::SplitPreview { split, output };
    }
}
