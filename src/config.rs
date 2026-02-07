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
