//! Keybinding configuration loading and validation.
//!
//! Config is loaded from embedded defaults and optionally overlaid by user settings from
//! `JK_KEYBINDS` or `~/.config/jk/keybinds.toml`.

mod raw;

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::error::JkError;
use crate::keys::KeyBinding;

use self::raw::{PartialConfig, RawConfig, apply_partial};

/// Embedded default keymap shipped with the binary.
pub(super) const DEFAULT_KEYBINDS: &str = include_str!("../../config/keybinds.default.toml");

/// Fully-resolved keybinding configuration for all runtime modes.
#[derive(Debug, Clone)]
pub struct KeybindConfig {
    pub normal: NormalKeys,
    pub command: CommandKeys,
    pub confirm: ConfirmKeys,
}

/// Normal-mode bindings (navigation, command launchers, and high-frequency mutations).
#[derive(Debug, Clone)]
pub struct NormalKeys {
    pub quit: Vec<KeyBinding>,
    pub refresh: Vec<KeyBinding>,
    pub up: Vec<KeyBinding>,
    pub down: Vec<KeyBinding>,
    pub top: Vec<KeyBinding>,
    pub bottom: Vec<KeyBinding>,
    pub command_mode: Vec<KeyBinding>,
    pub help: Vec<KeyBinding>,
    pub keymap: Vec<KeyBinding>,
    pub aliases: Vec<KeyBinding>,
    pub show: Vec<KeyBinding>,
    pub diff: Vec<KeyBinding>,
    pub status: Vec<KeyBinding>,
    pub log: Vec<KeyBinding>,
    pub operation_log: Vec<KeyBinding>,
    pub bookmark_list: Vec<KeyBinding>,
    pub resolve_list: Vec<KeyBinding>,
    pub file_list: Vec<KeyBinding>,
    pub tag_list: Vec<KeyBinding>,
    pub root: Vec<KeyBinding>,
    pub repeat_last: Vec<KeyBinding>,
    pub toggle_patch: Vec<KeyBinding>,
    pub fetch: Vec<KeyBinding>,
    pub push: Vec<KeyBinding>,
    pub rebase_main: Vec<KeyBinding>,
    pub rebase_trunk: Vec<KeyBinding>,
    pub new: Vec<KeyBinding>,
    pub next: Vec<KeyBinding>,
    pub prev: Vec<KeyBinding>,
    pub edit: Vec<KeyBinding>,
    pub commit: Vec<KeyBinding>,
    pub describe: Vec<KeyBinding>,
    pub bookmark_set: Vec<KeyBinding>,
    pub abandon: Vec<KeyBinding>,
    pub rebase: Vec<KeyBinding>,
    pub squash: Vec<KeyBinding>,
    pub split: Vec<KeyBinding>,
    pub restore: Vec<KeyBinding>,
    pub revert: Vec<KeyBinding>,
    pub undo: Vec<KeyBinding>,
    pub redo: Vec<KeyBinding>,
}

/// Command-mode and prompt-mode editing bindings.
#[derive(Debug, Clone)]
pub struct CommandKeys {
    pub submit: Vec<KeyBinding>,
    pub cancel: Vec<KeyBinding>,
    pub backspace: Vec<KeyBinding>,
    pub history_prev: Vec<KeyBinding>,
    pub history_next: Vec<KeyBinding>,
}

/// Confirmation-mode bindings for dangerous command approval/rejection.
#[derive(Debug, Clone)]
pub struct ConfirmKeys {
    pub accept: Vec<KeyBinding>,
    pub reject: Vec<KeyBinding>,
}

impl KeybindConfig {
    /// Load keybinds from defaults plus optional user overrides.
    ///
    /// Parse/validation failures include path context when sourced from user config.
    pub fn load() -> Result<Self, JkError> {
        let mut raw: RawConfig = toml::from_str(DEFAULT_KEYBINDS)
            .map_err(|err| JkError::ConfigParse(format!("default keybind parse error: {err}")))?;

        if let Some(user_path) = user_keybind_path().filter(|path| path.exists()) {
            let content = fs::read_to_string(&user_path).map_err(|source| JkError::ConfigRead {
                path: user_path.display().to_string(),
                source,
            })?;
            let user: PartialConfig = toml::from_str(&content).map_err(|err| {
                JkError::ConfigParse(format!("{} parse error: {err}", user_path.display()))
            })?;
            apply_partial(&mut raw, user);
        }

        raw.into_config()
    }
}

/// Resolve user override path from environment.
///
/// `JK_KEYBINDS` has precedence over the default `$HOME/.config/jk/keybinds.toml` location.
fn user_keybind_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("JK_KEYBINDS") {
        return Some(PathBuf::from(path));
    }

    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/jk/keybinds.toml"))
}

/// Parse configured key token strings into runtime [`KeyBinding`] values.
pub(super) fn parse_bindings(values: &[String]) -> Result<Vec<KeyBinding>, JkError> {
    values
        .iter()
        .map(|value| {
            KeyBinding::parse(value).ok_or_else(|| {
                JkError::ConfigParse(format!("unknown keybinding `{value}` in keybind config"))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
