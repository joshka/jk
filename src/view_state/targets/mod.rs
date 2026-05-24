use color_eyre::Result;

use crate::actions::{JjBookmarkForgetTarget, JjBookmarkTarget, JjGitPushTarget};
use crate::menus::ExactActionContext;
use crate::view_state::ViewState;

mod bookmark_targets;
mod exact_context;

struct ViewActionTargets<'a> {
    /// Active view whose selection is being projected into action targets.
    view: &'a ViewState,
}

impl<'a> ViewActionTargets<'a> {
    /// Builds the action-target projector for the current active view.
    fn new(view: &'a ViewState) -> Self {
        Self { view }
    }

    /// Projects the active view into a push target or reports why the current selection is unsafe.
    fn push_target(&self) -> Result<Option<JjGitPushTarget>> {
        bookmark_targets::push_target(self.view)
    }

    /// Projects the active view into a bookmark target or reports why the current selection is
    /// unsafe.
    fn bookmark_target(&self) -> Result<Option<JjBookmarkTarget>> {
        bookmark_targets::bookmark_target(self.view)
    }

    /// Returns the selected exact local bookmark name for delete-like actions.
    fn selected_local_bookmark_name(&self) -> Result<Option<&'a str>> {
        self.selected_local_bookmark_name_for("delete")
    }

    /// Returns the selected exact local bookmark name for one named action or reports why it is
    /// unsafe.
    fn selected_local_bookmark_name_for(&self, action: &str) -> Result<Option<&'a str>> {
        bookmark_targets::selected_local_bookmark_name_for(self.view, action)
    }

    /// Projects the selected bookmarks row into a forget target or reports why it is unsafe.
    fn bookmark_forget_target(&self) -> Result<Option<(String, JjBookmarkForgetTarget)>> {
        bookmark_targets::bookmark_forget_target(self.view)
    }

    /// Projects the active detail, file, or status view into an exact restore/revert action
    /// context.
    fn exact_restore_revert_context(&self) -> Result<Option<ExactActionContext>> {
        exact_context::exact_restore_revert_context(self.view)
    }
}

impl ViewState {
    pub fn push_target(&self) -> Result<Option<JjGitPushTarget>> {
        ViewActionTargets::new(self).push_target()
    }

    pub fn bookmark_target(&self) -> Result<Option<JjBookmarkTarget>> {
        ViewActionTargets::new(self).bookmark_target()
    }

    pub fn selected_local_bookmark_name(&self) -> Result<Option<&str>> {
        ViewActionTargets::new(self).selected_local_bookmark_name()
    }

    pub fn selected_local_bookmark_name_for(&self, action: &str) -> Result<Option<&str>> {
        ViewActionTargets::new(self).selected_local_bookmark_name_for(action)
    }

    pub fn bookmark_forget_target(&self) -> Result<Option<(String, JjBookmarkForgetTarget)>> {
        ViewActionTargets::new(self).bookmark_forget_target()
    }

    pub fn exact_restore_revert_context(&self) -> Result<Option<ExactActionContext>> {
        ViewActionTargets::new(self).exact_restore_revert_context()
    }
}
