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

pub fn hint(stdout: &mut Stdout, window: &Window, hint: &str) -> Result<(), Box<dyn Error>> {
    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
    stdout.queue(style::SetForegroundColor(style::Color::Magenta))?;
    stdout.queue(Print(hint))?;
    stdout.queue(style::ResetColor)?;
    stdout.queue(MoveTo(0, window.height as u16))?;
    stdout.flush()?;
    Ok(())
}
