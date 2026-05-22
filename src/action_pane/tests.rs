use super::*;

fn output_with_lines(count: usize) -> ActionPane {
    ActionPane::pending(
        "jj action --preview".to_owned(),
        (0..count)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        Some("context".to_owned()),
    )
}

#[test]
fn scroll_clamps_to_readable_boundaries() {
    let mut output = output_with_lines(8);

    output.page_down(4);
    output.page_down(4);
    output.page_down(4);

    assert_eq!(output.scroll(), output.max_scroll(4));

    output.page_up(4);
    output.page_up(4);
    output.page_up(4);

    assert_eq!(output.scroll(), 0);
}

#[test]
fn body_lines_keep_command_context_and_multiline_output() {
    let output = ActionPane::pending(
        "jj git push --preview".to_owned(),
        "first\n\nthird".to_owned(),
        Some("status push uses jj default target".to_owned()),
    );

    assert_eq!(
        output.body_lines(),
        [
            "command: jj git push --preview",
            "context: status push uses jj default target",
            "output:",
            "  first",
            "  ",
            "  third",
        ]
    );
}

#[test]
fn key_handling_maps_preview_commands_and_scrolls_output() {
    let mut output = output_with_lines(8);

    assert_eq!(
        handle_action_pane_key(KeyCode::PageDown, &mut output, 4),
        ActionPaneKey::Handled
    );
    assert_eq!(output.scroll(), 4);

    assert_eq!(
        handle_action_pane_key(KeyCode::Enter, &mut output, 4),
        ActionPaneKey::Primary
    );
    assert_eq!(
        handle_action_pane_key(KeyCode::Esc, &mut output, 4),
        ActionPaneKey::Cancel
    );
    assert_eq!(
        handle_action_pane_key(KeyCode::Char('x'), &mut output, 4),
        ActionPaneKey::Ignored
    );
}

#[test]
fn visible_lines_never_drop_below_one() {
    assert_eq!(action_pane_visible_lines(0), 1);
    assert_eq!(action_pane_visible_lines(1), 1);
    assert_eq!(action_pane_visible_lines(5), 4);
}
