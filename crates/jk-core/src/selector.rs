//! Provider-neutral selector decisions and pure resolution.
//!
//! A view owns its cursor, marks, search, and refresh policy. This module starts at the handoff
//! from that interaction state to a workflow: the caller submits a cursor plus optional ordered
//! choices, or reports cancellation, and receives an explicit resolution outcome.

/// The kind of repository object being selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectorKind {
    /// A revision or revset result.
    Revision,
    /// One or more file expressions.
    Fileset,
    /// A repository operation.
    Operation,
    /// A bookmark.
    Bookmark,
    /// A tag.
    Tag,
    /// A Git remote.
    Remote,
    /// A jj workspace.
    Workspace,
}

/// The command-facing role filled by a selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectorRole {
    /// Objects changed or inspected by a workflow.
    Source,
    /// The destination of a move or relationship.
    Destination,
    /// Parents assigned to a new or existing revision.
    Parent,
    /// A single general-purpose target.
    Target,
}

/// The number of values accepted for a selector role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionCardinality {
    /// Exactly one value is required.
    One,
    /// One or more values are required.
    OneOrMore,
}

/// Where the ordering of a resolved selection came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionOrder {
    /// The current cursor supplied the only value.
    Cursor,
    /// The user explicitly ordered values, such as revision marks.
    Explicit,
}

/// The selector contract requested by a workflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionRequest {
    kind: SelectorKind,
    role: SelectorRole,
    cardinality: SelectionCardinality,
}

impl SelectionRequest {
    /// Requires exactly one value for `role`.
    #[must_use]
    pub const fn one(kind: SelectorKind, role: SelectorRole) -> Self {
        Self {
            kind,
            role,
            cardinality: SelectionCardinality::One,
        }
    }

    /// Requires one or more ordered values for `role`.
    #[must_use]
    pub const fn one_or_more(kind: SelectorKind, role: SelectorRole) -> Self {
        Self {
            kind,
            role,
            cardinality: SelectionCardinality::OneOrMore,
        }
    }

    /// Returns the selected object kind.
    #[must_use]
    pub const fn kind(self) -> SelectorKind {
        self.kind
    }

    /// Returns the command-facing role.
    #[must_use]
    pub const fn role(self) -> SelectorRole {
        self.role
    }

    /// Returns the required cardinality.
    #[must_use]
    pub const fn cardinality(self) -> SelectionCardinality {
        self.cardinality
    }
}

/// Cursor and ordered choices submitted by an active selector.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionCandidates<T> {
    cursor: Option<T>,
    ordered: Vec<T>,
}

impl<T> SelectionCandidates<T> {
    /// Creates candidates from the current cursor and explicitly ordered choices.
    #[must_use]
    pub const fn new(cursor: Option<T>, ordered: Vec<T>) -> Self {
        Self { cursor, ordered }
    }

    /// Creates candidates with only a cursor value.
    #[must_use]
    pub const fn cursor(cursor: Option<T>) -> Self {
        Self {
            cursor,
            ordered: Vec::new(),
        }
    }
}

/// The user's terminal decision for a selector interaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionDecision<T> {
    /// Resolve the submitted cursor and ordered choices.
    Submit(SelectionCandidates<T>),
    /// Close the selector without choosing anything.
    Cancel,
}

/// Why submitted selector state is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidSelection {
    /// No cursor or ordered choice was supplied.
    Missing,
    /// An explicitly ordered value appeared more than once.
    Duplicate,
}

/// A successful selection with explicit role and ordering provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSelection<T> {
    request: SelectionRequest,
    order: SelectionOrder,
    values: Vec<T>,
}

impl<T> ResolvedSelection<T> {
    /// Returns the contract this selection fills.
    #[must_use]
    pub const fn request(&self) -> SelectionRequest {
        self.request
    }

    /// Returns the source of the resolved ordering.
    #[must_use]
    pub const fn order(&self) -> SelectionOrder {
        self.order
    }

    /// Returns selected values in command-facing order.
    #[must_use]
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Consumes the resolution and returns selected values in command-facing order.
    #[must_use]
    pub fn into_values(self) -> Vec<T> {
        self.values
    }
}

/// The explicit outcome of resolving selector interaction state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionResolution<T> {
    /// The selection satisfies the requested role and cardinality.
    Resolved(ResolvedSelection<T>),
    /// More values were submitted than a single-value role accepts.
    Ambiguous {
        /// The contract that could not choose one value.
        request: SelectionRequest,
        /// Candidate values in their explicit order.
        candidates: Vec<T>,
    },
    /// Submitted state could not form a valid selection.
    Invalid {
        /// The contract that rejected the state.
        request: SelectionRequest,
        /// The invalid-state category.
        reason: InvalidSelection,
    },
    /// The user cancelled without submitting a selection.
    Cancelled {
        /// The contract that was cancelled.
        request: SelectionRequest,
    },
}

/// Resolves provider-neutral selector state without rendering, I/O, or command construction.
#[must_use]
pub fn resolve_selection<T: Eq>(
    request: SelectionRequest,
    decision: SelectionDecision<T>,
) -> SelectionResolution<T> {
    let SelectionDecision::Submit(candidates) = decision else {
        return SelectionResolution::Cancelled { request };
    };

    if has_duplicates(&candidates.ordered) {
        return SelectionResolution::Invalid {
            request,
            reason: InvalidSelection::Duplicate,
        };
    }

    let (values, order) = if candidates.ordered.is_empty() {
        let Some(cursor) = candidates.cursor else {
            return SelectionResolution::Invalid {
                request,
                reason: InvalidSelection::Missing,
            };
        };
        (vec![cursor], SelectionOrder::Cursor)
    } else {
        (candidates.ordered, SelectionOrder::Explicit)
    };

    if request.cardinality == SelectionCardinality::One && values.len() > 1 {
        return SelectionResolution::Ambiguous {
            request,
            candidates: values,
        };
    }

    SelectionResolution::Resolved(ResolvedSelection {
        request,
        order,
        values,
    })
}

fn has_duplicates<T: Eq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARENTS: SelectionRequest =
        SelectionRequest::one_or_more(SelectorKind::Revision, SelectorRole::Parent);
    const OPERATION: SelectionRequest =
        SelectionRequest::one(SelectorKind::Operation, SelectorRole::Target);

    #[test]
    fn ordered_values_override_cursor_and_preserve_explicit_order() {
        let decision = SelectionDecision::Submit(SelectionCandidates::new(
            Some("cursor"),
            vec!["third", "first"],
        ));

        let SelectionResolution::Resolved(resolved) = resolve_selection(PARENTS, decision) else {
            panic!("ordered revision parents should resolve");
        };

        assert_eq!(resolved.request(), PARENTS);
        assert_eq!(resolved.order(), SelectionOrder::Explicit);
        assert_eq!(resolved.values(), ["third", "first"]);
    }

    #[test]
    fn cursor_is_the_fallback_for_an_empty_ordered_selection() {
        let decision = SelectionDecision::Submit(SelectionCandidates::cursor(Some("cursor")));

        let SelectionResolution::Resolved(resolved) = resolve_selection(PARENTS, decision) else {
            panic!("cursor revision should resolve");
        };

        assert_eq!(resolved.order(), SelectionOrder::Cursor);
        assert_eq!(resolved.values(), ["cursor"]);
    }

    #[test]
    fn one_value_role_reports_ambiguous_ordered_candidates() {
        let decision =
            SelectionDecision::Submit(SelectionCandidates::new(None, vec!["op-2", "op-1"]));

        assert_eq!(
            resolve_selection(OPERATION, decision),
            SelectionResolution::Ambiguous {
                request: OPERATION,
                candidates: vec!["op-2", "op-1"],
            }
        );
    }

    #[test]
    fn missing_selection_is_invalid_instead_of_an_empty_success() {
        let decision = SelectionDecision::Submit(SelectionCandidates::<&str>::default());

        assert_eq!(
            resolve_selection(PARENTS, decision),
            SelectionResolution::Invalid {
                request: PARENTS,
                reason: InvalidSelection::Missing,
            }
        );
    }

    #[test]
    fn duplicate_ordered_values_are_invalid() {
        let decision =
            SelectionDecision::Submit(SelectionCandidates::new(None, vec!["same", "same"]));

        assert_eq!(
            resolve_selection(PARENTS, decision),
            SelectionResolution::Invalid {
                request: PARENTS,
                reason: InvalidSelection::Duplicate,
            }
        );
    }

    #[test]
    fn cancellation_is_distinct_from_invalid_state() {
        assert_eq!(
            resolve_selection::<&str>(OPERATION, SelectionDecision::Cancel),
            SelectionResolution::Cancelled { request: OPERATION }
        );
    }

    #[test]
    fn extension_kinds_share_resolution_without_provider_metadata() {
        for kind in [
            SelectorKind::Bookmark,
            SelectorKind::Tag,
            SelectorKind::Remote,
            SelectorKind::Workspace,
        ] {
            let request = SelectionRequest::one(kind, SelectorRole::Target);
            let decision = SelectionDecision::Submit(SelectionCandidates::cursor(Some("id")));
            assert!(matches!(
                resolve_selection(request, decision),
                SelectionResolution::Resolved(_)
            ));
        }
    }
}
