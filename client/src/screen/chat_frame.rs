use super::{Rect, screen_buffer::{ScreenBuffer, ScreenCell}};
use crossterm::style::Color;

pub struct ChatFrame {
    left: Rect,
    right: Rect,
    top: Rect,
    bottom: Rect,
}

impl ChatFrame {
    pub fn new(parent_rect: &Rect) -> Self {
        Self {
            left: Rect::new(parent_rect.x.into(), parent_rect.y as usize + 2, 1, parent_rect.h as usize - 6),
            right: Rect::new(parent_rect.w as usize - 1, parent_rect.y as usize + 2, 1, parent_rect.h as usize - 6),
            top: Rect::new(parent_rect.x.into(), parent_rect.y as usize + 1, parent_rect.w.into(), 1),
            bottom: Rect::new(parent_rect.x.into(), parent_rect.h as usize - 3, parent_rect.w.into(), 1),
        }
    }

    pub fn render(&self, buf: &mut ScreenBuffer) {
        Self::_render_col(&self.left, buf);
        Self::_render_col(&self.right, buf);
        Self::_render_row(&self.bottom, buf);
        Self::_render_row(&self.top, buf);
    }

    fn _render_col(rect: &Rect, buf: &mut ScreenBuffer) {
        buf.fill_col(
            ScreenCell::new(' ', Color::DarkGrey, Color::Reset, false),
            rect.x as usize,
            rect.y as usize,
            rect.y as usize + rect.h as usize
        );
    }

    fn _render_row(rect: &Rect, buf: &mut ScreenBuffer) {
        buf.fill(
            ScreenCell::new(' ', Color::DarkGrey, Color::Reset, false),
            rect.x as usize,
            rect.y as usize,
            rect.w as usize,
        );
    }
}