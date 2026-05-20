//! `jj status` view state, rendering, and scroll navigation.
//!
//! Status stays close to rendered `jj status` output, but it also carries a
//! narrow row model for exact file-path actions. Rows that do not confidently
//! name one repo-relative tracked path remain visible and selectable, but file
//! mutation actions report why they are disabled instead of guessing.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
#[cfg(test)]
use crate::jj::JjCommand;
use crate::jj::{ViewSpec, document_plain_text};
use crate::rendered_jj::DocumentLines;
use crate::search::{SearchQuery, highlight_line, line_matches};
use crate::selection::Selection;
use crate::sticky_file_view::load_document;
use crate::theme;

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(KeyPattern::char(' '), Command::View(ViewCommand::PageDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageDown),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char('f', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char(' ', crossterm::event::KeyModifiers::SHIFT),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageUp),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::modified_char('b', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenFiles),
    ),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

/// Rendered `jj status` output plus exact row action contracts.
pub struct StatusView {
    spec: ViewSpec,
    rows: Vec<StatusRow>,
    selection: Selection,
}

impl StatusView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            rows: load_status_rows(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(row_list(&self.rows, search), area, &mut state);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
            ViewCommand::MoveDown => {
                self.move_down(1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.move_up(1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.move_down(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.move_up(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.rows.len());
                ViewEffect::Handled
            }
            ViewCommand::OpenFiles => ViewEffect::OpenView(ViewSpec::file_list(None, None)),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(context.viewport_height, query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::OpenActionMenu => self
                .selected_exact_path()
                .map_or_else(ViewEffect::StatusError, |_| ViewEffect::Ignored),
            ViewCommand::ToggleSelect => ViewEffect::Ignored,
            ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_status_rows)?;
        Ok(())
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn line_count(&self) -> usize {
        self.rows.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.selection.index()
    }

    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.selection.set(scroll_offset, self.rows.len());
    }

    #[cfg(test)]
    pub fn scroll_to_bottom(&mut self, _viewport_height: u16) {
        self.selection.last(self.rows.len());
    }

    #[cfg(test)]
    pub fn scroll_down(&mut self, _viewport_height: u16, amount: usize) {
        self.move_down(amount);
    }

    pub fn clamp(&mut self, _viewport_height: u16) {
        self.selection.clamp(self.rows.len());
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.rows
            .iter()
            .filter(|row| line_matches(row.line(), query))
            .count()
    }

    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.rows.len())
            .chain(0..self.selection.index().min(self.rows.len()))
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.rows.len()).rev())
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    pub fn selected_exact_path(&self) -> std::result::Result<&str, String> {
        let Some(row) = self.rows.get(self.selection.index()) else {
            return Err("status file action unavailable: status output is empty".to_owned());
        };
        row.exact_path()
    }

    fn copy_options(&self) -> Vec<CopyOption> {
        let lines = self
            .rows
            .iter()
            .map(|row| row.line().clone())
            .collect::<Vec<_>>();
        let text = document_plain_text(&lines);
        if text.is_empty() {
            Vec::new()
        } else {
            vec![CopyOption::new("status text", text)]
        }
    }

    fn move_down(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.next(self.rows.len());
        }
    }

    fn move_up(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.previous();
        }
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<StatusRow>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self
            .rows
            .get(previous_index)
            .and_then(StatusRow::exact_path_option)
            .map(str::to_owned);
        let previous_text = self.rows.get(previous_index).map(StatusRow::row_text);

        self.rows = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.rows,
            previous_index,
            previous_path,
            previous_text,
        );
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct StatusRow {
    line: Line<'static>,
    path: StatusPathContract,
}

impl StatusRow {
    fn new(line: Line<'static>, path: StatusPathContract) -> Self {
        Self { line, path }
    }

    fn line(&self) -> &Line<'static> {
        &self.line
    }

    fn exact_path(&self) -> std::result::Result<&str, String> {
        match &self.path {
            StatusPathContract::Exact(path) => Ok(path),
            StatusPathContract::Disabled(message) => Err((*message).to_owned()),
            StatusPathContract::None => Err(
                "status file action unavailable: selected row has no exact file path".to_owned(),
            ),
        }
    }

    fn exact_path_option(&self) -> Option<&str> {
        match &self.path {
            StatusPathContract::Exact(path) => Some(path),
            StatusPathContract::Disabled(_) | StatusPathContract::None => None,
        }
    }

    fn row_text(&self) -> String {
        line_text(&self.line)
    }
}

#[derive(Clone, Debug)]
enum StatusPathContract {
    Exact(String),
    Disabled(&'static str),
    None,
}

fn load_status_rows(spec: &ViewSpec) -> Result<Vec<StatusRow>> {
    Ok(status_rows_from_document(load_document(spec)?))
}

fn status_rows_from_document(document: DocumentLines) -> Vec<StatusRow> {
    document
        .lines()
        .iter()
        .cloned()
        .map(parse_status_row)
        .collect()
}

fn parse_status_row(line: Line<'static>) -> StatusRow {
    let text = line_text(&line);
    let path = parse_status_path_contract(&text);
    StatusRow::new(line, path)
}

fn parse_status_path_contract(text: &str) -> StatusPathContract {
    let Some(separator_index) = text.find(char::is_whitespace) else {
        return StatusPathContract::None;
    };
    let status = &text[..separator_index];
    if status.is_empty()
        || status.len() > 2
        || status.chars().any(|character| !is_status_code(character))
    {
        return StatusPathContract::None;
    }
    let separator_and_path = &text[separator_index..];
    let Some(path) = separator_and_path.strip_prefix(' ') else {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected path separator is ambiguous",
        );
    };
    if path.is_empty() {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected status row has no file path",
        );
    }
    if status.contains('?') || status.contains('!') {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected status row is not a tracked jj path",
        );
    }
    if status == "R" {
        return StatusPathContract::Disabled(
            "status file action unavailable: renamed status rows contain multiple paths",
        );
    }
    if !matches!(status, "M" | "A" | "D") {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected status kind is not path-restorable",
        );
    }

    match validate_repo_relative_path(path) {
        Ok(()) => StatusPathContract::Exact(path.to_owned()),
        Err(message) => StatusPathContract::Disabled(message),
    }
}

fn is_status_code(character: char) -> bool {
    character.is_ascii_alphabetic() || matches!(character, '?' | '!')
}

fn validate_repo_relative_path(path: &str) -> std::result::Result<(), &'static str> {
    if path.trim() != path {
        return Err(
            "status file action unavailable: selected path has ambiguous surrounding whitespace",
        );
    }
    if path.starts_with('/') {
        return Err("status file action unavailable: selected path is absolute");
    }
    if path.contains('\0') || path.contains('\n') {
        return Err("status file action unavailable: selected path contains invalid control text");
    }
    if path.contains(" => ") || (path.starts_with('{') && path.ends_with('}')) {
        return Err(
            "status file action unavailable: selected row appears to contain multiple paths",
        );
    }
    if path
        .split('/')
        .any(|component| matches!(component, "" | "." | ".."))
    {
        return Err(
            "status file action unavailable: selected path is not a clean repo-relative path",
        );
    }
    Ok(())
}

fn row_list(rows: &[StatusRow], search: Option<&SearchQuery>) -> List<'static> {
    let items = rows
        .iter()
        .map(|row| ListItem::new(highlight_line(row.line().clone(), search)))
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

fn restore_selection(
    selection: &mut Selection,
    rows: &[StatusRow],
    previous_index: usize,
    previous_path: Option<String>,
    previous_text: Option<String>,
) {
    if let Some(path) = previous_path
        && let Some(index) = rows
            .iter()
            .position(|row| row.exact_path_option() == Some(path.as_str()))
    {
        selection.set(index, rows.len());
        return;
    }

    if let Some(text) = previous_text
        && let Some(index) = rows.iter().position(|row| row.row_text() == text)
    {
        selection.set(index, rows.len());
        return;
    }

    selection.set(previous_index, rows.len());
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[cfg(test)]
impl StatusView {
    pub(crate) fn test_new(lines: &[&str]) -> Self {
        Self {
            spec: ViewSpec::new(JjCommand::Status, Vec::new()),
            rows: lines
                .iter()
                .map(|line| parse_status_row(Line::from((*line).to_owned())))
                .collect(),
            selection: Selection::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    fn status_view(lines: &[&str]) -> StatusView {
        StatusView::test_new(lines)
    }

    #[test]
    fn copy_options_use_full_status_text() {
        let view = status_view(&["Working copy changes:", "M src/app.rs"]);

        let options = view.copy_options();

        assert_eq!(options.len(), 1);
        assert_eq!(options[0].label(), "status text");
        assert_eq!(options[0].value(), "Working copy changes:\nM src/app.rs");
    }

    #[test]
    fn status_navigation_moves_through_rows() {
        let mut view = status_view(&["one", "two", "three"]);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 3,
                search: None,
            },
        );
        assert_eq!(view.scroll_offset(), 1);

        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 3,
                search: None,
            },
        );
        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn status_search_moves_to_next_match() {
        let mut view = status_view(&["first", "beta", "third", "beta again"]);
        let query = SearchQuery::new("beta".to_owned()).unwrap();

        let effect = view.execute(
            ViewCommand::StartSearch,
            CommandContext {
                viewport_height: 3,
                search: Some(&query),
            },
        );

        assert_eq!(effect, ViewEffect::SearchStarted { matches: 2 });
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn clamp_preserves_readable_selection_after_document_shrinks() {
        let mut view = status_view(&["one", "two", "three"]);
        view.scroll_to_bottom(3);
        assert_eq!(view.scroll_offset(), 2);

        view.rows = vec![parse_status_row(Line::from("one".to_owned()))];
        view.clamp(3);

        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn status_rows_keep_rendered_sections_readable() {
        let view = status_view(&[
            "The working copy has conflicts:",
            "UU src/app.rs",
            "",
            "Working copy changes:",
            "M src/status.rs",
            "A docs/plan/progress.md",
            "",
            "Working copy  (@) : yostqsxw 12345678 Slice 6 work",
            "Parent commit (@-): mzvwutkl 87654321 Prior change",
        ]);

        let rendered = view
            .rows
            .iter()
            .map(StatusRow::row_text)
            .collect::<Vec<_>>()
            .join("\n");

        assert_snapshot!(rendered, @r"
        The working copy has conflicts:
        UU src/app.rs

        Working copy changes:
        M src/status.rs
        A docs/plan/progress.md

        Working copy  (@) : yostqsxw 12345678 Slice 6 work
        Parent commit (@-): mzvwutkl 87654321 Prior change
        ");
    }

    #[test]
    fn status_parser_accepts_modified_added_and_deleted_paths() {
        let modified = parse_status_row(Line::from("M src/status.rs".to_owned()));
        let added = parse_status_row(Line::from("A docs/plan/progress.md".to_owned()));
        let deleted = parse_status_row(Line::from("D dir/with spaces/file.txt".to_owned()));

        assert_eq!(modified.exact_path().unwrap(), "src/status.rs");
        assert_eq!(added.exact_path().unwrap(), "docs/plan/progress.md");
        assert_eq!(deleted.exact_path().unwrap(), "dir/with spaces/file.txt");
    }

    #[test]
    fn status_parser_disables_renamed_conflict_and_untracked_looking_rows() {
        let renamed = parse_status_row(Line::from("R {old.rs => new.rs}".to_owned()));
        let conflict = parse_status_row(Line::from("UU src/lib.rs".to_owned()));
        let untracked = parse_status_row(Line::from("? scratch.txt".to_owned()));

        assert_eq!(
            renamed.exact_path().unwrap_err(),
            "status file action unavailable: renamed status rows contain multiple paths"
        );
        assert_eq!(
            conflict.exact_path().unwrap_err(),
            "status file action unavailable: selected status kind is not path-restorable"
        );
        assert_eq!(
            untracked.exact_path().unwrap_err(),
            "status file action unavailable: selected status row is not a tracked jj path"
        );
    }

    #[test]
    fn status_parser_disables_ambiguous_paths() {
        let absolute = parse_status_row(Line::from("M /tmp/file.txt".to_owned()));
        let parent = parse_status_row(Line::from("M ../outside.txt".to_owned()));
        let multiple = parse_status_row(Line::from("M {old => new}".to_owned()));
        let leading_space = parse_status_row(Line::from("M  leading-space.txt".to_owned()));

        assert_eq!(
            absolute.exact_path().unwrap_err(),
            "status file action unavailable: selected path is absolute"
        );
        assert_eq!(
            parent.exact_path().unwrap_err(),
            "status file action unavailable: selected path is not a clean repo-relative path"
        );
        assert_eq!(
            multiple.exact_path().unwrap_err(),
            "status file action unavailable: selected row appears to contain multiple paths"
        );
        assert_eq!(
            leading_space.exact_path().unwrap_err(),
            "status file action unavailable: selected path has ambiguous surrounding whitespace"
        );
    }

    #[test]
    fn status_action_path_follows_selected_exact_row() {
        let mut view = status_view(&["Working copy changes:", "M src/status.rs"]);

        assert_eq!(
            view.selected_exact_path().unwrap_err(),
            "status file action unavailable: selected row has no exact file path"
        );

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 4,
                search: None,
            },
        );

        assert_eq!(view.selected_exact_path().unwrap(), "src/status.rs");
    }

    #[test]
    fn status_refresh_preserves_selected_exact_path_when_present() {
        let mut view = status_view(&["M alpha", "M beta"]);
        view.selection.set(1, view.rows.len());

        view.refresh_with_loader(|_| {
            Ok(vec![
                parse_status_row(Line::from("M beta".to_owned())),
                parse_status_row(Line::from("M alpha".to_owned())),
            ])
        })
        .unwrap();

        assert_eq!(view.scroll_offset(), 0);
        assert_eq!(view.selected_exact_path().unwrap(), "beta");
    }

    #[test]
    fn status_refresh_clamps_when_selected_path_disappears() {
        let mut view = status_view(&["M alpha", "M beta"]);
        view.selection.set(1, view.rows.len());

        view.refresh_with_loader(|_| Ok(vec![parse_status_row(Line::from("M alpha".to_owned()))]))
            .unwrap();

        assert_eq!(view.scroll_offset(), 0);
        assert_eq!(view.selected_exact_path().unwrap(), "alpha");
    }
}
