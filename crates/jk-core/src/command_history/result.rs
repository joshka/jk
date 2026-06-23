use std::path::PathBuf;

use super::redaction::redact_text;

/// Process result summary retained by history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResultSummary {
    /// Exit status summary when the process started and exited.
    pub exit_status: Option<ExitStatusSummary>,
    /// Spawn or runner error when no exit status was available.
    pub spawn_error: Option<String>,
    /// Bounded stdout summary.
    pub stdout: StreamSummary,
    /// Bounded stderr summary.
    pub stderr: StreamSummary,
}

impl Default for CommandResultSummary {
    fn default() -> Self {
        Self {
            exit_status: None,
            spawn_error: None,
            stdout: StreamSummary::empty(),
            stderr: StreamSummary::empty(),
        }
    }
}

impl CommandResultSummary {
    pub(super) fn apply_retention(&mut self, retention: &OutputRetention) {
        self.stdout.apply_limit(retention.stdout_limit());
        self.stderr.apply_limit(retention.stderr_limit());
    }
}

/// Exit status data that does not require retaining platform-specific process types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExitStatusSummary {
    /// Numeric process exit code, when available.
    pub code: Option<i32>,
    /// Platform signal number, when available.
    pub signal: Option<i32>,
    /// Whether the command succeeded.
    pub success: bool,
}

impl ExitStatusSummary {
    /// Creates a normal exit-code status summary.
    #[must_use]
    pub const fn code(code: i32) -> Self {
        Self {
            code: Some(code),
            signal: None,
            success: code == 0,
        }
    }

    /// Creates a signal status summary.
    #[must_use]
    pub const fn signal(signal: i32) -> Self {
        Self {
            code: None,
            signal: Some(signal),
            success: false,
        }
    }
}

/// Bounded output retention policy for a command record.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OutputRetention {
    /// Retain only bounded stdout and stderr summaries.
    SummaryOnly {
        /// Maximum stdout snippet bytes.
        stdout_limit: usize,
        /// Maximum stderr snippet bytes.
        stderr_limit: usize,
    },
    /// Retain summaries and point to a local full-output artifact.
    Artifact {
        /// Maximum stdout snippet bytes.
        stdout_limit: usize,
        /// Maximum stderr snippet bytes.
        stderr_limit: usize,
        /// Local artifact path that owns full output.
        full_output_path: PathBuf,
    },
}

impl OutputRetention {
    /// Creates a summary-only retention policy.
    #[must_use]
    pub const fn summary_only(stdout_limit: usize, stderr_limit: usize) -> Self {
        Self::SummaryOnly {
            stdout_limit,
            stderr_limit,
        }
    }

    const fn stdout_limit(&self) -> usize {
        match self {
            Self::SummaryOnly { stdout_limit, .. } | Self::Artifact { stdout_limit, .. } => {
                *stdout_limit
            }
        }
    }

    const fn stderr_limit(&self) -> usize {
        match self {
            Self::SummaryOnly { stderr_limit, .. } | Self::Artifact { stderr_limit, .. } => {
                *stderr_limit
            }
        }
    }
}

/// Bounded and redacted stream summary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StreamSummary {
    /// Original stream byte length before redaction or truncation.
    pub byte_len: usize,
    /// Original stream line count before redaction or truncation.
    pub line_count: usize,
    /// Redacted snippet retained in memory.
    pub snippet: String,
    /// Whether the redacted snippet was truncated.
    pub truncated: bool,
    /// Whether any obvious secret-looking text was redacted.
    pub redacted: bool,
}

impl StreamSummary {
    /// Creates an empty stream summary.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            byte_len: 0,
            line_count: 0,
            snippet: String::new(),
            truncated: false,
            redacted: false,
        }
    }

    /// Creates a bounded stream summary from bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8], limit: usize) -> Self {
        Self::from_text(&String::from_utf8_lossy(bytes), limit).with_byte_len(bytes.len())
    }

    /// Creates a bounded stream summary from text.
    #[must_use]
    pub fn from_text(text: &str, limit: usize) -> Self {
        let byte_len = text.len();
        let line_count = text.lines().count();
        let (redacted_text, redacted) = redact_text(text);
        let (snippet, truncated) = truncate_to_boundary(&redacted_text, limit);

        Self {
            byte_len,
            line_count,
            snippet,
            truncated,
            redacted,
        }
    }

    fn with_byte_len(mut self, byte_len: usize) -> Self {
        self.byte_len = byte_len;
        self
    }

    fn apply_limit(&mut self, limit: usize) {
        let (snippet, truncated) = truncate_to_boundary(&self.snippet, limit);
        self.truncated |= truncated;
        self.snippet = snippet;
    }
}

fn truncate_to_boundary(text: &str, limit: usize) -> (String, bool) {
    if text.len() <= limit {
        return (text.to_owned(), false);
    }

    if limit == 0 {
        return (String::new(), true);
    }

    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_owned(), true)
}
