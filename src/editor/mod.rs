mod buffer;

use buffer::Buffer;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::{Color::*, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::size,
};
use std::io::{Stdout, Write};
use std::{cmp::min, path::PathBuf};

/// Main editor data structure.
pub struct Editor {
    /// Main text buffer. One buffer represents one open file. Currently a single editor contains
    /// only a single buffer.
    pub buffer: Buffer,
}

impl Editor {
    /// The string shown on an out-of-bounds line.
    const EMPTY_LINE_NOTATION: &str = "~";
    const TAB_WIDTH: usize = 4;

    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let buffer = match Buffer::from_path(path) {
            Ok(buf) => buf,
            Err(err) => return Err(err),
        };

        return Ok(Editor { buffer: buffer });
    }

    /// Renders the entire editor to stdout. This is the only `render` function that should be
    /// called in `main.rs`.
    pub fn render(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        execute!(stdout, Hide)?; // Hide the cursor while drawing.

        let (_, rows) = size().unwrap();

        if rows >= 3 {
            // -1 for the footer bar and -1 for the buffer status bar.
            for i in 0..((self.buffer.visual_height - 2) as usize) {
                let line_idx = self.buffer.visual_origin_row + i;

                let mut text: String;

                if line_idx < self.buffer.len_lines() {
                    // Fetch the line from from the buffer and strip the trailing newline.
                    text = self.buffer.get_line(line_idx);

                    // Remove `n` characters from the front of the line, where `n` is
                    // buffer.visual_origin_col.
                    text = text.chars().skip(self.buffer.visual_origin_col).collect();

                    // Replace tab characters with spaces when rendering.
                    text = text.replace('\t', &" ".repeat(Editor::TAB_WIDTH));
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
        }
        if rows >= 2 {
            self.render_status_bar(stdout)?;
        }
        if rows >= 1 {
            self.render_footer_bar(stdout)?;
        }
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

    fn render_status_bar(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let (cols, rows) = size()?;

        // We only want to render the status bar if there are 2 or more rows being rendered to the
        // screen.
        if rows < 2 {
            return Ok(());
        }

        let text = self.buffer.get_status_bar_text();
        let blank_space = cols - min(text.len() as u16, cols);

        execute!(stdout, MoveTo(0, rows - 2))?;
        write!(stdout, "{}{}", text, " ".repeat(blank_space as usize))?;
        Ok(())
    }

    /// Draws the footer bar. The footer bar is a property of the entire editor rather than a single
    /// buffer.
    fn render_footer_bar(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let (cols, rows) = size()?;
        execute!(stdout, MoveTo(0, rows - 1))?;
        execute!(stdout, SetBackgroundColor(White), SetForegroundColor(Black))?;

        let footer_bar = format!(
            "Line: {}, Column: {}. Press Ctrl-D or F10 to quit.",
            self.buffer.get_logical_cursor_line(),
            self.buffer.get_logical_cursor_col()
        );

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

            // Handle Ctrl-<X>
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                match key_event.code {
                    KeyCode::Char('d') => {
                        return true;
                    }
                    KeyCode::Char('s') => {
                        self.buffer.save_file();
                    }
                    _ => self.buffer.handle_key_event(key_event),
                }
            } else {
                match key_event.code {
                    KeyCode::F(10) => return true,
                    _ => {
                        self.buffer.handle_key_event(key_event);
                    }
                }
            }
        }
        false
    }

    /// Ensures the cursor remains on screen at all times by moving the viewport if the cursor has
    /// gone out-of-bounds since the last input event.
    pub fn align_cursor(&mut self) {
        let (_, rows) = size().unwrap();

        // It doesn't matter where the cursor is in this case because no part of the buffer will be
        // shown on-screen.
        if rows < 3 {
            return;
        }

        let line_idx = self.buffer.char_to_line(self.buffer.cursor_idx);
        let col_idx = self.buffer.cursor_idx - self.buffer.line_to_char(line_idx);

        // If the cursor is above the first visual line, then set the line the cursor is on to be
        // the first visual line.
        if line_idx < self.buffer.visual_origin_row {
            self.buffer.visual_origin_row = line_idx;
        }

        // Similarly, if the cursor is below the last line, then the last line needs to be the line
        // the cursor is on. NOTE: the `-2` in the conditional is to ensure the cursor doesn't enter
        // the status bar or the footer bar.
        if line_idx >= self.buffer.visual_origin_row + self.buffer.visual_height - 2 {
            self.buffer.visual_origin_row = line_idx - (self.buffer.visual_height - 3);
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
