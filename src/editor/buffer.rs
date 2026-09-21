use crossterm::event::{KeyCode, KeyEvent};
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use crate::editor::{Editor, selection::Selection};

/// Acts as a wrapper around Ropey::rope. It's only job is to hold and manipulate text.
#[derive(Clone, PartialEq)]
pub struct Buffer {
    /// Holds the buffer's text.
    text: Rope,

    /// Index into the Rope characters of self.text.
    cursor_idx: usize,

    /// Indicates the section of text that is being selected (highlighted).
    selection: Selection,
}

impl Buffer {
    /// Creates a new `Buffer` as a wrapper around the given String.
    pub fn new(text: String) -> Self {
        // Replace CRLF with LF.
        let replaced = text.replace("\r\n", "\n");
        Buffer {
            text: Rope::from_str(&replaced),
            cursor_idx: 0,
            selection: Selection::new(),
        }
    }

    /// Gets the logical position of the grapheme under the cursor - that is, the line it's on and
    /// its grapheme position within the line.
    pub fn get_logical_cursor_pos(&self) -> (usize, usize) {
        let line_idx = self.text.char_to_line(self.cursor_idx);
        let line_char_idx = self.text.line_to_char(line_idx);
        let relevant_line_str: String = self
            .text
            .line(line_idx)
            .chars()
            .take(self.cursor_idx - line_char_idx)
            .collect();
        (self.visual_str_len(&relevant_line_str), line_idx)
    }

    /// Gets this `Buffer`'s text as a `String`.
    pub fn get_text_as_string(&self) -> String {
        self.text.to_string()
    }

    /// Gets this `Buffer`'s text as a `Ropey::Rope`.
    pub fn get_text_as_rope(&self) -> &Rope {
        &self.text
    }

    pub fn handle_keystroke(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => {
                self.move_cursor_backwards();
            }
            KeyCode::Right => {
                self.move_cursor_forward();
            }
            KeyCode::Up => {
                self.move_cursor_up();
            }
            KeyCode::Down => {
                self.move_cursor_down();
            }
            KeyCode::End => {
                self.move_to_end(key_event);
            }
            KeyCode::Home => {
                self.move_to_home(key_event);
            }

            _ => {}
        }
    }

    fn grapheme_visual_width(grapheme: &str) -> usize {
        // ZWJ emoji sequences are rendered as a single emoji, normally width 2.
        if grapheme.contains('\u{200D}') {
            return 2;
        }

        grapheme.chars().map(|c| c.width_cjk().unwrap_or(0)).sum()
    }

    /// Returns the visual length of the first line of a string.
    fn visual_str_len(&self, string: &str) -> usize {
        let mut len = 0;

        for grapheme in string.graphemes(true) {
            if grapheme == "\t" {
                let diff = len % 4;
                len += if diff == 0 { 4 } else { 4 - diff };
            } else if grapheme == "\n" || grapheme == "\r" {
                break;
            } else {
                len += grapheme_visual_width(grapheme);
            }
        }

        len
    }

    /// Moves cursor_idx one grapheme forwards.
    pub fn move_cursor_forward(&mut self) {
        self.cursor_idx = self.next_grapheme_boundary(self.cursor_idx);
    }

    /// Moves cursor_idx one grapheme backwards.
    pub fn move_cursor_backwards(&mut self) {
        self.cursor_idx = self.previous_grapheme_boundary(self.cursor_idx);
    }

    fn move_cursor_up(&mut self) {
        // Determine the visual length of the current line up to the grapheme under the cursor.
        let current_line = self.text.line(self.text.char_to_line(self.cursor_idx));
        let cursor_idx_within_line = self.cursor_idx
            - self
                .text
                .line_to_char(self.text.char_to_line(self.cursor_idx));
        let current_line_string: String =
            current_line.chars().take(cursor_idx_within_line).collect();
        let line_visual_length = self.visual_str_len(&current_line_string);

        // Move the cursor one line up, either to the grapheme at `line_visual_length`, or to the
        // end of the line, whichever is closer.
        let cur_line_idx = self.text.char_to_line(self.cursor_idx);
        if cur_line_idx == 0 {
            self.cursor_idx = 0;
        } else {
            let length_of_line_above = self.text.line(cur_line_idx - 1).len_chars() - 1;
            let target = length_of_line_above.min(line_visual_length);
            self.cursor_idx = self.text.line_to_char(cur_line_idx - 1) + target;
        }
    }

    fn move_cursor_down(&mut self) {
        // Determine the visual length of the current line up to the grapheme under the cursor.
        let current_line = self.text.line(self.text.char_to_line(self.cursor_idx));
        let cursor_idx_within_line = self.cursor_idx
            - self
                .text
                .line_to_char(self.text.char_to_line(self.cursor_idx));
        let current_line_string: String =
            current_line.chars().take(cursor_idx_within_line).collect();
        let line_visual_length = self.visual_str_len(&current_line_string);

        // Move the cursor one line down, either to the grapheme at `line_visual_length`, or to the
        // end of the line, whichever is closer.
        let cur_line_idx = self.text.char_to_line(self.cursor_idx);
        if cur_line_idx == self.text.len_lines() - 1 {
            self.cursor_idx = self.text.len_chars();
        } else {
            let line_below = self.text.line(cur_line_idx + 1);
            let mut length_of_line_below = line_below.len_chars();
            if line_below.to_string().ends_with("\n") || line_below.to_string().ends_with("\r") {
                length_of_line_below -= 1;
            }
            let target = length_of_line_below.min(line_visual_length);
            self.cursor_idx = self.text.line_to_char(cur_line_idx + 1) + target;
        }
    }

    fn move_to_end(&mut self, key_event: KeyEvent) {
        // If MOD_1 is held, jump to the very end of the file.
        if key_event.modifiers.contains(Editor::MOD_KEY_1) {
            self.cursor_idx = self.text.len_chars();
        }
        // Otherwise, just jump to the end of current the line.
        else {
            // If we're on the last line, move to the end of the file. Otherwise, move to the end of
            // the current line.
            if self.text.char_to_line(self.cursor_idx) == self.text.len_lines() {
                self.cursor_idx = self.text.len_chars();
            } else {
                self.cursor_idx = self
                    .text
                    .line_to_char(self.text.char_to_line(self.cursor_idx) + 1)
                    - 1;
            }
        }
    }

    fn move_to_home(&mut self, key_event: KeyEvent) {
        // If MOD_1 is held, jump to the start of the file.
        if key_event.modifiers.contains(Editor::MOD_KEY_1) {
            self.cursor_idx = 0;
        }
        // Otherwise, just jump to the start of the current line.
        else {
            self.cursor_idx = self
                .text
                .line_to_char(self.text.char_to_line(self.cursor_idx));
        }
    }

    /// Returns the grapheme that is sat beneath the cursor - that is, the grapheme at
    /// `self.cursor_idx`.
    pub fn get_grapheme_under_cursor(&self) -> String {
        let end = self.next_grapheme_boundary(self.cursor_idx);
        self.text.slice(self.cursor_idx..end).to_string()
    }

    pub fn get_cursor_idx(&self) -> usize {
        self.cursor_idx
    }

    /// Gets the char index in self.text of the next grapheme boundary.
    fn next_grapheme_boundary(&self, char_idx: usize) -> usize {
        if char_idx >= self.text.len_chars() {
            return self.text.len_chars();
        }

        let text = self.text.slice(char_idx..).to_string();

        match text.grapheme_indices(true).next() {
            Some((_, grapheme)) => char_idx + grapheme.chars().count(),
            None => self.text.len_chars(),
        }
    }

    /// Gets the char index in self.text of the previous grapheme boundary.
    fn previous_grapheme_boundary(&self, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }

        let text = self.text.slice(..char_idx).to_string();

        match text.grapheme_indices(true).next_back() {
            Some((_, grapheme)) => char_idx - grapheme.chars().count(),
            None => 0,
        }
    }

    /// Creates a new empty `Buffer`.
    pub fn empty() -> Self {
        Buffer {
            text: Rope::from_str(""),
            cursor_idx: 0,
            selection: Selection::new(),
        }
    }

    /// Inserts a string at the current cursor location.
    pub fn insert_string(&mut self, s: &str) {
        for c in s.chars() {
            self.insert(c);
        }
    }

    /// Inserts a char at the current cursor location.
    pub fn insert(&mut self, c: char) {
        self.text.insert_char(self.cursor_idx, c);
        self.cursor_idx += 1;
    }

    /// Delete the character immediately before the cursor.
    pub fn backspace(&mut self) {
        let start = self.previous_grapheme_boundary(self.cursor_idx);
        self.text.remove(start..self.cursor_idx);
        self.cursor_idx = start;
    }

    /// Delete the character immediately after the cursor.
    pub fn delete(&mut self) {
        let next = self.next_grapheme_boundary(self.cursor_idx);
        self.text.remove(self.cursor_idx..next);
    }

    /// Remove all text from the buffer and set the cursor location back to zero.
    pub fn clear(&mut self) {
        self.text.remove(0..self.text.len_chars());
        self.cursor_idx = 0;
    }
}

pub fn grapheme_visual_width(grapheme: &str) -> usize {
    // ZWJ emoji sequences are rendered as a single emoji, normally width 2.
    if grapheme.contains('\u{200D}') {
        return 2;
    }

    grapheme.chars().map(|c| c.width_cjk().unwrap_or(0)).sum()
}