use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
enum Cell {
    Empty,            // width 1
    Grapheme(String), // may be width 1 or 2
    Continuation,     // marks the second cell of a wide char
}

pub struct VisualBox {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl VisualBox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Empty; (width * height) as usize],
        }
    }

    /// Returns the index within self.cells of the visual coordinate (x, y) measured from the top
    /// left of the screen.
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn compile(&self) -> String {
        let mut out = String::new();

        for y in 0..self.height {
            for x in 0..self.width {
                match &self.cells[self.index(x, y) as usize] {
                    Cell::Empty => out.push(' '),
                    Cell::Grapheme(g) => out.push_str(g),
                    Cell::Continuation => {} // skip wide-char continuation
                }
            }
            out.push('\n');
        }

        out
    }

    pub fn draw(&mut self, mut x: usize, y: usize, text: &str) {
        if y >= self.height || x >= self.width {
            return;
        }

        let real_text = match text {
            "\n" => " ",
            _ => text.trim_end(),
        };

        for grapheme in real_text.graphemes(true) {
            let width = UnicodeWidthStr::width(grapheme);

            // Combine zero-width graphemes with the previous grapheme.
            if width == 0 {
                if x > 0 {
                    let i = self.index(x - 1, y);
                    if let Cell::Grapheme(g) = &mut self.cells[i as usize] {
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
            self.cells[i as usize] = Cell::Grapheme(grapheme.to_string());
            for dx in 1..width {
                self.cells[(i + dx) as usize] = Cell::Continuation;
            }

            x += width;
        }
    }
}
