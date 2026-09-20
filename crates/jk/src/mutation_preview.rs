use jk_core::{
    CommandPreview, InvalidSelection, SelectionCandidates, SelectionDecision, SelectionRequest,
    SelectionResolution, SelectorKind, SelectorRole, SourceAction, resolve_selection,
};
use jk_tui::log_view::LogView;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommandPreview {
    pub(crate) preview: CommandPreview,
    pub(crate) source_action: SourceAction,
    pub(crate) source_key: &'static str,
    pub(crate) failure_label: &'static str,
    pub(crate) success_message: &'static str,
}

impl PendingCommandPreview {
    pub(crate) const fn describe(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::DescribeRevision,
            source_key: "a m",
            failure_label: "jj describe",
            success_message: "Described revision",
        }
    }

    pub(crate) const fn abandon(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::AbandonRevision,
            source_key: "a a",
            failure_label: "jj abandon",
            success_message: "Abandoned revision",
        }
    }

    pub(crate) const fn new_change(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::NewRevision,
            source_key: "a n",
            failure_label: "jj new",
            success_message: "Created new change",
        }
    }

    pub(crate) const fn edit(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::EditRevision,
            source_key: "a e",
            failure_label: "jj edit",
            success_message: "Edited revision",
        }
    }

    pub(crate) const fn undo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Undo,
            source_key: "a u",
            failure_label: "jj undo",
            success_message: "Undid operation",
        }
    }

    pub(crate) const fn redo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Redo,
            source_key: "a U",
            failure_label: "jj redo",
            success_message: "Redid operation",
        }
    }
}

pub fn selected_new_parents(log: &LogView) -> SelectionResolution<String> {
    let request = SelectionRequest::one_or_more(SelectorKind::Revision, SelectorRole::Parent);
    let mut parents = Vec::new();
    for change_id in log.marked_change_ids() {
        let Some(commit_id) = log.commit_id_for_change_id(change_id) else {
            return SelectionResolution::Invalid {
                request,
                reason: InvalidSelection::UnresolvedIdentity,
            };
        };
        parents.push(commit_id.to_owned());
    }
    let candidates =
        SelectionCandidates::new(log.selected_commit_id().map(ToOwned::to_owned), parents);
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
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();
    if !stderr.is_empty() {
        return format!("{command} failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(stdout).trim().to_owned();
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
    fn selected_new_parents_use_exact_selected_commit() {
        let log = log_view(["abcdefghijklmnop", "zyxwvutsrqponmlk"]);

        let SelectionResolution::Resolved(resolved) = selected_new_parents(&log) else {
            panic!("selected revision should resolve as a parent");
        };
        assert_eq!(
            resolved.values(),
            ["0000000000000000000000000000000000000001"]
        );
    }

    #[test]
    fn selected_new_parents_use_exact_marked_commits_in_order() {
        let mut log = log_view(["abcdefghijklmnop", "bbbbbbbbcccccccc", "zyxwvutsrqponmlk"]);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::ToggleMark);

        let SelectionResolution::Resolved(resolved) = selected_new_parents(&log) else {
            panic!("ordered revision marks should resolve as parents");
        };
        assert_eq!(
            resolved.values(),
            [
                "0000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000003",
            ]
        );
    }

    #[test]
    fn marked_parents_with_the_same_short_prefix_remain_distinct() {
        let mut log = log_view(["abcdefgh11111111", "abcdefgh22222222"]);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::ToggleMark);

        let SelectionResolution::Resolved(resolved) = selected_new_parents(&log) else {
            panic!("distinct marked commits should resolve even when display prefixes collide");
        };
        assert_eq!(resolved.values().len(), 2);
        assert_ne!(resolved.values()[0], resolved.values()[1]);
    }

    #[test]
    fn marked_divergent_parent_is_rejected_without_falling_back_to_cursor() {
        let mut log = log_view(["divergent", "divergent", "cursor"]);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Last);

        assert!(matches!(
            selected_new_parents(&log),
            SelectionResolution::Invalid {
                reason: InvalidSelection::UnresolvedIdentity,
                ..
            }
        ));
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
                LogEntry::new(*change_id, format!("{:040x}", index + 1), "summary")
                    .with_rendered_line(index)
            })
            .collect();
        LogView::new(LogSnapshot::new(rendered, entries))
    }
}
