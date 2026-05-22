use super::{LogItem, LogView};
use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::jj::JjCommand;
use crate::menus::{CopyOption, ExactActionContext, build_action_menu};
use crate::search::{SearchQuery, entry_matches};

impl LogView {
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => match self.cycle_mode() {
                Ok(mode) => ViewEffect::StatusMessage(format!("mode: {}", mode.label())),
                Err(error) => ViewEffect::StatusError(error.to_string()),
            },
            ViewCommand::NewTrunk => ViewEffect::RunNewTrunk,
            ViewCommand::MoveDown => {
                self.select_next();
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.select_previous();
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.page_down(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.page_up(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.select_first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.select_last();
                ViewEffect::Handled
            }
            ViewCommand::OpenShow => self
                .current_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Show, revset.to_owned()))
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::OpenDiff => self
                .current_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Diff, revset.to_owned()))
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::ToggleSelect => self.toggle_selection(),
            ViewCommand::OpenActionMenu => self.open_action_menu(),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    pub fn current_revset(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::action_id)
            .or_else(|| self.spec.target())
    }

    pub fn selected_revision(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::action_id)
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(selected) = next_matching_entry(&self.entries, self.selection.index(), query)
        else {
            return false;
        };
        self.selection.set(selected, self.entries.len());
        true
    }

    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(selected) = previous_matching_entry(&self.entries, self.selection.index(), query)
        else {
            return false;
        };
        self.selection.set(selected, self.entries.len());
        true
    }

    pub fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.entries.get(self.selection.index()) else {
            return Vec::new();
        };
        let mut options = Vec::new();
        if let Some(change_id) = entry.change_id() {
            options.push(CopyOption::new("change id", change_id));
        }
        if let Some(commit_id) = entry.commit_id() {
            options.push(CopyOption::new("commit id", commit_id));
        }
        options.push(CopyOption::new("row text", entry.row_text()));
        options
    }

    pub fn select_first(&mut self) {
        self.selection.first();
    }

    pub fn select_next(&mut self) {
        self.selection.next(self.entries.len());
    }

    pub fn select_previous(&mut self) {
        self.selection.previous();
    }

    pub fn select_last(&mut self) {
        self.selection.last(self.entries.len());
    }

    fn page_down(&mut self, page_size: usize) {
        self.selection.set(
            self.selection.index().saturating_add(page_size),
            self.entries.len(),
        );
    }

    fn page_up(&mut self, page_size: usize) {
        self.selection.set(
            self.selection.index().saturating_sub(page_size),
            self.entries.len(),
        );
    }

    fn toggle_selection(&mut self) -> ViewEffect {
        let Some(change_id) = self.current_exact_change_id() else {
            return ViewEffect::StatusMessage(
                "selection only works on rows with exact change ids".to_owned(),
            );
        };
        let change_id = change_id.to_owned();

        let Some(position) = self
            .selected_change_ids
            .iter()
            .position(|selected| selected == &change_id)
        else {
            self.selected_change_ids.push(change_id.clone());
            return ViewEffect::StatusMessage(format!("selected {change_id}"));
        };
        self.selected_change_ids.remove(position);
        ViewEffect::StatusMessage(format!("unselected {change_id}"))
    }

    fn open_action_menu(&mut self) -> ViewEffect {
        let Some(current_revision) = self.current_exact_change_id() else {
            return ViewEffect::StatusMessage(
                "action menu requires current row to have an exact revision id".to_owned(),
            );
        };
        let selected_revisions = self.selected_revisions_in_log_order();
        let mut context =
            ExactActionContext::with_current(current_revision).with_sources(selected_revisions);
        if self.current_row_is_visible_working_copy() {
            context = context.with_visible_working_copy();
        }
        let menu = build_action_menu(&context);
        if menu.is_empty() {
            ViewEffect::StatusMessage("no preview actions available for selection".to_owned())
        } else {
            ViewEffect::OpenActionMenu(menu)
        }
    }

    fn current_exact_change_id(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::action_id)
    }

    fn current_row_is_visible_working_copy(&self) -> bool {
        self.entries
            .get(self.selection.index())
            .is_some_and(LogItem::is_visible_working_copy)
    }

    fn selected_revisions_in_log_order(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(LogItem::action_id)
            .filter(|action_id| {
                self.selected_change_ids
                    .iter()
                    .any(|selected| selected == action_id)
            })
            .map(str::to_owned)
            .collect()
    }
}

fn next_matching_entry(entries: &[LogItem], selected: usize, query: &SearchQuery) -> Option<usize> {
    ((selected + 1)..entries.len())
        .chain(0..selected.min(entries.len()))
        .find(|index| entry_matches(&entries[*index].lines(), query))
}

fn previous_matching_entry(
    entries: &[LogItem],
    selected: usize,
    query: &SearchQuery,
) -> Option<usize> {
    (0..selected)
        .rev()
        .chain(((selected + 1)..entries.len()).rev())
        .find(|index| entry_matches(&entries[*index].lines(), query))
}
