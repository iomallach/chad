extern crate shared;
use crossterm::queue;
use shared::read_message;
use shared::Message;
use std::io::Stdout;
use std::io::{Write as _, self};
use std::str::FromStr;
use std::{net::SocketAddr, error::Error, io::stdout};
use std::result::Result;
use crossterm::cursor::{MoveTo, MoveDown, MoveLeft};
use crossterm::event::{read, poll, Event, KeyCode, KeyModifiers};
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

fn fetch_message(client: &mut Client) {
    if let Some(stream) = &mut client.stream {
        match read_message(stream) {
            Ok(m) => {
                let msg = Message::from_str(&m).expect("No fucking errors");
                if msg.has_message {
                    let chat_message = chat::ChatMessage::default(msg.message.unwrap(), msg.timestamp, msg.username);
                    client.chat_log.put_line(chat_message);
                }
            },
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => {},
                    io::ErrorKind::BrokenPipe => return,
                    _ => panic!("Something went really wrong {:?}", e),
                }
            }
        }
    }
}

fn render_chat(client: &Client, stdout: &mut Stdout) -> Result<(), Box<dyn Error>> {
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
    Ok(())
}

fn clear_prompt(stdout: &mut Stdout, move_left: u16) -> Result<(), Box<dyn Error>> {
    queue!(
        stdout,
        cursor::MoveLeft(move_left),
        Clear(terminal::ClearType::UntilNewLine),
    )?;
    Ok(stdout.flush()?)
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
        if poll(std::time::Duration::from_millis(50))? {
            match read()? {
                Event::Key(event) => {
                    fetch_message(&mut client);
                    render_chat(&client, &mut stdout)?;
                    match event.code {
                        KeyCode::Char(c) if event.modifiers.is_empty() => {
                            client_input.push(c);
                            stdout.execute(Print(c))?;
                        }
                        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::SHIFT) => {
                            client_input.push_uppercase(c.to_uppercase());
                            stdout.execute(Print(c.to_uppercase()))?;
                        }
                        KeyCode::Backspace => {
                            client_input.backspace();
                            clear_prompt(&mut stdout, 1)?;
                        }
                        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Enter => {
                            let source: String = client_input.inner.drain(..).collect();
                            if source.is_empty() {
                                continue
                            }
                            let maybe_command = CommandParser::new(&source).next_command();
                            match maybe_command {
                                Ok(cmd) => {
                                    match cmd {
                                        Command::Disconnect => {
                                            if client.stream.is_some() {
                                                client.disconnect()?;
                                                hint(&mut stdout, &client.window, "Disconnected")?;
                                            } else {
                                                hint(&mut stdout, &client.window, "Not connected")?;
                                            }
                                            clear_prompt(&mut stdout, source.len() as u16)?;
                                        }
                                        Command::Connect => {
                                            if client.login_name.is_none() {
                                                clear_prompt(&mut stdout, source.len() as u16)?;
                                                hint(&mut stdout, &client.window, "You are not logged in, login via /login")?;
                                                continue;
                                            }
                                            let socket_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                                            match client.connect(socket_addr) {
                                                Ok(_) => {
                                                    clear_prompt(&mut stdout, source.len() as u16)?;
                                                    status_bar(&mut stdout, &client.window, "Online", 1)?;
                                                    hint(&mut stdout, &client.window, "Connected")?;
                                                    continue;
                                                },
                                                Err(e) => {
                                                    clear_prompt(&mut stdout, source.len() as u16)?;
                                                    hint(&mut stdout, &client.window, &e.to_string())?;
                                                    continue;
                                                }
                                            }
                                        },
                                        Command::Login(l) => {
                                            client.login(&l);
                                            clear_prompt(&mut stdout, source.len() as u16)?;
                                            hint(&mut stdout, &client.window, &format!("Logged in as {}, now connect via /connect", l))?;
                                        }
                                    }
                                },
                                Err(e) => {
                                    match e {
                                        ParseError::InvalidCommand => {
                                            clear_prompt(&mut stdout, source.len() as u16)?;
                                            hint(&mut stdout, &client.window, "No such command, try /login or /connect")?;
                                        },
                                        ParseError::NoArgument => {
                                            clear_prompt(&mut stdout, source.len() as u16)?;
                                            hint(&mut stdout, &client.window, "No argument provided for /login")?;
                                        },
                                        ParseError::NotACommand => {
                                            clear_prompt(&mut stdout, source.len() as u16)?;
                                            // TODO: move enirely into send_message
                                            if let Some(_) = client.stream {
                                                client.send_message(&source)?;
                                            } else {
                                                hint(&mut stdout, &client.window, "You are offline, connect with /connect")?;
                                            }
                                        },
                                        ParseError::UnexpectedToken => {
                                            clear_prompt(&mut stdout, source.len() as u16)?;
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
        } else {
            fetch_message(&mut client);
            render_chat(&client, &mut stdout)?;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
