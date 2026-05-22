use crate::command::{Binding, Command};

use super::metadata::help_metadata;
use super::{HelpContext, HelpRow, HelpSection, HelpSectionKind};

/// Projects global and view-local bindings into grouped help sections for one context.
pub fn project_help(
    global_bindings: &[Binding],
    view_bindings: &[Binding],
    context: HelpContext,
) -> Vec<HelpSection> {
    let global_rows = collect_help_rows(global_bindings, context);
    let view_rows = collect_help_rows(view_bindings, context);

    [
        HelpSectionKind::Navigation,
        HelpSectionKind::Views,
        HelpSectionKind::SearchCopy,
        HelpSectionKind::RepositoryActions,
        HelpSectionKind::Actions,
        HelpSectionKind::Recovery,
        HelpSectionKind::App,
    ]
    .into_iter()
    .filter_map(|kind| {
        let rows = global_rows
            .iter()
            .chain(&view_rows)
            .filter(|(row_kind, _)| *row_kind == kind)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        (!rows.is_empty()).then(|| HelpSection::new(kind, rows))
    })
    .collect()
}

/// Returns whether a command should appear in help for the given context.
pub(crate) fn command_is_visible_in_help(command: Command, context: HelpContext) -> bool {
    help_metadata(command, context).is_some()
}

/// Collects visible help rows from one binding slice, merging duplicate commands by label.
fn collect_help_rows(
    bindings: &[Binding],
    context: HelpContext,
) -> Vec<(HelpSectionKind, HelpRow)> {
    let mut rows: Vec<(HelpSectionKind, Command, HelpRow)> = Vec::new();

    for binding in bindings {
        let command = binding.command();
        let Some((kind, action)) = help_metadata(command, context) else {
            continue;
        };
        let key = binding.key_label();

        if let Some((_, _, row)) = rows.iter_mut().find(|(row_kind, row_command, row)| {
            *row_kind == kind && *row_command == command && row.action() == action
        }) {
            row.push_key_label(&key);
        } else {
            rows.push((kind, command, HelpRow::new(key, action)));
        }
    }

    rows.into_iter().map(|(kind, _, row)| (kind, row)).collect()
}
