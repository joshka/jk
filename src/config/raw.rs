use serde::Deserialize;

use crate::error::JkError;

use super::{CommandKeys, ConfirmKeys, KeybindConfig, NormalKeys, parse_bindings};

#[derive(Debug, Deserialize)]
pub(super) struct RawConfig {
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
    help: Vec<String>,
    keymap: Vec<String>,
    aliases: Vec<String>,
    show: Vec<String>,
    diff: Vec<String>,
    status: Vec<String>,
    log: Vec<String>,
    operation_log: Vec<String>,
    bookmark_list: Vec<String>,
    resolve_list: Vec<String>,
    file_list: Vec<String>,
    tag_list: Vec<String>,
    root: Vec<String>,
    repeat_last: Vec<String>,
    toggle_patch: Vec<String>,
    fetch: Vec<String>,
    push: Vec<String>,
    rebase_main: Vec<String>,
    rebase_trunk: Vec<String>,
    new: Vec<String>,
    next: Vec<String>,
    prev: Vec<String>,
    edit: Vec<String>,
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
    history_prev: Vec<String>,
    history_next: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawConfirm {
    accept: Vec<String>,
    reject: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PartialConfig {
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
    help: Option<Vec<String>>,
    keymap: Option<Vec<String>>,
    aliases: Option<Vec<String>>,
    show: Option<Vec<String>>,
    diff: Option<Vec<String>>,
    status: Option<Vec<String>>,
    log: Option<Vec<String>>,
    operation_log: Option<Vec<String>>,
    bookmark_list: Option<Vec<String>>,
    resolve_list: Option<Vec<String>>,
    file_list: Option<Vec<String>>,
    tag_list: Option<Vec<String>>,
    root: Option<Vec<String>>,
    repeat_last: Option<Vec<String>>,
    toggle_patch: Option<Vec<String>>,
    fetch: Option<Vec<String>>,
    push: Option<Vec<String>>,
    rebase_main: Option<Vec<String>>,
    rebase_trunk: Option<Vec<String>>,
    new: Option<Vec<String>>,
    next: Option<Vec<String>>,
    prev: Option<Vec<String>>,
    edit: Option<Vec<String>>,
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
    history_prev: Option<Vec<String>>,
    history_next: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PartialConfirm {
    accept: Option<Vec<String>>,
    reject: Option<Vec<String>>,
}

impl RawConfig {
    pub(super) fn into_config(self) -> Result<KeybindConfig, JkError> {
        Ok(KeybindConfig {
            normal: NormalKeys {
                quit: parse_bindings(&self.normal.quit)?,
                refresh: parse_bindings(&self.normal.refresh)?,
                up: parse_bindings(&self.normal.up)?,
                down: parse_bindings(&self.normal.down)?,
                top: parse_bindings(&self.normal.top)?,
                bottom: parse_bindings(&self.normal.bottom)?,
                command_mode: parse_bindings(&self.normal.command_mode)?,
                help: parse_bindings(&self.normal.help)?,
                keymap: parse_bindings(&self.normal.keymap)?,
                aliases: parse_bindings(&self.normal.aliases)?,
                show: parse_bindings(&self.normal.show)?,
                diff: parse_bindings(&self.normal.diff)?,
                status: parse_bindings(&self.normal.status)?,
                log: parse_bindings(&self.normal.log)?,
                operation_log: parse_bindings(&self.normal.operation_log)?,
                bookmark_list: parse_bindings(&self.normal.bookmark_list)?,
                resolve_list: parse_bindings(&self.normal.resolve_list)?,
                file_list: parse_bindings(&self.normal.file_list)?,
                tag_list: parse_bindings(&self.normal.tag_list)?,
                root: parse_bindings(&self.normal.root)?,
                repeat_last: parse_bindings(&self.normal.repeat_last)?,
                toggle_patch: parse_bindings(&self.normal.toggle_patch)?,
                fetch: parse_bindings(&self.normal.fetch)?,
                push: parse_bindings(&self.normal.push)?,
                rebase_main: parse_bindings(&self.normal.rebase_main)?,
                rebase_trunk: parse_bindings(&self.normal.rebase_trunk)?,
                new: parse_bindings(&self.normal.new)?,
                next: parse_bindings(&self.normal.next)?,
                prev: parse_bindings(&self.normal.prev)?,
                edit: parse_bindings(&self.normal.edit)?,
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
                history_prev: parse_bindings(&self.command.history_prev)?,
                history_next: parse_bindings(&self.command.history_next)?,
            },
            confirm: ConfirmKeys {
                accept: parse_bindings(&self.confirm.accept)?,
                reject: parse_bindings(&self.confirm.reject)?,
            },
        })
    }
}

pub(super) fn apply_partial(base: &mut RawConfig, user: PartialConfig) {
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
        if let Some(value) = normal.help {
            base.normal.help = value;
        }
        if let Some(value) = normal.keymap {
            base.normal.keymap = value;
        }
        if let Some(value) = normal.aliases {
            base.normal.aliases = value;
        }
        if let Some(value) = normal.show {
            base.normal.show = value;
        }
        if let Some(value) = normal.diff {
            base.normal.diff = value;
        }
        if let Some(value) = normal.status {
            base.normal.status = value;
        }
        if let Some(value) = normal.log {
            base.normal.log = value;
        }
        if let Some(value) = normal.operation_log {
            base.normal.operation_log = value;
        }
        if let Some(value) = normal.bookmark_list {
            base.normal.bookmark_list = value;
        }
        if let Some(value) = normal.resolve_list {
            base.normal.resolve_list = value;
        }
        if let Some(value) = normal.file_list {
            base.normal.file_list = value;
        }
        if let Some(value) = normal.tag_list {
            base.normal.tag_list = value;
        }
        if let Some(value) = normal.root {
            base.normal.root = value;
        }
        if let Some(value) = normal.repeat_last {
            base.normal.repeat_last = value;
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
        if let Some(value) = normal.next {
            base.normal.next = value;
        }
        if let Some(value) = normal.prev {
            base.normal.prev = value;
        }
        if let Some(value) = normal.edit {
            base.normal.edit = value;
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
        if let Some(value) = command.history_prev {
            base.command.history_prev = value;
        }
        if let Some(value) = command.history_next {
            base.command.history_next = value;
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
