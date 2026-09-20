//! Cancellable background execution for read-only refresh work.
//!
//! The first production slice deliberately owns only explicit log refresh. Preview-driven refresh
//! may use the same runner only after the preview opts in and the host validates that the work is
//! read-only. Timer-, watcher-, and repository-wide auto-refresh remain out of scope until every
//! target has a stable preservation identity.
//!
//! Result promotion follows two independent checks: the request generation must still be current,
//! and the loaded source must still identify the active view. A superseded success or failure may
//! contribute command history, but it cannot replace content or status. An accepted failure keeps
//! the last usable body and exposes the error in that view's status line.
//!
//! Refreshable views preserve interaction state by underlying-object identity: logs use full change
//! ids for selection, ordered marks, and expansion; diffs use file paths and exact path-plus-hunk
//! headers for folds; workspaces use canonical roots before display names; operation logs use full
//! operation ids. Scroll offsets are retained and clamped. Help and input overlays live outside the
//! refreshed snapshot and remain open. If a stable object disappears, list views fall back to the
//! current row, then the nearest old index, then the first row or no selection.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use jk_cli::{
    CancellableSystemJjCommandRunner, CancellationToken, JjLog, RecordingJjCommandRunner,
};
use jk_core::{CommandHistory, CommandSource, LogSnapshot, SourceAction, SourceView};

/// Host policy for refreshes requested after a command preview completes.
///
/// The default is deliberately disabled. A future preview may opt in to refreshing its originating
/// live view after a successful mutation; it must not start arbitrary background work itself.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PreviewRefreshPolicy {
    /// Keep refresh under explicit user control.
    #[default]
    Manual,
    /// Allow the host to refresh the originating live view after successful execution.
    OptInAfterSuccess,
}

#[derive(Debug)]
/// Completed log refresh data returned to the application thread.
pub struct LogRefreshResult {
    /// Exact log source used to produce the result.
    pub source: JjLog,
    /// Loaded snapshot or user-facing loader failure.
    pub outcome: Result<LogSnapshot, String>,
    /// Command records collected without sharing application state across threads.
    pub history: CommandHistory,
}

#[derive(Debug)]
/// Whether a completion still owns promotion rights for its target.
pub enum RefreshCompletion<T> {
    /// The newest request, eligible for target validation and promotion.
    Current(T),
    /// An older request whose state result must not be promoted.
    Superseded(T),
}

/// Owns the single active explicit log refresh and its completed-result queue.
#[derive(Debug, Default)]
pub struct LogRefreshRunner {
    tasks: CancellableTaskRunner<LogRefreshResult>,
}

impl LogRefreshRunner {
    /// Starts an explicit user-requested refresh and cancels any older log refresh.
    pub fn start_manual(&mut self, source: JjLog) {
        self.tasks.start(move |cancellation| {
            let mut history = CommandHistory::default();
            let command_source =
                CommandSource::new(SourceView::Log, SourceAction::Refresh).with_key("r");
            let system = CancellableSystemJjCommandRunner::new(cancellation);
            let mut runner = RecordingJjCommandRunner::new(system, &mut history, command_source);
            let outcome = source
                .load_with_runner(&mut runner)
                .map_err(|error| error.to_string());
            LogRefreshResult {
                source,
                outcome,
                history,
            }
        });
    }

    /// Drains completed work, labelling results that were superseded before promotion.
    pub fn drain(&mut self) -> Vec<RefreshCompletion<LogRefreshResult>> {
        self.tasks.drain()
    }

    /// Cancels and retires the active request so its eventual result cannot be promoted.
    pub fn cancel_active(&mut self) {
        self.tasks.cancel_active();
    }

    #[cfg(test)]
    fn start_with(
        &mut self,
        task: impl FnOnce(CancellationToken) -> LogRefreshResult + Send + 'static,
    ) {
        self.tasks.start(task);
    }

    #[cfg(test)]
    fn recv(&mut self) -> RefreshCompletion<LogRefreshResult> {
        self.tasks.recv()
    }
}

#[derive(Debug)]
struct TaskMessage<T> {
    generation: u64,
    value: T,
}

#[derive(Debug)]
struct ActiveTask {
    generation: u64,
    cancellation: CancellationToken,
}

#[derive(Debug)]
struct CancellableTaskRunner<T> {
    next_generation: u64,
    active: Option<ActiveTask>,
    sender: Sender<TaskMessage<T>>,
    receiver: Receiver<TaskMessage<T>>,
    workers: Vec<JoinHandle<()>>,
}

impl<T> Default for CancellableTaskRunner<T> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            next_generation: 1,
            active: None,
            sender,
            receiver,
            workers: Vec::new(),
        }
    }
}

impl<T: Send + 'static> CancellableTaskRunner<T> {
    fn start(&mut self, task: impl FnOnce(CancellationToken) -> T + Send + 'static) {
        if let Some(active) = &self.active {
            active.cancellation.cancel();
        }
        self.reap_finished_workers();

        let generation = self.next_generation;
        self.next_generation = self.next_generation.saturating_add(1);
        let cancellation = CancellationToken::new();
        let worker_cancellation = cancellation.clone();
        let sender = self.sender.clone();
        let worker = thread::spawn(move || {
            let value = task(worker_cancellation);
            let _ = sender.send(TaskMessage { generation, value });
        });
        self.active = Some(ActiveTask {
            generation,
            cancellation,
        });
        self.workers.push(worker);
    }

    fn drain(&mut self) -> Vec<RefreshCompletion<T>> {
        let mut completions = Vec::new();
        while let Ok(message) = self.receiver.try_recv() {
            completions.push(self.classify(message));
        }
        self.reap_finished_workers();
        completions
    }

    fn cancel_active(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
        }
    }

    #[cfg(test)]
    fn recv(&mut self) -> RefreshCompletion<T> {
        let message = self
            .receiver
            .recv()
            .expect("test refresh worker should send a completion");
        self.classify(message)
    }

    fn classify(&mut self, message: TaskMessage<T>) -> RefreshCompletion<T> {
        let is_current = self
            .active
            .as_ref()
            .is_some_and(|active| active.generation == message.generation);
        if is_current {
            self.active = None;
            RefreshCompletion::Current(message.value)
        } else {
            RefreshCompletion::Superseded(message.value)
        }
    }

    fn reap_finished_workers(&mut self) {
        let mut index = 0;
        while index < self.workers.len() {
            if self.workers[index].is_finished() {
                let worker = self.workers.swap_remove(index);
                let _ = worker.join();
            } else {
                index += 1;
            }
        }
    }
}

impl<T> Drop for CancellableTaskRunner<T> {
    fn drop(&mut self) {
        if let Some(active) = &self.active {
            active.cancellation.cancel();
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use jk_core::LogEntry;
    use jk_tui::log_view::{LogAction, LogView};

    use super::*;

    fn result(label: &str) -> LogRefreshResult {
        LogRefreshResult {
            source: JjLog::default(),
            outcome: Ok(LogSnapshot::new(label, Vec::new())),
            history: CommandHistory::default(),
        }
    }

    #[test]
    fn superseded_work_is_cancelled_and_cannot_be_current() {
        let (first_started, first_started_rx) = mpsc::channel();
        let (release_first, release_first_rx) = mpsc::channel();
        let (second_started, second_started_rx) = mpsc::channel();
        let (release_second, release_second_rx) = mpsc::channel();
        let mut runner = LogRefreshRunner::default();

        runner.start_with(move |cancellation| {
            first_started.send(()).expect("first start receiver");
            release_first_rx.recv().expect("first release");
            assert!(cancellation.is_cancelled());
            result("first")
        });
        first_started_rx.recv().expect("first worker starts");

        runner.start_with(move |cancellation| {
            second_started.send(()).expect("second start receiver");
            release_second_rx.recv().expect("second release");
            assert!(!cancellation.is_cancelled());
            result("second")
        });
        second_started_rx.recv().expect("second worker starts");

        release_second.send(()).expect("release second worker");
        assert!(matches!(runner.recv(), RefreshCompletion::Current(_)));

        release_first.send(()).expect("release first worker");
        assert!(matches!(runner.recv(), RefreshCompletion::Superseded(_)));
    }

    #[test]
    fn navigation_continues_while_fake_slow_refresh_is_pending() {
        let (started, started_rx) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let mut runner = LogRefreshRunner::default();
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));

        runner.start_with(move |_| {
            started.send(()).expect("slow worker start receiver");
            release_rx.recv().expect("slow worker release");
            result("refreshed")
        });
        started_rx.recv().expect("slow worker starts");

        let _ = view.apply(LogAction::Next);

        assert_eq!(view.selected_change_id(), Some("bbb"));
        release.send(()).expect("release slow worker");
        assert!(matches!(runner.recv(), RefreshCompletion::Current(_)));
    }

    #[test]
    fn preview_auto_refresh_is_opt_in_and_disabled_by_default() {
        assert_eq!(
            PreviewRefreshPolicy::default(),
            PreviewRefreshPolicy::Manual
        );
        assert_ne!(
            PreviewRefreshPolicy::default(),
            PreviewRefreshPolicy::OptInAfterSuccess
        );
    }
}
