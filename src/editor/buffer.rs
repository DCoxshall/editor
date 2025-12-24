use ropey::Rope;

/// Buffer acts as a wrapper around Ropey::rope. It's only job is to hold and manipulate text.
pub struct Buffer {
    pub text: Rope,
    cursor_idx: usize,
}

impl Buffer {
    pub fn new(text: String) -> Self {
        Buffer {
            text: Rope::from_str(&text),
            cursor_idx: 0,
        }
    }
}
