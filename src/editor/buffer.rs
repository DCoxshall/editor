use ropey::Rope;

/// One buffer represents one open file.
pub struct Buffer {
    // Contains the actual data in the buffer.
    pub text: Rope,

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
    pub fn from_string(string: String, width: usize, height: usize) -> Self {
        let new_buffer = Buffer {
            text: Rope::from(string),
            visual_origin_row: 0,
            visual_origin_col: 0,
            cursor_idx: 0,
            visual_width: width,
            visual_height: height,
        };

        new_buffer
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
        self.get_logical_cursor_col() - self.visual_origin_col
    }

    /// Gets the row that the cursor should be shown at visually.
    pub fn get_visual_cursor_line(&self) -> usize {
        self.get_logical_cursor_line() - self.visual_origin_row
    }
}
