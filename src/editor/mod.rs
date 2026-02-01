mod buffer;
pub mod tab;
pub mod view;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::editor::tab::Tab;
use crate::editor::view::View;
use crate::terminal::Terminal;

#[derive(PartialEq)]
pub enum Mode {
    Command,
    Edit,
}

/// Main editor data structure.
pub struct Editor {
    /// Vector of tabs - each one represents zero or more open views - think like a browser tab.
    pub tabs: Vec<Tab>,

    /// Index into `Self::tabs`. There is always at least one tab, even if that tab is empty.
    pub focused_tab_idx: usize,

    /// Represents whether the editor should quit on the next `mainloop`.
    pub quit: bool,

    /// Represents the command that the user is currently typing. Will be displayed in the command
    /// bar.
    pub command_bar_content: View,

    /// Represents which mode the editor is currently in.
    pub mode: Mode,
}

impl Editor {
    /// Creates a new editor from a Vector of relative file paths. If any of the files can't be
    /// opened, this is reported in the status bar and no view is created, unless no file can be
    /// opened, in which case one empty view is created.
    /// # Arguments
    /// * `paths: Vec<PathBuf>`: Paths to each file the user is attempting to open. Each path is
    ///   relative to the current working directory.
    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut views: Vec<View> = vec![];

        for path in paths {
            match View::from_path(path) {
                Ok(view) => views.push(view),
                Err(_) => {}
            }
        }

        let mut tabs = vec![];

        for view in views {
            tabs.push(Tab::from_view(view));
        }

        if tabs.len() == 0 {
            tabs.push(Tab::new());
        }

        let mut new_editor = Editor {
            tabs: tabs,
            focused_tab_idx: 0,
            quit: false,
            command_bar_content: View::new(),
            mode: Mode::Command,
        };

        new_editor.command_bar_content.has_status_bar = false;
        return new_editor;
    }

    pub fn handle_input(&mut self, event: Event, terminal: &Terminal) {
        match event {
            Event::Key(key_event) => {
                if key_event.is_press() {
                    self.handle_keystroke(key_event, terminal);
                }
            }

            _ => {}
        }
    }

    /// Runs the command currently stored in `self.command_input`. Clears the command input after
    /// running.
    fn run_user_command(&mut self) {
        let command = self.command_bar_content.buffer.text.to_string();
        if command == String::from("quit") {
            self.quit = true;
        } else if let Some(filename) = command.strip_prefix("hsplit") {
            let filename = filename.trim_start();
            let pathbuf = PathBuf::from(filename);
            let view = View::from_path(pathbuf).unwrap();
            self.get_current_tab_mut()
                .insert_new_view(view, tab::Axis::Horizontal);
        } else if let Some(filename) = command.strip_prefix("vsplit") {
            let filename = filename.trim_start();
            let pathbuf = PathBuf::from(filename);
            let view = View::from_path(pathbuf).unwrap();
            self.get_current_tab_mut()
                .insert_new_view(view, tab::Axis::Vertical);
        }

        self.command_bar_content.buffer.clear();
    }

    // Changes the focused tab to the requested tab. If the requested tab does
    // not exist, pass silently.
    fn set_focused_tab(&mut self, requested_tab: usize) {
        if requested_tab < self.tabs.len() {
            self.focused_tab_idx = requested_tab;
        }
    }

    fn handle_keystroke(&mut self, key_event: KeyEvent, terminal: &Terminal) {
        if key_event.modifiers.contains(KeyModifiers::ALT) {
            match key_event.code {
                KeyCode::Char('d') => {
                    self.quit = true;
                }

                KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                    let requested_tab = (c as u8 - b'1') as usize;
                    self.set_focused_tab(requested_tab);
                }

                _ => {
                    // Pass Alt+other keys to the tab to handle (e.g., Alt+Arrow for focus movement)
                    self.get_current_tab_mut()
                        .handle_keystroke(key_event, terminal);
                }
            }
        } else {
            match key_event.code {
                KeyCode::F(10) => {
                    self.quit = true;
                }

                KeyCode::Enter => {
                    if self.mode == Mode::Command {
                        self.run_user_command();
                    }
                }

                KeyCode::F(1) => {
                    if self.mode == Mode::Command {
                        self.mode = Mode::Edit;
                    } else {
                        self.mode = Mode::Command;
                    }
                }

                KeyCode::Backspace => {
                    if self.mode == Mode::Command {
                        self.command_bar_content.buffer.backspace();
                    } else {
                        self.get_current_tab_mut()
                            .handle_keystroke(key_event, terminal);
                    }
                }

                KeyCode::Char(c) => {
                    if self.mode == Mode::Command {
                        self.command_bar_content.buffer.insert(c);
                    } else {
                        self.get_current_tab_mut()
                            .handle_keystroke(key_event, terminal);
                    }
                }

                // If we can't match here, pass down to the focused tab to deal with.
                _ => self
                    .get_current_tab_mut()
                    .handle_keystroke(key_event, terminal),
            }
        }
    }

    pub fn get_current_tab_mut(&mut self) -> &mut Tab {
        return &mut self.tabs[self.focused_tab_idx];
    }

    pub fn get_current_tab(&self) -> &Tab {
        return &self.tabs[self.focused_tab_idx];
    }

    /// Scrolls the currently focused view to make sure that the cursor is currently
    /// being shown.
    pub fn ensure_cursor_shown(&mut self, width: usize, height: usize) {
        self.get_current_tab_mut()
            .ensure_cursor_shown(width, height);
    }
}
