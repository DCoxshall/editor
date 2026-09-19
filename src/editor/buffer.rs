use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

/// Acts as a wrapper around Ropey::rope. It's only job is to hold and manipulate text.
#[derive(Clone, PartialEq)]
pub struct Buffer {
    /// Holds the buffer's text.
    pub text: Rope,

    /// Index into the graphemes of self.text.
    pub cursor_idx: usize,
}

impl Buffer {
    /// Creates a new `Buffer` as a wrapper around the given String.
    pub fn new(text: String) -> Self {
        // Replace CRLF with LF.
        let replaced = text.replace("\r\n", "\n");
        Buffer {
            text: Rope::from_str(&replaced),
            cursor_idx: 0,
        }
    }

    /// Creates a new empty `Buffer`.
    pub fn empty() -> Self {
        Buffer {
            text: Rope::from_str(""),
            cursor_idx: 0,
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

    /// Delete the grapheme immediately before the cursor.
    pub fn backspace(&mut self) {
        if self.cursor_idx == 0 {
            return;
        }

        let text_before_cursor = self.text.slice(..self.cursor_idx);

        let binding = text_before_cursor.to_string();
        let mut graphemes = binding.grapheme_indices(true);

        let Some((grapheme_start, _)) = graphemes.next_back() else {
            return;
        };

        let grapheme_len = self.cursor_idx - grapheme_start;

        self.text.remove(grapheme_start..self.cursor_idx);
        self.cursor_idx -= grapheme_len;
    }

    /// Delete the character immediately after the cursor.
    pub fn delete(&mut self) {
        if self.cursor_idx != self.text.len_chars() {
            self.text.remove(self.cursor_idx..self.cursor_idx + 1);
        }
    }

    /// Remove all text from the buffer and set the cursor location back to zero.
    pub fn clear(&mut self) {
        self.text.remove(0..self.text.len_chars());
        self.cursor_idx = 0;
    }
}
