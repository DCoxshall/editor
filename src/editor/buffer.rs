use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::size,
};
use ropey::Rope;
use std::{cmp::min, fs, io::Write, path::PathBuf};
use encoding_rs::UTF_16LE;

use crate::editor::Editor;

/// One buffer represents one open file.
pub struct Buffer {
    // Contains the relative path of the file being displayed in this buffer.
    pub file_path: PathBuf,

    // Contains the actual data in the buffer.
    text: Rope,

    // Represents the height and width in columns and rows of the area of the screen that
    // we're drawing `buffer` to.
    pub visual_width: usize,
    pub visual_height: usize,

    /// Represent the line index in `text` that should be shown at the buffer's (0, 0).
    pub visual_origin_row: usize,

    /// Represents the column index in `text` that should be shown at the buffer's (0, 0).
    pub visual_origin_col: usize,

    /// Represents where in `text` the cursor is. Cursor location is a property of the buffer and
    /// not the editor. Measured in chars, not bytes.
    /// Due to how we handle resize events and cursor movement, the cursor is guranteed to always be
    /// inside the viewport.
    pub cursor_idx: usize,
}

impl Buffer {
    /// Creates a buffer from a given file path.
    /// Loads contents if the file exists and is readable.
    /// Creates an empty buffer if the file does not exist.
    /// Returns Err if the file exists but it can't be read.
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        // First, we read the text from the file. If the file can't be read, we simply return an
        // error.
        // Next, we iterate through the text and replace CRLF with just LF.

        let (cols, rows) = size().unwrap();

        // Read raw bytes from the file
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(err),
        };

        // Attempt UTF-8 first, then UTF-16 LE, then fallback lossily
        let contents = if let Ok(s) = String::from_utf8(bytes.clone()) {
            s
        } else if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            // UTF-16 LE BOM detected
            let (cow, _, _) = UTF_16LE.decode(&bytes[2..]); // skip BOM
            cow.into_owned()
        } else {
            // Fallback: replace invalid sequences
            String::from_utf8_lossy(&bytes).into_owned()
        };

        let mut rope: Rope = Rope::from_str(&contents);

        let mut line_idx = 0;
        while line_idx < rope.len_lines() {
            let line = rope.line(line_idx);
            let len = line.len_chars();
            if len >= 2 {
                let last_char = line.char(len - 2);
                let newline_char = line.char(len - 1);
                if last_char == '\r' && newline_char == '\n' {
                    rope.remove(
                        rope.line_to_char(line_idx) + len - 2
                            ..rope.line_to_char(line_idx) + len - 1,
                    );
                }
            }
            line_idx += 1;
        }

        Ok(Buffer {
            file_path: path,
            text: rope,
            visual_width: cols as usize,
            visual_height: rows as usize,
            visual_origin_row: 0,
            visual_origin_col: 0,
            cursor_idx: 0,
        })
    }

    /// Save the current contents of the file.
    pub fn save_file(&self) {
        let mut output_file = fs::File::create(&self.file_path).unwrap();
        output_file
            .write_all(self.text.to_string().as_bytes())
            .unwrap();
    }

    /// Return a string for the editor to use as a status bar for this buffer.
    pub fn get_status_bar_text(&self) -> String {
        let mut text = String::from("Viewing file ");
        let filename = self
            .file_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unnamed>".to_string());
        text.push_str(&filename);
        return text;
    }

    /// Moves the cursor right by one character.
    pub fn move_right(&mut self) {
        if self.cursor_idx < self.len_chars() {
            self.cursor_idx += 1;
        }
    }

    /// Moves the cursor left by one character.
    pub fn move_left(&mut self) {
        if self.cursor_idx > 0 {
            self.cursor_idx -= 1;
        }
    }

    /// Moves the cursor up a line.
    pub fn move_up(&mut self) {
        let cursor_line = self.get_logical_cursor_line();
        // If we're on the first line, go to the beginning of the line.
        if cursor_line == 0 {
            self.cursor_idx = 0;
        } else {
            let next_line_idx = cursor_line - 1;
            let next_line_char_idx = self.text.line_to_char(next_line_idx);
            let next_line_len = self.text.line(next_line_idx).len_chars();
            if next_line_len <= 1 || self.get_logical_cursor_col() == 0 {
                self.cursor_idx = next_line_char_idx;
            } else {
                self.cursor_idx =
                    next_line_char_idx + min(next_line_len, self.get_logical_cursor_col());
            }
        }
    }

    /// Moves the cursor down a line.
    pub fn move_down(&mut self) {
        let cursor_line = self.get_logical_cursor_line();
        // If we're on the last line, go the end of the line.
        if cursor_line == self.text.len_lines() - 1 {
            self.cursor_idx = self.text.len_chars();
        } else {
            let next_line_idx = cursor_line + 1;
            let next_line_char_idx = self.text.line_to_char(next_line_idx);
            let next_line_len = self.text.line(next_line_idx).len_chars();
            if next_line_len <= 1 || self.get_logical_cursor_col() == 0 {
                self.cursor_idx = next_line_char_idx;
            } else {
                self.cursor_idx =
                    next_line_char_idx + min(next_line_len, self.get_logical_cursor_col());
            }
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        let (current_line_idx, _) = self.get_logical_cursor_pos();
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Right => self.move_right(),
                KeyCode::Left => {
                    self.move_left();
                }
                KeyCode::Up => {
                    self.move_up();
                }
                KeyCode::Down => {
                    self.move_down();
                }
                KeyCode::Home => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.cursor_idx = 0;
                    } else {
                        self.cursor_idx = self.line_to_char(current_line_idx);
                    }
                }
                KeyCode::End => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.cursor_idx = self.len_chars();
                    } else {
                        let current_line_len = self.get_line(current_line_idx).len();
                        let current_line_char_idx = self.line_to_char(current_line_idx);
                        self.cursor_idx =
                            current_line_char_idx + current_line_len - min(current_line_len, 1);
                    }
                }
                KeyCode::Char(x) => {
                    let mut buf = [0u8; 4];
                    self.text.insert(self.cursor_idx, x.encode_utf8(&mut buf));
                    self.cursor_idx += 1;
                }
                KeyCode::Enter => {
                    self.text.insert(self.cursor_idx, "\n");
                    self.cursor_idx += 1;
                }
                KeyCode::Backspace => {
                    if self.cursor_idx != 0 {
                        self.text.remove(self.cursor_idx - 1..self.cursor_idx);
                        self.cursor_idx -= 1;
                    }
                }
                KeyCode::Tab => {
                    self.text.insert(self.cursor_idx, "\t");
                    self.cursor_idx += 1;
                }
                KeyCode::Delete => {
                    if self.cursor_idx != self.text.len_chars() {
                        self.text.remove(self.cursor_idx..self.cursor_idx + 1);
                    }
                }
                _ => {}
            }
        }
    }

    /// Returns the logical line and column that the cursor is on. (line, column).
    pub fn get_logical_cursor_pos(&self) -> (usize, usize) {
        (
            self.get_logical_cursor_line(),
            self.get_logical_cursor_col(),
        )
    }

    /// Gets the logical line that the cursor is on.
    pub fn get_logical_cursor_line(&self) -> usize {
        self.text.char_to_line(self.cursor_idx)
    }

    /// Gets the logical column that the cursor is on.
    pub fn get_logical_cursor_col(&self) -> usize {
        self.cursor_idx - self.text.line_to_char(self.get_logical_cursor_line())
    }

    /// Gets the column that the cursor should be shown at visually.
    pub fn get_visual_cursor_col(&self) -> usize {
        // Remember - tabs count as one logical character but TAB_WIDTH visual characters.
        let cursor_line = self.get_line(self.get_logical_cursor_line());
        let up_to_cursor: String = cursor_line.chars().take(self.get_logical_cursor_col()).collect();
        let tab_count = up_to_cursor.chars().filter(|&c| c == '\t').count();
        self.get_logical_cursor_col() + (Editor::TAB_WIDTH * tab_count)
            - self.visual_origin_col
            - tab_count
    }

    /// Gets the row that the cursor should be shown at visually.
    pub fn get_visual_cursor_line(&self) -> usize {
        self.get_logical_cursor_line() - self.visual_origin_row
    }

    /// Get the number of lines in the buffer.
    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    /// Get the character index of a given line.
    pub fn line_to_char(&self, idx: usize) -> usize {
        self.text.line_to_char(idx)
    }

    pub fn char_to_line(&self, idx: usize) -> usize {
        self.text.char_to_line(idx)
    }

    /// Get the length of the buffer in chars.
    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    /// Get the text of a line from the buffer as a string.
    pub fn get_line(&self, idx: usize) -> String {
        self.text.line(idx).to_string()
    }
}
