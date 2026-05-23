use super::{JjBookmarkMutationKind, JjBookmarkMutationPlan};
use crate::jj::exact_string_pattern;

impl JjBookmarkMutationPlan {
    pub fn command_label(&self) -> String {
        let label_args = self
            .command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjBookmarkMutationKind::Create => vec![
                "bookmark".to_owned(),
                "create".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Set => vec![
                "bookmark".to_owned(),
                "set".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Move => vec![
                "bookmark".to_owned(),
                "move".to_owned(),
                "--to".to_owned(),
                self.required_target().command_arg(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Rename => vec![
                "bookmark".to_owned(),
                "rename".to_owned(),
                self.name.clone(),
                self.required_new_name().to_owned(),
            ],
            JjBookmarkMutationKind::Delete => vec![
                "bookmark".to_owned(),
                "delete".to_owned(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Forget => {
                let mut argv = vec!["bookmark".to_owned(), "forget".to_owned()];
                if self.required_forget_target().include_remotes() {
                    argv.push("--include-remotes".to_owned());
                }
                argv.push(exact_string_pattern(&self.name));
                argv
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                vec![
                    "bookmark".to_owned(),
                    self.kind.label().to_owned(),
                    "--remote".to_owned(),
                    target.remote_pattern(),
                    target.bookmark_pattern(),
                ]
            }
        }
    }
}
