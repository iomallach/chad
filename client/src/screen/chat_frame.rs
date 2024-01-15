use super::{Rect, screen_buffer::{ScreenBuffer, ScreenCell}};
use crossterm::style::Color;
use itertools::Itertools;

pub struct ChatFrame {
    left: Rect,
    right: Rect,
    top: Rect,
    bottom: Rect,
    header: String,
}

enum ColBorder {
    LEFT,
    RIGHT,
}

enum RowBorder {
    TOP,
    BOTTOM,
}

impl ChatFrame {
    pub fn new(parent_rect: &Rect) -> Self {
        Self {
            left: Rect::new(parent_rect.x.into(), parent_rect.y as usize + 2, 1, parent_rect.h as usize - 6),
            right: Rect::new(parent_rect.w as usize - 1, parent_rect.y as usize + 2, 1, parent_rect.h as usize - 6),
            top: Rect::new(parent_rect.x.into(), parent_rect.y as usize + 1, parent_rect.w.into(), 1),
            bottom: Rect::new(parent_rect.x.into(), parent_rect.h as usize - 3, parent_rect.w.into(), 1),
            header: "Chat window".into(),
        }
    }

    pub fn render(&self, buf: &mut ScreenBuffer) {
        Self::_render_row(&self.bottom, buf, RowBorder::BOTTOM);
        Self::_render_row(&self.top, buf, RowBorder::TOP);
        Self::_render_col(&self.left, buf, ColBorder::LEFT);
        Self::_render_col(&self.right, buf, ColBorder::RIGHT);
    }

    fn _render_col(rect: &Rect, buf: &mut ScreenBuffer, col_border: ColBorder) {
        let special_border_top = match col_border {
            ColBorder::LEFT => '┌',
            ColBorder::RIGHT => '┐', 
        };
        let special_border_bottom = match col_border {
            ColBorder::LEFT => '└',
            ColBorder::RIGHT => '┘',
        };
        buf.fill_col(
            ScreenCell::new(special_border_top, Color::Reset, Color::White, false),
            rect.x,
            rect.y - 1,
            rect.y - 1
        );
        buf.fill_col(
            ScreenCell::new('│', Color::Reset, Color::White, false),
            rect.x,
            rect.y,
            rect.y + rect.h
        );
        buf.fill_col(
            ScreenCell::new(special_border_bottom, Color::Reset, Color::White, false),
            rect.x,
            rect.y + rect.h + 1,
            rect.y + rect.h + 1
        );
    }

    fn _render_row(rect: &Rect, buf: &mut ScreenBuffer, row_border: RowBorder) {
        let y = match row_border {
            RowBorder::BOTTOM => rect.y,
            RowBorder::TOP => rect.y,
        };
        // TODO: hardcoded to quickly test, need to make it percentage based and blahblah
        if let RowBorder::TOP = row_border {
            buf.fill(
                ScreenCell::new('─', Color::Reset, Color::White, false),
                rect.x + 1,
                y,
                4,
            );
            let header_cells = "Chat window".chars().map(|c| {
                ScreenCell::new(c, Color::Reset, Color::White, false)
            }).collect_vec();
            buf.put_cells(header_cells, rect.x + 1 + 5, y);
            buf.fill(
                ScreenCell::new('─', Color::Reset, Color::White, false),
                rect.x + 1 + 5 + 11 + 1,
                y,
                rect.w - 2 - 5 - 11,
            );
        } else {
            buf.fill(
                ScreenCell::new('─', Color::Reset, Color::White, false),
                rect.x + 1,
                y,
                rect.w - 2,
            );
        }
    }
}