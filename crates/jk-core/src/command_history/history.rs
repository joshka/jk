use std::collections::VecDeque;

use super::{CommandRecord, CommandRecordFinish, CommandRecordStart};

/// A stable command-history id for the current process.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandRecordId(u64);

impl CommandRecordId {
    /// Returns the numeric id assigned by the in-memory history.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Recent command records retained in memory.
#[derive(Clone, Debug)]
pub struct CommandHistory {
    records: VecDeque<CommandRecord>,
    limit: usize,
    next_id: u64,
}

impl CommandHistory {
    /// Creates an empty command history with a maximum record count.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            records: VecDeque::new(),
            limit,
            next_id: 1,
        }
    }

    /// Returns the maximum number of records retained in memory.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Starts a command record before execution.
    pub fn start(&mut self, input: CommandRecordStart) -> PendingCommandRecord {
        let id = self.allocate_id();
        let record = CommandRecord::started(id, input);
        self.push_record(record);
        PendingCommandRecord { id }
    }

    /// Finishes a pending command record.
    ///
    /// If the pending record has already been evicted by the history limit, this method does
    /// nothing.
    pub fn finish(&mut self, pending: &PendingCommandRecord, finish: CommandRecordFinish) -> bool {
        if let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.id == pending.id)
        {
            return record.finish(finish);
        }
        false
    }

    /// Appends an already-completed command record.
    ///
    /// This is useful for synchronous command paths where the caller already has all timing and
    /// result data at the recording boundary.
    pub fn append(
        &mut self,
        input: CommandRecordStart,
        finish: CommandRecordFinish,
    ) -> CommandRecordId {
        let pending = self.start(input);
        let id = pending.id;
        self.finish(&pending, finish);
        id
    }

    /// Returns retained records from oldest to newest.
    pub fn records(&self) -> impl DoubleEndedIterator<Item = &CommandRecord> {
        self.records.iter()
    }

    fn allocate_id(&mut self) -> CommandRecordId {
        let id = CommandRecordId(self.next_id);
        self.next_id += 1;
        id
    }

    fn push_record(&mut self, record: CommandRecord) {
        if self.limit == 0 {
            return;
        }

        while self.records.len() >= self.limit {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(128)
    }
}

/// A pending command record handle returned by [`CommandHistory::start`].
#[derive(Debug, Eq, PartialEq)]
pub struct PendingCommandRecord {
    id: CommandRecordId,
}

impl PendingCommandRecord {
    /// Returns the id of the pending command record.
    #[must_use]
    pub const fn id(&self) -> CommandRecordId {
        self.id
    }
}
