use std::collections::VecDeque;
use std::io;
use std::process::Output;

use jk_cli::{DiffFormat, DiffQuery, JjCommandRunner};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::diff_view::DiffView;
use jk_tui::log_view::LogView;
use jk_tui::workspaces_view::{WorkspaceViewRow, WorkspaceViewSnapshot, WorkspacesView};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use crate::state::{AppState, AppView};

pub fn diff_app_view(change_id: &str) -> AppView {
    AppView::Diff {
        view: diff_view(change_id),
        query: diff_query(change_id),
    }
}

pub fn diff_query(change_id: &str) -> DiffQuery {
    DiffQuery::Revision {
        rev: change_id.to_owned(),
        format: DiffFormat::Patch,
    }
}

pub fn diff_view(change_id: &str) -> DiffView {
    DiffView::from_error(
        change_id,
        format!("jj diff -r {change_id}"),
        "synthetic diff fixture".to_owned(),
    )
}

pub fn real_diff_view(change_id: &str) -> DiffView {
    DiffView::new(
        jk_core::DiffSnapshot::new(
            change_id,
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        )
        .with_title(format!("jj diff -r {change_id}")),
    )
}

pub fn log_app_view(change_id: &str) -> AppView {
    log_app_view_with_changes([change_id])
}

pub fn log_app_view_with_description(change_id: &str, description: &str) -> AppView {
    AppView::Log(LogView::new(
        jk_core::LogSnapshot::new(
            format!("@  {change_id} {description}\n"),
            vec![jk_core::LogEntry::new(change_id, "commit", description).with_rendered_line(0)],
        )
        .with_title("jj log"),
    ))
}

pub fn log_app_view_with_changes<const N: usize>(change_ids: [&str; N]) -> AppView {
    let rendered = change_ids
        .iter()
        .enumerate()
        .map(|(index, change_id)| {
            let marker = if index == 0 { "@" } else { "○" };
            format!("{marker}  {change_id} {change_id} summary\n")
        })
        .collect::<String>();
    let entries = change_ids
        .iter()
        .enumerate()
        .map(|(index, change_id)| {
            jk_core::LogEntry::new(*change_id, "commit", format!("{change_id} summary"))
                .with_rendered_line(index)
        })
        .collect();
    AppView::Log(LogView::new(
        jk_core::LogSnapshot::new(rendered, entries).with_title("jj log"),
    ))
}

pub fn active_log_status(state: &mut AppState) -> String {
    let AppView::Log(log) = state.views.active_mut() else {
        panic!("expected active log view");
    };
    let backend = TestBackend::new(96, 4);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => match error {},
    };
    let draw_result = terminal.draw(|frame| log.render(frame));
    assert!(draw_result.is_ok());
    buffer_line(terminal.backend().buffer(), 3)
}

pub fn buffer_line(buffer: &ratatui::buffer::Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect::<String>()
}

pub fn workspace_app_view() -> WorkspacesView {
    WorkspacesView::new(WorkspaceViewSnapshot::new(vec![
        WorkspaceViewRow::new("default", "/repo/default", false).with_root("/repo/default"),
        WorkspaceViewRow::new("dogfood", "/repo/dogfood", true).with_root("/repo/dogfood"),
    ]))
}

pub fn append_history_record(
    history: &mut CommandHistory,
    spec: jk_core::JjCommandSpec,
    view: SourceView,
    action: SourceAction,
) {
    append_history_record_with_operation_id(history, spec, view, action, None);
}

pub fn append_history_record_with_operation_id(
    history: &mut CommandHistory,
    spec: jk_core::JjCommandSpec,
    view: SourceView,
    action: SourceAction,
    operation_id: Option<&str>,
) {
    let start = jk_core::CommandRecordStart::from_spec(&spec, CommandSource::new(view, action))
        .with_started_at(std::time::SystemTime::UNIX_EPOCH);
    let mut finish = jk_core::CommandRecordFinish::from_exit_code(
        0,
        "",
        "",
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1),
    );
    finish.operation_id = operation_id.map(str::to_owned);
    history.append(start, finish);
}

pub fn output(code: i32, stdout: &str, stderr: &str) -> Output {
    Output {
        status: exit_status(code),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

pub struct SequencedRunner {
    outputs: VecDeque<io::Result<Output>>,
}

impl SequencedRunner {
    pub(crate) fn successes(outputs: Vec<Output>) -> Self {
        Self {
            outputs: outputs.into_iter().map(Ok).collect(),
        }
    }
}

impl JjCommandRunner for SequencedRunner {
    fn run(&mut self, _spec: &jk_core::JjCommandSpec) -> io::Result<Output> {
        self.outputs
            .pop_front()
            .expect("runner called too many times")
    }
}

#[cfg(unix)]
fn exit_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;

    std::process::ExitStatus::from_raw(code << 8)
}

#[cfg(not(unix))]
fn exit_status(code: i32) -> std::process::ExitStatus {
    std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
        .args(if cfg!(windows) {
            vec!["/C".into(), format!("exit {code}").into()]
        } else {
            vec!["-c".into(), format!("exit {code}").into()]
        })
        .status()
        .unwrap()
}
