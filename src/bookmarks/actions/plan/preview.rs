use super::{JjBookmarkMutationKind, JjBookmarkMutationPlan};
use crate::jj::exact_string_pattern;

impl JjBookmarkMutationPlan {
    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("bookmark: {}", self.name),
        ];

        match self.kind {
            JjBookmarkMutationKind::Create => {
                lines.extend([
                    "source/current: new local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: creates one local bookmark at the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark create".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Set => {
                lines.extend([
                    "source/current: jj resolves the literal local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: sets one local bookmark to the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark set".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Move => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: moves one exactly named local bookmark to the destination target"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark move".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Rename => {
                lines.extend([
                    format!("old name: {}", self.name),
                    format!("new name: {}", self.required_new_name()),
                    "target: exact selected local bookmark row; rendered labels are not parsed"
                        .to_owned(),
                    "effect: renames one local bookmark without --overwrite-existing".to_owned(),
                    "duplicate name: jj failure output is preserved if the new name already exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark rename".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Delete => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    "destination: none".to_owned(),
                    "effect: deletes one local bookmark; this is delete, not forget".to_owned(),
                    "tracking: track/untrack stay disabled until exact tracking metadata exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark delete".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Forget => {
                let target = self.required_forget_target();
                lines.extend([
                    format!(
                        "target: exact bookmark {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("visible state: {}", target.visible_state()),
                    format!("scope: {}", target.scope_summary()),
                    "effect: forgets tracking relationship metadata; this is forget, not delete"
                        .to_owned(),
                    "output: full jj failure output remains inspectable in this pane".to_owned(),
                    "confirmation: press Enter to run jj bookmark forget".to_owned(),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                lines.extend([
                    format!("local bookmark: {}", target.local_bookmark_label()),
                    format!("remote bookmark: {}", target.remote_bookmark()),
                    format!("remote: {}", target.remote()),
                    format!("remote pattern: {}", target.remote_pattern()),
                    format!("bookmark pattern: {}", target.bookmark_pattern()),
                    format!("visible state: {}", target.visible_state()),
                    tracking_effect(self.kind),
                    "output: full jj result or failure output remains inspectable in this pane"
                        .to_owned(),
                    format!(
                        "confirmation: press Enter to run jj bookmark {}",
                        self.kind.label()
                    ),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
        }

        lines.join("\n")
    }
}

fn tracking_effect(kind: JjBookmarkMutationKind) -> String {
    match kind {
        JjBookmarkMutationKind::Track => {
            "effect: tracks the exact remote bookmark for the exact local bookmark; this does not fetch, push, delete, or rename".to_owned()
        }
        JjBookmarkMutationKind::Untrack => {
            "effect: untracks the exact remote bookmark relationship; this does not delete the local or remote bookmark".to_owned()
        }
        JjBookmarkMutationKind::Create
        | JjBookmarkMutationKind::Set
        | JjBookmarkMutationKind::Move
        | JjBookmarkMutationKind::Rename
        | JjBookmarkMutationKind::Delete
        | JjBookmarkMutationKind::Forget => {
            unreachable!("tracking target effects only apply to track/untrack")
        }
    }
}
