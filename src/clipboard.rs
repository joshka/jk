//! Thin wrapper around the system clipboard.

use color_eyre::Result;

/// Write text to the system clipboard.
pub fn copy(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text.to_owned())?;
    Ok(())
}
