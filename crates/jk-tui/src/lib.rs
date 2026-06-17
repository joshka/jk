//! Ratatui views and interaction state for `jk`.
//!
//! The public surface is currently the log view in [`log_view`]. It accepts a
//! [`jk_core::LogSnapshot`], applies input actions, and renders a borderless view that keeps the
//! `jj` log body visually intact while adding title/status chrome and selected-row highlighting.

pub mod log_view;

mod ansi_text;
mod chrome;
mod log_state;
mod rendered_log;
mod selected_row;
