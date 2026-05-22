//! `jj bookmark list` feature root.
//!
//! Bookmark views keep rendered `jj` output close to row metadata so copy,
//! search, refresh, and action targeting can preserve user-visible `jj`
//! presentation while using trusted parsed fields only where needed.
//!
//! `rows` owns metadata pairing and row classification, `view` owns the
//! selectable bookmark list surface, `targets` owns action-target safety
//! checks, and `actions` owns the exact bookmark mutation plans.

pub(crate) mod actions;
mod rows;
mod targets;
mod view;

pub(crate) use self::rows::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
pub use self::view::BookmarksView;

pub const BINDINGS: &[crate::command::Binding] = self::view::BINDINGS;

#[cfg(test)]
mod tests;
