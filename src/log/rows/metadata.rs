use color_eyre::Result;

use crate::jj::{ViewSpec, run_jj_template_lines};
use crate::rendered_rows::RowMetadata;

/// Load row metadata using a narrow template that can be paired back to rendered rows.
pub(super) fn run_jj_with_template(
    spec: &ViewSpec,
    template: &str,
) -> Result<RowMetadata<RevisionMetadata>> {
    Ok(parse_revision_metadata_lines(run_jj_template_lines(
        spec, template, false,
    )?))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RevisionMetadata {
    /// Full change id emitted by the metadata template for one rendered revision row.
    pub(super) change_id: String,
    /// Full commit id emitted alongside the change id when metadata pairing remains aligned.
    pub(super) commit_id: Option<String>,
}

pub(super) fn parse_revision_metadata_lines(lines: Vec<String>) -> RowMetadata<RevisionMetadata> {
    let mut metadata = Vec::new();
    for line in lines {
        if is_graph_only_revision_metadata_line(&line) {
            continue;
        }
        let Some(row) = parse_metadata_line(&line) else {
            return RowMetadata::Drifted;
        };
        metadata.push(row);
    }
    RowMetadata::Valid(metadata)
}

pub(super) fn parse_metadata_line(line: &str) -> Option<RevisionMetadata> {
    let line = line
        .char_indices()
        .find(|(_, character)| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
        .map(|(index, _)| &line[index..])?;

    let line = line
        .strip_prefix("@  ")
        .or_else(|| line.strip_prefix("○  "))
        .or_else(|| line.strip_prefix("◆  "))?;

    let mut tokens = line.split_whitespace();
    let change_id = tokens.next()?;
    let commit_id = tokens.next()?;

    if tokens.next().is_some() || line != format!("{change_id} {commit_id}") {
        return None;
    }

    if !is_full_change_id(change_id) || !is_full_commit_id(commit_id) {
        return None;
    }

    Some(RevisionMetadata {
        change_id: change_id.to_owned(),
        commit_id: Some(commit_id.to_owned()),
    })
}

fn is_graph_only_revision_metadata_line(line: &str) -> bool {
    let text = line.trim_start_matches(|character| {
        matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮')
    });

    text.is_empty() || text == "~" || text == "~  (elided revisions)"
}

fn is_full_commit_id(token: &str) -> bool {
    token.len() == 40 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_full_change_id(token: &str) -> bool {
    token.len() == 32 && token.bytes().all(|byte| byte.is_ascii_lowercase())
}
