mod buffer;
pub mod tab;
pub mod view;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::terminal::Terminal;
use crate::{command::Command, editor::view::View};
use crate::{command::command_bar::CommandBar, editor::tab::Tab};

#[derive(PartialEq)]
pub enum Mode {
    Command,
    Edit,
}

/// Main editor data structure.
pub struct Editor {
    /// Vector of tabs - each one represents zero or more open views - think
    /// like a browser tab.
    pub tabs: Vec<Tab>,

    /// Index into `Self::tabs`. There is always at least one tab, even if that
    /// tab is empty.
    pub focused_tab_idx: usize,

    /// Represents whether the editor should quit on the next `mainloop`.
    pub quit: bool,

    /// Represents the command that the user is currently typing. Will be
    /// displayed in the command bar.
    pub command_bar: CommandBar,

    /// Represents which mode the editor is currently in.
    pub mode: Mode,
}

impl Editor {
    /// MOD_KEY_1 is used for movement and windowing commands.
    const MOD_KEY_1: KeyModifiers = KeyModifiers::CONTROL;

    /// MOD_KEY_2 is used for editor commands - saving, finding etc.
    const MOD_KEY_2: KeyModifiers = KeyModifiers::ALT;

    /// Creates a new editor from a Vector of relative file paths. If any of the
    /// files can't be opened, this is reported in the status bar and no view is
    /// created, unless no file can be opened, in which case one empty view is
    /// created.
    /// # Arguments
    /// * `paths: Vec<PathBuf>`: Paths to each file the user is attempting to
    ///   open. Each path is relative to the current working directory.
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

        let new_editor = Editor {
            tabs: tabs,
            focused_tab_idx: 0,
            quit: false,
            command_bar: CommandBar::new(),
            mode: Mode::Edit,
        };

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

    /// Runs the command currently stored in `self.command_input`. Clears the
    /// command input after running.
    fn attempt_to_run_command(&mut self) {
        // Attempt to create a command from the text that's currently in the
        // command bar. If this succeeds, we simply apply the command to the
        // editor. If we can't successfully parse a command from the command
        // bar, we warn the user why.
        match self.command_bar.parse_command() {
            Ok(mut command) => command.apply(self),
            Err(e) => self.show_warning(&e),
        };
    }

    // Changes the focused tab to the requested tab. If the requested tab does
    // not exist, pass silently.
    fn set_focused_tab(&mut self, requested_tab: usize) {
        if requested_tab < self.tabs.len() {
            self.focused_tab_idx = requested_tab;
        }
    }

    /// Set the contents of the command bar to the given warning text.
    pub fn show_warning(&mut self, warning_text: &str) {
        self.command_bar.set_contents(warning_text);
    }

    fn handle_keystroke(&mut self, key_event: KeyEvent, terminal: &Terminal) {
        if key_event.modifiers.contains(Self::MOD_KEY_1) {
            match key_event.code {
                KeyCode::Char('d') => Command::quit(self),

                KeyCode::Char('s') => {
                    // Typing Ctrl-S is equivalent to running the `save` command
                    // (with no arguments).
                    Command::save_no_args(self);
                }

                _ => {
                    // Pass MOD_KEY_1+other keys to the tab to handle.
                    self.get_current_tab_mut()
                        .handle_keystroke(key_event, terminal);
                }
            }
        } else if key_event.modifiers.contains(Self::MOD_KEY_2) {
            match key_event.code {
                KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                    let requested_tab = (c as u8 - b'1') as usize;
                    self.set_focused_tab(requested_tab);
                }

                _ => {
                    // Pass MOD_KEY_2+other keys to the tab to handle (e.g.,
                    // Alt+Arrow for focus movement)
                    self.get_current_tab_mut()
                        .handle_keystroke(key_event, terminal);
                }
            }
        } else {
            match key_event.code {
                KeyCode::F(10) => Command::quit(self),

                KeyCode::Enter => match self.mode {
                    Mode::Command => {
                        self.attempt_to_run_command();
                    }
                    Mode::Edit => {
                        self.get_current_tab_mut()
                            .handle_keystroke(key_event, terminal);
                    }
                },

                KeyCode::F(1) | KeyCode::Esc => match self.mode {
                    Mode::Command => {
                        self.mode = Mode::Edit;
                    }
                    Mode::Edit => {
                        self.mode = Mode::Command;
                        self.command_bar.clear();
                    }
                },

                KeyCode::Backspace => match self.mode {
                    Mode::Command => {
                        self.command_bar.view.buffer.backspace();
                    }
                    Mode::Edit => self
                        .get_current_tab_mut()
                        .handle_keystroke(key_event, terminal),
                },

                KeyCode::Char(c) => match self.mode {
                    Mode::Command => {
                        self.command_bar.view.buffer.insert(c);
                    }
                    Mode::Edit => {
                        self.get_current_tab_mut()
                            .handle_keystroke(key_event, terminal);
                    }
                },

                // If we're in Command mode and we haven't been able to match
                // the keystroke at this level, we need to let the command bar
                // handle this keystroke. Otherwise, we let the currently
                // focused tab handle it.
                _ => match self.mode {
                    Mode::Command => self.command_bar.handle_keystroke(key_event),
                    Mode::Edit => {
                        self.get_current_tab_mut()
                            .handle_keystroke(key_event, terminal);
                    }
                },
            }
        }
    }

    /// Get a mutable reference to the currently focused tab.
    pub fn get_current_tab_mut(&mut self) -> &mut Tab {
        return &mut self.tabs[self.focused_tab_idx];
    }

    /// Get a reference to the currently focused tab.
    pub fn get_current_tab(&self) -> &Tab {
        return &self.tabs[self.focused_tab_idx];
    }

    /// Scrolls the currently focused view to make sure that the cursor is
    /// currently being shown.
    pub fn ensure_cursor_shown(&mut self, width: usize, height: usize) {
        self.get_current_tab_mut()
            .ensure_cursor_shown(width, height);
    }
}
