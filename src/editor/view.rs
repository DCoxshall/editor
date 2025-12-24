use std::{fs::read_to_string, path::PathBuf};

use crate::editor::buffer::Buffer;

/// "View" into a single file. Handles visualisation of the text in its buffer.
pub struct View {
    /// Text from the file we're currently editing.
    pub buffer: Buffer,

    // Path to said file.
    pub file_path: PathBuf,
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
            file_path: path,
        })
    }

    /// Creates a new empty view. The user can write in an empty view. Upon attempting to save, the
    /// editor should prompt them for a filename, which the view will be saved to.
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(String::from("")),
            file_path: PathBuf::new(),
        }
    }
}
