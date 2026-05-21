//! Rewrite action plans for graph-relative jj mutations.
//!
//! This module owns explicit rewrite source and destination roles, argv
//! construction, and preview wording for rebase, squash, and absorb. It
//! describes the selected revisions honestly, but it does not simulate jj's
//! line placement or final graph results.

use color_eyre::Result;

use crate::jj::run_direct_args;
use crate::jj_actions::CommandOutput;
use crate::jj_syntax::exact_change_id_revset;

// Rewrite plans share explicit source/destination roles and avoid parsing or
// predicting jj's final graph shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRebasePlan {
    sources: Vec<String>,
    destination: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjSquashPlan {
    sources: Vec<String>,
    destination: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbsorbPlan {
    source: String,
    destinations: Vec<String>,
}

impl JjRebasePlan {
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["rebase".to_owned()];
        for source in &self.sources {
            argv.push("-r".to_owned());
            argv.push(source.clone());
        }
        argv.push("-o".to_owned());
        argv.push(self.destination.clone());
        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "rebased",
        )
    }

    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source revision: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\nroles:\n{}\ndestination revision: {}\n\ncurrent graph context:\n- source rows are selected in jk\n- destination is the current row\n\nexpected jj effect:\n- semantics: jj rebase --revision <source> --onto <destination>\n- only listed source revisions are rebased\n- dependencies among listed sources are preserved\n- descendants outside the selected set may be rebased to fill holes\n- destination descendants are not inserted or rebased by -o\n\nnot a graph preview: jk has not run jj and is not simulating the final graph\n\nreview after run: jj op show -p\nundo path: jj undo\nconfirmation: press Enter to run jj rebase",
            self.command_label(false),
            sources,
            self.destination,
        )
    }

    fn normalize(mut self) -> Self {
        self.sources.retain(|source| !source.trim().is_empty());
        self
    }
}

impl JjSquashPlan {
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["squash".to_owned()];
        for source in &self.sources {
            argv.push("--from".to_owned());
            argv.push(source.clone());
        }
        argv.push("--into".to_owned());
        argv.push(self.destination.clone());
        argv.push("--use-destination-message".to_owned());
        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "squashed",
        )
    }

    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\n{}\n\ndestination: {}\n\ngraph effect: moves the selected source changes into the destination; jj may abandon emptied sources and rebase descendants\n\ndescription behavior: --use-destination-message keeps the destination description, discards source descriptions, and avoids an editor or prompt\n\nconfirmation: press Enter to run jj squash\nundo path: jj undo",
            self.command_label(false),
            sources,
            self.destination,
        )
    }

    fn normalize(mut self) -> Self {
        self.sources.retain(|source| !source.trim().is_empty());
        self
    }
}

impl JjAbsorbPlan {
    pub fn new(source: impl Into<String>, destinations: Vec<String>) -> Self {
        Self {
            source: source.into(),
            destinations,
        }
        .normalize()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn destinations(&self) -> &[String] {
        &self.destinations
    }

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
        let mut argv = vec![
            "absorb".to_owned(),
            "--from".to_owned(),
            exact_change_id_revset(&self.source),
        ];
        for destination in &self.destinations {
            argv.push("--into".to_owned());
            argv.push(exact_change_id_revset(destination));
        }
        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "absorbed")
    }

    pub fn preview_summary(&self) -> String {
        let destinations = self
            .destinations
            .iter()
            .map(|destination| format!("candidate destination: {destination}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            concat!(
                "command: {}\n\n",
                "source: {}\n",
                "{}\n\n",
                "selection: selected revisions are candidate destinations; jj absorb only ",
                "considers selected revisions that are ancestors of the source\n\n",
                "effect: jj splits source changes and moves each change to the closest ",
                "selected mutable ancestor where the corresponding lines were last modified\n\n",
                "opacity: jk does not simulate line-level placement or final graph shape\n\n",
                "ambiguity: changes remain in the source when jj cannot choose unambiguously\n\n",
                "source result: the source may become empty or abandoned depending on jj ",
                "semantics\n\n",
                "confirmation: press Enter to run jj absorb\n",
                "recovery: jj undo\n",
                "review: jj op show -p"
            ),
            self.command_label(),
            self.source,
            destinations,
        )
    }

    fn normalize(mut self) -> Self {
        self.destinations
            .retain(|destination| !destination.trim().is_empty() && destination != &self.source);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebase_command_args_use_explicit_sources_and_destination() {
        let rebase = JjRebasePlan::new(
            vec!["source-a".to_owned(), "source-b".to_owned()],
            "dest".to_owned(),
        );

        assert_eq!(
            rebase.command_argv(false),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
        assert_eq!(
            rebase.command_label(false),
            "jj rebase -r source-a -r source-b -o dest"
        );
    }

    #[test]
    fn rebase_preview_summary_includes_command_effect_and_undo_path() {
        let rebase = JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned());

        let preview = rebase.preview_summary();

        assert!(preview.contains("command: jj rebase -r source-a -o dest"));
        assert!(preview.contains("source revision: source-a"));
        assert!(preview.contains("destination revision: dest"));
        assert!(preview.contains("source rows are selected in jk"));
        assert!(preview.contains("destination is the current row"));
        assert!(preview.contains("semantics: jj rebase --revision <source> --onto <destination>"));
        assert!(preview.contains("only listed source revisions are rebased"));
        assert!(preview.contains("dependencies among listed sources are preserved"));
        assert!(preview.contains("descendants outside the selected set may be rebased"));
        assert!(preview.contains("destination descendants are not inserted or rebased by -o"));
        assert!(preview.contains("not a graph preview"));
        assert!(preview.contains("jk has not run jj and is not simulating the final graph"));
        assert!(preview.contains("review after run: jj op show -p"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("confirmation: press Enter to run jj rebase"));
    }

    #[test]
    fn squash_command_args_use_explicit_sources_destination_and_message_policy() {
        let squash = JjSquashPlan::new(
            vec!["source-a".to_owned(), "source-b".to_owned()],
            "dest".to_owned(),
        );

        assert_eq!(
            squash.command_argv(false),
            vec![
                "squash",
                "--from",
                "source-a",
                "--from",
                "source-b",
                "--into",
                "dest",
                "--use-destination-message"
            ]
        );
        assert_eq!(
            squash.command_label(false),
            "jj squash --from source-a --from source-b --into dest --use-destination-message"
        );
    }

    #[test]
    fn squash_preview_summary_includes_roles_effect_message_policy_and_undo_path() {
        let squash = JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned());

        let preview = squash.preview_summary();

        assert!(
            preview.contains(
                "command: jj squash --from source-a --into dest --use-destination-message"
            )
        );
        assert!(preview.contains("source: source-a"));
        assert!(preview.contains("destination: dest"));
        assert!(preview.contains("graph effect: moves the selected source changes"));
        assert!(preview.contains("--use-destination-message keeps the destination description"));
        assert!(preview.contains("confirmation: press Enter to run jj squash"));
        assert!(preview.contains("undo path: jj undo"));
    }

    #[test]
    fn absorb_command_args_use_exact_source_and_repeated_candidate_destinations() {
        let absorb = JjAbsorbPlan::new(
            "source-change",
            vec!["dest-a".to_owned(), "dest-b".to_owned()],
        );

        assert_eq!(
            absorb.command_argv(),
            vec![
                "absorb",
                "--from",
                "exactly(change_id(\"source-change\"), 1)",
                "--into",
                "exactly(change_id(\"dest-a\"), 1)",
                "--into",
                "exactly(change_id(\"dest-b\"), 1)",
            ]
        );
        assert_eq!(
            absorb.command_label(),
            "jj absorb --from exactly(change_id(\"source-change\"), 1) --into exactly(change_id(\"dest-a\"), 1) --into exactly(change_id(\"dest-b\"), 1)"
        );
    }

    #[test]
    fn absorb_preview_summary_names_bounded_opacity_and_recovery_paths() {
        let absorb = JjAbsorbPlan::new("source-change", vec!["dest-a".to_owned()]);

        let preview = absorb.preview_summary();

        assert!(preview.contains("source: source-change"));
        assert!(preview.contains("candidate destination: dest-a"));
        assert!(preview.contains("selected revisions are candidate destinations"));
        assert!(preview.contains("only considers selected revisions that are ancestors"));
        assert!(preview.contains("jk does not simulate line-level placement"));
        assert!(preview.contains("changes remain in the source"));
        assert!(preview.contains("source may become empty or abandoned"));
        assert!(preview.contains("recovery: jj undo"));
        assert!(preview.contains("review: jj op show -p"));
    }

    #[test]
    fn rebase_plan_argv_includes_repeated_sources_and_destination() {
        let rebase = JjRebasePlan::new(
            vec![
                "source-a".to_owned(),
                "source-b".to_owned(),
                "source-c".to_owned(),
            ],
            "dest".to_owned(),
        );

        assert_eq!(
            rebase.command_argv(false),
            vec![
                "rebase", "-r", "source-a", "-r", "source-b", "-r", "source-c", "-o", "dest"
            ]
        );
    }

    #[test]
    fn rebase_plan_argv_and_label_do_not_change_for_preview_flag() {
        let rebase = JjRebasePlan::new(vec!["source-a".to_owned(), "source-b".to_owned()], "dest");

        assert_eq!(
            rebase.command_argv(true),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
        assert_eq!(
            rebase.command_label(false),
            "jj rebase -r source-a -r source-b -o dest"
        );
        assert_eq!(
            rebase.command_label(true),
            "jj rebase -r source-a -r source-b -o dest"
        );
        assert_eq!(
            rebase.command_argv(false),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
    }
}
