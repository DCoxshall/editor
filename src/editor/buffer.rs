use ropey::Rope;

/// Buffer acts as a wrapper around Ropey::rope. It's only job is to hold and manipulate text.
#[derive(Clone, PartialEq)]
pub struct Buffer {
    pub text: Rope,

    /// Index into the chars of self.text.
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
}
