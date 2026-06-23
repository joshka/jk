use jk_cli::{WorkspaceListSnapshot, WorkspaceSummary};
use jk_tui::log_view::LogAction;
use jk_tui::rendered_view::RenderedAction;
use jk_tui::workspaces_view::{WorkspaceViewRow, WorkspaceViewSnapshot, WorkspacesAction};

pub fn workspace_view_snapshot(snapshot: WorkspaceListSnapshot) -> WorkspaceViewSnapshot {
    let rows = snapshot
        .workspaces
        .into_iter()
        .map(workspace_view_row)
        .collect();
    WorkspaceViewSnapshot::new(rows).with_title(snapshot.title)
}

fn workspace_view_row(workspace: WorkspaceSummary) -> WorkspaceViewRow {
    let root_display = workspace
        .root
        .as_ref()
        .map_or_else(|| "(no root)".to_owned(), |root| root.display().to_string());
    let mut row = WorkspaceViewRow::new(workspace.name, root_display, workspace.current);
    if let Some(root) = workspace.root {
        row = row.with_root(root);
    }
    if let Some(change_id) = workspace.change_id {
        row = row.with_change_id(change_id);
    }
    if let Some(commit_id) = workspace.commit_id {
        row = row.with_commit_id(commit_id);
    }
    row
}

pub fn update_stale_success_message(
    workspace_name: &str,
    command: &str,
    stderr: &str,
    stdout: &str,
) -> String {
    let command = compact_update_stale_command(command);
    let output = compact_command_output(stderr).or_else(|| compact_command_output(stdout));
    match output {
        Some(output) => format!("updated {workspace_name} via {command}: {output}"),
        None => format!("updated {workspace_name} via {command}"),
    }
}

pub const fn workspace_action_for_log_action(action: LogAction) -> WorkspacesAction {
    match action {
        LogAction::Previous => WorkspacesAction::Previous,
        LogAction::Next => WorkspacesAction::Next,
        LogAction::ScrollPreviousLine => WorkspacesAction::ScrollPreviousLine,
        LogAction::ScrollNextLine => WorkspacesAction::ScrollNextLine,
        LogAction::PagePrevious => WorkspacesAction::Previous,
        LogAction::PageNext => WorkspacesAction::Next,
        LogAction::First => WorkspacesAction::First,
        LogAction::Last => WorkspacesAction::Last,
        LogAction::Refresh => WorkspacesAction::Refresh,
        LogAction::OpenDiff => WorkspacesAction::OpenDiff,
        LogAction::ToggleExpanded => WorkspacesAction::OpenLog,
        LogAction::ToggleHelp => WorkspacesAction::ToggleHelp,
        LogAction::Quit => WorkspacesAction::Quit,
        LogAction::Home | LogAction::Log | LogAction::CollapseExpanded => {
            WorkspacesAction::ReturnBack
        }
        _ => WorkspacesAction::ReturnBack,
    }
}

pub const fn workspace_inspection_action_for_log_action(action: LogAction) -> RenderedAction {
    match action {
        LogAction::Previous => RenderedAction::ScrollPrevious,
        LogAction::Next => RenderedAction::ScrollNext,
        LogAction::ScrollPreviousLine => RenderedAction::ScrollPrevious,
        LogAction::ScrollNextLine => RenderedAction::ScrollNext,
        LogAction::PagePrevious => RenderedAction::PagePrevious,
        LogAction::PageNext => RenderedAction::PageNext,
        LogAction::ToggleMark => RenderedAction::PageNext,
        LogAction::ClearMarks => RenderedAction::Ignore,
        LogAction::First => RenderedAction::First,
        LogAction::Last => RenderedAction::Last,
        LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        LogAction::Refresh => RenderedAction::Refresh,
        LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    }
}

fn compact_update_stale_command(command: &str) -> &str {
    command
        .strip_prefix("jj -R ")
        .and_then(|command| command.split_once(" workspace update-stale"))
        .map_or(command, |_| "workspace update-stale")
}

fn compact_command_output(output: &str) -> Option<String> {
    let first_line = output.lines().find(|line| !line.trim().is_empty())?.trim();
    const MAX_STATUS_CHARS: usize = 120;
    if first_line.chars().count() <= MAX_STATUS_CHARS {
        Some(first_line.to_owned())
    } else {
        let mut summary = first_line
            .chars()
            .take(MAX_STATUS_CHARS.saturating_sub(3))
            .collect::<String>();
        summary.push_str("...");
        Some(summary)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn snapshot_mapping_preserves_row_fields() {
        let snapshot = WorkspaceListSnapshot {
            title: "jj workspace list".to_owned(),
            workspaces: vec![WorkspaceSummary {
                name: "dogfood".to_owned(),
                root: Some(PathBuf::from("/repo/dogfood")),
                current: true,
                change_id: Some("abc123".to_owned()),
                commit_id: Some("def456".to_owned()),
            }],
        };

        let snapshot = workspace_view_snapshot(snapshot);

        assert_eq!(snapshot.title(), "jj workspace list");
        assert_eq!(
            snapshot.rows(),
            &[WorkspaceViewRow::new("dogfood", "/repo/dogfood", true)
                .with_root("/repo/dogfood")
                .with_change_id("abc123")
                .with_commit_id("def456")]
        );
    }

    #[test]
    fn update_stale_success_message_prefers_stderr_then_stdout() {
        assert_eq!(
            update_stale_success_message(
                "dogfood",
                "jj -R /repo/dogfood workspace update-stale",
                "warning line",
                "stdout line",
            ),
            "updated dogfood via workspace update-stale: warning line"
        );

        assert_eq!(
            update_stale_success_message(
                "dogfood",
                "jj -R /repo/dogfood workspace update-stale",
                "",
                "stdout line",
            ),
            "updated dogfood via workspace update-stale: stdout line"
        );

        assert_eq!(
            update_stale_success_message("dogfood", "jj workspace update-stale", "", ""),
            "updated dogfood via jj workspace update-stale"
        );
    }

    #[test]
    fn workspace_action_mapping_preserves_workspace_policy() {
        assert_eq!(
            workspace_action_for_log_action(LogAction::Next),
            WorkspacesAction::Next
        );
        assert_eq!(
            workspace_action_for_log_action(LogAction::PageNext),
            WorkspacesAction::Next
        );
        assert_eq!(
            workspace_action_for_log_action(LogAction::OpenDiff),
            WorkspacesAction::OpenDiff
        );
        assert_eq!(
            workspace_action_for_log_action(LogAction::ToggleExpanded),
            WorkspacesAction::OpenLog
        );
        assert_eq!(
            workspace_action_for_log_action(LogAction::Home),
            WorkspacesAction::ReturnBack
        );
    }

    #[test]
    fn workspace_inspection_action_mapping_preserves_rendered_policy() {
        assert_eq!(
            workspace_inspection_action_for_log_action(LogAction::Next),
            RenderedAction::ScrollNext
        );
        assert_eq!(
            workspace_inspection_action_for_log_action(LogAction::ToggleMark),
            RenderedAction::PageNext
        );
        assert_eq!(
            workspace_inspection_action_for_log_action(LogAction::ClearMarks),
            RenderedAction::Ignore
        );
        assert_eq!(
            workspace_inspection_action_for_log_action(LogAction::Refresh),
            RenderedAction::Refresh
        );
        assert_eq!(
            workspace_inspection_action_for_log_action(LogAction::OpenDiff),
            RenderedAction::ReturnToLog
        );
    }
}
