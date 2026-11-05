//! Terminal presentation helpers: ANSI styling, the "Thinking..." spinner,
//! and clipboard access. Keeps escape codes out of the rest of the code.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::time::Duration;

fn paint(code: &str, text: impl Display) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

pub fn bold(text: impl Display) -> String {
    paint("1", text)
}

pub fn dim(text: impl Display) -> String {
    paint("90", text)
}

pub fn success(text: impl Display) -> String {
    paint("1;32", text)
}

pub fn warning(text: impl Display) -> String {
    paint("1;33", text)
}

pub fn error(text: impl Display) -> String {
    paint("1;31", text)
}

pub fn heading(text: impl Display) -> String {
    paint("1;36", text)
}

/// Starts a steady-ticking spinner with the given message.
pub fn spinner(message: &'static str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .expect("valid spinner template"),
    );
    spinner.set_message(message);
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    arboard::Clipboard::new()
        .context("failed to access clipboard")?
        .set_text(text.to_owned())
        .context("failed to copy to clipboard")
}
