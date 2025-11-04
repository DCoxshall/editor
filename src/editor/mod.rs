mod buffer;

use buffer::Buffer;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::{Color::*, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::size,
};
use std::cmp::min;
use std::io::{Result, Stdout, Write};

/// Main editor data structure.
pub struct Editor {
    /// Main text buffer. One buffer represents one open file. Currently a single editor contains
    /// only a single buffer.
    pub buffer: Buffer,

    pub filename: String,
}

impl Editor {
    /// The string shown on an out-of-bounds line.
    const EMPTY_LINE_NOTATION: &str = "~";

    pub fn from_string(string: String, filename: String) -> Self {
        let (cols, rows) = size().unwrap();
        let buffer_width: usize = cols as usize;
        let buffer_height: usize = (rows) as usize; // One for the status bar and one for the footer.

        return Editor {
            buffer: Buffer::from_string(string, buffer_width, buffer_height),
            filename: filename,
        };
    }

    /// Renders the entire editor to stdout. This is the only `render` function that should be
    /// called in `main.rs`.
    pub fn render(&self, stdout: &mut Stdout) -> Result<()> {
        execute!(stdout, Hide)?; // Hide the cursor while drawing.

        for i in 0..((self.buffer.visual_height - 1) as usize) {
            let line_idx = self.buffer.visual_origin_row + i;

            let mut text: String;

            if line_idx < self.buffer.text.len_lines() {
                // Fetch the line from from the buffer and strip the trailing newline.
                let line = self.buffer.text.line(line_idx);
                text = line.to_string();

                // Remove `n` characters from the front of the line, where `n` is
                // buffer.visual_origin_col.
                text = text.chars().skip(self.buffer.visual_origin_col).collect();
            } else {
                text = Editor::EMPTY_LINE_NOTATION.to_owned();
            }

            // Remove line feeds and carriage returns, in that order.
            if text.ends_with('\n') {
                text.pop();
            }

            if text.ends_with('\r') {
                text.pop();
            }

            // If the resulting string is longer than the width of the display, trim it.
            if text.chars().count() > self.buffer.visual_width {
                text = text.chars().take(self.buffer.visual_width).collect();
            }

            // If the resulting string is shorter than the width of the display, pad it.
            if text.chars().count() < self.buffer.visual_width {
                text += &(" ".repeat(self.buffer.visual_width - text.chars().count()));
            }

            execute!(stdout, MoveTo(0, i as u16))?;
            write!(stdout, "{}", text)?;
        }

        self.render_footer_bar(stdout)?;

        execute!(
            stdout,
            MoveTo(
                self.buffer.get_visual_cursor_col() as u16,
                self.buffer.get_visual_cursor_line() as u16
            )
        )?;
        execute!(stdout, Show)?; // Show the cursor again once we've finished drawing.

        Ok(())
    }

    /// Draws the footer bar. The footer bar is a property of the entire editor rather than a single
    /// buffer.
    fn render_footer_bar(&self, stdout: &mut Stdout) -> Result<()> {
        let (cols, rows) = size()?;
        execute!(stdout, MoveTo(0, rows))?;
        execute!(stdout, SetBackgroundColor(White), SetForegroundColor(Black))?;

        let footer_bar = format!("{}", self.filename);

        let message_len = min(footer_bar.len() as u16, cols);

        let footer_text: String = footer_bar.chars().take(message_len as usize).collect();

        let blank_space = cols - message_len;
        write!(
            stdout,
            "{}{}",
            footer_text,
            " ".repeat(blank_space as usize)
        )?;
        execute!(stdout, ResetColor)?;
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::F(1) => return true,
                KeyCode::Char('q') => return true,
                KeyCode::Right => {
                    if self.buffer.cursor_idx < self.buffer.text.len_chars() {
                        self.buffer.cursor_idx += 1;
                    }
                }
                KeyCode::Left => {
                    if self.buffer.cursor_idx > 0 {
                        self.buffer.cursor_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Up => {
                    let current_line_idx = self.buffer.get_logical_cursor_line();
                    let total_lines = self.buffer.text.len_lines();

                    // If the cursor is on the first line and we've pressed Up, it should jump to
                    // the start of that line.
                    if current_line_idx == 0 && key_event.code == KeyCode::Up {
                        self.buffer.cursor_idx = 0;
                    }
                    // If the cursor is on the last line and we've pressed Down, it should jump to
                    // the end of that line.
                    else if current_line_idx == total_lines - 1 && key_event.code == KeyCode::Down
                    {
                        self.buffer.cursor_idx = self.buffer.text.len_chars();
                    }
                    // Otherwise, if the length of the next line is 1 or we're currently at the
                    // start of a line, the cursor should jump to the start of the next line.
                    // Otherwise, it should jump to min(next_line_length, current_line_position).
                    else {
                        let next_line_idx = match key_event.code {
                            KeyCode::Down => current_line_idx + 1,
                            KeyCode::Up => current_line_idx - 1,
                            _ => unreachable!(),
                        };
                        let next_line_char_idx = self.buffer.text.line_to_char(next_line_idx);
                        let next_line_len = self.buffer.text.line(next_line_idx).len_chars();

                        if next_line_len <= 1 || self.buffer.get_logical_cursor_col() == 0 {
                            self.buffer.cursor_idx = next_line_char_idx;
                        } else {
                            let target_col =
                                min(next_line_len, self.buffer.get_logical_cursor_col());
                            self.buffer.cursor_idx = next_line_char_idx + target_col - 1;
                        }
                    }
                }
                KeyCode::Home => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.buffer.cursor_idx = 0;
                    } else {
                        self.buffer.cursor_idx = self
                            .buffer
                            .text
                            .line_to_char(self.buffer.get_logical_cursor_line());
                    }
                }
                KeyCode::End => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.buffer.cursor_idx = self.buffer.text.len_chars();
                    } else {
                        let current_line_len = self
                            .buffer
                            .text
                            .line(self.buffer.get_logical_cursor_line())
                            .len_chars();
                        let current_line_char_idx = self
                            .buffer
                            .text
                            .line_to_char(self.buffer.get_logical_cursor_line());
                        self.buffer.cursor_idx =
                            current_line_char_idx + current_line_len - min(current_line_len, 1);
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Ensures the cursor remains on screen at all times by moving the viewport if the cursor has
    /// gone out-of-bounds since the last input event.
    pub fn align_cursor(&mut self) {
        let line_idx = self.buffer.text.char_to_line(self.buffer.cursor_idx);
        let col_idx = self.buffer.cursor_idx - self.buffer.text.line_to_char(line_idx);

        // If the cursor is above the first visual line, then set the line the cursor is on to be
        // the first visual line.
        if line_idx < self.buffer.visual_origin_row {
            self.buffer.visual_origin_row = line_idx;
        }

        // Similarly, if the cursor is below the last line, then the last line needs to be the line
        // the cursor is on. NOTE: the `-1` in the conditional is to ensure the cursor doesn't enter
        // the status bar.
        if line_idx >= self.buffer.visual_origin_row + self.buffer.visual_height - 1 {
            self.buffer.visual_origin_row = line_idx - (self.buffer.visual_height - 2);
        }

        // If the cursor is left of the first column being displayed, then the first column needs to
        // be the column that the cursor is on.
        if col_idx < self.buffer.visual_origin_col {
            self.buffer.visual_origin_col = col_idx;
        }

        // And finally, if the cursor is right of the last column being displayed, then the last
        // line needs to be the line that the cursor is on.
        if col_idx >= self.buffer.visual_origin_col + self.buffer.visual_width {
            self.buffer.visual_origin_col = col_idx - self.buffer.visual_width + 1;
        }
    }
}
