//! `jj bookmark list` view state, rendering, and item-based navigation.
//!
//! The first pass keeps bookmark rows close to rendered `jj` output while
//! carrying exact bookmark names and target ids separately for copy,
//! search, refresh, and open-show behavior.

use color_eyre::Result;
use color_eyre::eyre::eyre;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec};
use crate::jj_actions::{JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget};
use crate::jj_rows::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;
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
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Enter),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Selectable bookmark output from `jj bookmark list`.
pub struct BookmarksView {
    spec: ViewSpec,
    entries: Vec<BookmarkItem>,
    selection: Selection,
}

impl BookmarksView {
    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<BookmarkItem>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, Vec::new()),
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_new_with_args(entries: Vec<BookmarkItem>, args: Vec<String>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, args),
            selection: Selection::default(),
        }
    }

    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_bookmark_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.selection.next(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.selection.previous();
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::OpenShow => self
                .selected_entry()
                .and_then(BookmarkItem::target_change_id)
                .map(|change_id| ViewEffect::OpenDetail(JjCommand::Show, change_id.to_owned()))
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage(
                        "selected bookmark has no target change id".to_owned(),
                    )
                }),
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
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem
            | ViewCommand::OpenDiff
            | ViewCommand::ToggleSelect
            | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_bookmark_entries)
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn line_count(&self) -> usize {
        self.entries.iter().map(BookmarkItem::line_count).sum()
    }

    fn selected_entry(&self) -> Option<&BookmarkItem> {
        self.entries.get(self.selection.index())
    }

    pub fn selected_bookmark_name(&self) -> Option<&str> {
        self.selected_entry().map(BookmarkItem::bookmark_name)
    }

    pub fn selected_local_bookmark_name(&self) -> Option<&str> {
        self.selected_entry()
            .filter(|entry| matches!(entry.state(), BookmarkRowState::Local { .. }))
            .map(BookmarkItem::bookmark_name)
    }

    pub fn selected_bookmark_forget_target(
        &self,
    ) -> Result<Option<(&str, JjBookmarkForgetTarget)>> {
        let Some(entry) = self.selected_entry() else {
            return Ok(None);
        };
        let name = entry.bookmark_name();

        match entry.state() {
            BookmarkRowState::Local { tracking } => {
                let target = local_forget_target(tracking)?;
                Ok(Some((name, target)))
            }
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => {
                let target = self.remote_forget_target(name, remote, tracking, local_peer)?;
                Ok(Some((name, target)))
            }
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark forget requires trusted bookmark metadata for the selected row"
            )),
        }
    }

    pub fn selected_bookmark_tracking_target(
        &self,
        kind: JjBookmarkMutationKind,
    ) -> Result<Option<(&str, JjBookmarkTrackingTarget)>> {
        let Some(entry) = self.selected_entry() else {
            return Ok(None);
        };
        match kind {
            JjBookmarkMutationKind::Track => self.selected_bookmark_track_target(entry),
            JjBookmarkMutationKind::Untrack => self.selected_bookmark_untrack_target(entry),
            JjBookmarkMutationKind::Create
            | JjBookmarkMutationKind::Set
            | JjBookmarkMutationKind::Move
            | JjBookmarkMutationKind::Rename
            | JjBookmarkMutationKind::Delete
            | JjBookmarkMutationKind::Forget => Err(eyre!(
                "bookmark tracking target requires track or untrack action"
            )),
        }
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.entries.len()).rev())
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.selected_entry() else {
            return Vec::new();
        };

        let mut options = Vec::new();
        options.push(CopyOption::new("bookmark name", entry.bookmark_name()));
        if let Some(change_id) = entry.target_change_id() {
            options.push(CopyOption::new("change id", change_id));
        }
        if let Some(commit_id) = entry.target_commit_id() {
            options.push(CopyOption::new("commit id", commit_id));
        }
        options.push(CopyOption::new("row text", entry.row_text()));
        options
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<BookmarkItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_bookmark_name = self
            .selected_entry()
            .map(|entry| entry.bookmark_name().to_owned());

        self.entries = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_bookmark_name,
        );
        Ok(())
    }

    fn remote_forget_target(
        &self,
        name: &str,
        remote: &str,
        tracking: &RemoteBookmarkTrackingState,
        local_peer: &BookmarkLocalPeerState,
    ) -> Result<JjBookmarkForgetTarget> {
        match local_peer {
            BookmarkLocalPeerState::Present => {
                return Err(eyre!(
                    "bookmark forget disabled: selected remote bookmark has a local peer"
                ));
            }
            BookmarkLocalPeerState::Unknown => {
                return Err(eyre!(
                    "bookmark forget disabled: selected remote bookmark has unknown local-peer metadata"
                ));
            }
            BookmarkLocalPeerState::Absent => {}
        }

        if matches!(
            tracking,
            RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                ..
            }
        ) {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark tracking metadata says a local peer is present"
            ));
        }

        if self.has_local_bookmark_peer(name) {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark has a local peer"
            ));
        }

        let remote_siblings = self.remote_bookmark_peer_count(name);
        if remote_siblings != 1 {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark is not unique; found {remote_siblings} remote peers named '{name}'"
            ));
        }

        Ok(JjBookmarkForgetTarget::remote_only(
            remote,
            remote_tracking_summary(tracking),
        ))
    }

    fn has_local_bookmark_peer(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| {
            entry.bookmark_name() == name && matches!(entry.state(), BookmarkRowState::Local { .. })
        })
    }

    fn remote_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Remote { .. })
            })
            .count()
    }

    fn selected_bookmark_track_target<'a>(
        &self,
        entry: &'a BookmarkItem,
    ) -> Result<Option<(&'a str, JjBookmarkTrackingTarget)>> {
        let name = entry.bookmark_name();
        self.require_selected_target("track", entry)?;

        match entry.state() {
            BookmarkRowState::Local { tracking } => self
                .local_bookmark_track_target(name, entry, tracking)
                .map(Some),
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => remote_bookmark_track_target(name, remote, tracking, local_peer, entry, self)
                .map(|target| Some((name, target))),
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark track disabled: selected row has unknown bookmark metadata"
            )),
        }
    }

    fn selected_bookmark_untrack_target<'a>(
        &self,
        entry: &'a BookmarkItem,
    ) -> Result<Option<(&'a str, JjBookmarkTrackingTarget)>> {
        let name = entry.bookmark_name();
        self.require_selected_target("untrack", entry)?;

        match entry.state() {
            BookmarkRowState::Local { tracking } => self
                .local_bookmark_untrack_target(name, entry, tracking)
                .map(Some),
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => remote_bookmark_untrack_target(name, remote, tracking, local_peer, entry, self)
                .map(|target| Some((name, target))),
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark untrack disabled: selected row has unknown bookmark metadata"
            )),
        }
    }

    fn local_bookmark_track_target<'a>(
        &self,
        name: &'a str,
        local: &BookmarkItem,
        tracking: &LocalBookmarkRemoteState,
    ) -> Result<(&'a str, JjBookmarkTrackingTarget)> {
        self.require_safe_local_tracking_context("track", name)?;

        match tracking {
            LocalBookmarkRemoteState::UntrackedRemotePresent => {
                let remote =
                    self.exactly_one_remote_peer(name, "track", remote_peer_is_untracked)?;
                self.require_matching_peer_targets("track", local, remote)?;
                let remote_name = remote_name(remote).expect("remote peer has remote state");
                Ok((
                    name,
                    JjBookmarkTrackingTarget::local(
                        name,
                        remote.bookmark_name(),
                        remote_name,
                        format!(
                            "local bookmark with one untracked remote peer on {}; {}",
                            remote_name,
                            remote_tracking_summary(
                                remote_tracking(remote).expect("remote peer has remote state")
                            )
                        ),
                    ),
                ))
            }
            LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
                "bookmark track disabled: selected local bookmark is local-only"
            )),
            LocalBookmarkRemoteState::Tracked { .. } => Err(eyre!(
                "bookmark track disabled: selected local bookmark already has tracked remote metadata"
            )),
            LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
                "bookmark track disabled: selected local bookmark has ambiguous remote metadata"
            )),
        }
    }

    fn local_bookmark_untrack_target<'a>(
        &self,
        name: &'a str,
        local: &BookmarkItem,
        tracking: &LocalBookmarkRemoteState,
    ) -> Result<(&'a str, JjBookmarkTrackingTarget)> {
        self.require_safe_local_tracking_context("untrack", name)?;

        match tracking {
            LocalBookmarkRemoteState::Tracked { .. } => {
                let remote =
                    self.exactly_one_remote_peer(name, "untrack", remote_peer_is_tracked)?;
                self.require_matching_peer_targets("untrack", local, remote)?;
                let remote_name = remote_name(remote).expect("remote peer has remote state");
                Ok((
                    name,
                    JjBookmarkTrackingTarget::local(
                        name,
                        remote.bookmark_name(),
                        remote_name,
                        format!(
                            "local bookmark with one tracked remote peer on {}; {}",
                            remote_name,
                            remote_tracking_summary(
                                remote_tracking(remote).expect("remote peer has remote state")
                            )
                        ),
                    ),
                ))
            }
            LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark is local-only"
            )),
            LocalBookmarkRemoteState::UntrackedRemotePresent => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark has only untracked remote metadata"
            )),
            LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark has ambiguous remote metadata"
            )),
        }
    }

    fn require_safe_local_tracking_context(&self, action: &str, name: &str) -> Result<()> {
        if !self.has_unfiltered_all_remotes_metadata() {
            return Err(eyre!(
                "bookmark {action} disabled: selected local bookmark requires unfiltered all-remotes metadata"
            ));
        }

        let local_peers = self.local_bookmark_peer_count(name);
        if local_peers != 1 {
            return Err(eyre!(
                "bookmark {action} disabled: selected local bookmark is ambiguous; found {local_peers} local rows named '{name}'"
            ));
        }

        Ok(())
    }

    fn exactly_one_remote_peer(
        &self,
        name: &str,
        action: &str,
        is_eligible: impl Fn(&BookmarkItem) -> bool,
    ) -> Result<&BookmarkItem> {
        let peers = self
            .entries
            .iter()
            .filter(|entry| entry.bookmark_name() == name && is_eligible(entry))
            .collect::<Vec<_>>();

        match peers.as_slice() {
            [peer] => Ok(peer),
            [] => Err(eyre!(
                "bookmark {action} disabled: selected local bookmark has no exactly typed eligible remote sibling"
            )),
            _ => Err(eyre!(
                "bookmark {action} disabled: selected local bookmark has ambiguous remote siblings; found {} eligible remote rows named '{name}'",
                peers.len()
            )),
        }
    }

    fn require_selected_target(&self, action: &str, entry: &BookmarkItem) -> Result<()> {
        if entry.target_change_id().is_none() {
            return Err(eyre!(
                "bookmark {action} disabled: selected bookmark row has no exact target metadata"
            ));
        }
        Ok(())
    }

    fn require_matching_peer_targets(
        &self,
        action: &str,
        local: &BookmarkItem,
        remote: &BookmarkItem,
    ) -> Result<()> {
        let Some(local_target) = local.target_change_id() else {
            return Err(eyre!(
                "bookmark {action} disabled: selected local bookmark has no exact target metadata"
            ));
        };
        let Some(remote_target) = remote.target_change_id() else {
            return Err(eyre!(
                "bookmark {action} disabled: selected remote sibling has no exact target metadata"
            ));
        };
        if local_target != remote_target {
            return Err(eyre!(
                "bookmark {action} disabled: local and remote bookmark targets differ"
            ));
        }
        Ok(())
    }

    fn visible_local_peer_target(&self, action: &str, name: &str) -> Result<Option<&BookmarkItem>> {
        let peers = self
            .entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Local { .. })
            })
            .collect::<Vec<_>>();

        match peers.as_slice() {
            [] => Ok(None),
            [peer] => Ok(Some(peer)),
            _ => Err(eyre!(
                "bookmark {action} disabled: selected bookmark has ambiguous local peers; found {} local rows named '{name}'",
                peers.len()
            )),
        }
    }

    fn local_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Local { .. })
            })
            .count()
    }

    fn has_unfiltered_all_remotes_metadata(&self) -> bool {
        !self.spec.args().is_empty()
            && self
                .spec
                .args()
                .iter()
                .all(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    }
}

fn remote_bookmark_track_target(
    name: &str,
    remote: &str,
    tracking: &RemoteBookmarkTrackingState,
    local_peer: &BookmarkLocalPeerState,
    entry: &BookmarkItem,
    view: &BookmarksView,
) -> Result<JjBookmarkTrackingTarget> {
    match tracking {
        RemoteBookmarkTrackingState::Untracked { .. } => {
            let local = validated_visible_local_peer("track", name, local_peer, entry, view)?;
            let visible_state = format!(
                "remote bookmark on {remote}; {}; local peer: {}",
                remote_tracking_summary(tracking),
                local_peer_summary(local_peer)
            );
            Ok(tracking_target(name, remote, visible_state, local))
        }
        RemoteBookmarkTrackingState::Tracked { .. } => Err(eyre!(
            "bookmark track disabled: selected remote bookmark is already tracked"
        )),
    }
}

fn remote_bookmark_untrack_target(
    name: &str,
    remote: &str,
    tracking: &RemoteBookmarkTrackingState,
    local_peer: &BookmarkLocalPeerState,
    entry: &BookmarkItem,
    view: &BookmarksView,
) -> Result<JjBookmarkTrackingTarget> {
    match tracking {
        RemoteBookmarkTrackingState::Tracked { .. } => {
            let local = validated_visible_local_peer("untrack", name, local_peer, entry, view)?;
            let visible_state = format!(
                "remote bookmark on {remote}; {}; local peer: {}",
                remote_tracking_summary(tracking),
                local_peer_summary(local_peer)
            );
            Ok(tracking_target(name, remote, visible_state, local))
        }
        RemoteBookmarkTrackingState::Untracked { .. } => Err(eyre!(
            "bookmark untrack disabled: selected remote bookmark is not tracked"
        )),
    }
}

fn validated_visible_local_peer<'a>(
    action: &str,
    name: &str,
    local_peer: &BookmarkLocalPeerState,
    remote: &BookmarkItem,
    view: &'a BookmarksView,
) -> Result<Option<&'a BookmarkItem>> {
    match local_peer {
        BookmarkLocalPeerState::Present => {
            let Some(local) = view.visible_local_peer_target(action, name)? else {
                return Err(eyre!(
                    "bookmark {action} disabled: selected remote bookmark local-peer metadata drifted"
                ));
            };
            view.require_matching_peer_targets(action, local, remote)?;
            Ok(Some(local))
        }
        BookmarkLocalPeerState::Absent | BookmarkLocalPeerState::Unknown => Ok(None),
    }
}

fn tracking_target(
    name: &str,
    remote: &str,
    visible_state: String,
    local: Option<&BookmarkItem>,
) -> JjBookmarkTrackingTarget {
    match local {
        Some(local) => JjBookmarkTrackingTarget::local(
            local.bookmark_name(),
            name.to_owned(),
            remote.to_owned(),
            visible_state,
        ),
        None => {
            JjBookmarkTrackingTarget::remote_only(name.to_owned(), remote.to_owned(), visible_state)
        }
    }
}

fn remote_peer_is_tracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Tracked { .. },
            ..
        }
    )
}

fn remote_peer_is_untracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Untracked { .. },
            ..
        }
    )
}

fn remote_name(entry: &BookmarkItem) -> Option<&str> {
    match entry.state() {
        BookmarkRowState::Remote { remote, .. } => Some(remote),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

fn remote_tracking(entry: &BookmarkItem) -> Option<&RemoteBookmarkTrackingState> {
    match entry.state() {
        BookmarkRowState::Remote { tracking, .. } => Some(tracking),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

fn local_peer_summary(local_peer: &BookmarkLocalPeerState) -> &'static str {
    match local_peer {
        BookmarkLocalPeerState::Present => "present",
        BookmarkLocalPeerState::Absent => "absent",
        BookmarkLocalPeerState::Unknown => "unknown in visible rows",
    }
}

fn local_forget_target(tracking: &LocalBookmarkRemoteState) -> Result<JjBookmarkForgetTarget> {
    match tracking {
        LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
            "bookmark forget disabled: selected local bookmark is local-only"
        )),
        LocalBookmarkRemoteState::Tracked {
            untracked_remote_present,
        } => Ok(JjBookmarkForgetTarget::local(
            if *untracked_remote_present {
                "tracked local bookmark with an untracked remote peer present"
            } else {
                "tracked local bookmark"
            },
        )),
        LocalBookmarkRemoteState::UntrackedRemotePresent => Ok(JjBookmarkForgetTarget::local(
            "local bookmark with an untracked remote peer present",
        )),
        LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
            "bookmark forget disabled: selected local bookmark has ambiguous remote metadata"
        )),
    }
}

fn remote_tracking_summary(tracking: &RemoteBookmarkTrackingState) -> String {
    match tracking {
        RemoteBookmarkTrackingState::Tracked {
            local_present,
            synced,
        } => format!("tracked remote bookmark; local present: {local_present}; synced: {synced}"),
        RemoteBookmarkTrackingState::Untracked { synced } => {
            format!("untracked remote bookmark; synced: {synced}")
        }
    }
}

fn entry_list(entries: &[BookmarkItem], search: Option<&SearchQuery>) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let lines = entry
                .lines()
                .into_iter()
                .map(|line| highlight_line(line, search))
                .collect::<Vec<_>>();
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

fn restore_selection(
    selection: &mut Selection,
    entries: &[BookmarkItem],
    previous_index: usize,
    previous_bookmark_name: Option<String>,
) {
    if let Some(bookmark_name) = previous_bookmark_name
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.bookmark_name() == bookmark_name.as_str())
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::text::Line;

    use super::*;
    use crate::command::{Command, find_binding};
    use crate::jj::JjCommand;

    fn bookmark_item(
        text: &[&str],
        bookmark_name: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkItem {
        BookmarkItem::new(
            text.iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect::<Vec<_>>(),
            bookmark_name.to_owned(),
            target_change_id.map(str::to_owned),
            target_commit_id.map(str::to_owned),
        )
    }

    fn bookmarks_view(entries: Vec<BookmarkItem>) -> BookmarksView {
        BookmarksView {
            spec: ViewSpec::new(JjCommand::Bookmarks, Vec::new()),
            entries,
            selection: Selection::default(),
        }
    }

    fn all_remotes_bookmarks_view(entries: Vec<BookmarkItem>) -> BookmarksView {
        BookmarksView {
            spec: ViewSpec::new(JjCommand::Bookmarks, vec!["--all-remotes".to_owned()]),
            entries,
            selection: Selection::default(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn movement_is_bookmark_item_based() {
        let mut view = bookmarks_view(vec![
            bookmark_item(
                &["@  feature", "│  target change"],
                "feature",
                Some("a"),
                Some("aa"),
            ),
            bookmark_item(&["○  trunk"], "trunk", Some("b"), Some("bb")),
        ]);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );

        assert_eq!(view.selection.index(), 1);
        view.execute(
            ViewCommand::MoveUp,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 0);
        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 1);
    }

    #[test]
    fn copy_options_include_exact_name_and_target_ids_when_known() {
        let view = bookmarks_view(vec![bookmark_item(
            &["@  feature", "│  target change"],
            "feature",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        )]);

        let options = view.copy_options();

        assert_eq!(options.len(), 4);
        assert_eq!(options[0].label(), "bookmark name");
        assert_eq!(options[0].value(), "feature");
        assert_eq!(options[1].label(), "change id");
        assert_eq!(options[1].value().len(), 64);
        assert_eq!(options[2].label(), "commit id");
        assert_eq!(options[2].value().len(), 64);
        assert_eq!(options[3].label(), "row text");
        assert_eq!(options[3].value(), "@  feature\n│  target change");
    }

    #[test]
    fn refresh_preserves_selected_bookmark_name() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| {
            Ok(vec![
                bookmark_item(&["@  second"], "second", Some("b2"), Some("bb2")),
                bookmark_item(&["○  third"], "third", Some("c"), Some("cc")),
            ])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(view.entries[0].bookmark_name(), "second");
    }

    #[test]
    fn refresh_clamps_when_selected_bookmark_disappears() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| {
            Ok(vec![bookmark_item(
                &["@  first"],
                "first",
                Some("a"),
                Some("aa"),
            )])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn selected_bookmark_name_returns_current_row() {
        let view = bookmarks_view(vec![
            bookmark_item(
                &["@  feature", "│  target"],
                "feature",
                Some("a"),
                Some("aa"),
            ),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);

        assert_eq!(view.selected_bookmark_name(), Some("feature"));
    }

    #[test]
    fn selected_local_bookmark_name_ignores_nonlocal_rows() {
        let remote = bookmark_item(&["  @origin: abc"], "@origin", None, None).with_local(false);
        let view = bookmarks_view(vec![remote]);

        assert_eq!(view.selected_bookmark_name(), Some("@origin"));
        assert_eq!(view.selected_local_bookmark_name(), None);
    }

    #[test]
    fn selected_local_bookmark_name_ignores_unknown_metadata_rows() {
        let unknown = bookmark_item(&["maybe-local: abc"], "maybe-local", None, None)
            .with_state(BookmarkRowState::Unknown);
        let view = bookmarks_view(vec![unknown]);

        assert_eq!(view.selected_bookmark_name(), Some("maybe-local"));
        assert_eq!(view.selected_local_bookmark_name(), None);
    }

    #[test]
    fn bookmark_forget_target_accepts_tracked_or_remote_backed_local_rows() {
        let tracked =
            bookmark_item(&["main: abc"], "main", None, None).with_state(BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                },
            });
        let view = bookmarks_view(vec![tracked]);

        let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

        assert_eq!(name, "main");
        assert_eq!(
            target,
            JjBookmarkForgetTarget::local("tracked local bookmark")
        );

        let remote_backed = bookmark_item(&["feature: abc"], "feature", None, None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
            },
        );
        let view = bookmarks_view(vec![remote_backed]);

        let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

        assert_eq!(name, "feature");
        assert_eq!(
            target,
            JjBookmarkForgetTarget::local("local bookmark with an untracked remote peer present")
        );
    }

    #[test]
    fn bookmark_forget_target_accepts_single_remote_without_local_peer() {
        let remote = bookmark_item(&["main@origin: abc"], "main", None, None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Absent,
            },
        );
        let view = bookmarks_view(vec![remote]);

        let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

        assert_eq!(name, "main");
        assert_eq!(
            target,
            JjBookmarkForgetTarget::remote_only(
                "origin",
                "untracked remote bookmark; synced: false"
            )
        );
    }

    #[test]
    fn bookmark_forget_target_rejects_disabled_metadata_states() {
        let local_only = bookmark_item(&["scratch: abc"], "scratch", None, None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::LocalOnly,
            },
        );
        let view = bookmarks_view(vec![local_only]);
        assert_eq!(
            view.selected_bookmark_forget_target()
                .unwrap_err()
                .to_string(),
            "bookmark forget disabled: selected local bookmark is local-only"
        );

        let ambiguous =
            bookmark_item(&["main: abc"], "main", None, None).with_state(BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous,
            });
        let view = bookmarks_view(vec![ambiguous]);
        assert_eq!(
            view.selected_bookmark_forget_target()
                .unwrap_err()
                .to_string(),
            "bookmark forget disabled: selected local bookmark has ambiguous remote metadata"
        );

        let unknown = bookmark_item(&["main?: abc"], "main", None, None)
            .with_state(BookmarkRowState::Unknown);
        let view = bookmarks_view(vec![unknown]);
        assert_eq!(
            view.selected_bookmark_forget_target()
                .unwrap_err()
                .to_string(),
            "bookmark forget requires trusted bookmark metadata for the selected row"
        );
    }

    #[test]
    fn bookmark_forget_target_rejects_remote_rows_with_local_or_nonunique_peers() {
        let remote_with_local = bookmark_item(&["main@origin: abc"], "main", None, None)
            .with_state(BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            });
        let view = bookmarks_view(vec![remote_with_local]);
        assert_eq!(
            view.selected_bookmark_forget_target()
                .unwrap_err()
                .to_string(),
            "bookmark forget disabled: selected remote bookmark has a local peer"
        );

        let first = bookmark_item(&["main@origin: abc"], "main", None, None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Absent,
            },
        );
        let second = bookmark_item(&["main@upstream: abc"], "main", None, None).with_state(
            BookmarkRowState::Remote {
                remote: "upstream".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Absent,
            },
        );
        let view = bookmarks_view(vec![first, second]);
        assert_eq!(
            view.selected_bookmark_forget_target()
                .unwrap_err()
                .to_string(),
            "bookmark forget disabled: selected remote bookmark is not unique; found 2 remote peers named 'main'"
        );
    }

    #[test]
    fn bookmark_track_target_accepts_remote_untracked_rows() {
        let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Absent,
            },
        );
        let view = bookmarks_view(vec![remote]);

        let (name, target) = view
            .selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
            .unwrap()
            .unwrap();

        assert_eq!(name, "main");
        assert_eq!(
            target,
            JjBookmarkTrackingTarget::remote_only(
                "main",
                "origin",
                "remote bookmark on origin; untracked remote bookmark; synced: false; local peer: absent",
            )
        );
    }

    #[test]
    fn bookmark_untrack_target_accepts_remote_tracked_rows() {
        let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Unknown,
            },
        );
        let view = bookmarks_view(vec![remote]);

        let (name, target) = view
            .selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap()
            .unwrap();

        assert_eq!(name, "main");
        assert_eq!(
            target,
            JjBookmarkTrackingTarget::remote_only(
                "main",
                "origin",
                "remote bookmark on origin; tracked remote bookmark; local present: true; synced: true; local peer: unknown in visible rows",
            )
        );
    }

    #[test]
    fn bookmark_tracking_targets_accept_safe_local_rows() {
        let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                },
            },
        );
        let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let view = all_remotes_bookmarks_view(vec![local, remote]);

        let (name, target) = view
            .selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap()
            .unwrap();

        assert_eq!(name, "main");
        assert_eq!(
            target,
            JjBookmarkTrackingTarget::local(
                "main",
                "main",
                "origin",
                "local bookmark with one tracked remote peer on origin; tracked remote bookmark; local present: true; synced: true",
            )
        );

        let local = bookmark_item(&["topic: abc"], "topic", Some("b"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
            },
        );
        let remote = bookmark_item(&["topic@origin: abc"], "topic", Some("b"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let view = all_remotes_bookmarks_view(vec![local, remote]);

        let (name, target) = view
            .selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
            .unwrap()
            .unwrap();

        assert_eq!(name, "topic");
        assert_eq!(
            target,
            JjBookmarkTrackingTarget::local(
                "topic",
                "topic",
                "origin",
                "local bookmark with one untracked remote peer on origin; untracked remote bookmark; synced: false",
            )
        );
    }

    #[test]
    fn bookmark_tracking_targets_reject_disabled_row_states() {
        let local_only = bookmark_item(&["scratch: abc"], "scratch", Some("a"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::LocalOnly,
            },
        );
        let view = all_remotes_bookmarks_view(vec![local_only]);

        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
                .unwrap_err()
                .to_string(),
            "bookmark track disabled: selected local bookmark is local-only"
        );
        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
                .unwrap_err()
                .to_string(),
            "bookmark untrack disabled: selected local bookmark is local-only"
        );

        let unknown = bookmark_item(&["maybe: abc"], "maybe", Some("a"), None)
            .with_state(BookmarkRowState::Unknown);
        let view = bookmarks_view(vec![unknown]);
        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
                .unwrap_err()
                .to_string(),
            "bookmark track disabled: selected row has unknown bookmark metadata"
        );
    }

    #[test]
    fn bookmark_tracking_targets_reject_unsafe_local_context_and_drift() {
        let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                },
            },
        );
        let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let view = bookmarks_view(vec![local, remote]);

        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
                .unwrap_err()
                .to_string(),
            "bookmark untrack disabled: selected local bookmark requires unfiltered all-remotes metadata"
        );

        let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                },
            },
        );
        let remote = bookmark_item(&["main@origin: def"], "main", Some("b"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: false,
                },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let view = all_remotes_bookmarks_view(vec![local, remote]);

        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
                .unwrap_err()
                .to_string(),
            "bookmark untrack disabled: local and remote bookmark targets differ"
        );
    }

    #[test]
    fn bookmark_tracking_targets_reject_ambiguous_local_remote_siblings() {
        let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                },
            },
        );
        let origin = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let upstream = bookmark_item(&["main@upstream: abc"], "main", Some("a"), None).with_state(
            BookmarkRowState::Remote {
                remote: "upstream".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            },
        );
        let view = all_remotes_bookmarks_view(vec![local, origin, upstream]);

        assert_eq!(
            view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
                .unwrap_err()
                .to_string(),
            "bookmark untrack disabled: selected local bookmark has ambiguous remote siblings; found 2 eligible remote rows named 'main'"
        );
    }

    #[test]
    fn search_wraps_by_bookmark_item() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  alpha", "│  target"], "alpha", Some("a"), Some("aa")),
            bookmark_item(&["○  beta"], "beta", Some("b"), Some("bb")),
            bookmark_item(&["○  alpha"], "gamma", Some("c"), Some("cc")),
        ]);
        view.selection.set(1, view.entries.len());
        let query = SearchQuery::new("alpha".to_owned()).unwrap();

        assert_eq!(view.search_matches(&query), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 0);
        assert!(view.previous_match(&query));
        assert_eq!(view.selection.index(), 2);
    }

    #[test]
    fn open_show_uses_target_change_id_and_reports_missing_targets() {
        let view = bookmarks_view(vec![bookmark_item(
            &["@  feature", "│  target change"],
            "feature",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        )]);

        assert_eq!(
            find_binding(view.bindings(), key(KeyCode::Enter)).map(Binding::command),
            Some(Command::View(ViewCommand::OpenShow))
        );
        assert_eq!(
            find_binding(view.bindings(), key(KeyCode::Char('s'))).map(Binding::command),
            Some(Command::View(ViewCommand::OpenShow))
        );

        let mut view = view;
        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::OpenDetail(
                JjCommand::Show,
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned()
            )
        );

        let mut view = bookmarks_view(vec![bookmark_item(&["○  feature"], "feature", None, None)]);
        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage("selected bookmark has no target change id".to_owned())
        );
    }
}
