//! Bookmark action-target resolution for selected rows.
//!
//! The resolver owns the fail-closed policy between rendered bookmark rows
//! and mutation plans. It only enables forget, track, and untrack when the
//! selected row and its visible peers carry enough trusted metadata to name an
//! exact local or remote bookmark target. Unknown, filtered, drifted, or
//! ambiguous metadata stays disabled with the existing user-facing wording.

mod helpers;
mod peers;
mod resolver;

pub use resolver::BookmarkActionTargetResolver;
