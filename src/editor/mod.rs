mod buffer;

use buffer::Buffer;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read},
    execute,
    style::{Color::*, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};
use std::{
    cmp::max,
    io::{Stdout, Write, stdout},
};
use std::{cmp::min, path::PathBuf};
use unicode_width::UnicodeWidthStr;

/// Main editor data structure.
pub struct Editor {
    /// Main text buffer. One buffer represents one open file. Currently a single editor contains
    /// only a single buffer.
    pub buffer: Buffer,

    /// Text to be displayed in the footer.
    pub footer_text: String,

    stdout: Stdout,
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
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        return Ok(Editor {
            buffer: buffer,
            footer_text: String::from(""),
            stdout,
        });
    }

    /// Renders the entire editor to stdout. This is the only `render` function that should be
    /// called in `main.rs`.
    pub fn render(&mut self) -> std::io::Result<()> {
        execute!(self.stdout, Hide)?; // Hide the cursor while drawing.

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
                    text += &(" ".repeat(self.buffer.visual_width - text.width_cjk()));
                }

                execute!(self.stdout, MoveTo(0, i as u16))?;
                write!(self.stdout, "{}", text)?;
            }
        }
        if rows >= 2 {
            self.render_status_bar()?;
        }
        if rows >= 1 {
            self.render_footer_bar()?;
        }
        execute!(
            self.stdout,
            MoveTo(
                self.buffer.get_visual_cursor_col() as u16,
                self.buffer.get_visual_cursor_line() as u16
            )
        )?;
        execute!(self.stdout, Show)?; // Show the cursor again once we've finished drawing.

        Ok(())
    }

    fn render_status_bar(&mut self) -> std::io::Result<()> {
        let (cols, rows) = size()?;

        // We only want to render the status bar if there are 2 or more rows being rendered to the
        // screen.
        if rows < 2 {
            return Ok(());
        }

        let text = self.buffer.get_status_bar_text();
        let blank_space = cols - min(text.len() as u16, cols);

        execute!(self.stdout, MoveTo(0, rows - 2))?;
        write!(self.stdout, "{}{}", text, " ".repeat(blank_space as usize))?;
        Ok(())
    }

    /// Draws the footer bar. The footer bar is a property of the entire editor rather than a single
    /// buffer.
    fn render_footer_bar(&mut self) -> std::io::Result<()> {
        let (cols, rows) = size()?;
        execute!(self.stdout, MoveTo(0, rows - 1))?;
        execute!(
            self.stdout,
            SetBackgroundColor(White),
            SetForegroundColor(Black)
        )?;

        let footer_bar = &self.footer_text;

        let message_len = min(footer_bar.len() as u16, cols);

        let footer_text: String = footer_bar.chars().take(message_len as usize).collect();

        let blank_space = cols - message_len;
        write!(
            self.stdout,
            "{}{}",
            footer_text,
            " ".repeat(blank_space as usize)
        )?;
        execute!(self.stdout, ResetColor)?;
        Ok(())
    }

    fn save_buffer(&mut self) {
        // If the buffer does not have a file path, prompt the user for one.
        if self.buffer.file_path.as_os_str().is_empty() {
            let new_filename = self.editor_prompt("Enter new filename> ");
            match new_filename {
                Some(name) => {
                    self.buffer.file_path.push(&name);
                    match self.buffer.save_file() {
                        Ok(()) => self.footer_text = format!("New file saved as {}", &name),
                        Err(_) => self.footer_text = format!("File save failed. Please try again."),
                    }
                }
                None => self.footer_text = String::from("No file name given, cancelled save."),
            }
        } else {
            match self.buffer.save_file() {
                Ok(_) => self.footer_text = format!("File saved."),
                Err(_) => self.footer_text = format!("File save failed. Please try again."),
            }
        }
    }

    /// If the buffer is dirty, we need to ask the user whether they really meant to exit without
    /// saving. Otherwise, just exit.
    fn attempt_exit(&mut self) -> bool {
        if self.buffer.dirty_buffer {
            let response =
                self.editor_prompt("The buffer is unsaved. Do you really want to exit? (y/n): ");
            match response {
                Some(str) => {
                    if str == "y" || str == "Y" || str == "yes" {
                        return true;
                    } else {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        } else {
            return true;
        }
    }

    /// Returns true if the user wants to quit, false otherwise.
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if key_event.kind == KeyEventKind::Press {
            // Handle Ctrl-<X>
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                match key_event.code {
                    KeyCode::Char('d') => {
                        return self.attempt_exit();
                    }
                    KeyCode::Char('s') => {
                        self.save_buffer();
                    }
                    KeyCode::Char('f') => {
                        let target = match self.editor_prompt("Enter target text> ") {
                            Some(text) => text,
                            None => {
                                self.footer_text = String::from("Search cancelled.");
                                return false;
                            }
                        };

                        let found = self.buffer.go_to_next_instance(&target);

                        if !found {
                            let user_response =
                                match self.editor_prompt("No match found. Search from top? y/n> ") {
                                    Some(text) => text,
                                    None => {
                                        self.footer_text = String::from("Search cancelled.");
                                        return false;
                                    }
                                };
                            if user_response == "y" {
                                let past_cursor_idx = self.buffer.cursor_idx;
                                self.buffer.cursor_idx = 0;
                                let found = self.buffer.go_to_next_instance(&target);
                                if !found {
                                    self.buffer.cursor_idx = past_cursor_idx;
                                    self.footer_text = String::from("No match found.");
                                } else {
                                    self.footer_text = String::from("Match found.");
                                }
                            }
                        } else {
                            self.footer_text = String::from("Match found.");
                        }
                    }
                    _ => self.buffer.handle_key_event(key_event),
                }
            } else {
                match key_event.code {
                    KeyCode::F(10) => return true,
                    // KeyCode::F(1) => {
                    //     let user_text = self.editor_prompt("> ");
                    //     match user_text {
                    //         Some(text) => {
                    //             self.footer_text = format!("You entered a command: {}", text)
                    //         }
                    //         None => {}
                    //     }
                    // }
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

    pub fn clear_terminal(&mut self) -> std::io::Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        execute!(self.stdout, MoveTo(0, 0))?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Prompt the user for some input, and return that input as a string. The prompt will appear in
    /// the footer bar, a la Vim.
    pub fn editor_prompt(&mut self, prompt_text: &str) -> Option<String> {
        self.footer_text = prompt_text.to_owned();
        let mut user_input = String::new();

        let (cols, _) = size().unwrap();

        loop {
            self.footer_text = format!("{}{}", prompt_text, user_input);
            self.render().ok();
            let _ = execute!(self.stdout, MoveTo(self.footer_text.len() as u16, cols - 1));
            let _ = self.stdout.flush();

            match read() {
                Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char(x) => {
                            user_input.push(x);
                        }
                        KeyCode::Backspace => {
                            user_input.pop();
                        }
                        KeyCode::Enter => {
                            self.footer_text.clear();
                            return Some(user_input);
                        }
                        KeyCode::Esc => {
                            self.footer_text.clear();
                            return None;
                        }
                        _ => {}
                    }
                }
                Err(_) => return None,
                _ => {}
            }
        }
    }

    pub fn mainloop(&mut self) -> std::io::Result<()> {
        loop {
            self.render()?;
            self.stdout.flush()?;
            match read() {
                Ok(Event::Key(key_event)) => {
                    let quit = self.handle_key_event(key_event);
                    if quit {
                        break;
                    }
                }
                Ok(Event::Resize(w, h)) => {
                    self.buffer.visual_width = w as usize;
                    self.buffer.visual_height = max(h, 0) as usize;
                }
                Err(err) => {
                    return Err(err);
                }
                _ => {}
            }

            // After every input event, we need to ensure that the cursor remains on screen.
            self.align_cursor();
        }

        disable_raw_mode()?;
        execute!(self.stdout, LeaveAlternateScreen, Show)?;

        return Ok(());
    }
}
