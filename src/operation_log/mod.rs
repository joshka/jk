//! `jj operation log` feature root.
//!
//! Operation-log views keep rendered `jj` output close to row metadata so copy,
//! search, refresh, operation detail navigation, and recovery action targets can
//! preserve user-visible `jj` presentation while using exact operation ids only
//! where needed.
//!
//! `rows` owns rendered-row grouping and metadata pairing, `view` owns the
//! selectable list surface, `detail` owns rendered operation show/diff documents,
//! and `actions` owns repository-wide and exact-operation recovery plans.

pub(crate) mod actions;
pub(crate) mod detail;
mod rows;
mod view;

#[cfg(test)]
pub(crate) use self::rows::OPERATION_ID_TEMPLATE;
pub(crate) use self::rows::{OperationLogItem, load_operation_log_entries};
pub use self::view::OperationLogView;

pub const BINDINGS: &[crate::command::Binding] = self::view::BINDINGS;

#[cfg(test)]
mod tests;
