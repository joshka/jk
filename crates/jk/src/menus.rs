use jk_cli::{DiffFormat, LogTemplateSelection};
use jk_tui::command_discovery::{BindingContext, filtered_discovery_len};
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

pub fn clamp_command_discovery_selection(
    context: BindingContext,
    query: &str,
    selected: &mut usize,
) {
    let row_count = filtered_discovery_len(context, query);
    if row_count == 0 {
        *selected = 0;
    } else {
        *selected = (*selected).min(row_count - 1);
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
