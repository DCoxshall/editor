use crossterm::event::KeyEvent;

use crate::{
    command::{Command, CommandKeyword::Save},
    editor::view::View,
};

pub struct CommandBar {
    pub view: View,
}

impl CommandBar {
    pub fn new() -> CommandBar {
        CommandBar { view: View::new() }
    }

    pub fn clear(&mut self) {
        self.view.buffer.clear();
    }

    pub fn set_contents(&mut self, content: &str) {
        self.clear();
        for c in content.chars() {
            self.view.buffer.insert(c);
        }
    }

    /// Gets the raw textual contents of the command bar's internal view.
    pub fn get_contents(&self) -> String {
        return self.view.buffer.text.to_string();
    }

    fn parse_save_command(args: Vec<&str>) -> Result<Command, String> {
        if args.len() == 0 {
            return Ok(Command {
                keyword: Save,
                arguments: vec![],
            });
        } else if args.len() == 1 {
            return Ok(Command {
                keyword: Save,
                arguments: vec![args[0].to_owned()],
            });
        } else if args.len() == 2 {
            if args[1] != "!" {
                return Err("Usage: save [filename] [!]".to_string());
            }
            return Ok(Command {
                keyword: Save,
                arguments: vec![args[0].to_owned(), "!".to_string()],
            });
        }
        Err("Usage: save [filename] [!]".to_string())
    }

    /// Attempt to parse and return the command currently stored in the command
    /// bar's view.
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

            _ => {
                return Err("Command not recognised.".to_string());
            }
        }
    }

    pub fn handle_keystroke(&mut self, key_event: KeyEvent) {
        // Since the command bar structure itself doesn't handle anything, we
        // just pass every keystroke down to the underlying View.
        match key_event.code {
            // The command bar has nothing to pass a missed key event down to.
            _ => self.view.handle_keystroke(key_event),
        }
    }
}
