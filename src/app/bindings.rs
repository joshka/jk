use crossterm::event::KeyCode;

use crate::command::Binding;
use crate::command::Command;

/// Global bindings that resolve before view-local bindings.
///
/// Feature views add their own commands on top; these entries stay focused on app-level dispatch
/// and shared mode changes.
pub const APP_BINDINGS: &[Binding] = &[
    Binding::char('q', Command::Quit),
    Binding::code(KeyCode::Esc, Command::Quit),
    Binding::char('?', Command::Help),
    Binding::char('/', Command::SearchPrompt),
    Binding::char('W', Command::PromptLogRevset),
    Binding::char('S', Command::OpenStatus),
    Binding::char('R', Command::OpenResolve),
    Binding::char('B', Command::OpenBookmarks),
    Binding::char('X', Command::OpenWorkspaces),
    Binding::char('O', Command::OpenOperationLog),
    Binding::char('D', Command::Describe),
    Binding::char('C', Command::Commit),
    Binding::char('b', Command::BookmarkCreate),
    Binding::chord('b', 'c', Command::BookmarkCreate),
    Binding::chord('b', 'r', Command::BookmarkRename),
    Binding::chord('b', 'f', Command::BookmarkForget),
    Binding::chord('b', 't', Command::BookmarkTrack),
    Binding::chord('b', 'u', Command::BookmarkUntrack),
    Binding::char('=', Command::BookmarkSet),
    Binding::char('m', Command::BookmarkMove),
    Binding::char('f', Command::Fetch),
    Binding::char('F', Command::FetchRemote),
    Binding::char('y', Command::Copy),
    Binding::char('p', Command::Push),
    Binding::char('v', Command::ViewFormat),
    Binding::char('r', Command::Refresh),
    Binding::char('h', Command::Back),
    Binding::code(KeyCode::Left, Command::Back),
    Binding::char('L', Command::SwitchLog),
    Binding::char('J', Command::SwitchDefault),
];
