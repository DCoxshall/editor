use crossterm::style::Color;
use crossterm::style::{Attributes, ContentStyle};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, PartialEq)]
pub enum Cell {
    Empty,            // width 1
    Grapheme(String), // may be width 1 or 2
    Continuation,     // marks the second cell of a wide char
}
#[derive(Clone, PartialEq)]
pub struct StyledCell {
    pub cell: Cell,
    pub style: ContentStyle,
}

pub struct VisualBox {
    pub width: usize,
    pub height: usize,
    cells: Vec<StyledCell>,
}

fn reset_style() -> ContentStyle {
    ContentStyle {
        foreground_color: Some(Color::Reset),
        background_color: Some(Color::Reset),
        underline_color: Some(Color::Reset),
        attributes: Attributes::none(),
    }
}

impl VisualBox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![StyledCell { cell: Cell::Empty, style: reset_style() }; (width * height) as usize],
        }
    }

    /// Returns the index within self.cells of the visual coordinate (x, y) measured from the top
    /// left of the screen.
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn at(&self, x: usize, y: usize) -> &StyledCell {
        let idx = self.index(x, y);
        &self.cells[idx]
    }

    pub fn draw(&mut self, x: usize, y: usize, text: &str) {
        self.draw_with_style(x, y, text, reset_style());
    }

    pub fn set(&mut self, x: usize, y: usize, cell: &StyledCell) {
        let idx = self.index(x, y);
        if idx >= self.cells.len() {
            return;
        }
        self.cells[idx] = cell.clone();
    }

    pub fn draw_with_style(&mut self, mut x: usize, y: usize, text: &str, style: ContentStyle) {
        if y >= self.height || x >= self.width {
            return;
        }

        let mut real_text = text.to_owned();
        if real_text.ends_with('\n') {
            real_text.pop();
        }
        if real_text.ends_with('\r') {
            real_text.pop();
        }

        for grapheme in real_text.graphemes(true) {
            let width = UnicodeWidthStr::width(grapheme);

            // Combine zero-width graphemes with the previous grapheme.
            if width == 0 {
                if x > 0 {
                    let i = self.index(x - 1, y);
                    if let Cell::Grapheme(g) = &mut self.cells[i].cell {
                        g.push_str(grapheme);
                    }
                }
                continue;
            }

            // If it won't fit, stop drawing.
            if x + width > self.width {
                break;
            }

            // Write the grapheme to self.cells.
            let i = self.index(x, y);
            self.cells[i].cell = Cell::Grapheme(grapheme.to_string());
            self.cells[i].style = style;
            for dx in 1..width {
                self.cells[i + dx].cell = Cell::Continuation;
                self.cells[i + dx].style = style;
            }

            x += width;
        }
    }
}
