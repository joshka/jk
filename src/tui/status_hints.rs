//! Status-bar hint vocabulary and width-fit projection.

use ratatui::text::{Line, Span};
use ratatui_macros::span;

use crate::theme;

#[derive(Clone, Copy, Debug)]
pub enum StatusHints {
    Graph,
    ShowDocument,
    DiffDocument,
    Status,
    Resolve,
    FileList,
    FileShowDocument,
    Bookmarks,
    Workspaces,
    OperationLog,
    OperationDetailDocument,
}

pub fn status_hint_spans(hints: StatusHints, width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used_width = 0;

    for hint in status_hint_candidates(hints) {
        let separator_width = if spans.is_empty() { 0 } else { 2 };
        let item_width = text_width(hint.key) + 1 + text_width(hint.label);
        if used_width + separator_width + item_width > usize::from(width) {
            break;
        }

        if !spans.is_empty() {
            spans.push(Span::raw("  "));
        }
        spans.push(key(hint.key));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(hint.label));
        used_width += separator_width + item_width;
    }

    Line::from(spans)
}

#[derive(Clone, Copy)]
struct StatusHint {
    key: &'static str,
    label: &'static str,
}

impl StatusHint {
    const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

const GRAPH_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("PgUp/PgDn", "page"),
    StatusHint::new("Enter/l", "open"),
    StatusHint::new("Space", "select"),
    StatusHint::new("a", "action"),
    StatusHint::new("S", "status"),
    StatusHint::new("f/p/r", "sync"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const DOCUMENT_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("g/G", "ends"),
    StatusHint::new("[/]", "file"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const STATUS_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("a", "actions"),
    StatusHint::new("D/C", "describe/commit @"),
    StatusHint::new("b/=/m", "bookmark @"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const RESOLVE_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/l", "inspect"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const FILE_LIST_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/l", "open"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("a", "action"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const FILE_SHOW_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("/", "search"),
    StatusHint::new("a", "action"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const OPERATION_DETAIL_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("s/d", "show/diff"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const BOOKMARKS_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/s", "show"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("x", "delete"),
    StatusHint::new("br", "rename"),
    StatusHint::new("bf", "forget"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const WORKSPACES_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const OPERATION_LOG_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("u", "undo"),
    StatusHint::new("C-r", "redo"),
    StatusHint::new("s", "show"),
    StatusHint::new("d", "diff"),
    StatusHint::new("a", "action"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy id"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];

fn status_hint_candidates(hints: StatusHints) -> &'static [StatusHint] {
    match hints {
        StatusHints::Graph => GRAPH_STATUS_HINTS,
        StatusHints::ShowDocument | StatusHints::DiffDocument => DOCUMENT_STATUS_HINTS,
        StatusHints::Status => STATUS_STATUS_HINTS,
        StatusHints::Resolve => RESOLVE_STATUS_HINTS,
        StatusHints::FileList => FILE_LIST_STATUS_HINTS,
        StatusHints::FileShowDocument => FILE_SHOW_STATUS_HINTS,
        StatusHints::OperationDetailDocument => OPERATION_DETAIL_STATUS_HINTS,
        StatusHints::Bookmarks => BOOKMARKS_STATUS_HINTS,
        StatusHints::Workspaces => WORKSPACES_STATUS_HINTS,
        StatusHints::OperationLog => OPERATION_LOG_STATUS_HINTS,
    }
}

fn text_width(text: &str) -> usize {
    text.chars().count()
}

fn key(label: &'static str) -> Span<'static> {
    span!(theme::key_style(); "{label}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_text(line: Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn status_hint_spans_fit_complete_items_only() {
        assert_eq!(
            plain_text(status_hint_spans(StatusHints::Graph, 27)),
            "j/k move  PgUp/PgDn page"
        );
    }

    #[test]
    fn status_hint_spans_return_empty_when_first_item_does_not_fit() {
        assert_eq!(plain_text(status_hint_spans(StatusHints::Graph, 7)), "");
    }
}
