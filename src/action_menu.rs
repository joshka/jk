//! Action-menu presentation models for future graph mutation preparation.
//!
//! This module owns the user-visible action vocabulary for mutation prep.
//! It intentionally does not execute commands or invoke `jj`.

const PREVIEW_REQUIRED_MARKER: &str = "Preview required before execution.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyTier {
    PreviewFirst,
}

impl SafetyTier {
    #[cfg(test)]
    pub fn is_preview_first(&self) -> bool {
        matches!(self, Self::PreviewFirst)
    }

    pub fn preview_marker(&self) -> &'static str {
        PREVIEW_REQUIRED_MARKER
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionKind {
    New,
    Split,
    Abandon,
    Rebase,
    Squash,
}

impl ActionKind {
    fn label(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Split => "split",
            Self::Abandon => "abandon",
            Self::Rebase => "rebase",
            Self::Squash => "squash",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePromptOption {
    role: &'static str,
    value: String,
}

impl RolePromptOption {
    pub fn new(role: &'static str, value: impl Into<String>) -> Self {
        Self {
            role,
            value: value.into(),
        }
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.role, self.value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePrompt {
    title: &'static str,
    options: Vec<RolePromptOption>,
    preview_required_message: &'static str,
}

impl RolePrompt {
    pub fn new(
        title: &'static str,
        options: Vec<RolePromptOption>,
        preview_required_message: &'static str,
    ) -> Self {
        Self {
            title,
            options,
            preview_required_message,
        }
    }

    pub fn title(&self) -> &str {
        self.title
    }

    pub fn options(&self) -> &[RolePromptOption] {
        &self.options
    }

    pub fn status_message(&self) -> String {
        let mut lines = self
            .options
            .iter()
            .map(RolePromptOption::label)
            .collect::<Vec<_>>();
        lines.push(self.preview_required_message.to_owned());
        lines.join("\n")
    }

    pub fn source_revisions(&self) -> Vec<&str> {
        self.options
            .iter()
            .filter(|option| option.role() == "source")
            .map(RolePromptOption::value)
            .collect()
    }

    pub fn destination_revision(&self) -> Option<&str> {
        self.options
            .iter()
            .find(|option| option.role() == "destination")
            .map(RolePromptOption::value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowUp {
    StatusMessage(String),
    ExactRevision { revision: String },
    NewParents { parents: Vec<String> },
    RolePrompt(RolePrompt),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionMenuItem {
    action: ActionKind,
    label: String,
    safety_tier: SafetyTier,
    follow_up: FollowUp,
}

impl ActionMenuItem {
    pub fn action(&self) -> ActionKind {
        self.action
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn safety_tier(&self) -> SafetyTier {
        self.safety_tier
    }

    pub fn follow_up(&self) -> &FollowUp {
        &self.follow_up
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionMenu {
    items: Vec<ActionMenuItem>,
}

impl ActionMenu {
    pub fn new(items: Vec<ActionMenuItem>) -> Self {
        Self { items }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[ActionMenuItem] {
        &self.items
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactActionContext {
    current_revision: Option<String>,
    source_revisions: Vec<String>,
}

impl ExactActionContext {
    pub fn with_current(current_revision: impl Into<String>) -> Self {
        Self {
            current_revision: Some(current_revision.into()),
            source_revisions: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            current_revision: None,
            source_revisions: Vec::new(),
        }
    }

    pub fn with_sources<I, S>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.source_revisions = sources.into_iter().map(Into::into).collect();
        self
    }

    pub fn current_revision(&self) -> Option<&str> {
        self.current_revision.as_deref()
    }

    pub fn source_revisions(&self) -> &[String] {
        &self.source_revisions
    }
}

pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    let Some(current_revision) = context.current_revision() else {
        return ActionMenu::default();
    };

    let new_parents = if context.source_revisions().is_empty() {
        vec![current_revision.to_owned()]
    } else {
        context.source_revisions().to_vec()
    };

    if context.source_revisions().is_empty()
        || context
            .source_revisions()
            .iter()
            .all(|source| source == current_revision)
    {
        let new = menu_item_for_new_parents(&new_parents);
        let split = menu_item_for_single_revision(ActionKind::Split, current_revision);
        let abandon = menu_item_for_single_revision(ActionKind::Abandon, current_revision);
        return ActionMenu::new(vec![new, split, abandon]);
    }

    let sources = context
        .source_revisions()
        .iter()
        .filter(|source| *source != &current_revision)
        .cloned()
        .collect::<Vec<_>>();
    if sources.is_empty() {
        return ActionMenu::default();
    }

    ActionMenu::new(vec![
        menu_item_for_new_parents(&new_parents),
        menu_item_for_multirev_action(ActionKind::Rebase, &sources, current_revision),
        menu_item_for_multirev_action(ActionKind::Squash, &sources, current_revision),
    ])
}

fn menu_item_for_new_parents(parent_revisions: &[String]) -> ActionMenuItem {
    let label = if parent_revisions.len() == 1 {
        format!("new child of {}", short_id(&parent_revisions[0]))
    } else {
        format!("new merge child of {} parents", parent_revisions.len())
    };
    ActionMenuItem {
        action: ActionKind::New,
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::NewParents {
            parents: parent_revisions.to_vec(),
        },
    }
}

fn menu_item_for_single_revision(action: ActionKind, revision: &str) -> ActionMenuItem {
    let label = format!(
        "{} selected revision {}",
        action.label(),
        short_id(revision)
    );
    let follow_up = match action {
        ActionKind::Abandon => FollowUp::ExactRevision {
            revision: revision.to_owned(),
        },
        ActionKind::New | ActionKind::Split | ActionKind::Rebase | ActionKind::Squash => {
            let message = format!("{} {}", label, PREVIEW_REQUIRED_MARKER);
            FollowUp::StatusMessage(message)
        }
    };
    ActionMenuItem {
        action,
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up,
    }
}

fn menu_item_for_multirev_action(
    action: ActionKind,
    source_revisions: &[String],
    destination_revision: &str,
) -> ActionMenuItem {
    let label = format!(
        "{} {} source revision{} into destination {}",
        action.label(),
        source_revisions.len(),
        if source_revisions.len() == 1 { "" } else { "s" },
        short_id(destination_revision),
    );
    let options = source_revisions
        .iter()
        .map(|source| RolePromptOption::new("source", source))
        .chain(std::iter::once(RolePromptOption::new(
            "destination",
            destination_revision.to_owned(),
        )))
        .collect();
    let role_prompt = RolePrompt::new(
        "confirm role assignment",
        options,
        SafetyTier::PreviewFirst.preview_marker(),
    );
    ActionMenuItem {
        action,
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RolePrompt(role_prompt),
    }
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_exact_revision_builds_preview_first_split_and_abandon_menu() {
        let context = ExactActionContext::with_current("0000000011111111222222223333333344444444");
        let menu = build_action_menu(&context);

        assert_eq!(menu.items().len(), 3);
        assert_eq!(menu.items()[0].action(), ActionKind::New);
        assert_eq!(menu.items()[1].action(), ActionKind::Split);
        assert_eq!(menu.items()[2].action(), ActionKind::Abandon);
        assert!(menu.items()[0].safety_tier().is_preview_first());
        assert!(menu.items()[1].safety_tier().is_preview_first());
        assert!(menu.items()[2].safety_tier().is_preview_first());
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::NewParents { parents }
                if parents == &vec!["0000000011111111222222223333333344444444".to_owned()]
        ));
        assert!(matches!(
            menu.items()[1].follow_up(),
            FollowUp::StatusMessage(message)
                if message.ends_with(PREVIEW_REQUIRED_MARKER)
        ));
        assert!(matches!(
            menu.items()[2].follow_up(),
            FollowUp::ExactRevision { revision }
                if revision == "0000000011111111222222223333333344444444"
        ));
    }

    #[test]
    fn selected_sources_and_destination_prompt_with_explicit_roles() {
        let context =
            ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
                .with_sources([
                    "aaaabbbb1111111111111111111111111111111111",
                    "eeeeffff2222222222222222222222222222222222",
                ]);
        let menu = build_action_menu(&context);

        assert_eq!(menu.items().len(), 3);
        assert!(menu.items()[0].label().contains("new merge child"));
        assert!(
            menu.items()[1]
                .label()
                .contains("source revisions into destination ccccdddd")
        );
        assert!(
            menu.items()[2]
                .label()
                .contains("source revisions into destination ccccdddd")
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::NewParents { parents }
                if parents == &vec![
                    "aaaabbbb1111111111111111111111111111111111".to_owned(),
                    "eeeeffff2222222222222222222222222222222222".to_owned()
                ]
        ));
        assert!(menu.items()[1].label().contains("rebase"));
        assert!(matches!(
            menu.items()[1].follow_up(),
            FollowUp::RolePrompt(prompt)
                if prompt.title() == "confirm role assignment"
        ));
        if let FollowUp::RolePrompt(prompt) = menu.items()[1].follow_up() {
            assert_eq!(menu.items()[1].action(), ActionKind::Rebase);
            assert_eq!(prompt.options()[0].role(), "source");
            assert_eq!(
                prompt.options()[0].value(),
                "aaaabbbb1111111111111111111111111111111111"
            );
            assert_eq!(
                prompt.source_revisions(),
                vec![
                    "aaaabbbb1111111111111111111111111111111111",
                    "eeeeffff2222222222222222222222222222222222"
                ]
            );
            assert_eq!(
                prompt.destination_revision(),
                Some("ccccdddd1111111111111111111111111111111111")
            );
            let labels = prompt
                .options()
                .iter()
                .map(|option| option.label())
                .collect::<Vec<_>>();
            assert_eq!(
                labels[0],
                "source: aaaabbbb1111111111111111111111111111111111"
            );
            assert_eq!(
                labels[1],
                "source: eeeeffff2222222222222222222222222222222222"
            );
            assert_eq!(
                labels[2],
                "destination: ccccdddd1111111111111111111111111111111111"
            );
            assert!(prompt.status_message().ends_with(PREVIEW_REQUIRED_MARKER));
        } else {
            panic!("expected role prompt follow-up");
        }
        if let FollowUp::RolePrompt(prompt) = menu.items()[2].follow_up() {
            assert_eq!(menu.items()[2].action(), ActionKind::Squash);
            assert_eq!(
                prompt.source_revisions(),
                vec![
                    "aaaabbbb1111111111111111111111111111111111",
                    "eeeeffff2222222222222222222222222222222222"
                ]
            );
            assert_eq!(
                prompt.destination_revision(),
                Some("ccccdddd1111111111111111111111111111111111")
            );
        } else {
            panic!("expected squash role prompt follow-up");
        }
    }

    #[test]
    fn no_exact_actionable_ids_returns_empty_menu() {
        let context = ExactActionContext::none();
        let menu = build_action_menu(&context);

        assert!(menu.is_empty());
    }

    #[test]
    fn multi_source_menu_excludes_abandon() {
        let context =
            ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
                .with_sources(["aaaabbbb1111111111111111111111111111111111"]);
        let menu = build_action_menu(&context);

        let actions = menu
            .items()
            .iter()
            .map(ActionMenuItem::action)
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![ActionKind::New, ActionKind::Rebase, ActionKind::Squash]
        );
    }

    #[test]
    fn self_selection_keeps_new_parent_and_single_revision_actions() {
        let context =
            ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
                .with_sources(["ccccdddd1111111111111111111111111111111111"]);
        let menu = build_action_menu(&context);

        let actions = menu
            .items()
            .iter()
            .map(ActionMenuItem::action)
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![ActionKind::New, ActionKind::Split, ActionKind::Abandon]
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::NewParents { parents }
                if parents == &vec!["ccccdddd1111111111111111111111111111111111".to_owned()]
        ));
    }
}
