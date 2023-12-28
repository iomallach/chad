use std::io::{Write as _, Stdout, Read, self};
use std::net::TcpStream;
use std::{net::SocketAddr, error::Error, io::stdout};
use std::result::Result;
use crossterm::cursor::{MoveTo, MoveDown, MoveLeft};
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::Print;
use crossterm::{execute, ExecutableCommand, QueueableCommand, cursor};
use crossterm::terminal::{self, Clear};
use crate::clientrs::{Client, ClientInput};
use crate::draw::{header, status_bar, hint};
use crate::parser::{CommandParser, ParseError, Command};

mod clientrs;
mod chat;
mod draw;
mod parser;
mod message;

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
    hint(&mut stdout, &client.window, "Type in /login <name> to join the fun")?;
    status_bar(&mut stdout, &client.window, "Offline", 0)?;

    loop {
        match read()? {
            Event::Key(event) => {
                if let Some(stream) = &mut client.stream {
                    let mut buf: [u8; 4096] = [0; 4 * 1024];
                    match stream.read(&mut buf) {
                        Ok(0) => {},
                        Ok(n) => {
                            hint(&mut stdout, &client.window, &format!("Got {} bytes", n))?;
                            let mut buf_iter = buf.iter();
                            let name_length = buf_iter.next().unwrap();
                            let delim: Vec<u8> = vec![32, 58, 32];
                            let msg_sender = buf_iter.clone().cloned().take(*name_length as usize).chain(delim);
                            let msg_itself = buf_iter.cloned().skip(*name_length as usize).take_while(|e| *e != 0);
                            let msg: Vec<u8> = msg_sender.chain(msg_itself).collect();
                            client.chat_log.put_line(std::str::from_utf8(&msg).unwrap().to_string());
                        },
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {},
                        e => { panic!("Unexpected error {:?}", e)},
                    }
                }
                if !client.chat_log.is_empty() {
                    stdout.queue(cursor::SavePosition)?;
                    stdout.queue(cursor::Hide)?;
                    stdout.queue(MoveTo(0, 2))?;
                    for m in client.chat_log.get() {
                        stdout.queue(Print(m))?;
                        stdout.queue(MoveDown(1))?;
                        stdout.queue(MoveLeft(client.window.width as u16))?;
                    }
                    stdout.queue(cursor::RestorePosition)?;
                    stdout.queue(cursor::Show)?;
                    stdout.flush()?;
                }
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
                        break;
                    }
                    KeyCode::Enter => {
                        let source: String = client_input.buffer.drain(..).collect();
                        let maybe_command = CommandParser::new(&source).next_command();
                        match maybe_command {
                            Ok(cmd) => {
                                match cmd {
                                    Command::Connect => {
                                        if client.login_name.is_none() {
                                            stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                            stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                            hint(&mut stdout, &client.window, "You are not logged in, login via /login")?;
                                            continue;
                                        }
                                        let socket_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                                        match client.connect(socket_addr) {
                                            Ok(_) => {
                                                stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                                stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                                status_bar(&mut stdout, &client.window, "Online", 1)?;
                                                hint(&mut stdout, &client.window, "Connected")?;
                                                continue;
                                            },
                                            Err(e) => {
                                                stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                                stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                                hint(&mut stdout, &client.window, &e.to_string())?;
                                                continue;
                                            }
                                        }
                                    },
                                    Command::Login(l) => {
                                        client.login(&l);
                                        stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                        hint(&mut stdout, &client.window, &format!("Logged in as {}, now connect via /connect", l))?;
                                    }
                                }
                            },
                            Err(e) => {
                                match e {
                                    ParseError::InvalidCommand => {
                                        stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                        stdout.flush()?;
                                        hint(&mut stdout, &client.window, "No such command, try /login or /connect")?;
                                    },
                                    ParseError::NoArgument => {
                                        stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                        stdout.flush()?;
                                        hint(&mut stdout, &client.window, "No argument provided for /login")?;
                                    },
                                    ParseError::NotACommand => {
                                        stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                        stdout.flush()?;
                                        if let Some(stream) = &mut client.stream {
                                            write!(stream, "{}", source)?
                                        } else {
                                            hint(&mut stdout, &client.window, "You are offline, connect with /connect")?;
                                        }
                                    },
                                    ParseError::UnexpectedToken => {
                                        stdout.queue(cursor::MoveLeft(source.len() as u16))?;
                                        stdout.queue(Clear(terminal::ClearType::UntilNewLine))?;
                                        stdout.flush()?;
                                        hint(&mut stdout, &client.window, "No such command, try /login or /connect")?;
                                    },
                                }
                            }
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
