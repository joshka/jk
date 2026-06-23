use std::time::{Duration, SystemTime};

use super::{
    CommandExecutionContext, CommandIdentity, CommandRecordId, CommandResultSummary, CommandSource,
    DEFAULT_STREAM_LIMIT, ExitStatusSummary, OutputRetention, StreamSummary,
};
use crate::{ExecutionMode, JjCommandSpec, RefreshPlan, SafetyClass};

/// One retained command-history record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRecord {
    /// Stable id for this in-memory history.
    pub id: CommandRecordId,
    /// Command identity captured from a typed spec.
    pub command: CommandIdentity,
    /// View and action that triggered the command.
    pub source: CommandSource,
    /// Execution context captured before running the command.
    pub context: CommandExecutionContext,
    /// Start and finish timing.
    pub timing: CommandTiming,
    /// Process or spawn result summary.
    pub result: CommandResultSummary,
    /// Output retention policy used for this record.
    pub retention: OutputRetention,
    /// Refresh behavior requested by the command spec.
    pub refresh: RefreshPlan,
    /// Safety classification requested by the command spec.
    pub safety: SafetyClass,
    /// Execution mode requested by the command spec.
    pub execution_mode: ExecutionMode,
    /// Operation id reported by `jj`, when cheaply available.
    pub operation_id: Option<String>,
}

impl CommandRecord {
    pub(super) fn started(id: CommandRecordId, input: CommandRecordStart) -> Self {
        Self {
            id,
            command: input.command,
            source: input.source,
            context: input.context,
            timing: CommandTiming {
                started_at: input.started_at,
                ended_at: None,
                duration: None,
            },
            result: CommandResultSummary::default(),
            retention: input.retention,
            refresh: input.refresh,
            safety: input.safety,
            execution_mode: input.execution_mode,
            operation_id: None,
        }
    }

    pub(super) fn finish(&mut self, mut finish: CommandRecordFinish) -> bool {
        if self.timing.ended_at.is_some() {
            return false;
        }

        let duration = finish
            .ended_at
            .duration_since(self.timing.started_at)
            .unwrap_or_default();
        finish.result.apply_retention(&self.retention);
        self.timing.ended_at = Some(finish.ended_at);
        self.timing.duration = Some(duration);
        self.result = finish.result;
        self.operation_id = finish.operation_id;
        true
    }
}

/// Fields required when a command starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRecordStart {
    /// Command identity captured from a typed spec.
    pub command: CommandIdentity,
    /// View and action that triggered the command.
    pub source: CommandSource,
    /// Execution context captured before running the command.
    pub context: CommandExecutionContext,
    /// Start time.
    pub started_at: SystemTime,
    /// Output retention policy for this record.
    pub retention: OutputRetention,
    /// Refresh behavior requested by the command spec.
    pub refresh: RefreshPlan,
    /// Safety classification requested by the command spec.
    pub safety: SafetyClass,
    /// Execution mode requested by the command spec.
    pub execution_mode: ExecutionMode,
}

impl CommandRecordStart {
    /// Creates start data from a typed `jj` command spec and source.
    #[must_use]
    pub fn from_spec(spec: &JjCommandSpec, source: CommandSource) -> Self {
        Self {
            command: CommandIdentity::from_spec(spec),
            source,
            context: CommandExecutionContext::from_spec(spec),
            started_at: SystemTime::now(),
            retention: OutputRetention::summary_only(DEFAULT_STREAM_LIMIT, DEFAULT_STREAM_LIMIT),
            refresh: spec.refresh_plan(),
            safety: spec.safety(),
            execution_mode: spec.mode(),
        }
    }

    /// Sets the start time for deterministic tests or caller-owned timing.
    #[must_use]
    pub const fn with_started_at(mut self, started_at: SystemTime) -> Self {
        self.started_at = started_at;
        self
    }

    /// Sets the output retention policy.
    #[must_use]
    pub fn with_retention(mut self, retention: OutputRetention) -> Self {
        self.retention = retention;
        self
    }
}

/// Fields required when a command finishes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRecordFinish {
    /// Finish time.
    pub ended_at: SystemTime,
    /// Process or spawn result summary.
    pub result: CommandResultSummary,
    /// Operation id reported by `jj`, when cheaply available.
    pub operation_id: Option<String>,
}

impl CommandRecordFinish {
    /// Creates finish data for a process result.
    #[must_use]
    pub const fn from_result(
        result: CommandResultSummary,
        operation_id: Option<String>,
        ended_at: SystemTime,
    ) -> Self {
        Self {
            ended_at,
            result,
            operation_id,
        }
    }

    /// Creates finish data for a process exit code and captured output.
    #[must_use]
    pub fn from_exit_code(
        code: i32,
        stdout: impl AsRef<[u8]>,
        stderr: impl AsRef<[u8]>,
        ended_at: SystemTime,
    ) -> Self {
        Self::from_result(
            CommandResultSummary {
                exit_status: Some(ExitStatusSummary::code(code)),
                spawn_error: None,
                stdout: StreamSummary::from_bytes(stdout.as_ref(), DEFAULT_STREAM_LIMIT),
                stderr: StreamSummary::from_bytes(stderr.as_ref(), DEFAULT_STREAM_LIMIT),
            },
            None,
            ended_at,
        )
    }

    /// Creates finish data for a command that failed before producing an exit status.
    #[must_use]
    pub fn from_spawn_error(
        error: impl Into<String>,
        stdout: impl AsRef<[u8]>,
        stderr: impl AsRef<[u8]>,
        ended_at: SystemTime,
    ) -> Self {
        Self::from_result(
            CommandResultSummary {
                exit_status: None,
                spawn_error: Some(error.into()),
                stdout: StreamSummary::from_bytes(stdout.as_ref(), DEFAULT_STREAM_LIMIT),
                stderr: StreamSummary::from_bytes(stderr.as_ref(), DEFAULT_STREAM_LIMIT),
            },
            None,
            ended_at,
        )
    }
}

/// Timing captured for a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandTiming {
    /// Time at which the command was recorded as started.
    pub started_at: SystemTime,
    /// Time at which the command finished.
    pub ended_at: Option<SystemTime>,
    /// Duration between start and finish.
    pub duration: Option<Duration>,
}
