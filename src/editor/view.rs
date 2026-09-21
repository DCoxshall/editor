use std::{
    fs::{OpenOptions, read_to_string},
    io::{ErrorKind, Write},
    path::PathBuf,
};

use crossterm::event::{KeyCode, KeyEvent};

use crate::editor::{Editor, buffer::Buffer, selection::Selection};

/// Handles visualisation of the text in its buffer. Can be used wherever text needs to
/// be rendered to the screen, for example the text in a file, or the text in the
/// command bar.
#[derive(Clone, PartialEq)]
pub struct View {
    /// Text from the file we're currently editing.
    pub buffer: Buffer,

    // Path to said file. A view doesn't necessarily have a corresponding file - for
    // example, if the user has just opened a new view, or the view is being used as the
    // view for the command bar.
    pub file_path: Option<PathBuf>,

    /// Not every View will have a status bar. If this view is being used as the view
    /// for the command bar, there won't be a status bar, for instance.
    pub has_status_bar: bool,

    pub start_row: usize,
    pub start_col: usize,

    /// Represents whether this view has been edited since it was last saved to disk.
    pub dirty: bool,
}

impl View {
    pub const TAB_WIDTH: usize = 4;

    pub const EMPTY_LINE_NOTATION: &str = "~";

    /// Creates a new `View` from a `PathBuf`. If `path` does not exist, we return an empty `View`
    /// with `path` as its `file_path`. If `path` exists but cannot be read, we return an error.
    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let buffer = match read_to_string(&path) {
            Ok(text) => Buffer::new(text),
            Err(e) if e.kind() == ErrorKind::NotFound => Buffer::empty(),
            Err(e) => return Err(e),
        };

        Ok(View {
            buffer,
            file_path: Some(path),
            has_status_bar: true,
            start_row: 0,
            start_col: 0,
            dirty: false,
        })
    }

    /// Creates a new empty view.
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(String::from("")),
            file_path: None,
            start_row: 0,
            start_col: 0,
            has_status_bar: true,
            dirty: false,
        }
    }

    pub fn handle_keystroke(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                // We should only be inserting characters if no key modifiers are pressed.
                if !(key_event.modifiers.contains(Editor::MOD_KEY_1)
                    || key_event.modifiers.contains(Editor::MOD_KEY_2))
                {
                    self.buffer.insert(c);
                }
                self.dirty = true;
            }

            KeyCode::Tab => {
                self.buffer.insert('\t');
                self.dirty = true;
            }

            KeyCode::Enter => {
                self.buffer.insert('\n');
                self.dirty = true;
            }
            KeyCode::Backspace => {
                self.buffer.backspace();
                self.dirty = true;
            }
            KeyCode::Delete => {
                self.buffer.delete();
                self.dirty = true;
            }

            // Couldn't match at editor level, tab level, or here, so disregard.
            _ => self.buffer.handle_keystroke(key_event),
        }
    }

    /// Adjust `self.start_row` and `self.start_col` to ensure the cursor is within the
    /// visual bounds of the view.
    pub fn ensure_cursor_shown(&mut self, width: usize, height: usize) {
        // `view_height` is required because `height` represents the size of the entire focused
        // View, not the height of the actual viewport.
        let view_height = height - 1;

        let (cursor_x, cursor_y) = self.buffer.get_logical_cursor_pos();
        if self.start_col + width <= cursor_x {
            self.start_col = cursor_x - width + 1;
        }
        if self.start_row + view_height <= cursor_y {
            self.start_row = cursor_y + 1 - view_height;
        }
        if cursor_x < self.start_col {
            self.start_col = cursor_x;
        }
        if cursor_y < self.start_row {
            self.start_row = cursor_y;
        }
    }

    pub fn save(&self) -> Result<(), String> {
        match &self.file_path {
            Some(path) => {
                let mut file = match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                {
                    Ok(f) => f,
                    Err(_) => {
                        return Err(String::from("Error: File could not be opened for writing."));
                    }
                };
                match write!(file, "{}", self.buffer.get_text_as_string()) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(
                            "Error: File was opened but could not be written to.".to_string()
                        );
                    }
                }

                Ok(())
            }
            None => panic!("Attempted to save a View without a name."),
        }
    }
}
