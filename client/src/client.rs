use core::panic;
use std::error::Error;
use std::result::Result;
use std::io::{stdout, Write, self, Read, Stdout};
use std::net::{SocketAddr, TcpStream};
use std::usize;

use crossterm::terminal::Clear;
use crossterm::{QueueableCommand, cursor, cursor::MoveTo, style::ResetColor, style::Print, style::PrintStyledContent, style::SetBackgroundColor, style::SetForegroundColor, style::Color, terminal};
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::{execute, ExecutableCommand};

struct Grid {
    grid_items: Vec<GridItem>,
    w: usize,
    h: usize,
}

impl Grid {
    fn new(w: usize, h: usize) -> Self {
        Self {
            grid_items: vec![GridItem::new(); w*h],
            w,
            h,
        }
    }

    fn put_item(&mut self, x: usize, y: usize, c: char) {
        if let Some(e) = self.grid_items.get_mut(self.w * y + x) {
            *e = GridItem::from_char(c)
        }
    }

    fn render(&self, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
        stdout.queue(MoveTo(0, 0))?;
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        for gi in &self.grid_items {
            stdout.queue(Print(gi.ch))?;
        }
        stdout.flush()?;
        Ok(())
    }
}

#[derive(Clone)]
struct GridItem {
    ch: char
}

impl GridItem {
    fn new() -> Self {
        Self {
            ch: ' ',
        }
    }

    fn from_char(c: char) -> Self {
        Self {
            ch: c
        }
    }
}

fn fetch_updates(stdout: &mut Stdout, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf: [u8; 8192] = [0; 8 * 1024];
    match stream.read(&mut buf) {
        Ok(0) => {Ok(())},
        Ok(n) => Ok(execute!(stdout, Print(String::from_utf8_lossy(&buf[..n])))?),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {Ok(())},
        e => panic!("Error reading from socket {:?}", e)
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    let mut client = Client::new();
    let mut client_input = ClientInput::new();
    header(&mut stdout, &client.window, "Chad")?;
    stdout.flush()?;
    hint(&mut stdout, &client.window, "Type in /connect to join the fun")?;
    status_bar(&mut stdout, &client.window, "Offline", 0)?;

    // execute!(stdout, Print(format!("Connected to {}\n", socket_addr)))?;
    // if let Some(stream) = &mut client.stream {
    //     fetch_updates(&mut stdout, stream)?;
    // } else {
    //     client.connect(socket_addr)?;
    // }

    loop {
        match read()? {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Char(c) if event.modifiers.is_empty() => {
                        client_input.push(c);
                        stdout.execute(Print(c))?;
                    }
                    KeyCode::Backspace => {
                        client_input.backspace();
                        stdout.queue(cursor::MoveLeft(1))?;
                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                        stdout.flush()?;
                    }
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        execute!(stdout, terminal::LeaveAlternateScreen)?;
                        terminal::disable_raw_mode()?;
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        match client_input.buffer.get(0) {
                            Some('/') => {
                                if client_input.buffer.iter().cloned().skip(1).collect::<String>() == "connect".to_string() {
                                    let inp = client_input.buffer.iter().cloned().collect::<String>();
                                    match client.connect(parse_connect_command(&inp)?) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            stdout.queue(cursor::MoveLeft(client_input.buffer.len() as u16))?;
                                            stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                            client_input.buffer.clear();
                                            hint(&mut stdout, &client.window, &e.to_string()).unwrap();
                                            continue;
                                        }
                                    }
                                    stdout.queue(cursor::MoveLeft(client_input.buffer.len() as u16))?;
                                    stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                    client_input.buffer.clear();
                                    status_bar(&mut stdout, &client.window, "Online", 1)?;
                                    hint(&mut stdout, &client.window, "")?;
                                    // stdout.flush()?;
                                }
                            },
                            Some(_) => {
                                stdout.queue(cursor::MoveLeft(client_input.buffer.len() as u16))?;
                                stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                stdout.flush()?;
                                if let Some(stream) = &mut client.stream {
                                    write!(stream, "{}", client_input.buffer.iter().collect::<String>())?
                                }
                                client_input.buffer.clear();
                            }
                            None => {}
                        }
                    },
                    _ => {}
                }
            }
            // Event::Key(event) => {
            //     execute!(stdout, Print(format!("{:?}\n", event)))?;
            //     fetch_updates(&mut stdout, &mut stream)?;
            // },
            _ => {}
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}