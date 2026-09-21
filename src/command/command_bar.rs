use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::{
    command::Command,
    editor::{
        tab::Direction::{Down, Left, Right, Up},
        view::View,
    },
};

/// Stores a `View` to input text into, and handles parsing of commands.
pub struct CommandBar {
    /// The command bar's textual `View`. This `View` stores the text that the user inputs, with the
    /// intention of executing.
    pub view: View,

    /// Represents whether the last command that the user executed failed. `warning_set` is designed
    /// to control the presence of a warning in the terminal - individual commands can choose to
    /// call `CommandBar::show_warning`, which sets the `warning_set` to true.
    pub warning_set: bool,
}

impl CommandBar {
    /// Creates a new empty command bar.
    pub fn new() -> CommandBar {
        CommandBar {
            view: View::new(),
            warning_set: false,
        }
    }

    /// Sets the contents of the command bar to the given warning text, and set `warning_set` to
    /// `true`.
    pub fn show_warning(&mut self, warning_text: &str) {
        self.set_contents(warning_text);
        self.warning_set = true;
    }

    /// Attempts to parse the user's command currently stored in the command bar's view.
    pub fn parse_command(&mut self) -> Result<Command, String> {
        let contents = self.get_contents();
        let tokens: Vec<&str> = contents.split_whitespace().collect();

        if tokens.len() == 0 {
            return Err("No command given.".to_string());
        }

        match tokens[0] {
            "save" => {
                return CommandBar::parse_save_command(tokens.into_iter().skip(1).collect());
            }

            "vsplit" => {
                return CommandBar::parse_vsplit_command(tokens.into_iter().skip(1).collect());
            }

            "hsplit" => {
                return CommandBar::parse_hsplit_command(tokens.into_iter().skip(1).collect());
            }

            "quit" => {
                return CommandBar::parse_quit_command(tokens.into_iter().skip(1).collect());
            }

            "move_focus" | "m" => {
                return CommandBar::parse_move_focus_command(tokens.into_iter().skip(1).collect());
            }

            "focus_tab" => {
                return CommandBar::parse_focus_tab_command(tokens.into_iter().skip(1).collect());
            }

            "next_tab" => {
                return CommandBar::parse_next_tab_command(tokens.into_iter().skip(1).collect());
            }

            "prev_tab" => {
                return CommandBar::parse_prev_tab_command(tokens.into_iter().skip(1).collect());
            }

            "open" => {
                return CommandBar::parse_open_command(tokens.into_iter().skip(1).collect());
            }

            _ => {
                return Err("Command not recognised.".to_string());
            }
        }
    }

    /// Parses an `open` command from given arguments, or return an error to display to the user.
    fn parse_open_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() != 1 {
            return Err("Usage: open filename".to_string());
        }

        Ok(Command::Open {
            file_path: PathBuf::from(args[0]),
        })
    }

    /// Parses a `next_tab` command from given arguments, or return an error to display to the user.
    fn parse_next_tab_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() != 0 {
            return Err("Usage: next_tab".to_string());
        }

        Ok(Command::NextTab)
    }

    /// Parses a `prev_tab` command from given arguments, or return an error to display to the user.
    fn parse_prev_tab_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() != 0 {
            return Err("Usage: prev_tab".to_string());
        }

        Ok(Command::PreviousTab)
    }

    /// Parses a `focus_tab` command from given arguments, or return an error to display to the
    /// user.
    fn parse_focus_tab_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() != 1 {
            return Err("Usage: focus_tab index".to_string());
        }

        let index = match args[0].parse::<usize>() {
            Ok(idx) => idx - 1, // Remember that tabs are indexed from 1 in the GUI.
            Err(_) => return Err("Usage: focus_tab index".to_string()),
        };

        Ok(Command::FocusTab { index })
    }

    /// Parses a `save` command from given arguments, or return an error to display to the user.
    fn parse_save_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() == 0 {
            return Ok(Command::Save {
                file_path: None,
                force: false,
            });
        } else if args.len() == 1 {
            return Ok(Command::Save {
                file_path: Some(PathBuf::from(args[0])),
                force: false,
            });
        } else if args.len() == 2 {
            if args[1] != "!" {
                return Err("Usage: save [filename] [!]".to_string());
            }
            return Ok(Command::Save {
                file_path: Some(PathBuf::from(args[0])),
                force: true,
            });
        }
        Err("Usage: save [filename] [!]".to_string())
    }

    /// Parses a `vsplit_tab` command from given arguments, or return an error to display to the
    /// user.
    fn parse_vsplit_command(args: Vec<&str>) -> Result<Command, String> {
        let is_left = match args.len() {
            1 => false,
            2 => match args[1] {
                "l" => true,
                "r" => false,
                _ => {
                    return Err("Usage: vsplit filename [l|r]".to_string());
                }
            },
            _ => {
                return Err("Usage: vsplit filename [l|r]".to_string());
            }
        };

        Ok(Command::Vsplit {
            file_path: PathBuf::from(args[0]),
            is_left,
        })
    }

    /// Parses a `hsplit_tab` command from given arguments, or return an error to display to the
    /// user.
    fn parse_hsplit_command(args: Vec<&str>) -> Result<Command, String> {
        let is_up = match args.len() {
            1 => false,
            2 => match args[1] {
                "u" => true,
                "d" => false,
                _ => {
                    return Err("Usage: hsplit filename [u|d]".to_string());
                }
            },
            _ => {
                return Err("Usage: hsplit filename [u|d]".to_string());
            }
        };

        Ok(Command::Hsplit {
            file_path: PathBuf::from(args[0]),
            is_up,
        })
    }

    /// Parses a `quit` command from given arguments, or return an error to display to the user.
    fn parse_quit_command(args: Vec<&str>) -> Result<Command, String> {
        match args.len() {
            0 => Ok(Command::Quit { force: false }),
            1 => match args[0] {
                "!" => Ok(Command::Quit { force: true }),
                _ => Err("Usage: quit [!]".to_string()),
            },
            _ => Err("Usage: quit [!]".to_string()),
        }
    }

    /// Parses a `move_focus` command from given arguments, or return an error to display to the
    /// user.
    fn parse_move_focus_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() != 1 {
            return Err("Usage: move_focus [left|right|up|down]".to_string());
        }

        let direction = match args[0] {
            "left" | "l" => Left,
            "right" | "r" => Right,
            "up" | "u" => Up,
            "down" | "d" => Down,
            _ => return Err("Usage: move_focus left|right|up|down OR <m l|r|u|d>".to_string()),
        };

        Ok(Command::MoveFocus { direction })
    }

    /// Clears the command line.
    pub fn clear(&mut self) {
        self.view.buffer.clear();
    }

    /// Sets the contents of the command line to `content`. "Contents of the command line" here
    /// means everything succeeding the prompt symbols.
    pub fn set_contents(&mut self, content: &str) {
        self.clear();
        for c in content.chars() {
            self.view.buffer.insert(c);
        }
    }

    /// Gets the raw textual contents of the command bar's internal view.
    pub fn get_contents(&self) -> String {
        return self.view.buffer.get_text_as_string();
    }

    /// Handles a keystroke. If the command bar struct itself cannot handle the keystroke, pass the
    /// `KeyEvent` down to the owned `View`.
    pub fn handle_keystroke(&mut self, key_event: KeyEvent) {
        // Since the command bar structure itself doesn't handle anything, we just pass every
        // keystroke down to the underlying View.
        match key_event.code {
            // The command bar has nothing to pass a missed key event down to.
            _ => self.view.handle_keystroke(key_event),
        }
    }
}
