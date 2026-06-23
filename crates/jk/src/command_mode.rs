use std::io;
use std::path::Path;
use std::process::Output;

use jk_core::{ExecutionMode, InspectionSnapshot, JjCommandSpec, RefreshPlan, SafetyClass};

pub fn command_mode_spec(argv: Vec<String>, repository: Option<&Path>) -> JjCommandSpec {
    let mut spec = JjCommandSpec::render_read_only(argv)
        .with_mode(ExecutionMode::CommandMode)
        .with_safety(SafetyClass::LocalMetadata)
        .with_refresh_plan(RefreshPlan::None);
    if let Some(repository) = repository {
        spec = spec.with_repository(repository);
    }
    let title = format!(": {}", spec.preview());
    spec.with_title(title)
}

pub fn jj_command_lines(input: &str, error: Option<&str>) -> Vec<String> {
    let mut lines = vec![format!(": {input}")];
    if let Some(error) = error {
        lines.push(format!("error: {error}"));
    }
    lines.push(String::new());
    lines.push("enter run   Ctrl-u clear   backspace edit   esc cancel".to_owned());
    lines
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QuoteMode {
    Single,
    Double,
}

pub fn parse_jj_command_args(input: &str) -> std::result::Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut in_arg = false;
    let mut chars = input.chars();

    while let Some(character) = chars.next() {
        match quote {
            Some(QuoteMode::Single) => {
                if character == '\'' {
                    quote = None;
                } else {
                    current.push(character);
                    in_arg = true;
                }
            }
            Some(QuoteMode::Double) => match character {
                '"' => quote = None,
                '\\' => {
                    let Some(next) = chars.next() else {
                        return Err("dangling escape".to_owned());
                    };
                    current.push(next);
                    in_arg = true;
                }
                _ => {
                    current.push(character);
                    in_arg = true;
                }
            },
            None => match character {
                character if character.is_whitespace() => {
                    if in_arg {
                        args.push(std::mem::take(&mut current));
                        in_arg = false;
                    }
                }
                '\'' => {
                    quote = Some(QuoteMode::Single);
                    in_arg = true;
                }
                '"' => {
                    quote = Some(QuoteMode::Double);
                    in_arg = true;
                }
                '\\' => {
                    let Some(next) = chars.next() else {
                        return Err("dangling escape".to_owned());
                    };
                    current.push(next);
                    in_arg = true;
                }
                _ => {
                    current.push(character);
                    in_arg = true;
                }
            },
        }
    }

    match quote {
        Some(QuoteMode::Single) => Err("unterminated single quote".to_owned()),
        Some(QuoteMode::Double) => Err("unterminated double quote".to_owned()),
        None => {
            if in_arg {
                args.push(current);
            }
            Ok(args)
        }
    }
}

pub fn command_mode_snapshot(
    command_line: &str,
    result: std::result::Result<&Output, &io::Error>,
) -> InspectionSnapshot {
    let rendered = command_mode_rendered(command_line, result);
    InspectionSnapshot::new(command_line, rendered).with_title(command_line)
}

pub fn command_mode_rendered(
    command_line: &str,
    result: std::result::Result<&Output, &io::Error>,
) -> String {
    let mut rendered = String::new();
    rendered.push_str(&format!("Command: {command_line}\n"));
    match result {
        Ok(output) => {
            rendered.push_str(&format!("Status: {}\n", exit_status_label(output)));
            push_command_stream(&mut rendered, "Stdout", &output.stdout);
            push_command_stream(&mut rendered, "Stderr", &output.stderr);
        }
        Err(error) => {
            rendered.push_str("Status: spawn error\n");
            rendered.push_str(&format!("Spawn error: {error}\n"));
            push_command_stream(&mut rendered, "Stdout", &[]);
            push_command_stream(&mut rendered, "Stderr", &[]);
        }
    }
    rendered.push_str("\nActions: e edit/retry command   : run another jj command\n");
    rendered
}

fn exit_status_label(output: &Output) -> String {
    if output.status.success() {
        return "success".to_owned();
    }

    output
        .status
        .code()
        .map_or_else(|| "failed".to_owned(), |code| format!("exit {code}"))
}

fn push_command_stream(rendered: &mut String, label: &str, bytes: &[u8]) {
    rendered.push_str(&format!("\n{label}:\n"));
    let text = String::from_utf8_lossy(bytes);
    if text.is_empty() {
        rendered.push_str("<empty>\n");
    } else {
        rendered.push_str(&text);
        if !text.ends_with('\n') {
            rendered.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: exit_status(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;

        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(not(unix))]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["/C".into(), format!("exit {code}").into()]
            } else {
                vec!["-c".into(), format!("exit {code}").into()]
            })
            .status()
            .expect("exit status fixture")
    }

    #[test]
    fn parser_handles_quotes_and_escapes() {
        assert_eq!(
            parse_jj_command_args("status").expect("valid args"),
            vec!["status"]
        );
        assert_eq!(
            parse_jj_command_args("describe -m 'two words' @").expect("valid args"),
            vec!["describe", "-m", "two words", "@"]
        );
        assert_eq!(
            parse_jj_command_args("log -r \"description(\\\"done\\\")\"").expect("valid args"),
            vec!["log", "-r", "description(\"done\")"]
        );
    }

    #[test]
    fn parser_reports_incomplete_quotes() {
        assert_eq!(
            parse_jj_command_args("describe -m 'unfinished"),
            Err("unterminated single quote".to_owned())
        );
        assert_eq!(
            parse_jj_command_args("log \\"),
            Err("dangling escape".to_owned())
        );
    }

    #[test]
    fn rendered_output_shows_edit_retry_hint() {
        let result = output(0, "clean\n", "");
        let rendered = command_mode_rendered("jj status", Ok(&result));

        assert!(rendered.contains("Actions: e edit/retry command"));
    }

    #[test]
    fn rendered_output_preserves_failure_stderr() {
        let result = output(1, "", "bad revset\n");
        let rendered = command_mode_rendered("jj log -r bad", Ok(&result));

        assert!(rendered.contains("Command: jj log -r bad"));
        assert!(rendered.contains("Status: exit 1"));
        assert!(rendered.contains("Stdout:\n<empty>"));
        assert!(rendered.contains("Stderr:\nbad revset"));
    }
}
