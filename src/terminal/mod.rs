use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{self, Event},
    execute, queue,
    style::SetStyle,
    terminal::{self, ClearType, disable_raw_mode, size},
};
use std::io::{Stdout, Write, stdout};

use crate::ui::visual_box::{Cell::*, VisualBox};

pub struct Terminal {
    stdout: Stdout,

    /// Represents what is set to be rendered to the screen at the end of the
    /// next cycle. When the UI layer calls into the Terminal layer to draw to
    /// the screen, instead of actually queueing something up to be drawn to the
    /// screen, we just draw to this box, and queue up only once at the end of
    /// each cycle.
    visual_box: VisualBox,

    /// Represents what has already been rendered to the screen (i.e. it's a
    /// historical copy of `Terminal::visual_box`).
    rendered: VisualBox,
}

impl Terminal {
    pub fn new() -> Result<Self, std::io::Error> {
        let (width, height) = size()?;
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        Ok(Self {
            stdout: stdout(),
            visual_box: VisualBox::new(width as usize, height as usize),
            rendered: VisualBox::new(width as usize, height as usize),
        })
    }

    /// Reset the internal VisualBoxes to the terminal's new size.
    pub fn resize(&mut self) {
        let (width, height) = size().unwrap();
        let width = width as usize;
        let height = height as usize;
        self.visual_box = VisualBox::new(width, height);
        self.rendered = VisualBox::new(width, height);
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
        for dx in 0..visual_box.width {
            for dy in 0..visual_box.height {
                self.visual_box.set(x + dx, y + dy, visual_box.at(dx, dy));
            }
        }
    }

    fn render_cell_at(&mut self, x: usize, y: usize) {
        let styled_cell = self.visual_box.at(x, y);
        let text = match &styled_cell.cell {
            Empty => " ",
            Grapheme(g) => g,
            Continuation => "",
        };
        queue!(
            self.stdout,
            cursor::MoveTo(x as u16, y as u16),
            SetStyle(styled_cell.style),
            crossterm::style::Print(text)
        )
        .unwrap();
        self.rendered.set(x, y, styled_cell);
    }

    pub fn render(&mut self) {
        for i in 0..self.visual_box.height {
            for j in 0..self.visual_box.width {
                // Render each cell if and only if that cell has changed.
                if self.visual_box.at(j, i) != self.rendered.at(j, i) {
                    self.render_cell_at(j, i);
                }
            }
        }

        // For now, we explicitly want to move to (0, 0) between renders so that
        // the cursor doesn't appear at the last-rendered cell.        
        queue!(self.stdout, MoveTo(0, 0)).unwrap();
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
