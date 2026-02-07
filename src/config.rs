use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::JkError;
use crate::keys::KeyBinding;

const DEFAULT_KEYBINDS: &str = include_str!("../config/keybinds.default.toml");

#[derive(Debug, Clone)]
pub struct KeybindConfig {
    pub normal: NormalKeys,
    pub command: CommandKeys,
    pub confirm: ConfirmKeys,
}

#[derive(Debug, Clone)]
pub struct NormalKeys {
    pub quit: Vec<KeyBinding>,
    pub refresh: Vec<KeyBinding>,
    pub up: Vec<KeyBinding>,
    pub down: Vec<KeyBinding>,
    pub top: Vec<KeyBinding>,
    pub bottom: Vec<KeyBinding>,
    pub command_mode: Vec<KeyBinding>,
    pub show: Vec<KeyBinding>,
    pub diff: Vec<KeyBinding>,
    pub toggle_patch: Vec<KeyBinding>,
    pub fetch: Vec<KeyBinding>,
    pub push: Vec<KeyBinding>,
    pub rebase_main: Vec<KeyBinding>,
    pub rebase_trunk: Vec<KeyBinding>,
    pub new: Vec<KeyBinding>,
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

#[derive(Debug, Clone)]
pub struct CommandKeys {
    pub submit: Vec<KeyBinding>,
    pub cancel: Vec<KeyBinding>,
    pub backspace: Vec<KeyBinding>,
}

#[derive(Debug, Clone)]
pub struct ConfirmKeys {
    pub accept: Vec<KeyBinding>,
    pub reject: Vec<KeyBinding>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    normal: RawNormal,
    command: RawCommand,
    confirm: RawConfirm,
}

#[derive(Debug, Deserialize)]
struct RawNormal {
    quit: Vec<String>,
    refresh: Vec<String>,
    up: Vec<String>,
    down: Vec<String>,
    top: Vec<String>,
    bottom: Vec<String>,
    command_mode: Vec<String>,
    show: Vec<String>,
    diff: Vec<String>,
    toggle_patch: Vec<String>,
    fetch: Vec<String>,
    push: Vec<String>,
    rebase_main: Vec<String>,
    rebase_trunk: Vec<String>,
    new: Vec<String>,
    commit: Vec<String>,
    describe: Vec<String>,
    bookmark_set: Vec<String>,
    abandon: Vec<String>,
    rebase: Vec<String>,
    squash: Vec<String>,
    split: Vec<String>,
    restore: Vec<String>,
    revert: Vec<String>,
    undo: Vec<String>,
    redo: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawCommand {
    submit: Vec<String>,
    cancel: Vec<String>,
    backspace: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawConfirm {
    accept: Vec<String>,
    reject: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    normal: Option<PartialNormal>,
    command: Option<PartialCommand>,
    confirm: Option<PartialConfirm>,
}

#[derive(Debug, Deserialize)]
struct PartialNormal {
    quit: Option<Vec<String>>,
    refresh: Option<Vec<String>>,
    up: Option<Vec<String>>,
    down: Option<Vec<String>>,
    top: Option<Vec<String>>,
    bottom: Option<Vec<String>>,
    command_mode: Option<Vec<String>>,
    show: Option<Vec<String>>,
    diff: Option<Vec<String>>,
    toggle_patch: Option<Vec<String>>,
    fetch: Option<Vec<String>>,
    push: Option<Vec<String>>,
    rebase_main: Option<Vec<String>>,
    rebase_trunk: Option<Vec<String>>,
    new: Option<Vec<String>>,
    commit: Option<Vec<String>>,
    describe: Option<Vec<String>>,
    bookmark_set: Option<Vec<String>>,
    abandon: Option<Vec<String>>,
    rebase: Option<Vec<String>>,
    squash: Option<Vec<String>>,
    split: Option<Vec<String>>,
    restore: Option<Vec<String>>,
    revert: Option<Vec<String>>,
    undo: Option<Vec<String>>,
    redo: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PartialCommand {
    submit: Option<Vec<String>>,
    cancel: Option<Vec<String>>,
    backspace: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PartialConfirm {
    accept: Option<Vec<String>>,
    reject: Option<Vec<String>>,
}

impl KeybindConfig {
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

impl RawConfig {
    fn into_config(self) -> Result<KeybindConfig, JkError> {
        Ok(KeybindConfig {
            normal: NormalKeys {
                quit: parse_bindings(&self.normal.quit)?,
                refresh: parse_bindings(&self.normal.refresh)?,
                up: parse_bindings(&self.normal.up)?,
                down: parse_bindings(&self.normal.down)?,
                top: parse_bindings(&self.normal.top)?,
                bottom: parse_bindings(&self.normal.bottom)?,
                command_mode: parse_bindings(&self.normal.command_mode)?,
                show: parse_bindings(&self.normal.show)?,
                diff: parse_bindings(&self.normal.diff)?,
                toggle_patch: parse_bindings(&self.normal.toggle_patch)?,
                fetch: parse_bindings(&self.normal.fetch)?,
                push: parse_bindings(&self.normal.push)?,
                rebase_main: parse_bindings(&self.normal.rebase_main)?,
                rebase_trunk: parse_bindings(&self.normal.rebase_trunk)?,
                new: parse_bindings(&self.normal.new)?,
                commit: parse_bindings(&self.normal.commit)?,
                describe: parse_bindings(&self.normal.describe)?,
                bookmark_set: parse_bindings(&self.normal.bookmark_set)?,
                abandon: parse_bindings(&self.normal.abandon)?,
                rebase: parse_bindings(&self.normal.rebase)?,
                squash: parse_bindings(&self.normal.squash)?,
                split: parse_bindings(&self.normal.split)?,
                restore: parse_bindings(&self.normal.restore)?,
                revert: parse_bindings(&self.normal.revert)?,
                undo: parse_bindings(&self.normal.undo)?,
                redo: parse_bindings(&self.normal.redo)?,
            },
            command: CommandKeys {
                submit: parse_bindings(&self.command.submit)?,
                cancel: parse_bindings(&self.command.cancel)?,
                backspace: parse_bindings(&self.command.backspace)?,
            },
            confirm: ConfirmKeys {
                accept: parse_bindings(&self.confirm.accept)?,
                reject: parse_bindings(&self.confirm.reject)?,
            },
        })
    }
}

fn user_keybind_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("JK_KEYBINDS") {
        return Some(PathBuf::from(path));
    }

    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/jk/keybinds.toml"))
}

fn parse_bindings(values: &[String]) -> Result<Vec<KeyBinding>, JkError> {
    values
        .iter()
        .map(|value| {
            KeyBinding::parse(value).ok_or_else(|| {
                JkError::ConfigParse(format!("unknown keybinding `{value}` in keybind config"))
            })
        })
        .collect()
}

fn apply_partial(base: &mut RawConfig, user: PartialConfig) {
    if let Some(normal) = user.normal {
        if let Some(value) = normal.quit {
            base.normal.quit = value;
        }
        if let Some(value) = normal.refresh {
            base.normal.refresh = value;
        }
        if let Some(value) = normal.up {
            base.normal.up = value;
        }
        if let Some(value) = normal.down {
            base.normal.down = value;
        }
        if let Some(value) = normal.top {
            base.normal.top = value;
        }
        if let Some(value) = normal.bottom {
            base.normal.bottom = value;
        }
        if let Some(value) = normal.command_mode {
            base.normal.command_mode = value;
        }
        if let Some(value) = normal.show {
            base.normal.show = value;
        }
        if let Some(value) = normal.diff {
            base.normal.diff = value;
        }
        if let Some(value) = normal.toggle_patch {
            base.normal.toggle_patch = value;
        }
        if let Some(value) = normal.fetch {
            base.normal.fetch = value;
        }
        if let Some(value) = normal.push {
            base.normal.push = value;
        }
        if let Some(value) = normal.rebase_main {
            base.normal.rebase_main = value;
        }
        if let Some(value) = normal.rebase_trunk {
            base.normal.rebase_trunk = value;
        }
        if let Some(value) = normal.new {
            base.normal.new = value;
        }
        if let Some(value) = normal.commit {
            base.normal.commit = value;
        }
        if let Some(value) = normal.describe {
            base.normal.describe = value;
        }
        if let Some(value) = normal.bookmark_set {
            base.normal.bookmark_set = value;
        }
        if let Some(value) = normal.abandon {
            base.normal.abandon = value;
        }
        if let Some(value) = normal.rebase {
            base.normal.rebase = value;
        }
        if let Some(value) = normal.squash {
            base.normal.squash = value;
        }
        if let Some(value) = normal.split {
            base.normal.split = value;
        }
        if let Some(value) = normal.restore {
            base.normal.restore = value;
        }
        if let Some(value) = normal.revert {
            base.normal.revert = value;
        }
        if let Some(value) = normal.undo {
            base.normal.undo = value;
        }
        if let Some(value) = normal.redo {
            base.normal.redo = value;
        }
    }

    if let Some(command) = user.command {
        if let Some(value) = command.submit {
            base.command.submit = value;
        }
        if let Some(value) = command.cancel {
            base.command.cancel = value;
        }
        if let Some(value) = command.backspace {
            base.command.backspace = value;
        }
    }

    if let Some(confirm) = user.confirm {
        if let Some(value) = confirm.accept {
            base.confirm.accept = value;
        }
        if let Some(value) = confirm.reject {
            base.confirm.reject = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_KEYBINDS, KeybindConfig, RawConfig};
    use crate::keys::KeyBinding;

    #[test]
    fn default_keybinds_include_high_frequency_shortcuts() {
        let raw: RawConfig =
            toml::from_str(DEFAULT_KEYBINDS).expect("default keybinds should parse");
        let config = raw
            .into_config()
            .expect("default keybinds should convert into runtime config");

        assert_eq!(config.normal.fetch, vec![KeyBinding::Char('F')]);
        assert_eq!(config.normal.push, vec![KeyBinding::Char('P')]);
        assert_eq!(config.normal.rebase_main, vec![KeyBinding::Char('M')]);
        assert_eq!(config.normal.rebase_trunk, vec![KeyBinding::Char('T')]);
        assert_eq!(config.normal.new, vec![KeyBinding::Char('n')]);
        assert_eq!(config.normal.commit, vec![KeyBinding::Char('c')]);
        assert_eq!(config.normal.describe, vec![KeyBinding::Char('D')]);
        assert_eq!(config.normal.bookmark_set, vec![KeyBinding::Char('b')]);
        assert_eq!(config.normal.abandon, vec![KeyBinding::Char('a')]);
        assert_eq!(config.normal.toggle_patch, vec![KeyBinding::Char('p')]);
        assert_eq!(config.normal.rebase, vec![KeyBinding::Char('B')]);
        assert_eq!(config.normal.squash, vec![KeyBinding::Char('S')]);
        assert_eq!(config.normal.split, vec![KeyBinding::Char('X')]);
        assert_eq!(config.normal.restore, vec![KeyBinding::Char('O')]);
        assert_eq!(config.normal.revert, vec![KeyBinding::Char('R')]);
        assert_eq!(config.normal.undo, vec![KeyBinding::Char('u')]);
        assert_eq!(config.normal.redo, vec![KeyBinding::Char('U')]);
    }

    #[test]
    fn load_keeps_keybind_configuration_valid() {
        let config = KeybindConfig::load().expect("keybind config should load");
        assert!(!config.normal.fetch.is_empty());
        assert!(!config.normal.push.is_empty());
        assert!(!config.normal.rebase_main.is_empty());
        assert!(!config.normal.rebase_trunk.is_empty());
        assert!(!config.normal.new.is_empty());
        assert!(!config.normal.commit.is_empty());
        assert!(!config.normal.describe.is_empty());
        assert!(!config.normal.bookmark_set.is_empty());
        assert!(!config.normal.abandon.is_empty());
        assert!(!config.normal.toggle_patch.is_empty());
        assert!(!config.normal.rebase.is_empty());
        assert!(!config.normal.squash.is_empty());
        assert!(!config.normal.split.is_empty());
        assert!(!config.normal.restore.is_empty());
        assert!(!config.normal.revert.is_empty());
        assert!(!config.normal.undo.is_empty());
        assert!(!config.normal.redo.is_empty());
    }
}
