use std::io::Write as _;
use std::{io::Stdout, error::Error};
use crate::clientrs::Window;
use crossterm::{style, terminal};
use crossterm::QueueableCommand;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;



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
