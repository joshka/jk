use ratatui::Frame;
use ratatui::widgets::Clear;

use crate::app::actions::ActionPane;
use crate::app::status_line::StatusLine;
use crate::command::HelpSection;
use crate::menus::{ActionMenu, CopyOption, RolePrompt};
use crate::modes::ViewMenuOption;

mod action_pane;
mod menus;

#[cfg(test)]
pub use action_pane::{render_abandon_confirm, render_action_pane};
#[cfg(test)]
pub use menus::{action_menu, help_overlay, help_overlay_text, role_prompt};

/// Borrowed overlay projection for the current interaction mode.
///
/// The enum carries references so drawing never takes ownership of prompt/menu/action state. Add
/// only shared modal presentation here; feature-specific availability and command policy belong in
/// the view, action menu, or action plan that produced the state.
pub enum Overlay<'a> {
    /// No modal overlay is active.
    None,
    Help {
        /// Help sections already projected from the command vocabulary.
        sections: Vec<HelpSection>,
    },
    CopyMenu {
        /// Copy options offered by the active view.
        options: &'a [CopyOption],
        /// Highlighted option index.
        selected: usize,
    },
    ViewMenu {
        /// Top-level view entries offered by the app.
        options: &'a [ViewMenuOption],
        /// Highlighted option index.
        selected: usize,
    },
    ActionMenu {
        /// Action rows offered for the current exact selection.
        menu: &'a ActionMenu,
        /// Highlighted action row index.
        selected: usize,
    },
    PushRemotePrompt {
        /// Remote names offered for push.
        remotes: &'a [String],
        /// Highlighted remote index.
        selected: usize,
    },
    FetchRemotePrompt {
        /// Remote names offered for fetch.
        remotes: &'a [String],
        /// Highlighted remote index.
        selected: usize,
    },
    ActionPane {
        /// Shared overlay title stem such as "Split" or "Fetch".
        title: &'static str,
        /// Scrollable preview/result output owned by action state.
        output: &'a ActionPane,
    },
    AbandonConfirm {
        /// User-typed exact revision confirmation text.
        input: &'a str,
        /// Existing preview output shown above the confirmation footer.
        output: &'a ActionPane,
    },
    RolePrompt {
        /// Immutable role prompt model for rewrite assignment.
        prompt: &'a RolePrompt,
        /// Highlighted role row index.
        selected: usize,
    },
}

/// Draw the active modal overlay over an already rendered frame.
///
/// Overlays are presentation-only. Selection indexes and output scroll offsets are owned by
/// `InteractionMode` or `ActionPane`; this function only sizes, clears, and renders the modal.
pub fn render_overlay(frame: &mut Frame<'_>, _status: &StatusLine, overlay: Overlay<'_>) {
    match overlay {
        Overlay::None => {}
        Overlay::Help { sections } => {
            let content = menus::help_overlay_text(&sections);
            let area = menus::centered_area(frame.area(), 84, content.lines.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::help_overlay(content), area);
        }
        Overlay::CopyMenu { options, selected } => {
            let area = menus::centered_area(frame.area(), 54, options.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::copy_menu(options, selected), area);
        }
        Overlay::ViewMenu { options, selected } => {
            let area = menus::centered_area(frame.area(), 54, options.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::view_menu(options, selected), area);
        }
        Overlay::ActionMenu { menu, selected } => {
            let area = menus::centered_area(frame.area(), 64, menu.items().len() as u16 + 3);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::action_menu(menu, selected), area);
        }
        Overlay::PushRemotePrompt { remotes, selected } => {
            let area = menus::centered_area(frame.area(), 46, remotes.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::remote_prompt("Push remote", remotes, selected), area);
        }
        Overlay::FetchRemotePrompt { remotes, selected } => {
            let area = menus::centered_area(frame.area(), 46, remotes.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(
                menus::remote_prompt("Fetch remote", remotes, selected),
                area,
            );
        }
        Overlay::ActionPane { title, output } => {
            let title = action_pane::action_pane_title(title, output);
            let area = action_pane::action_pane_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            action_pane::render_action_pane(frame, area, &title, output);
        }
        Overlay::AbandonConfirm { input, output } => {
            let title = "Abandon confirm";
            let area = action_pane::action_pane_area_with_footer(
                frame.area(),
                title,
                output,
                &action_pane::abandon_confirm_footer_text(input),
            );
            frame.render_widget(Clear, area);
            action_pane::render_abandon_confirm(frame, area, title, input, output);
        }
        Overlay::RolePrompt { prompt, selected } => {
            let area = menus::centered_area(frame.area(), 54, prompt.options().len() as u16 + 4);
            frame.render_widget(Clear, area);
            frame.render_widget(menus::role_prompt(prompt, selected), area);
        }
    }
}
