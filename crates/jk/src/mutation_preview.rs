use jk_core::{CommandPreview, SourceAction};
use jk_tui::log_view::LogView;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingCommandPreview {
    pub(crate) preview: CommandPreview,
    pub(crate) source_action: SourceAction,
    pub(crate) source_key: &'static str,
    pub(crate) failure_label: &'static str,
    pub(crate) copy_status: Option<String>,
}

impl PendingCommandPreview {
    pub(crate) const fn describe(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::DescribeRevision,
            source_key: "m",
            failure_label: "jj describe",
            copy_status: None,
        }
    }

    pub(crate) const fn abandon(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::AbandonRevision,
            source_key: "a",
            failure_label: "jj abandon",
            copy_status: None,
        }
    }

    pub(crate) const fn new_change(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::NewRevision,
            source_key: "n",
            failure_label: "jj new",
            copy_status: None,
        }
    }

    pub(crate) const fn edit(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::EditRevision,
            source_key: "e",
            failure_label: "jj edit",
            copy_status: None,
        }
    }

    pub(crate) const fn undo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Undo,
            source_key: "u",
            failure_label: "jj undo",
            copy_status: None,
        }
    }

    pub(crate) const fn redo(preview: CommandPreview) -> Self {
        Self {
            preview,
            source_action: SourceAction::Redo,
            source_key: "U",
            failure_label: "jj redo",
            copy_status: None,
        }
    }
}

pub fn selected_new_parents(log: &LogView) -> Vec<String> {
    if log.has_marks() {
        return log.marked_revision_ids();
    }

    log.selected_revision_id()
        .map(ToOwned::to_owned)
        .into_iter()
        .collect()
}

pub fn describe_message_lines(rev: &str, message: &str) -> Vec<String> {
    vec![
        format!("Revision: {rev}"),
        format!("Message: {message}"),
        String::new(),
        "type message   enter preview   Ctrl-u clear   backspace edit   esc cancel".to_owned(),
    ]
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
        assert_eq!(describe.source_key, "m");
        assert_eq!(describe.failure_label, "jj describe");

        let abandon = PendingCommandPreview::abandon(preview());
        assert_eq!(abandon.source_action, SourceAction::AbandonRevision);
        assert_eq!(abandon.source_key, "a");
        assert_eq!(abandon.failure_label, "jj abandon");

        let redo = PendingCommandPreview::redo(preview());
        assert_eq!(redo.source_action, SourceAction::Redo);
        assert_eq!(redo.source_key, "U");
        assert_eq!(redo.failure_label, "jj redo");
    }

    #[test]
    fn describe_prompt_lines_include_revision_message_and_controls() {
        assert_eq!(
            describe_message_lines("abc123", "current message"),
            vec![
                "Revision: abc123".to_owned(),
                "Message: current message".to_owned(),
                String::new(),
                "type message   enter preview   Ctrl-u clear   backspace edit   esc cancel"
                    .to_owned(),
            ]
        );
    }

    #[test]
    fn selected_new_parents_use_short_selected_revision() {
        let log = log_view(["abcdefghijklmnop", "zyxwvutsrqponmlk"]);

        assert_eq!(selected_new_parents(&log), ["abcdefgh"]);
    }

    #[test]
    fn selected_new_parents_use_short_marked_revisions() {
        let mut log = log_view(["abcdefghijklmnop", "bbbbbbbbcccccccc", "zyxwvutsrqponmlk"]);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::ToggleMark);

        assert_eq!(selected_new_parents(&log), ["abcdefgh", "zyxwvuts"]);
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
