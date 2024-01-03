use std::io::Write as _;
use std::{io::Stdout, error::Error};
use crate::clientrs::Window;
use crossterm::{style, terminal, queue};
use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;


pub struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
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

#[derive(Clone, Copy)]
pub enum StatusBarComponentKind {
    Status,
    Login,
    ConnectedClients,
}

impl PartialEq for StatusBarComponentKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StatusBarComponentKind::ConnectedClients, StatusBarComponentKind::ConnectedClients) => true,
            (StatusBarComponentKind::Login, StatusBarComponentKind::Login) => true,
            (StatusBarComponentKind::Status, StatusBarComponentKind::Status) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct StatusBarComponent {
    kind: StatusBarComponentKind,
    display: String,
    value: String,
    display_fg_color: style::Color,
    value_fg_color: style::Color,
}

impl StatusBarComponent {
    pub fn new(kind: StatusBarComponentKind, display: String, value: String, display_fg_color: style::Color, value_fg_color: style::Color) -> Self {
        Self {
            kind,
            display,
            value,
            display_fg_color,
            value_fg_color,
        }
    }

    pub fn status(value: String) -> Self {
        Self::new(
            StatusBarComponentKind::Status,
            "Status".to_owned(),
            value,
            style::Color::Black,
            style::Color::Red
        )
    }

    pub fn connected_clients(value: String) -> Self {
        Self::new(
            StatusBarComponentKind::ConnectedClients,
            "Connected clients".to_owned(),
            value,
            style::Color::Black,
            style::Color::Green
        )
    }

    pub fn login(value: String) -> Self {
        Self::new(
            StatusBarComponentKind::Login,
            "Logged in as".to_owned(),
            value,
            style::Color::Black,
            style::Color::Black
        )
    }

    pub fn render(&self, stdout: &mut Stdout, rect: &Rect) -> Result<(), Box<dyn Error>> {
        let content_length = self.display.len() + self.value.len() + 2;
        let empty_space = rect.w as usize - content_length + 1;
        queue!(
            stdout,
            style::SetBackgroundColor(style::Color::White),
            MoveTo(rect.x, rect.y),
            Print(format!("{:indent$}", "", indent=empty_space/2)),
            style::SetForegroundColor(self.display_fg_color),
            Print(&self.display),
            Print(format!(":{:indent$}", "", indent=1)),
            style::SetForegroundColor(self.value_fg_color),
            Print(&self.value),
            Print(format!("{:indent$}", "", indent=empty_space/2))
        )?;
        Ok(())
    }
}

pub struct StatusBarBox{
    rect: Rect,
    components: Vec<StatusBarComponent>,
    component_rects: Vec<Rect>
}

impl StatusBarBox {
    pub fn new(rect: Rect, components: Vec<StatusBarComponent>) -> Self {
        let rects = (0..components.len()).map(|i| {
            let component_width = rect.w / components.len() as u16;
            let cursor_x_pos = component_width * i as u16;
            Rect::new(cursor_x_pos as usize, rect.y as usize, component_width as usize, 1)
        }).collect();
        Self {
            rect,
            components,
            component_rects: rects,
        }
    }

    pub fn render(&mut self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        queue!(
            stdout,
            MoveTo(self.rect.x, self.rect.y),
            style::SetBackgroundColor(style::Color::White)
        )?;
        for (i, c) in self.components.iter().enumerate() {
            let this_rect = self.component_rects.get(i).unwrap();
            c.render(stdout, this_rect)?;
        }
        queue!(
            stdout,
            style::ResetColor,
            MoveTo(0, self.rect.y + 2)
        )?;
        stdout.flush()?;
        Ok(())
    }

    // TODO: figure out what the fuck is going on here with the clones
    pub fn patch(&mut self, mut patches: Vec<StatusBarComponent>) -> &mut Self {
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


pub fn status_bar(stdout: &mut Stdout, window: &Window, status: &str, clients_connected: usize) -> Result<(), Box<dyn Error>> {
    let color = match status {
        "Offline" => style::Color::Red,
        "Online" => style::Color::Green,
        _ => panic!("Unrecognized status"),
    };
    stdout.queue(MoveTo(0, window.height as u16 - 2))?;
    stdout.queue(style::SetBackgroundColor(style::Color::White))?;
    stdout.queue(style::SetForegroundColor(color))?;
    stdout.queue(Print(format!("Status: {}", status)))?;
    stdout.queue(style::SetForegroundColor(style::Color::Black))?;
    stdout.queue(Print("  |  "))?;
    stdout.queue(Print(format!("Clients connected: {}", clients_connected)))?;
    for _ in status.len() + 8 + 5 + 20..window.width {
        stdout.queue(Print(" "))?;
    }
    stdout.queue(style::ResetColor)?;
    stdout.queue(MoveTo(0, window.height as u16))?;
    stdout.flush()?;
    Ok(())
}

pub fn header(stdout: &mut Stdout, window: &Window, label: &str) -> Result<(), Box<dyn Error>> {
    let center = window.width/2 - label.len()/2;
    stdout.queue(style::SetBackgroundColor(style::Color::White))?;
    stdout.queue(style::SetForegroundColor(style::Color::Red))?;
    for _ in 0..center as usize {
        stdout.queue(Print(" "))?;
    }
    stdout.queue(MoveTo(center as u16, 0))?;
    stdout.queue(Print(label))?;
    for _ in center + label.len()..window.width as usize {
        stdout.queue(Print(" "))?;
    }
    stdout.queue(style::ResetColor)?;
    Ok(())
}

pub fn hint(stdout: &mut Stdout, window: &Window, hint: &str) -> Result<(), Box<dyn Error>> {
    stdout.queue(MoveTo(0, 1))?;
    stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
    stdout.queue(style::SetForegroundColor(style::Color::Magenta))?;
    stdout.queue(Print(hint))?;
    stdout.queue(style::ResetColor)?;
    stdout.queue(MoveTo(0, window.height as u16))?;
    stdout.flush()?;
    Ok(())
}
