//! Applies the TUI viewport width at the process boundary without altering global environment.

use jk_cli::{JjCommandRunner, RecordingJjCommandRunner, SystemJjCommandRunner};
use jk_core::{CommandHistory, CommandSource, JjCommandSpec};

/// Gives each jj invocation the width left after jk's interaction gutter.
pub(crate) struct TuiJjCommandRunner<R> {
    inner: R,
    columns: Option<u16>,
}

impl<R> TuiJjCommandRunner<R> {
    pub(crate) fn new(inner: R) -> Self {
        let columns = crossterm::terminal::size()
            .ok()
            .map(|(columns, _)| jk_tui::content_width(columns).max(1));
        Self { inner, columns }
    }
}

impl<R: JjCommandRunner> JjCommandRunner for TuiJjCommandRunner<R> {
    fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<std::process::Output> {
        match self.columns {
            Some(columns) => self.inner.run(&spec.clone().with_display_columns(columns)),
            None => self.inner.run(spec),
        }
    }
}

pub(crate) fn system_runner() -> TuiJjCommandRunner<SystemJjCommandRunner> {
    TuiJjCommandRunner::new(SystemJjCommandRunner)
}

pub fn recording_runner(
    history: &mut CommandHistory,
    source: CommandSource,
) -> RecordingJjCommandRunner<'_, TuiJjCommandRunner<SystemJjCommandRunner>> {
    RecordingJjCommandRunner::new(system_runner(), history, source)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WidthProbe;

    impl JjCommandRunner for WidthProbe {
        fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<std::process::Output> {
            assert_eq!(spec.display_columns(), Some(37));
            assert_eq!(spec.preview(), "jj log");
            Err(std::io::Error::other("probe complete"))
        }
    }

    #[test]
    fn renderer_width_reaches_runner_without_changing_command() {
        let mut runner = TuiJjCommandRunner {
            inner: WidthProbe,
            columns: Some(37),
        };
        let spec = JjCommandSpec::render_read_only(["log"]);
        assert!(runner.run(&spec).is_err());
        assert_eq!(spec.display_columns(), None);
    }
}
