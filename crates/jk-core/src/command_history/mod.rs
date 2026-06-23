//! In-memory command history records.
//!
//! The history model stores command specs as structured data so future runner and UI layers can
//! show what ran without reconstructing process details from rendered output.

mod history;
mod identity;
mod record;
mod redaction;
mod result;

pub use history::{CommandHistory, CommandRecordId, PendingCommandRecord};
pub use identity::{
    CommandExecutionContext, CommandFamily, CommandIdentity, CommandSource, GlobalOptionsSnapshot,
    SourceAction, SourceView,
};
pub use record::{CommandRecord, CommandRecordFinish, CommandRecordStart, CommandTiming};
pub use result::{CommandResultSummary, ExitStatusSummary, OutputRetention, StreamSummary};

const DEFAULT_STREAM_LIMIT: usize = 8 * 1024;
const REDACTED: &str = "<redacted>";

#[cfg(test)]
mod tests;
