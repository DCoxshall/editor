use std::path::PathBuf;

use crate::editor::{
    Editor,
    tab::{
        Axis::{Horizontal, Vertical},
        Direction, Tab,
    },
    view::View,
};

pub mod command_bar;

/// Represents a command - an action that can be taken by the user at the command bar. Each variant
/// of `Command` represents one command that the user might execute, along with its potential
/// arguments.
pub enum Command {
    Quit {
        force: bool,
    },
    Save {
        file_path: Option<PathBuf>,
        force: bool,
    },
    Hsplit {
        file_path: PathBuf,
        is_up: bool,
    },
    Vsplit {
        file_path: PathBuf,
        is_left: bool,
    },
    MoveFocus {
        direction: Direction,
    },
    FocusTab {
        index: usize,
    },
    NextTab,
    PreviousTab,
    Open {
        file_path: PathBuf,
    },
}

impl Command {
    /// Applies the `Command` to the given `Editor`, setting `CommandBar::warning_set` to `false`.
    pub fn apply(&mut self, editor: &mut Editor) {
        editor.command_bar.warning_set = false;
        match self {
            Command::Quit { force } => Command::quit(editor, *force),
            Command::Save { file_path, force } => Command::save(editor, file_path.clone(), *force),
            Command::Hsplit { file_path, is_up } => {
                Command::hsplit(editor, file_path.clone(), *is_up)
            }
            Command::Vsplit { file_path, is_left } => {
                Command::vsplit(editor, file_path.clone(), *is_left)
            }
            Command::MoveFocus { direction } => Command::move_focus(editor, direction.clone()),
            Command::FocusTab { index } => Command::focus_tab(editor, *index),
            Command::NextTab => Command::next_tab(editor),
            Command::PreviousTab => Command::previous_tab(editor),
            Command::Open { file_path } => Command::open(editor, file_path.clone()),
        }
        if !editor.command_bar.warning_set {
            editor.command_bar.clear();
        }
        editor.toggle_command_bar_focus();
    }

    /// Apply the `open` command to `editor`. Functionally, this opens the given file, creating a new tab in the process, and swaps focus to this file in the new tab.
    pub fn open(editor: &mut Editor, file_path: PathBuf) {
        if !file_path.exists() {
            editor
                .command_bar
                .show_warning("Error: File could not be found.");
            return;
        }

        let new_view = match View::from_path(file_path) {
            Ok(v) => v,
            Err(_) => {
                editor
                    .command_bar
                    .show_warning("Error: File could not be opened.");
                return;
            }
        };

        let new_tab = Tab::from_view(new_view);
        editor.tabs.insert(editor.focused_tab_idx + 1, new_tab);
        editor.focused_tab_idx += 1;
    }

    /// Apply the `next_tab` command to `editor`. Swaps focus from the current tab to the tab immediately to its right, wrapping back to the first tab if necessary.
    pub fn next_tab(editor: &mut Editor) {
        editor.focused_tab_idx += 1;
        if editor.focused_tab_idx == editor.tabs.len() {
            editor.focused_tab_idx = 0;
        }
    }

    /// Apply the `prev_tab` command to `editor`. Swaps focus from the current tab to the tab immediately to its left, wrapping back to the last tab if necessary.
    pub fn previous_tab(editor: &mut Editor) {
        if editor.focused_tab_idx == 0 {
            editor.focused_tab_idx = editor.tabs.len() - 1;
        } else {
            editor.focused_tab_idx -= 1;
        }
    }

    /// Apply the `focus_tab` command to `editor`. Swaps focus from the current tab to the tab at the provided index.
    pub fn focus_tab(editor: &mut Editor, index: usize) {
        editor.set_focused_tab(index);
    }

    /// Apply the `move_focus` command to `editor`. Swaps focus from the current view to the view immediately in the cardinal `direction` from the current view.
    fn move_focus(editor: &mut Editor, direction: Direction) {
        editor.get_current_tab_mut().move_focus_direction(direction);
    }

    /// Sub-branch of the `save` command, where the user exeuctes `save` with no arguments, or taps the key combination for `save`. Saves the currently-focused view's contents to the given file.
    fn save_no_args(editor: &mut Editor) {
        let current_view = editor.get_current_tab().get_current_focused_view();

        if current_view.file_path.is_none() {
            editor
                .command_bar
                .show_warning("Error: A name is required to save.");
            return;
        }

        match current_view.save() {
            Ok(_) => editor.command_bar.clear(),
            Err(e) => editor.command_bar.show_warning(&e),
        }
    }

    /// Sub-branch of the `save` command, where the user executes `save` with a file path as its only argument. Saves the currently-focused view's contents to its given file.
    fn save_as(editor: &mut Editor, file_path: PathBuf, force: bool) {
        let current_view = editor.get_current_tab_mut().get_current_focused_view_mut();

        if current_view.file_path.is_some() && file_path == current_view.file_path.clone().unwrap()
        {
            match current_view.save() {
                Ok(_) => editor.command_bar.clear(),
                Err(e) => editor.command_bar.show_warning(&e),
            }
        } else {
            if file_path.exists() {
                if force {
                    current_view.file_path = Some(file_path);
                    match current_view.save() {
                        Ok(_) => editor.command_bar.clear(),
                        Err(e) => editor.command_bar.show_warning(&e),
                    }
                } else {
                    editor
                        .command_bar
                        .show_warning("Warn: File already exists. Append '!' to overwrite.");
                }
            } else {
                current_view.file_path = Some(file_path);
                match current_view.save() {
                    Ok(_) => editor.command_bar.clear(),
                    Err(e) => editor.command_bar.show_warning(&e),
                }
            }
        }
    }

    /// Apply the `save` command to `editor`. Chooses between saving with or without a file name.
    /// Upon a successful save, this function sets `View::dirty` to `false`.
    pub fn save(editor: &mut Editor, file_path: Option<PathBuf>, force: bool) {
        match file_path {
            Some(path) => Command::save_as(editor, path, force),
            None => Command::save_no_args(editor),
        }
        if !editor.command_bar.warning_set {
            editor
                .get_current_tab_mut()
                .get_current_focused_view_mut()
                .dirty = false;
        }
    }

    /// Apply the `quit` command to `editor`. Checks if any view is dirty - if yes, then we show a
    /// warning, if no, then we immediately quit.
    pub fn quit(editor: &mut Editor, force: bool) {
        match force {
            true => editor.quit = true,
            false => {
                let mut any_dirty_views = false;
                'outer: for tab in &editor.tabs {
                    for view in tab.views() {
                        if view.dirty {
                            any_dirty_views = true;
                            break 'outer;
                        }
                    }
                }
                if any_dirty_views {
                    editor.command_bar.show_warning("Warn: Not all views have been saved. Append a '!' to override this warning.");
                } else {
                    editor.quit = true;
                }
            }
        }
    }

    /// Apply the `vsplit` command to `editor`. Attempts to open the given file to either the left or the right of the currently focused view.
    fn vsplit(editor: &mut Editor, file_path: PathBuf, is_left: bool) {
        let new_view = View::from_path(file_path).unwrap();
        editor
            .get_current_tab_mut()
            .insert_new_view(new_view, Vertical, is_left);
    }

    /// Apply the `hsplit` command to `editor`. Attempts to open the given file either above or below the currently focused view.
    fn hsplit(editor: &mut Editor, file_path: PathBuf, is_up: bool) {
        let new_view = View::from_path(file_path).unwrap();
        editor
            .get_current_tab_mut()
            .insert_new_view(new_view, Horizontal, is_up);
    }
}
