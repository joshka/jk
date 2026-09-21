//! Cancellable background execution for read-only refresh work.
//!
//! Starting a log refresh cancels the previous request without blocking the application thread.
//! Dropping the runner cancels pending work and joins its workers.
//!
//! The runner labels results by request generation. Before applying a current result, the caller
//! must also check that its source still matches the log view. Superseded results can contribute
//! command history but must not replace content or status. On failure, the caller keeps the last
//! usable snapshot and displays the error in the view's status line.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use jk_cli::{
    CancellableSystemJjCommandRunner, CancellationToken, JjLog, RecordingJjCommandRunner,
};
use jk_core::{CommandHistory, CommandSource, LogSnapshot, SourceAction, SourceView};

/// Host policy for refreshes requested after a command preview completes.
///
/// Defaults to explicit user refresh. If a preview opts in, the host may refresh its originating
/// view after a successful command.
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
/// Whether a result belongs to the latest request.
pub enum RefreshCompletion<T> {
    /// The newest request; the caller must still check that its source matches the view.
    Current(T),
    /// An older or cancelled request; its result must not replace view content or status.
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
            let system = crate::runner::TuiJjCommandRunner::new(system);
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

    /// Returns completed results, labelling those superseded or cancelled before this call.
    pub fn drain(&mut self) -> Vec<RefreshCompletion<LogRefreshResult>> {
        self.tasks.drain()
    }

    /// Requests cancellation without waiting and marks the eventual result as superseded.
    pub fn cancel_active(&mut self) {
        self.tasks.cancel_active();
    }

    #[cfg(test)]
    pub(crate) fn start_with(
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

    #[test]
    fn dropping_runner_cancels_and_joins_its_worker() {
        let (started_tx, started_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let mut runner = LogRefreshRunner::default();
        runner.start_with(move |cancellation| {
            started_tx.send(()).expect("report worker start");
            while !cancellation.is_cancelled() {
                thread::yield_now();
            }
            finished_tx.send(()).expect("report cancellation observed");
            result("cancelled")
        });
        started_rx.recv().expect("worker started");

        drop(runner);

        finished_rx
            .try_recv()
            .expect("drop joined the cancelled worker");
    }
}
