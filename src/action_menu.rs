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
    Edit,
    New,
    Split,
    Duplicate,
    Abandon,
    Restore,
    Revert,
    Rebase,
    Squash,
    Absorb,
}

impl ActionKind {
    fn label(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::New => "new",
            Self::Split => "split",
            Self::Duplicate => "duplicate",
            Self::Abandon => "abandon",
            Self::Restore => "restore",
            Self::Revert => "revert",
            Self::Rebase => "rebase",
            Self::Squash => "squash",
            Self::Absorb => "absorb",
        }
    }

    fn shortcut(self) -> char {
        match self {
            Self::Edit => 'e',
            Self::New => 'n',
            Self::Split => 's',
            Self::Duplicate => 'd',
            Self::Abandon => 'x',
            Self::Restore => 'r',
            Self::Revert => 'v',
            Self::Rebase => 'b',
            Self::Squash => 'u',
            Self::Absorb => 'a',
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

    pub fn preview_required_message(&self) -> &str {
        self.preview_required_message
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
    ExactRevision {
        revision: String,
    },
    SplitExactTarget {
        revision: String,
    },
    SplitCurrentWorkingCopy,
    DuplicateExactTarget {
        revision: String,
    },
    EditExactTarget {
        revision: String,
    },
    RestoreExactTarget {
        revision: String,
        path: Option<String>,
    },
    RestoreWorkingCopyPath {
        path: String,
    },
    RevertExactTarget {
        revision: String,
    },
    OperationRestoreExactTarget {
        operation_id: String,
    },
    OperationRevertExactTarget {
        operation_id: String,
    },
    NewParents {
        parents: Vec<String>,
    },
    RolePrompt(RolePrompt),
    AbsorbCandidates {
        source: String,
        destinations: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionMenuItem {
    action: ActionKind,
    shortcut: char,
    label: String,
    safety_tier: SafetyTier,
    follow_up: FollowUp,
}

impl ActionMenuItem {
    pub fn new(
        action: ActionKind,
        label: impl Into<String>,
        safety_tier: SafetyTier,
        follow_up: FollowUp,
    ) -> Self {
        Self {
            action,
            shortcut: action.shortcut(),
            label: label.into(),
            safety_tier,
            follow_up,
        }
    }

    pub fn action(&self) -> ActionKind {
        self.action
    }

    pub fn shortcut(&self) -> char {
        self.shortcut
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

    pub fn item_for_shortcut(&self, shortcut: char) -> Option<&ActionMenuItem> {
        self.items.iter().find(|item| item.shortcut() == shortcut)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactActionContext {
    current_revision: Option<String>,
    source_revisions: Vec<String>,
    selected_path: Option<String>,
    current_is_visible_working_copy: bool,
    surface: ActionSurface,
}

impl ExactActionContext {
    pub fn with_current(current_revision: impl Into<String>) -> Self {
        Self {
            current_revision: Some(current_revision.into()),
            source_revisions: Vec::new(),
            selected_path: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Graph,
        }
    }

    pub fn detail(current_revision: impl Into<String>) -> Self {
        Self {
            current_revision: Some(current_revision.into()),
            source_revisions: Vec::new(),
            selected_path: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Detail,
        }
    }

    pub fn status_path(path: impl Into<String>) -> Self {
        Self {
            current_revision: Some("@".to_owned()),
            source_revisions: Vec::new(),
            selected_path: Some(path.into()),
            current_is_visible_working_copy: false,
            surface: ActionSurface::Status,
        }
    }

    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            current_revision: None,
            source_revisions: Vec::new(),
            selected_path: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Graph,
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

    pub fn with_selected_path(mut self, path: impl Into<String>) -> Self {
        self.selected_path = Some(path.into());
        self
    }

    pub fn with_visible_working_copy(mut self) -> Self {
        self.current_is_visible_working_copy = true;
        self
    }

    pub fn current_revision(&self) -> Option<&str> {
        self.current_revision.as_deref()
    }

    pub fn source_revisions(&self) -> &[String] {
        &self.source_revisions
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.selected_path.as_deref()
    }

    fn current_is_visible_working_copy(&self) -> bool {
        self.current_is_visible_working_copy
    }

    fn is_detail_surface(&self) -> bool {
        matches!(self.surface, ActionSurface::Detail)
    }

    fn is_status_surface(&self) -> bool {
        matches!(self.surface, ActionSurface::Status)
    }
}

pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    let Some(current_revision) = context.current_revision() else {
        return ActionMenu::default();
    };

    if context.is_status_surface() {
        return context
            .selected_path()
            .map(status_path_action_menu)
            .unwrap_or_default();
    }

    let mutation_items = mutation_menu_items(current_revision, context.selected_path(), true);

    if context.is_detail_surface() {
        return ActionMenu::new(mutation_items);
    }

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
        let edit = menu_item_for_edit(current_revision);
        let new = menu_item_for_new_parents(&new_parents);
        let split =
            menu_item_for_split(current_revision, context.current_is_visible_working_copy());
        let abandon = menu_item_for_single_revision(ActionKind::Abandon, current_revision);
        let mut items = vec![edit, new, split, abandon];
        items.extend(mutation_items);
        return ActionMenu::new(items);
    }

    let selected_revisions = context
        .source_revisions()
        .iter()
        .filter(|source| *source != current_revision)
        .cloned()
        .collect::<Vec<_>>();
    if selected_revisions.is_empty() {
        return ActionMenu::default();
    }

    let mut items = vec![
        menu_item_for_new_parents(&new_parents),
        menu_item_for_multirev_action(ActionKind::Rebase, &selected_revisions, current_revision),
        menu_item_for_multirev_action(ActionKind::Squash, &selected_revisions, current_revision),
        menu_item_for_absorb(current_revision, &selected_revisions),
    ];
    items.extend(mutation_menu_items(current_revision, None, false));
    ActionMenu::new(items)
}

fn menu_item_for_new_parents(parent_revisions: &[String]) -> ActionMenuItem {
    let label = if parent_revisions.len() == 1 {
        format!("new child of {}", short_id(&parent_revisions[0]))
    } else {
        format!("new merge child of {} parents", parent_revisions.len())
    };
    ActionMenuItem {
        action: ActionKind::New,
        shortcut: ActionKind::New.shortcut(),
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
        ActionKind::Edit
        | ActionKind::New
        | ActionKind::Duplicate
        | ActionKind::Restore
        | ActionKind::Revert
        | ActionKind::Rebase
        | ActionKind::Squash
        | ActionKind::Absorb
        | ActionKind::Split => {
            let message = format!("{} {}", label, PREVIEW_REQUIRED_MARKER);
            FollowUp::StatusMessage(message)
        }
    };
    ActionMenuItem {
        action,
        shortcut: action.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up,
    }
}

fn menu_item_for_split(revision: &str, current_is_visible_working_copy: bool) -> ActionMenuItem {
    let label = if current_is_visible_working_copy {
        "split current working-copy change @".to_owned()
    } else {
        format!("split selected revision {}", short_id(revision))
    };
    let follow_up = if current_is_visible_working_copy {
        FollowUp::SplitCurrentWorkingCopy
    } else {
        FollowUp::SplitExactTarget {
            revision: revision.to_owned(),
        }
    };

    ActionMenuItem {
        action: ActionKind::Split,
        shortcut: ActionKind::Split.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up,
    }
}

fn menu_item_for_edit(revision: &str) -> ActionMenuItem {
    ActionMenuItem {
        action: ActionKind::Edit,
        shortcut: ActionKind::Edit.shortcut(),
        label: format!("edit selected revision {}", short_id(revision)),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::EditExactTarget {
            revision: revision.to_owned(),
        },
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
        shortcut: action.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RolePrompt(role_prompt),
    }
}

fn menu_item_for_absorb(source_revision: &str, destination_revisions: &[String]) -> ActionMenuItem {
    let label = format!(
        "absorb current revision {} into {} candidate destination{}",
        short_id(source_revision),
        destination_revisions.len(),
        if destination_revisions.len() == 1 {
            ""
        } else {
            "s"
        },
    );
    ActionMenuItem {
        action: ActionKind::Absorb,
        shortcut: ActionKind::Absorb.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::AbsorbCandidates {
            source: source_revision.to_owned(),
            destinations: destination_revisions.to_vec(),
        },
    }
}

fn mutation_menu_items(
    current_revision: &str,
    selected_path: Option<&str>,
    include_duplicate: bool,
) -> Vec<ActionMenuItem> {
    let mut items = Vec::new();
    if let Some(path) = selected_path {
        items.push(ActionMenuItem {
            action: ActionKind::Restore,
            shortcut: 'p',
            label: format!("restore selected path from {}", short_id(current_revision)),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::RestoreExactTarget {
                revision: current_revision.to_owned(),
                path: Some(path.to_owned()),
            },
        });
    }
    if include_duplicate {
        items.push(ActionMenuItem {
            action: ActionKind::Duplicate,
            shortcut: ActionKind::Duplicate.shortcut(),
            label: format!("duplicate selected revision {}", short_id(current_revision)),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::DuplicateExactTarget {
                revision: current_revision.to_owned(),
            },
        });
    }
    items.push(ActionMenuItem {
        action: ActionKind::Restore,
        shortcut: ActionKind::Restore.shortcut(),
        label: format!("restore selected revision {}", short_id(current_revision)),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RestoreExactTarget {
            revision: current_revision.to_owned(),
            path: None,
        },
    });
    items.push(ActionMenuItem {
        action: ActionKind::Revert,
        shortcut: ActionKind::Revert.shortcut(),
        label: format!(
            "revert selected revision {} into @",
            short_id(current_revision)
        ),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RevertExactTarget {
            revision: current_revision.to_owned(),
        },
    });
    items
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionSurface {
    Graph,
    Detail,
    Status,
}

fn status_path_action_menu(path: &str) -> ActionMenu {
    ActionMenu::new(vec![ActionMenuItem {
        action: ActionKind::Restore,
        shortcut: ActionKind::Restore.shortcut(),
        label: format!("restore selected status path {path}"),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RestoreWorkingCopyPath {
            path: path.to_owned(),
        },
    }])
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_exact_revision_builds_graph_menu_with_duplicate_restore_and_revert() {
        let context = ExactActionContext::with_current("0000000011111111222222223333333344444444");
        let menu = build_action_menu(&context);

        assert_eq!(menu.items().len(), 7);
        assert_eq!(menu.items()[0].action(), ActionKind::Edit);
        assert_eq!(menu.items()[0].shortcut(), 'e');
        assert_eq!(menu.items()[1].action(), ActionKind::New);
        assert_eq!(menu.items()[1].shortcut(), 'n');
        assert_eq!(menu.items()[2].action(), ActionKind::Split);
        assert_eq!(menu.items()[2].shortcut(), 's');
        assert_eq!(menu.items()[3].action(), ActionKind::Abandon);
        assert_eq!(menu.items()[3].shortcut(), 'x');
        assert_eq!(menu.items()[4].action(), ActionKind::Duplicate);
        assert_eq!(menu.items()[4].shortcut(), 'd');
        assert_eq!(menu.items()[5].action(), ActionKind::Restore);
        assert_eq!(menu.items()[5].shortcut(), 'r');
        assert_eq!(menu.items()[6].action(), ActionKind::Revert);
        assert_eq!(menu.items()[6].shortcut(), 'v');
        assert!(menu.items()[0].safety_tier().is_preview_first());
        assert!(menu.items()[1].safety_tier().is_preview_first());
        assert!(menu.items()[2].safety_tier().is_preview_first());
        assert!(menu.items()[3].safety_tier().is_preview_first());
        assert!(menu.items()[4].safety_tier().is_preview_first());
        assert!(menu.items()[5].safety_tier().is_preview_first());
        assert!(menu.items()[6].safety_tier().is_preview_first());
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::EditExactTarget { revision }
                if revision == "0000000011111111222222223333333344444444"
        ));
        assert!(matches!(
            menu.items()[1].follow_up(),
            FollowUp::NewParents { parents }
                if parents == &vec!["0000000011111111222222223333333344444444".to_owned()]
        ));
        assert!(matches!(
            menu.items()[2].follow_up(),
            FollowUp::SplitExactTarget { revision }
                if revision == "0000000011111111222222223333333344444444"
        ));
        assert!(matches!(
            menu.items()[3].follow_up(),
            FollowUp::ExactRevision { revision }
                if revision == "0000000011111111222222223333333344444444"
        ));
        assert!(matches!(
            menu.items()[4].follow_up(),
            FollowUp::DuplicateExactTarget { revision }
                if revision == "0000000011111111222222223333333344444444"
        ));
        assert!(matches!(
            menu.items()[5].follow_up(),
            FollowUp::RestoreExactTarget { revision, path }
                if revision == "0000000011111111222222223333333344444444" && path.is_none()
        ));
        assert!(matches!(
            menu.items()[6].follow_up(),
            FollowUp::RevertExactTarget { revision }
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

        assert_eq!(menu.items().len(), 6);
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
        assert!(
            menu.items()[3]
                .label()
                .contains("absorb current revision ccccdddd into 2 candidate destinations")
        );
        assert!(matches!(
            menu.items()[3].follow_up(),
            FollowUp::AbsorbCandidates {
                source,
                destinations,
            } if source == "ccccdddd1111111111111111111111111111111111"
                && destinations == &vec![
                    "aaaabbbb1111111111111111111111111111111111".to_owned(),
                    "eeeeffff2222222222222222222222222222222222".to_owned()
                ]
        ));
        assert!(matches!(
            menu.items()[4].follow_up(),
            FollowUp::RestoreExactTarget { revision, path }
                if revision == "ccccdddd1111111111111111111111111111111111" && path.is_none()
        ));
        assert!(matches!(
            menu.items()[5].follow_up(),
            FollowUp::RevertExactTarget { revision }
                if revision == "ccccdddd1111111111111111111111111111111111"
        ));
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
            vec![
                ActionKind::New,
                ActionKind::Rebase,
                ActionKind::Squash,
                ActionKind::Absorb,
                ActionKind::Restore,
                ActionKind::Revert
            ]
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
            vec![
                ActionKind::Edit,
                ActionKind::New,
                ActionKind::Split,
                ActionKind::Abandon,
                ActionKind::Duplicate,
                ActionKind::Restore,
                ActionKind::Revert
            ]
        );
        assert!(matches!(
            menu.items()[1].follow_up(),
            FollowUp::NewParents { parents }
                if parents == &vec!["ccccdddd1111111111111111111111111111111111".to_owned()]
        ));
        assert!(matches!(
            menu.items()[2].follow_up(),
            FollowUp::SplitExactTarget { revision }
                if revision == "ccccdddd1111111111111111111111111111111111"
        ));
    }

    #[test]
    fn visible_working_copy_split_uses_current_follow_up() {
        let context =
            ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
                .with_visible_working_copy();
        let menu = build_action_menu(&context);

        assert_eq!(menu.items()[2].action(), ActionKind::Split);
        assert_eq!(
            menu.items()[2].label(),
            "split current working-copy change @"
        );
        assert!(matches!(
            menu.items()[2].follow_up(),
            FollowUp::SplitCurrentWorkingCopy
        ));
    }

    #[test]
    fn detail_context_offers_duplicate_restore_and_revert() {
        let menu = build_action_menu(&ExactActionContext::detail(
            "ccccdddd1111111111111111111111111111111111",
        ));

        let actions = menu
            .items()
            .iter()
            .map(ActionMenuItem::action)
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![
                ActionKind::Duplicate,
                ActionKind::Restore,
                ActionKind::Revert
            ]
        );
    }

    #[test]
    fn detail_context_with_selected_path_offers_path_restore_first() {
        let menu = build_action_menu(
            &ExactActionContext::detail("ccccdddd1111111111111111111111111111111111")
                .with_selected_path("src/quoted path.rs"),
        );

        let actions = menu
            .items()
            .iter()
            .map(ActionMenuItem::action)
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![
                ActionKind::Restore,
                ActionKind::Duplicate,
                ActionKind::Restore,
                ActionKind::Revert
            ]
        );
        assert_eq!(menu.items()[0].shortcut(), 'p');
        assert_eq!(menu.items()[1].shortcut(), 'd');
        assert_eq!(menu.items()[2].shortcut(), 'r');
        assert_eq!(menu.items()[3].shortcut(), 'v');
        assert_eq!(
            menu.item_for_shortcut('p').map(ActionMenuItem::label),
            Some("restore selected path from ccccdddd")
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::RestoreExactTarget { revision, path }
                if revision == "ccccdddd1111111111111111111111111111111111"
                    && path.as_deref() == Some("src/quoted path.rs")
        ));
    }

    #[test]
    fn status_path_context_offers_only_working_copy_path_restore() {
        let menu = build_action_menu(&ExactActionContext::status_path("src/status.rs"));

        assert_eq!(menu.items().len(), 1);
        assert_eq!(menu.items()[0].action(), ActionKind::Restore);
        assert_eq!(menu.items()[0].shortcut(), 'r');
        assert_eq!(
            menu.items()[0].label(),
            "restore selected status path src/status.rs"
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::RestoreWorkingCopyPath { path } if path == "src/status.rs"
        ));
    }
}
