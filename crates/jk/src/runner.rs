use jk_cli::{RecordingJjCommandRunner, SystemJjCommandRunner};
use jk_core::{CommandHistory, CommandSource};

pub const fn recording_runner(
    history: &mut CommandHistory,
    source: CommandSource,
) -> RecordingJjCommandRunner<'_, SystemJjCommandRunner> {
    RecordingJjCommandRunner::new(SystemJjCommandRunner, history, source)
}
