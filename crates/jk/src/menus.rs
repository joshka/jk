use jk_cli::{DiffFormat, LogTemplateSelection};
use jk_tui::command_discovery::{ActionMenuRow, BindingContext, action_menu_rows};
use jk_tui::diff_view::DiffView;

#[derive(Clone, Copy)]
pub enum MenuDirection {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewOptionRow {
    LogTemplate,
    DiffFormat(DiffFormat),
    Placeholder,
}

const DIFF_VIEW_OPTION_ROWS: &[ViewOptionRow] = &[
    ViewOptionRow::DiffFormat(DiffFormat::Patch),
    ViewOptionRow::DiffFormat(DiffFormat::Summary),
    ViewOptionRow::DiffFormat(DiffFormat::Stat),
    ViewOptionRow::DiffFormat(DiffFormat::Types),
    ViewOptionRow::DiffFormat(DiffFormat::NameOnly),
    ViewOptionRow::DiffFormat(DiffFormat::Git),
    ViewOptionRow::DiffFormat(DiffFormat::ColorWords),
];

pub fn wrapped_selection(selected: usize, row_count: usize, direction: MenuDirection) -> usize {
    if row_count == 0 {
        return 0;
    }

    let selected = selected.min(row_count - 1);
    match direction {
        MenuDirection::Previous => selected.checked_sub(1).unwrap_or(row_count - 1),
        MenuDirection::Next => (selected + 1) % row_count,
    }
}

pub fn action_menu_lines(context: BindingContext, selected: usize, width: usize) -> Vec<String> {
    let rows = action_menu_rows(context);
    if rows.is_empty() {
        return vec![
            "This view is read-only; no repository actions are available.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ];
    }

    let compact = width < 64;
    let key_width = rows.iter().map(|row| row.key.len()).max().unwrap_or(0);
    let label_width = rows.iter().map(|row| row.label.len()).max().unwrap_or(0);
    let mut lines = Vec::new();
    let mut previous_group = None;
    for (index, row) in rows.iter().enumerate() {
        if previous_group != Some(row.group) {
            if previous_group.is_some() {
                lines.push(String::new());
            }
            lines.push(format!("{}:", row.group.label()));
            previous_group = Some(row.group);
        }
        lines.push(action_menu_row_line(
            *row,
            index == selected,
            key_width,
            label_width,
            compact,
        ));
    }

    lines.push(String::new());
    lines.push(if compact {
        "enter open   key select   esc close".to_owned()
    } else {
        "Safety cues show whether actions preview, save, or run now; inspection views stay read-only."
            .to_owned()
    });
    if !compact {
        lines.push("j/k or arrows move   enter open   action key select   esc close".to_owned());
    }
    lines
}

fn action_menu_row_line(
    row: ActionMenuRow,
    selected: bool,
    key_width: usize,
    label_width: usize,
    compact: bool,
) -> String {
    let marker = if selected { ">" } else { " " };
    let safety = if compact {
        row.safety.compact_label()
    } else {
        row.safety.label()
    };
    if compact {
        format!("{marker} {:<key_width$}  {}  {safety}", row.key, row.label)
    } else {
        format!(
            "{marker} {:<key_width$}  {:<label_width$}  {safety}",
            row.key, row.label
        )
    }
}

pub const fn view_option_rows(context: BindingContext) -> &'static [ViewOptionRow] {
    match context {
        BindingContext::Log => &[ViewOptionRow::LogTemplate],
        BindingContext::Diff => DIFF_VIEW_OPTION_ROWS,
        BindingContext::Inspection
        | BindingContext::Workspaces
        | BindingContext::CommandHistory
        | BindingContext::OperationLog => &[ViewOptionRow::Placeholder],
    }
}

pub fn view_options_lines(
    context: BindingContext,
    selected: usize,
    template: &LogTemplateSelection,
    active_diff_format: Option<DiffFormat>,
) -> Vec<String> {
    match context {
        BindingContext::Log => {
            let marker = if selected == 0 { ">" } else { " " };
            vec![
                format!("{marker} {:<18} {}", "Template", template.label()),
                String::new(),
                "j/k or arrows move   enter open   esc close".to_owned(),
            ]
        }
        BindingContext::Diff => {
            let active = active_diff_format.unwrap_or(DiffFormat::Patch);
            let mut lines = DIFF_VIEW_OPTION_ROWS
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    let marker = if index == selected { ">" } else { " " };
                    let ViewOptionRow::DiffFormat(format) = row else {
                        unreachable!("diff view rows are all formats");
                    };
                    let active_marker = if *format == active { "*" } else { " " };
                    format!("{marker} {active_marker} {:<14}", format.label())
                })
                .collect::<Vec<_>>();
            lines.push(String::new());
            lines.push("j/k or arrows move   enter apply   esc close".to_owned());
            lines
        }
        BindingContext::Inspection => vec![
            "No view options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
        BindingContext::Workspaces => vec![
            "No workspace view options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
        BindingContext::CommandHistory => vec![
            "No command history options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
        BindingContext::OperationLog => vec![
            "No operation log options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
    }
}

pub fn diff_file_list_lines(view: &DiffView, selected: usize) -> Vec<String> {
    let paths = view.file_paths();
    if paths.is_empty() {
        return vec![
            "No files in this diff.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ];
    }

    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let marker = if index == selected { ">" } else { " " };
            format!("{marker} {:>2}/{} {path}", index + 1, paths.len())
        })
        .chain(std::iter::once(String::new()))
        .chain(std::iter::once(
            "j/k or arrows move   enter jump   esc close".to_owned(),
        ))
        .collect()
}

pub fn template_selector_lines(options: &[LogTemplateSelection], selected: usize) -> Vec<String> {
    options
        .iter()
        .enumerate()
        .map(|(index, template)| {
            let marker = if index == selected { ">" } else { " " };
            let name = template.template_name().unwrap_or("jj configured template");
            format!("{marker} {:<18} {name}", template.label())
        })
        .chain(std::iter::once(String::new()))
        .chain(std::iter::once(
            "j/k or arrows move   enter apply   esc cancel".to_owned(),
        ))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_selection_wraps_and_clamps() {
        assert_eq!(wrapped_selection(0, 3, MenuDirection::Previous), 2);
        assert_eq!(wrapped_selection(2, 3, MenuDirection::Next), 0);
        assert_eq!(wrapped_selection(99, 3, MenuDirection::Previous), 1);
        assert_eq!(wrapped_selection(99, 3, MenuDirection::Next), 0);
        assert_eq!(wrapped_selection(4, 0, MenuDirection::Next), 0);
    }

    #[test]
    fn action_menu_groups_ranked_actions_and_marks_selection() {
        let lines = action_menu_lines(BindingContext::Log, 3, 80);

        assert_eq!(lines[0], "Change actions:");
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("  m  Describe revision"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("> a  Abandon revision")
                    && line.contains("checks first"))
        );
        assert!(lines.iter().any(|line| line == "History and recovery:"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("inspection views stay read-only"))
        );
    }

    #[test]
    fn action_menu_compacts_without_losing_keys_or_safety() {
        let lines = action_menu_lines(BindingContext::Log, 0, 40);

        assert!(lines.iter().all(|line| line.chars().count() <= 40));
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("> m  Describe revision"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("a  Abandon revision  checks first"))
        );
    }

    #[test]
    fn action_menu_explains_read_only_contexts() {
        assert_eq!(
            action_menu_lines(BindingContext::Inspection, 0, 80),
            vec![
                "This view is read-only; no repository actions are available.".to_owned(),
                String::new(),
                "esc close".to_owned(),
            ]
        );
    }

    #[test]
    fn view_options_lines_show_template_or_placeholder() {
        assert_eq!(
            view_options_lines(BindingContext::Log, 0, &LogTemplateSelection::Oneline, None),
            vec![
                "> Template           oneline".to_owned(),
                String::new(),
                "j/k or arrows move   enter open   esc close".to_owned(),
            ]
        );
        assert_eq!(
            view_options_lines(
                BindingContext::Inspection,
                0,
                &LogTemplateSelection::Configured,
                None
            ),
            vec![
                "No view options in this slice.".to_owned(),
                String::new(),
                "esc close".to_owned(),
            ]
        );
    }

    #[test]
    fn template_selector_lines_show_template_names() {
        assert_eq!(
            template_selector_lines(
                &[
                    LogTemplateSelection::Configured,
                    LogTemplateSelection::Compact,
                ],
                1
            ),
            vec![
                "  configured         jj configured template".to_owned(),
                "> compact            builtin_log_compact".to_owned(),
                String::new(),
                "j/k or arrows move   enter apply   esc cancel".to_owned(),
            ]
        );
    }
}
