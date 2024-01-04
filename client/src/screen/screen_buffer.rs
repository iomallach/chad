use std::io::Write;

use crossterm::{style::{self, Stylize, Print}, QueueableCommand};

pub struct ScreenBuffer {
    buf: Vec<ScreenCell>,
    w: usize,
    diff: Vec<ChangedScreenCell>
}

impl ScreenBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buf: vec![ScreenCell::default(); w * h],
            w,
            diff: Vec::new(),
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

    pub fn fill_row(&mut self, cell: ScreenCell, row: usize, left_bound: Option<usize>, right_bound: Option<usize>) {
        match (left_bound, right_bound) {
            (Some(lb), Some(rb)) => {
                self.fill(
                    cell, 
                    lb, 
                    row * self.w, 
                    rb - lb
                )
            },
            (Some(lb), None) => {
                self.fill(cell, lb, row * self.w, self.w - lb)
            },
            (None, Some(rb)) => {
                self.fill(cell, 0, row * self.w, rb)
            },
            (None, None) => self.fill(cell, 0, row * self.w, self.w),
        }
    }
// [][][x][][] -> (0, 2) 2
// [][][x][][] -> (1, 2) 7
// [][][x][][] -> (3, 2) 12
// [][][x][][] -> (4, 2) 17
    pub fn fill_col(&mut self, cell: ScreenCell, col: usize, top_row: usize, bottow_row: usize) {
        for y in top_row..=bottow_row {
            self.fill(cell.clone(), col, y, 1);
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        self.fill(ScreenCell::default(), 0, row * self.w, self.w)
    }

    pub fn put_cell(&mut self, cell: ScreenCell, x: usize, y: usize) {
        if let Some(c) = self.buf.get_mut(self.w * y + x) {
            if *c != cell {
                self.diff.push(
                    ChangedScreenCell {
                        cell: cell.clone(),
                        x,
                        y,
                    }
                )
            }
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

    pub fn reset_diff(&mut self) {
        self.diff.clear()
    }

    pub fn render_diff(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        stdout.queue(crossterm::cursor::Hide)?;
        let mut x_prev = 0;
        let mut y_prev = 0;
        for cell in &self.diff {
            if cell.x.checked_sub(1).unwrap_or(0) != x_prev || cell.y != y_prev {
                stdout.queue(crossterm::cursor::MoveTo(cell.x as u16, cell.y as u16))?;
            }
            stdout.queue(Print(cell.cell.ch.with(cell.cell.fg).on(cell.cell.bg)))?;
            x_prev = cell.x;
            y_prev = cell.y;
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
        Self::new(' ', style::Color::Reset, style::Color::White)
    }

    pub fn bar_cell(ch: char, fg: style::Color) -> Self {
        Self::new(ch, style::Color::White, fg)
    }

    pub fn bar_empty_space() -> Self {
        Self::bar_cell(' ', style::Color::White)
    }
}

impl PartialEq for ScreenCell {
    fn eq(&self, other: &Self) -> bool {
        self.ch == other.ch && self.bg == other.bg && self.fg == other.fg
    }
}

struct ChangedScreenCell {
    cell: ScreenCell,
    x: usize,
    y: usize,
}