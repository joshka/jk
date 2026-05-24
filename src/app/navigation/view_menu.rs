use color_eyre::Result;

use crate::app::App;
use crate::app::status_line::StatusLine;
use crate::jj::{self, DiffFormat};
use crate::modes::{InteractionMode, ViewMenuAction, view_menu_options};

impl App {
    /// Open the top-level view menu with the current surface preselected when
    /// possible.
    pub fn open_view_menu(&mut self) {
        let selected = view_menu_options()
            .iter()
            .position(|option| self.view_menu_option_is_current(option.action()))
            .unwrap_or(0);
        self.mode = InteractionMode::ViewMenu { selected };
    }

    fn view_menu_option_is_current(&self, action: ViewMenuAction) -> bool {
        match action {
            ViewMenuAction::Open(command) => self.view.command() == command,
            ViewMenuAction::DiffFormat(format) => {
                matches!(self.view.command(), jj::Command::Show | jj::Command::Diff)
                    && self.diff_format == format
            }
        }
    }

    /// Apply one top-level view-menu choice.
    pub fn apply_view_menu_action(
        &mut self,
        action: ViewMenuAction,
        viewport_height: u16,
    ) -> Result<()> {
        match action {
            ViewMenuAction::Open(jj::Command::Log) => self.switch_to_log(),
            ViewMenuAction::Open(jj::Command::Default) => self.switch_to_default(),
            ViewMenuAction::Open(jj::Command::Status) => self.open_status(),
            ViewMenuAction::Open(jj::Command::Resolve) => self.open_resolve(),
            ViewMenuAction::Open(jj::Command::Bookmarks) => self.open_bookmarks(),
            ViewMenuAction::Open(jj::Command::Workspaces) => self.open_workspaces(),
            ViewMenuAction::Open(jj::Command::OperationLog) => self.open_operation_log(),
            ViewMenuAction::DiffFormat(diff_format) => {
                self.apply_diff_format(diff_format, viewport_height)
            }
            ViewMenuAction::Open(
                jj::Command::Show
                | jj::Command::Diff
                | jj::Command::FileList
                | jj::Command::FileShow
                | jj::Command::OperationShow
                | jj::Command::OperationDiff,
            ) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    "view menu only opens top-level shipped views",
                );
                Ok(())
            }
        }
    }

    /// Apply the app-level show/diff format toggle and reload the current
    /// detail view if needed.
    fn apply_diff_format(&mut self, diff_format: DiffFormat, viewport_height: u16) -> Result<()> {
        self.diff_format = diff_format;
        if !matches!(self.view.command(), jj::Command::Show | jj::Command::Diff) {
            self.status = StatusLine::with_message(
                &self.view,
                format!("show/diff format: {}", diff_format.label()),
            );
            return Ok(());
        }

        let scroll_offset = self.view.scroll_offset();
        let spec = self.view.spec().with_diff_format(diff_format);
        self.view = self.services.load_view(spec)?;
        self.view.set_scroll_offset(viewport_height, scroll_offset);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }
}
