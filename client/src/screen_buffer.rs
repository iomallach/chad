use crossterm::style;

pub struct ScreenBuffer {
    buf: Vec<ScreenCell>,
    w: usize,
}

impl ScreenBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buf: Vec::with_capacity(w * h),
            w,
        }
    }
// [][][][]
// [][][][x] -> (1 (y), 3 (x)) -> w * y + x
// [][][][]
    pub fn put_cell(&mut self, cell: ScreenCell, x: usize, y: usize) {
        if let Some(c) = self.buf.get_mut(self.w * y + x) {
            *c = cell;
        }
    }

    pub fn put_cells(&mut self, mut cells: Vec<ScreenCell>, x: usize, y: usize) {
        for (i, c) in cells.drain(..).enumerate() {
            self.put_cell(c, x + i, y);
        }
    }
}

#[derive(Clone)]
pub struct ScreenCell {
    ch: char,
    bg: style::Color,
    fg: style::Color,
}

impl ScreenCell {
    pub fn new(ch: char, bg: style::Color, fg: style::Color) -> Self {
        Self {
            ch,
            bg,
            fg,
        }
    }

    pub fn default() -> Self {
        Self::new(' ', style::Color::Black, style::Color::White)
    }

    pub fn bar_cell(ch: char, fg: style::Color) -> Self {
        Self::new(ch, style::Color::White, fg)
    }
}