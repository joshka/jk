pub fn validate_bookmark_rename_new_name(
    old_name: &str,
    new_name: &str,
) -> std::result::Result<(), String> {
    if new_name.is_empty() {
        return Err("empty bookmark name".to_owned());
    }
    if new_name == old_name {
        return Err("new bookmark name is unchanged".to_owned());
    }
    if new_name == "@" {
        return Err("bookmark name must not be @".to_owned());
    }
    if new_name.starts_with('-') {
        return Err("bookmark name must not start with '-'".to_owned());
    }
    if new_name.starts_with('/') || new_name.ends_with('/') || new_name.contains("//") {
        return Err("bookmark name must not contain empty path components".to_owned());
    }
    if new_name.starts_with('.') || new_name.contains("/.") {
        return Err("bookmark name components must not start with '.'".to_owned());
    }
    if new_name.ends_with('.') || new_name.ends_with(".lock") {
        return Err("bookmark name must not end with '.' or '.lock'".to_owned());
    }
    if new_name.contains("..") {
        return Err("bookmark name must not contain '..'".to_owned());
    }
    if new_name
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("bookmark name must not contain whitespace or control characters".to_owned());
    }
    if new_name
        .chars()
        .any(|character| matches!(character, '@' | ':' | '?' | '*' | '[' | '\\' | '^' | '~'))
    {
        return Err("bookmark name contains a reserved ref character".to_owned());
    }

    Ok(())
}
