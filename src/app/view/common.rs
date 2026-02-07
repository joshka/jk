pub(crate) fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub(crate) fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

pub(crate) fn is_working_copy_change_line(line: &str) -> bool {
    let stripped = super::strip_ansi(line);
    let mut chars = stripped.chars();
    match (chars.next(), chars.next()) {
        (Some(status), Some(' ')) => matches!(status, 'M' | 'A' | 'D' | 'R' | 'C' | '?' | 'U'),
        _ => false,
    }
}
