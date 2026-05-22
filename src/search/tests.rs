use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use super::*;

#[test]
fn search_query_uses_smart_case() {
    let query = SearchQuery::new("cargo".to_owned()).unwrap();

    assert!(query.matches("Cargo.toml"));

    let query = SearchQuery::new("Cargo".to_owned()).unwrap();

    assert!(query.matches("Cargo.toml"));
    assert!(!query.matches("cargo.toml"));
}

#[test]
fn highlight_line_reverses_only_matching_text() {
    let query = SearchQuery::new("arg".to_owned()).unwrap();
    let line = Line::from("Cargo.toml");

    let highlighted = highlight_line(line, Some(&query));

    assert_eq!(highlighted.spans.len(), 3);
    assert_eq!(highlighted.spans[0].content.as_ref(), "C");
    assert!(
        !highlighted.spans[0]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert_eq!(highlighted.spans[1].content.as_ref(), "arg");
    assert!(
        highlighted.spans[1]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert_eq!(highlighted.spans[2].content.as_ref(), "o.toml");
    assert!(
        !highlighted.spans[2]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
}

#[test]
fn highlight_line_preserves_existing_style_around_match() {
    let query = SearchQuery::new("cargo".to_owned()).unwrap();
    let style = ratatui::style::Style::default().fg(ratatui::style::Color::Green);
    let line = Line::from(Span::styled("Cargo.toml", style));

    let highlighted = highlight_line(line, Some(&query));

    assert_eq!(highlighted.spans[0].content.as_ref(), "Cargo");
    assert_eq!(
        highlighted.spans[0].style.fg,
        Some(ratatui::style::Color::Green)
    );
    assert!(
        highlighted.spans[0]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert_eq!(highlighted.spans[1].content.as_ref(), ".toml");
    assert_eq!(
        highlighted.spans[1].style.fg,
        Some(ratatui::style::Color::Green)
    );
    assert!(
        !highlighted.spans[1]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
}

#[test]
fn highlight_line_handles_matches_split_across_spans() {
    let query = SearchQuery::new("cargo".to_owned()).unwrap();
    let line = Line::from(vec![Span::raw("Car"), Span::raw("go.toml")]);

    let highlighted = highlight_line(line, Some(&query));

    assert_eq!(highlighted.spans.len(), 3);
    assert_eq!(highlighted.spans[0].content.as_ref(), "Car");
    assert!(
        highlighted.spans[0]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert_eq!(highlighted.spans[1].content.as_ref(), "go");
    assert!(
        highlighted.spans[1]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
    assert_eq!(highlighted.spans[2].content.as_ref(), ".toml");
    assert!(
        !highlighted.spans[2]
            .style
            .add_modifier
            .contains(Modifier::REVERSED)
    );
}
