use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event},
    execute, queue,
    terminal::{self, ClearType, disable_raw_mode},
};
use std::io::{Stdout, Write, stdout};

use crate::ui::visual_box::VisualBox;

pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn new() -> Result<Self, std::io::Error> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(Self { stdout: stdout() })
    }

    pub fn cleanup(&mut self) -> Result<(), std::io::Error> {
        self.clear();
        execute!(self.stdout, terminal::LeaveAlternateScreen)?;
        disable_raw_mode()?;
        self.show_cursor();
        Ok(())
    }

    pub fn size(&self) -> (usize, usize) {
        let (w, h) = terminal::size().unwrap();
        (w as usize, h as usize)
    }

    pub fn clear(&mut self) {
        queue!(self.stdout, terminal::Clear(ClearType::All)).unwrap();
    }

    pub fn hide_cursor(&mut self) {
        queue!(self.stdout, Hide).unwrap();
    }

    pub fn show_cursor(&mut self) {
        queue!(self.stdout, Show).unwrap();
    }

    /// Compiles and draws a visual box on screen at the given (x, y) coordinates.
    pub fn draw_visual_box(&mut self, x: usize, y: usize, visual_box: VisualBox) {
        let vb_text = visual_box.compile();
        let mut j = y;
        for line in vb_text.lines() {
            self.draw_line(x, j, line);
            j += 1;
        }
    }

    /// Draws a line on the terminal window at a given (x, y) coordinate.
    fn draw_line(&mut self, x: usize, y: usize, text: &str) {
        queue!(
            self.stdout,
            cursor::MoveTo(x as u16, y as u16),
            crossterm::style::Print(text)
        )
        .unwrap();
    }

    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    pub fn read_event(&self) -> Result<Event, std::io::Error> {
        event::read()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.cleanup().unwrap();
    }
}