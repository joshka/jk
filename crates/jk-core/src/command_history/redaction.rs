use std::ffi::OsString;

use super::REDACTED;

pub fn redact_argv(argv: Vec<OsString>) -> Vec<OsString> {
    argv.into_iter()
        .map(|arg| {
            let Some(arg) = arg.to_str() else {
                return arg;
            };
            OsString::from(redact_text(arg).0)
        })
        .collect()
}

pub(super) fn redact_text(text: &str) -> (String, bool) {
    let mut output = String::with_capacity(text.len());
    let mut redacted = false;
    let mut index = 0;

    while let Some((start, separator, redact_to_line)) = find_secret_assignment(&text[index..]) {
        let absolute_start = index + start;
        let value_start = absolute_start + separator;
        output.push_str(&text[index..value_start]);

        let value_end = if redact_to_line {
            find_line_end(text, value_start)
        } else {
            find_value_end(text, value_start)
        };
        output.push_str(REDACTED);
        redacted = true;
        index = value_end;
    }

    output.push_str(&text[index..]);
    (output, redacted)
}

fn find_secret_assignment(text: &str) -> Option<(usize, usize, bool)> {
    for (index, character) in text.char_indices() {
        if character != '=' && character != ':' {
            continue;
        }

        let key_start = text[..index]
            .char_indices()
            .rev()
            .find_map(|(position, character)| {
                if character.is_ascii_alphanumeric() || "_-. ".contains(character) {
                    None
                } else {
                    Some(position + character.len_utf8())
                }
            })
            .unwrap_or(0);
        let key = text[key_start..index].trim();
        if is_secret_key(key) {
            return Some((
                key_start,
                index + character.len_utf8() - key_start,
                is_authorization_key(key),
            ));
        }
    }
    None
}

fn find_value_end(text: &str, value_start: usize) -> usize {
    let mut saw_quote = None;
    let mut start = value_start;
    while let Some(character) = text[start..].chars().next() {
        if character.is_ascii_whitespace() {
            start += character.len_utf8();
        } else if character == '"' || character == '\'' {
            saw_quote = Some(character);
            start += character.len_utf8();
            break;
        } else {
            break;
        }
    }

    let value_end = text[start..]
        .char_indices()
        .find_map(|(offset, character)| {
            if Some(character) == saw_quote
                || (saw_quote.is_none() && (character.is_ascii_whitespace() || character == ','))
            {
                Some(start + offset)
            } else {
                None
            }
        })
        .unwrap_or(text.len());

    if let Some(quote) = saw_quote
        && text[value_end..].starts_with(quote)
    {
        return value_end + quote.len_utf8();
    }
    value_end
}

fn find_line_end(text: &str, value_start: usize) -> usize {
    text[value_start..]
        .find('\n')
        .map_or(text.len(), |offset| value_start + offset)
}

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["token", "secret", "password", "credential", "auth"]
        .iter()
        .any(|needle| key.contains(needle))
        || key
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|part| part == "key")
}

fn is_authorization_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("authorization") || key == "auth"
}
