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
            ScreenCell::new(c, Color::Reset, Color::Magenta)
        }).collect::<Vec<_>>();
        buf.fill_row(ScreenCell::default(), self.rect.y.into(), Some(self.rect.x.into()), Some(self.rect.w.into()));
        buf.put_cells(cells, self.rect.x.into(), self.rect.y.into());
    }
}