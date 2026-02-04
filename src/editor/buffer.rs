use ropey::Rope;

/// Buffer acts as a wrapper around Ropey::rope. It's only job is to hold and manipulate text.
#[derive(Clone, PartialEq)]
pub struct Buffer {
    pub text: Rope,

    /// Index into the graphemes of self.text.
    pub cursor_idx: usize,
}

impl Buffer {
    pub fn new(text: String) -> Self {
        // Replace CRLF with LF.
        let replaced = text.replace("\r\n", "\n");
        Buffer {
            text: Rope::from_str(&replaced),
            cursor_idx: 0,
        }
    }

    pub fn insert(&mut self, c: char) {
        self.text.insert_char(self.cursor_idx, c);
        self.cursor_idx += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_idx != 0 {
            self.text.remove(self.cursor_idx - 1..self.cursor_idx);
            self.cursor_idx -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_idx != self.text.len_chars() {
            self.text.remove(self.cursor_idx..self.cursor_idx + 1);
        }
    }

    pub fn clear(&mut self) {
        self.text.remove(0..self.text.len_chars());
        self.cursor_idx = 0;
    }
}
