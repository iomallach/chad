use crossterm::style::Color;
use itertools::Itertools as _;

use crate::screen::{Rect, screen_buffer::{ScreenBuffer, ScreenCell}};

pub enum BorderKind {
    DEFAULT,
    ROUNDED,
    DOUBLE,
    THICK
}

impl BorderKind {
    fn top_left(&self) -> char {
        match self {
            Self::DEFAULT => '┌',
            Self::ROUNDED => '╭',
            Self::DOUBLE => '╔',
            Self::THICK => '┏',
        }
    }
    
    fn top_right(&self) -> char {
        match self {
            Self::DEFAULT => '┐',
            Self::ROUNDED => '╮',
            Self::DOUBLE => '╗',
            Self::THICK => '┓',
        }
    }

    fn bottom_left(&self) -> char {
        match self {
            Self::DEFAULT => '└',
            Self::ROUNDED => '╰',
            Self::DOUBLE => '╚',
            Self::THICK => '┗',
        }
    }

    fn bottom_right(&self) -> char {
        match self {
            Self::DEFAULT => '┘',
            Self::ROUNDED => '╯',
            Self::DOUBLE => '╝',
            Self::THICK => '┛',
        }
    }

    fn horizontal(&self) -> char {
        match self {
            Self::DEFAULT => '─',
            Self::ROUNDED => '─',
            Self::DOUBLE => '═',
            Self::THICK => '━',
        }
    }

    fn vertical(&self) -> char {
        match self {
            Self::DEFAULT => '│',
            Self::ROUNDED => '│',
            Self::DOUBLE => '║',
            Self::THICK => '┃',
        }
    }
}

enum ColBorder {
    LEFT,
    RIGHT,
}

enum RowBorder {
    TOP,
    BOTTOM,
}

pub struct Borders {
    title: String,
    title_alignment_prc: f32,
    fg: Color,
    border_kind: BorderKind,
}

impl Borders {
    pub fn new(title: String, title_alignment_prc: f32, fg: Color, border_kind: BorderKind) -> Self {
        Self {
            title,
            title_alignment_prc,
            fg,
            border_kind,
        }
    }

    fn inner_rect(rect: Rect) -> Rect {
        rect.subrect(1, 1, 1, 1)
    }

    pub fn render(&self, rect: Rect, buf: &mut ScreenBuffer) {
        self._render_row(&rect.subrect(0, rect.h - 1, 0, rect.h - 1), buf, RowBorder::BOTTOM);
        self._render_row(&rect.subrect(0, 0, 0, rect.h - 1), buf, RowBorder::TOP);
        self._render_col(&rect.subrect(0, 0, rect.w - 1, 0), buf, ColBorder::LEFT);
        self._render_col(&rect.subrect(rect.w - 1, 0, rect.w - 1, 0), buf, ColBorder::RIGHT);
    }

    fn _render_col(&self, rect: &Rect, buf: &mut ScreenBuffer, col_border: ColBorder) {
        let special_border_top = match col_border {
            ColBorder::LEFT => self.border_kind.top_left(),
            ColBorder::RIGHT => self.border_kind.top_right(), 
        };
        let special_border_bottom = match col_border {
            ColBorder::LEFT => self.border_kind.bottom_left(),
            ColBorder::RIGHT => self.border_kind.bottom_right(),
        };
        buf.fill_col(
            ScreenCell::new(special_border_top, Color::Reset, self.fg, false),
            rect.x,
            rect.y,
            rect.y
        );
        buf.fill_col(
            ScreenCell::new(self.border_kind.vertical(), Color::Reset, self.fg, false),
            rect.x,
            rect.y + 1,
            rect.y + rect.h - 2
        );
        buf.fill_col(
            ScreenCell::new(special_border_bottom, Color::Reset, self.fg, false),
            rect.x,
            rect.y + rect.h - 1,
            rect.y + rect.h - 1
        );
    }

    fn _render_row(&self, rect: &Rect, buf: &mut ScreenBuffer, row_border: RowBorder) {
        let y = match row_border {
            RowBorder::BOTTOM => rect.y,
            RowBorder::TOP => rect.y,
        };
        if let RowBorder::TOP = row_border {
            let align_value =(rect.w as f32 * self.title_alignment_prc).floor() as usize;
            buf.fill(
                ScreenCell::new(self.border_kind.horizontal(), Color::Reset, self.fg, false),
                rect.x + 1,
                y,
                align_value,
            );
            let header_cells = self.title.chars().map(|c| {
                ScreenCell::new(c, Color::Reset, self.fg, false)
            }).collect_vec();
            buf.put_cells(header_cells, rect.x + 1 + align_value, y);
            buf.fill(
                ScreenCell::new(self.border_kind.horizontal(), Color::Reset, self.fg, false),
                rect.x + 1 + align_value + self.title.len(),
                y,
                rect.w - 1 - align_value - self.title.len(),
            );
        } else {
            buf.fill(
                ScreenCell::new(self.border_kind.horizontal(), Color::Reset, self.fg, false),
                rect.x + 1,
                y,
                rect.w - 2,
            );
        }
    }
}

pub struct Text {
    text: String,
    borders: Option<Borders>
}

impl Text {
    pub fn new(borders: Option<Borders>) -> Self {
        Self {
            text: "".into(),
            borders,
        }
    }

    pub fn with_borders(&mut self, title:  &str, title_alignment_prc: f32, fg: Color) -> &mut Self {
        self.borders = Some(Borders::new(title.into(), title_alignment_prc, fg, BorderKind::DEFAULT));
        self
    }

    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = text.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_renders() {
        let expected = "┌──W┐\n│XXX│\n│XXX│\n│XXX│\n└───┘\n";
        let h = 5;
        let w = 5;
        let mut buf = ScreenBuffer::new(w, h);
        let borders = Borders::new("W".into(), 0.4, Color::Black, BorderKind::DEFAULT);
        buf.fill(ScreenCell::new('X', Color::Reset, Color::Reset, false), 0, 0, h * w);
        borders.render(Rect::new(0, 0, w, h), &mut buf);
        assert_eq!(expected, format!("{}", buf));
    }
}