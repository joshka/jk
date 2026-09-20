use jk_core::{
    CommandPreview, SelectionCandidates, SelectionDecision, SelectionRequest, SelectionResolution,
    SelectorKind, SelectorRole, SourceAction, resolve_selection,
};
use jk_tui::log_view::LogView;

use crate::squash::SquashSelection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommandPreview {
    pub(crate) preview: CommandPreview,
    pub(crate) source_action: SourceAction,
    pub(crate) source_key: &'static str,
    pub(crate) failure_label: &'static str,
    pub(crate) success_message: &'static str,
    pub(crate) details: Vec<String>,
    pub(crate) reselect_change_id: Option<String>,
    pub(crate) copy_status: Option<String>,
    /// Updated by rendering; confirmation is disabled until its controls are visible.
    pub(crate) can_confirm: bool,
    pub(crate) scroll: u16,
    pub(crate) max_scroll: u16,
}

impl PendingCommandPreview {
    pub(crate) fn rebase(preview: CommandPreview, details: Vec<String>) -> Self {
        Self {
            preview,
            details,
            source_action: SourceAction::RebaseRevision,
            source_key: "R",
            failure_label: "jj rebase",
            success_message: "Rebased revisions · a u undo · C history · o operations",
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }
    pub(crate) const fn describe(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::DescribeRevision,
            source_key: "a m",
            failure_label: "jj describe",
            success_message: "Described revision",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn abandon(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::AbandonRevision,
            source_key: "a a",
            failure_label: "jj abandon",
            success_message: "Abandoned revision",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn new_change(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::NewRevision,
            source_key: "a n",
            failure_label: "jj new",
            success_message: "Created new change",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn edit(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::EditRevision,
            source_key: "a e",
            failure_label: "jj edit",
            success_message: "Edited revision",
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
            details: Vec::new(),
        }
    }

    pub(crate) fn restore(preview: CommandPreview, source: &str) -> Self {
        Self {
            preview,
            source_action: SourceAction::RestoreRevision,
            source_key: "a r",
            failure_label: "jj restore",
            success_message: "Restored all paths into working copy",
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
            details: vec![
                format!("Source: {source}"),
                "Destination: @ (working copy)".to_owned(),
                "Affected content: all paths".to_owned(),
            ],
        }
    }

    pub(crate) const fn undo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Undo,
            source_key: "a u",
            failure_label: "jj undo",
            success_message: "Undid operation",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn redo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Redo,
            source_key: "a U",
            failure_label: "jj redo",
            success_message: "Redid operation",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) fn squash(preview: CommandPreview, selection: &SquashSelection) -> Self {
        Self {
            preview,
            source_action: SourceAction::SquashRevision,
            source_key: "a s",
            failure_label: "jj squash",
            success_message: "Squashed source changes · a u undo · C history",
            details: selection.preview_details(),
            reselect_change_id: Some(selection.destination_change_id().to_owned()),
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn bookmark_create(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::BookmarkCreate,
            source_key: "c",
            failure_label: "jj bookmark create",
            success_message: "Created bookmark",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn bookmark_move(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::BookmarkMove,
            source_key: "m",
            failure_label: "jj bookmark move",
            success_message: "Moved bookmark",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn bookmark_delete(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::BookmarkDelete,
            source_key: "x",
            failure_label: "jj bookmark delete",
            success_message: "Deleted bookmark",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn git_fetch(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::GitFetch,
            source_key: "F",
            failure_label: "jj git fetch",
            success_message: "Fetched remote",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) const fn git_push_dry_run(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::GitPushDryRun,
            source_key: "P",
            failure_label: "jj git push --dry-run",
            success_message: "Push dry-run complete",
            details: Vec::new(),
            reselect_change_id: None,
            copy_status: None,
            can_confirm: false,
            scroll: 0,
            max_scroll: 0,
        }
    }
}

pub fn selected_new_parents(log: &LogView) -> SelectionResolution<String> {
    let candidates = SelectionCandidates::new(
        log.selected_revision_id().map(ToOwned::to_owned),
        log.marked_revision_ids(),
    );
    let request = SelectionRequest::one_or_more(SelectorKind::Revision, SelectorRole::Parent);
    resolve_selection(request, SelectionDecision::Submit(candidates))
}

pub(crate) fn new_change_id_from_output(stderr: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stderr).lines().find_map(|line| {
        let line = strip_ansi(line);
        if !line.contains("Working copy") {
            return None;
        }
        let (_, after_marker) = line.split_once("now at:")?;
        after_marker
            .split_whitespace()
            .next()
            .map(ToOwned::to_owned)
    })
}

fn strip_ansi(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\u{1b}' {
            stripped.push(character);
            continue;
        }

        if chars.next_if_eq(&'[').is_none() {
            continue;
        }
        for code in chars.by_ref() {
            if ('@'..='~').contains(&code) {
                break;
            }
        }
    }
    stripped
}

pub fn command_failure_message(command: &str, stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = strip_ansi(&String::from_utf8_lossy(stderr))
        .trim()
        .to_owned();
    if !stderr.is_empty() {
        return format!("{command} failed: {stderr}");
    }

    let stdout = strip_ansi(&String::from_utf8_lossy(stdout))
        .trim()
        .to_owned();
    if !stdout.is_empty() {
        return format!("{command} failed: {stdout}");
    }

    format!("{command} failed")
}

#[cfg(test)]
mod tests {
    use jk_core::{JjCommandSpec, LogEntry, LogSnapshot};
    use jk_tui::log_view::LogAction;

    use super::*;

    fn preview() -> CommandPreview {
        JjCommandSpec::render_read_only(["status"]).command_preview()
    }

    #[test]
    fn pending_preview_metadata_matches_source_actions() {
        let describe = PendingCommandPreview::describe(preview());
        assert_eq!(describe.source_action, SourceAction::DescribeRevision);
        assert_eq!(describe.source_key, "a m");
        assert_eq!(describe.failure_label, "jj describe");

        let abandon = PendingCommandPreview::abandon(preview());
        assert_eq!(abandon.source_action, SourceAction::AbandonRevision);
        assert_eq!(abandon.source_key, "a a");
        assert_eq!(abandon.failure_label, "jj abandon");

        let new_change = PendingCommandPreview::new_change(preview());
        assert_eq!(new_change.source_key, "a n");

        let edit = PendingCommandPreview::edit(preview());
        assert_eq!(edit.source_key, "a e");

        let redo = PendingCommandPreview::redo(preview());
        assert_eq!(redo.source_action, SourceAction::Redo);
        assert_eq!(redo.source_key, "a U");
        assert_eq!(redo.failure_label, "jj redo");
    }

    #[test]
    fn new_change_id_parser_reads_working_copy_output() {
        assert_eq!(
            new_change_id_from_output(
                b"\x1b[1mWorking copy  (@) now at: pomznpsy 72f6e428 second\x1b[0m\n"
            ),
            Some("pomznpsy".to_owned())
        );
        assert_eq!(new_change_id_from_output(b"no working copy output\n"), None);
    }

    #[test]
    fn selected_new_parents_use_short_selected_revision() {
        let log = log_view(["abcdefghijklmnop", "zyxwvutsrqponmlk"]);

        let SelectionResolution::Resolved(resolved) = selected_new_parents(&log) else {
            panic!("selected revision should resolve as a parent");
        };
        assert_eq!(resolved.values(), ["abcdefgh"]);
    }

    #[test]
    fn selected_new_parents_use_short_marked_revisions() {
        let mut log = log_view(["abcdefghijklmnop", "bbbbbbbbcccccccc", "zyxwvutsrqponmlk"]);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::ToggleMark);

        let SelectionResolution::Resolved(resolved) = selected_new_parents(&log) else {
            panic!("ordered revision marks should resolve as parents");
        };
        assert_eq!(resolved.values(), ["abcdefgh", "zyxwvuts"]);
    }

    #[test]
    fn failure_message_prefers_stderr_then_stdout() {
        assert_eq!(
            command_failure_message("jj abandon", b"bad rev\n", b"ignored\n"),
            "jj abandon failed: bad rev"
        );
        assert_eq!(
            command_failure_message("jj new", b"", b"stdout failure\n"),
            "jj new failed: stdout failure"
        );
        assert_eq!(
            command_failure_message("jj edit", b"", b""),
            "jj edit failed"
        );
    }

    fn log_view<const N: usize>(change_ids: [&str; N]) -> LogView {
        let rendered = change_ids
            .iter()
            .enumerate()
            .map(|(index, change_id)| {
                let marker = if index == 0 { "@" } else { "○" };
                format!("{marker}  {change_id} summary\n")
            })
            .collect::<String>();
        let entries = change_ids
            .iter()
            .enumerate()
            .map(|(index, change_id)| {
                LogEntry::new(*change_id, "commit", "summary").with_rendered_line(index)
            })
            .collect();
        LogView::new(LogSnapshot::new(rendered, entries))
    }
}
