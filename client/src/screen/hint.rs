use crossterm::style::Color;

use super::{Rect, screen_buffer::{ScreenBuffer, ScreenCell}};

pub struct Hint {
    rect: Rect,
    value: String,
}

impl Hint {
    pub fn new(value: &str, rect: Rect) -> Self {
        Self {
            rect,
            value: value.to_owned(),
        }
    }

    pub fn patch(&mut self, value: &str) {
        self.value = value.to_owned();
    }

    pub fn render(&self, buf: &mut ScreenBuffer) {
        let cells = self.value.chars().map(|c| {
            ScreenCell::new(c, Color::Reset, Color::Magenta, false)
        }).collect::<Vec<_>>();
        buf.fill_row(ScreenCell::default(), self.rect.y, Some(self.rect.x), Some(self.rect.w));
        buf.put_cells(cells, self.rect.x, self.rect.y);
    }
}