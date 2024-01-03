use crossterm::style;

pub mod screen_buffer;

#[derive(Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            x: x as u16,
            y: y as u16,
            w: w as u16,
            h: h as u16,
        }
    }
}

#[derive(Clone)]
pub enum BarComponentKind {
    Status,
    Login,
    ConnectedClients,
    Header,
}

impl PartialEq for BarComponentKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BarComponentKind::ConnectedClients, BarComponentKind::ConnectedClients) => true,
            (BarComponentKind::Login, BarComponentKind::Login) => true,
            (BarComponentKind::Status, BarComponentKind::Status) => true,
            (BarComponentKind::Header, BarComponentKind::Header) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct BarComponent {
    kind: BarComponentKind,
    rect: Rect,
    display: Vec<screen_buffer::ScreenCell>,
    value: Vec<screen_buffer::ScreenCell>,
    delim: char,
}

impl BarComponent {
    pub fn new(kind: BarComponentKind, parent_rect: Rect, display: String, value: String, delim: char) -> Self {
        Self {
            kind,
            rect: parent_rect,
            display: display.chars().map(|c| {
                screen_buffer::ScreenCell::bar_cell(c, style::Color::Black)
            }).collect(),
            value: value.chars().map(|c| {
                screen_buffer::ScreenCell::bar_cell(c, style::Color::Green)
            }).collect(),
            delim,
        }
    }

    pub fn status(value: String, rect: Rect) -> Self {
        Self::new(
            BarComponentKind::Status,
            rect,
            "Status".to_owned(),
            value,
            ':',
        )
    }

    pub fn connected_clients(value: String, rect: Rect) -> Self {
        Self::new(
            BarComponentKind::ConnectedClients,
            rect,
            "Connected clients".to_owned(),
            value,
            ':',
        )
    }

    pub fn login(value: String, rect: Rect) -> Self {
        Self::new(
            BarComponentKind::Login,
            rect,
            "Logged in as".to_owned(),
            value,
            ':',
        )
    }

    pub fn header(value: String, rect: Rect) -> Self {
        Self::new(
            BarComponentKind::Header,
            rect,
            "".to_owned(),
            value,
            ' ',
        )
    }

    pub fn render(&self, buf: &mut screen_buffer::ScreenBuffer) {
        let mut col_cursor: usize = self.rect.x.into();
        let content_length = self.display.len() + self.value.len() + 2;
        let empty_space = self.rect.w as usize - content_length + 1;
        buf.put_cells(vec![screen_buffer::ScreenCell::bar_empty_space(); empty_space/2], col_cursor.into(), self.rect.y.into());
        col_cursor += empty_space / 2;
        buf.put_cells(self.display.clone(), col_cursor, self.rect.y.into());
        col_cursor += self.display.len();
        buf.put_cell(screen_buffer::ScreenCell::bar_cell(self.delim, style::Color::White), col_cursor, self.rect.y.into());
        col_cursor += 1;
        buf.put_cell(screen_buffer::ScreenCell::bar_empty_space(), col_cursor, self.rect.y.into());
        col_cursor += 1;
        buf.put_cells(self.value.clone(), col_cursor, self.rect.y.into());
        col_cursor += self.value.len();
        buf.put_cells(vec![screen_buffer::ScreenCell::bar_empty_space(); empty_space/2 - 1], col_cursor.into(), self.rect.y.into())
    }
}

pub struct BarBox{
    rect: Rect,
    components: Vec<BarComponent>,
}

impl BarBox {
    pub fn new(rect: Rect, mut components: Vec<BarComponent>) -> Self {
        let components_len = components.len();
        for i in 0..components_len {
            if let Some(comp) = components.get_mut(i) {
                let component_width = rect.w / components_len as u16;
                let cursor_x_pos = rect.x + component_width * i as u16;
                comp.rect = Rect::new(cursor_x_pos.into(), rect.y.into(), component_width.into(), rect.h.into());
            }
        }
        Self {
            rect,
            components,
        }
    }

    pub fn render(&mut self, buf: &mut screen_buffer::ScreenBuffer) {
        for c in &self.components {
            c.render(buf);
        }
    }

    // TODO: figure out what the fuck is going on here with the clones
    pub fn patch(&mut self, mut patches: Vec<BarComponent>) -> &mut Self {
        for p in patches.iter_mut() {
            for c in self.components.iter_mut() {
                if p.kind == c.kind {
                    *c = p.clone();
                }
            }
        }
        self
    }

}