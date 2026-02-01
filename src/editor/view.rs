use std::{
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::buffer::Buffer;

pub enum SaveResult {
    Success,
    NeedsNaming
}

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
}

impl View {
    pub const EMPTY_LINE_NOTATION: &str = "~";
    /// Creates a new view from `path`. If the file referred to by `path` can't be found or can't be
    /// read from, returns a `std::io::Error`.
    /// # Arguments
    /// * `path: PathBuf`: a relative file path. If the path cannot be read from, we return an error
    ///   - otherwise, we read from the file and create a new buffer.
    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let text = read_to_string(&path)?;
        Ok(View {
            buffer: Buffer::new(text),
            file_path: Some(path),
            start_row: 0,
            start_col: 0,
            has_status_bar: true,
        })
    }

    /// Creates a new empty view. The user can write in an empty view. Upon attempting to save, the
    /// editor should prompt them for a filename, which the view will be saved to.
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(String::from("")),
            file_path: None,
            start_row: 0,
            start_col: 0,
            has_status_bar: true,
        }
    }

    fn line_len(&self, line_idx: usize) -> usize {
        let line = self.buffer.text.line(line_idx);
        if line.to_string().ends_with('\n') {
            return line.len_chars() - 1;
        } else if line.to_string().ends_with("\r\n") {
            return line.len_chars() - 2;
        }
        return line.len_chars();
    }

    pub fn handle_keystroke(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Up => {
                self.move_cursor_up();
            }
            KeyCode::Down => {
                self.move_cursor_down();
            }

            KeyCode::Char(c) => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {

                } else {
                    self.buffer.insert(c);
                }
            }

            KeyCode::Enter => {
                self.buffer.insert('\n');
            }

            KeyCode::Backspace => {
                self.buffer.backspace();
            }

            KeyCode::Delete => {
                self.buffer.delete();
            }

            // Couldn't match at editor level, tab level, or here, so disregard.
            _ => {}
        }
    }

    fn move_cursor_right(&mut self) {
        self.buffer.cursor_idx += 1;
    }

    fn move_cursor_left(&mut self) {
        if self.buffer.cursor_idx > 0 {
            self.buffer.cursor_idx -= 1;
        }
    }

    fn move_cursor_up(&mut self) {
        let (col, line) = self.get_logical_cursor_pos();
        if line != 0 {
            let prev_line_len = self.line_len(line - 1);

            self.buffer.cursor_idx =
                self.buffer.text.line_to_char(line - 1) + prev_line_len.min(col);
        }
    }

    fn move_cursor_down(&mut self) {
        let (col, line) = self.get_logical_cursor_pos();
        if line != self.buffer.text.len_lines() - 1 {
            let next_line_len = self.line_len(line + 1);

            self.buffer.cursor_idx =
                self.buffer.text.line_to_char(line + 1) + next_line_len.min(col);
        }
    }

    fn get_logical_cursor_pos(&self) -> (usize, usize) {
        let line = self.buffer.text.char_to_line(self.buffer.cursor_idx);
        let col = self.buffer.cursor_idx - self.buffer.text.line_to_char(line);
        (col, line)
    }

    /// Returns the location of the cursor relative to the view's own visual origin.
    pub fn get_visual_cursor_pos(&self) -> (usize, usize) {
        let (logical_x, logical_y) = self.get_logical_cursor_pos();
        (logical_x - self.start_col, logical_y - self.start_row)
    }

    /// Adjust `self.start_row` and `self.start_col` to ensure the cursor is within the
    /// visual bounds of the view.
    pub fn ensure_cursor_shown(&mut self, width: usize, height: usize) {
        let (cursor_x, cursor_y) = self.get_logical_cursor_pos();
        if self.start_col + width <= cursor_x {
            self.start_col = cursor_x - width + 1;
        }
        if self.start_row + height <= cursor_y {
            self.start_row = cursor_y - height + 1;
        }
        if cursor_x < self.start_col {
            self.start_col = cursor_x;
        }
        if cursor_y < self.start_row {
            self.start_row = cursor_y;
        }
    }

    pub fn save(&self) -> SaveResult {
        match &self.file_path {
            Some(path) => {
                let mut file = File::create(path).unwrap();
                for chunk in self.buffer.text.chunks() {
                    file.write_all(chunk.as_bytes()).unwrap();
                }

                SaveResult::Success
            }
            None => {
                SaveResult::NeedsNaming
            }
        }
    }
}
