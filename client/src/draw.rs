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
    // TODO: "-2" offset shall be accounted for here
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            x: x as u16,
            y: y as u16,
            w: w as u16,
            h: h as u16,
        }
    }
}

pub struct StatusBar{
    rect: Rect,
    delimiter: char,
    status_hint: String,
    clients_hint: String,
}

impl StatusBar {
    pub fn new(rect: Rect, delimiter: char, status_hint: &str, clients_hint: &str) -> Self {
        Self {
            rect,
            delimiter,
            status_hint: status_hint.to_owned(),
            clients_hint: clients_hint.to_owned(),
        }
    }

    pub fn render(&mut self, stdout: &mut Stdout, connection_state: &str, clients_state: usize) -> Result<(), Box<dyn Error>> {
        // -2 accounts for : and a whitespace
        // -1 is the hardcoded width of clients_state, I shall take the real width here instead
        let state_indent = self.rect.w - self.status_hint.len() as u16 - 2 - connection_state.len() as u16 - self.clients_hint.len() as u16 - 2 - 1;

        queue!(
            stdout,
            MoveTo(self.rect.y, self.rect.x),
            style::SetBackgroundColor(style::Color::White),
            style::SetForegroundColor(style::Color::Black),
            Print(&self.status_hint),
            style::SetForegroundColor(Self::status_hint_color(connection_state)),
            Print(format!(":{:indent$}{state}", "", indent=1, state=connection_state)),
            style::SetForegroundColor(style::Color::Black),
            Print(format!("{:indent$}", "", indent=state_indent as usize)),
            Print(&self.clients_hint),
            Print(format!(":{:indent$}{clients}", "", indent=1, clients=clients_state)),
            style::ResetColor,
            MoveTo(self.rect.y, self.rect.h),
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn status_hint_color(status_hint: &str) -> style::Color {
        match status_hint {
            "Offline" => style::Color::Red,
            "Online" => style::Color::Green,
            _ => unreachable!("This should be impossible to reach"),
        }
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
