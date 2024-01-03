use std::io::Write;

use crossterm::{style::{self, Stylize, Print}, QueueableCommand};

pub struct ScreenBuffer {
    buf: Vec<ScreenCell>,
    w: usize,
}

impl ScreenBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buf: vec![ScreenCell::default(); w * h],
            w,
        }
    }

    pub fn default(w: usize, h: usize) -> Self {
        let mut slf = Self::new(w, h);
        slf.fill(ScreenCell::default(), 0, 0, w*h);
        slf.fill(ScreenCell::bar_cell(' ', style::Color::White), 0, 0, w);
        slf.fill(ScreenCell::bar_cell(' ', style::Color::White), 0, h - 2, w);
        slf
    }

    pub fn fill(&mut self, cell: ScreenCell, x: usize, y: usize, w: usize) {
        self.put_cells(vec![cell; w], x, y)
    }

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

    pub fn render(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        self.render_subscreen(0, 0, self.buf.len()/self.w, stdout)
    }

    pub fn render_subscreen(&self, from_x: usize, from_y: usize, lines: usize, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        stdout.queue(crossterm::cursor::Hide)?;
        stdout.queue(crossterm::cursor::MoveTo(from_x as u16, from_y as u16))?;
        let start = from_x * self.w + from_y;
        let stop = start + lines * self.w;
        for cell in &self.buf[start..stop] {
            stdout.queue(Print(cell.ch.with(cell.fg).on(cell.bg)))?;
        }
        stdout.queue(crossterm::cursor::Show)?;
        stdout.flush()?;
        Ok(())
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

    pub fn bar_empty_space() -> Self {
        Self::bar_cell(' ', style::Color::White)
    }
}