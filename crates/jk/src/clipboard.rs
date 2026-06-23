use std::io::{self, Write};

pub fn copy_command_line(command_line: &str) -> String {
    match write_terminal_clipboard(command_line) {
        Ok(()) => "copied command".to_owned(),
        Err(error) => format!("copy failed: {error}"),
    }
}

fn write_terminal_clipboard(text: &str) -> io::Result<()> {
    let sequence = osc52_sequence(text);
    let mut stdout = io::stdout();
    stdout.write_all(sequence.as_bytes())?;
    stdout.flush()
}

fn osc52_sequence(text: &str) -> String {
    format!("\u{1b}]52;c;{}\u{7}", base64_encode(text.as_bytes()))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        let value = (u32::from(first) << 16) | (u32::from(second) << 8) | u32::from(third);

        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(value & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_handles_padding() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"j"), "ag==");
        assert_eq!(base64_encode(b"jj"), "amo=");
        assert_eq!(base64_encode(b"jj undo"), "amogdW5kbw==");
    }

    #[test]
    fn osc52_sequence_wraps_encoded_clipboard_payload() {
        assert_eq!(osc52_sequence("jj undo"), "\u{1b}]52;c;amogdW5kbw==\u{7}");
    }
}
