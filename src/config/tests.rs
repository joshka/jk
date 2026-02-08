use super::raw::RawConfig;
use super::{DEFAULT_KEYBINDS, KeybindConfig};

use crate::keys::KeyBinding;

#[test]
fn default_keybinds_include_high_frequency_shortcuts() {
    let raw: RawConfig = toml::from_str(DEFAULT_KEYBINDS).expect("default keybinds should parse");
    let config = raw
        .into_config()
        .expect("default keybinds should convert into runtime config");

    assert_eq!(config.normal.fetch, vec![KeyBinding::Char('F')]);
    assert_eq!(config.normal.push, vec![KeyBinding::Char('P')]);
    assert_eq!(config.normal.rebase_main, vec![KeyBinding::Char('M')]);
    assert_eq!(config.normal.rebase_trunk, vec![KeyBinding::Char('T')]);
    assert_eq!(config.normal.new, vec![KeyBinding::Char('n')]);
    assert_eq!(config.normal.next, vec![KeyBinding::Char(']')]);
    assert_eq!(config.normal.prev, vec![KeyBinding::Char('[')]);
    assert_eq!(config.normal.edit, vec![KeyBinding::Char('e')]);
    assert_eq!(config.normal.commit, vec![KeyBinding::Char('c')]);
    assert_eq!(config.normal.describe, vec![KeyBinding::Char('D')]);
    assert_eq!(config.normal.bookmark_set, vec![KeyBinding::Char('b')]);
    assert_eq!(config.normal.abandon, vec![KeyBinding::Char('a')]);
    assert_eq!(config.normal.help, vec![KeyBinding::Char('?')]);
    assert_eq!(config.normal.keymap, vec![KeyBinding::Char('K')]);
    assert_eq!(config.normal.aliases, vec![KeyBinding::Char('A')]);
    assert_eq!(config.normal.status, vec![KeyBinding::Char('s')]);
    assert_eq!(config.normal.log, vec![KeyBinding::Char('l')]);
    assert_eq!(config.normal.operation_log, vec![KeyBinding::Char('o')]);
    assert_eq!(config.normal.bookmark_list, vec![KeyBinding::Char('L')]);
    assert_eq!(config.normal.resolve_list, vec![KeyBinding::Char('v')]);
    assert_eq!(config.normal.file_list, vec![KeyBinding::Char('f')]);
    assert_eq!(config.normal.tag_list, vec![KeyBinding::Char('t')]);
    assert_eq!(config.normal.root, vec![KeyBinding::Char('w')]);
    assert_eq!(config.normal.page_up, vec![KeyBinding::PageUp]);
    assert_eq!(config.normal.page_down, vec![KeyBinding::PageDown]);
    assert_eq!(
        config.normal.back,
        vec![KeyBinding::Left, KeyBinding::Char('h')]
    );
    assert_eq!(config.normal.forward, vec![KeyBinding::Right]);
    assert_eq!(config.normal.repeat_last, vec![KeyBinding::Char('.')]);
    assert_eq!(config.normal.toggle_patch, vec![KeyBinding::Char('p')]);
    assert_eq!(config.normal.rebase, vec![KeyBinding::Char('B')]);
    assert_eq!(config.normal.squash, vec![KeyBinding::Char('S')]);
    assert_eq!(config.normal.split, vec![KeyBinding::Char('X')]);
    assert_eq!(config.normal.restore, vec![KeyBinding::Char('O')]);
    assert_eq!(config.normal.revert, vec![KeyBinding::Char('R')]);
    assert_eq!(config.normal.undo, vec![KeyBinding::Char('u')]);
    assert_eq!(config.normal.redo, vec![KeyBinding::Char('U')]);
    assert_eq!(config.command.history_prev, vec![KeyBinding::Up]);
    assert_eq!(config.command.history_next, vec![KeyBinding::Down]);
}

#[test]
fn load_keeps_keybind_configuration_valid() {
    let config = KeybindConfig::load().expect("keybind config should load");
    assert!(!config.normal.fetch.is_empty());
    assert!(!config.normal.push.is_empty());
    assert!(!config.normal.rebase_main.is_empty());
    assert!(!config.normal.rebase_trunk.is_empty());
    assert!(!config.normal.new.is_empty());
    assert!(!config.normal.next.is_empty());
    assert!(!config.normal.prev.is_empty());
    assert!(!config.normal.edit.is_empty());
    assert!(!config.normal.commit.is_empty());
    assert!(!config.normal.describe.is_empty());
    assert!(!config.normal.bookmark_set.is_empty());
    assert!(!config.normal.abandon.is_empty());
    assert!(!config.normal.help.is_empty());
    assert!(!config.normal.keymap.is_empty());
    assert!(!config.normal.aliases.is_empty());
    assert!(!config.normal.status.is_empty());
    assert!(!config.normal.log.is_empty());
    assert!(!config.normal.operation_log.is_empty());
    assert!(!config.normal.bookmark_list.is_empty());
    assert!(!config.normal.resolve_list.is_empty());
    assert!(!config.normal.file_list.is_empty());
    assert!(!config.normal.tag_list.is_empty());
    assert!(!config.normal.root.is_empty());
    assert!(!config.normal.page_up.is_empty());
    assert!(!config.normal.page_down.is_empty());
    assert!(!config.normal.back.is_empty());
    assert!(!config.normal.forward.is_empty());
    assert!(!config.normal.repeat_last.is_empty());
    assert!(!config.normal.toggle_patch.is_empty());
    assert!(!config.normal.rebase.is_empty());
    assert!(!config.normal.squash.is_empty());
    assert!(!config.normal.split.is_empty());
    assert!(!config.normal.restore.is_empty());
    assert!(!config.normal.revert.is_empty());
    assert!(!config.normal.undo.is_empty());
    assert!(!config.normal.redo.is_empty());
    assert!(!config.command.history_prev.is_empty());
    assert!(!config.command.history_next.is_empty());
}
