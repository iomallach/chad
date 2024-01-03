extern crate shared;
use crossterm::queue;
use draw::{StatusBarBox, StatusBarComponent};
use parser::ParseResult;
use screen::BarComponent;
use shared::read_message;
use std::io::Stdout;
use std::io::{Write as _, self};
use std::{net::SocketAddr, error::Error, io::stdout};
use std::result::Result;
use crossterm::cursor::{MoveTo, MoveDown, MoveLeft};
use crossterm::event::{read, poll, Event, KeyCode, KeyModifiers};
use crossterm::style::Print;
use crossterm::{execute, ExecutableCommand, QueueableCommand, cursor};
use crossterm::terminal::{self, Clear};
use crate::clientrs::{Client, ClientInput};
use crate::draw::{header, hint, Rect};
use crate::parser::{CommandParser, Command};

mod clientrs;
mod chat;
mod draw;
mod parser;
mod screen;


fn fetch_message(client: &mut Client, status_bar: &mut StatusBarBox) {
    if let Some(stream) = &mut client.stream {
        match read_message(stream) {
            Ok(m) => {
                if m.message.is_some() {
                    let chat_message = chat::ChatMessage::default(m.message.unwrap(), m.timestamp, m.username);
                    client.chat_log.put_line(chat_message);
                }
                // TODO: this likely doesn't belong here
                if let Some(n_conn) = m.connections {
                    status_bar.patch(
                        vec![
                            StatusBarComponent::status("Online".to_owned()),
                            StatusBarComponent::connected_clients(format!("{}", n_conn)),
                            StatusBarComponent::login(client.login_name.clone().unwrap())
                        ]
                    ).render(&mut stdout()).expect("Failed rendering status bar");
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
        stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        for m in client.chat_log.get() {
            stdout.queue(Print(m))?;
            stdout.queue(MoveDown(1))?;
            stdout.queue(MoveLeft(client.window.width as u16))?;
            stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
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

struct TerminalState;

impl TerminalState {
    fn enter() -> Result<Self, Box<dyn Error>> {
        execute!(stdout(), terminal::EnterAlternateScreen, terminal::Clear(terminal::ClearType::All))?;
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        let _ = execute!(stdout(), terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    let _terminal_state = TerminalState::enter()?;
    let mut client = Client::new();
    let mut screen_buf = screen::screen_buffer::ScreenBuffer::default(client.window.width, client.window.height);
    let mut status_bar = screen::BarBox::new(
        screen::Rect::new(0, client.window.height - 2, client.window.width, client.window.height),
        vec![
            screen::BarComponent::status("Offline".to_owned(), screen::Rect::new(0, 0, client.window.width, client.window.height)),
            screen::BarComponent::connected_clients(format!("{}", 0), screen::Rect::new(0, 0, client.window.width, client.window.height)),
            screen::BarComponent::login("Not logged in".to_owned(), screen::Rect::new(0, 0, client.window.width, client.window.height))
        ]
    );
    let mut header_bar = screen::BarBox::new(
        screen::Rect::new(0, 0, client.window.width, client.window.height),
        vec![
            screen::BarComponent::header("Chad".to_owned(), screen::Rect::new(0, 0, client.window.width, client.window.height))
        ]
    );
    let mut client_input = ClientInput::new(screen::Rect::new(0, client.window.height - 1, client.window.width, 1));
    // let mut stat_bar = StatusBarBox::new(
    //     Rect::new(0, client.window.height - 2, client.window.width, 1),
    //     vec![
    //         StatusBarComponent::status("Offline".to_owned()),
    //         StatusBarComponent::connected_clients(format!("{}", 0)),
    //         StatusBarComponent::login("Not logged in".to_owned()),
    //     ]
    // );
    // header(&mut stdout, &client.window, "Chad")?;
    // stdout.flush()?;
    // hint(&mut stdout, &client.window, "Type in /login <name> to join the fun")?;
    // stat_bar.render(&mut stdout)?;
    status_bar.render(&mut screen_buf);
    header_bar.render(&mut screen_buf);
    screen_buf.render(&mut stdout)?;
    stdout.queue(MoveTo(0, client.window.height as u16 - 1))?;
    stdout.flush()?;

    loop {
        if poll(std::time::Duration::from_millis(400))? {
            match read()? {
                Event::Key(event) => {
                    // fetch_message(&mut client, &mut stat_bar);
                    // render_chat(&client, &mut stdout)?;
                    match event.code {
                        KeyCode::Char(c) if event.modifiers.is_empty() => {
                            client_input.push(c);
                            // stdout.execute(Print(c))?;
                        }
                        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::SHIFT) => {
                            client_input.push_uppercase(c.to_uppercase());
                            // stdout.execute(Print(c.to_uppercase()))?;
                        }
                        KeyCode::Backspace => {
                            client_input.backspace();
                            // clear_prompt(&mut stdout, 1)?;
                        }
                        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Enter => {
                            let source: String = client_input.inner.iter().collect();
                            let maybe_command = CommandParser::new(&source).next_command();
                            match maybe_command {
                                ParseResult::Command(cmd) => {
                                    match cmd {
                                        Command::Disconnect => {
                                            if client.stream.is_some() {
                                                client.disconnect()?;
                                                hint(&mut stdout, &client.window, "Disconnected")?;
                                            } else {
                                                hint(&mut stdout, &client.window, "Not connected")?;
                                            }
                                            // clear_prompt(&mut stdout, source.len() as u16)?;
                                        }
                                        Command::Connect => {
                                            if client.login_name.is_none() {
                                                // clear_prompt(&mut stdout, source.len() as u16)?;
                                                client_input.clear();
                                                hint(&mut stdout, &client.window, "You are not logged in, login via /login")?;
                                            }
                                            let socket_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                                            match client.connect(socket_addr) {
                                                Ok(_) => {
                                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                                    // stat_bar.patch(
                                                    //     vec![
                                                    //         StatusBarComponent::status("Online".to_owned()),
                                                    //         StatusBarComponent::connected_clients(format!("{}", 1)),
                                                    //         StatusBarComponent::login(client.login_name.clone().unwrap()),
                                                    //     ]
                                                    // ).render(&mut stdout)?;
                                                    hint(&mut stdout, &client.window, "Connected")?;
                                                },
                                                Err(e) => {
                                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                                    hint(&mut stdout, &client.window, &e.to_string())?;
                                                }
                                            }
                                            client_input.clear();
                                        },
                                        Command::Login(l) => {
                                            client.login(&l);
                                            client_input.clear();
                                            // clear_prompt(&mut stdout, source.len() as u16)?;
                                            hint(&mut stdout, &client.window, &format!("Logged in as {}, now connect via /connect", l))?;
                                        }
                                    }
                                },
                                ParseResult::InvalidCommand => {
                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                    hint(&mut stdout, &client.window, "No such command, try /login or /connect")?;
                                },
                                ParseResult::NoArgument => {
                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                    hint(&mut stdout, &client.window, "No argument provided for /login")?;
                                },
                                ParseResult::NotACommand => {
                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                    // TODO: move enirely into send_message
                                    if let Some(_) = client.stream {
                                        client.send_message(&source)?;
                                    } else {
                                        hint(&mut stdout, &client.window, "You are offline, connect with /connect")?;
                                    }
                                },
                                ParseResult::UnexpectedToken => {
                                    // clear_prompt(&mut stdout, source.len() as u16)?;
                                    hint(&mut stdout, &client.window, "No such command, try /login or /connect")?;
                                },
                                ParseResult::EmptyInput => {},
                            }
                            client_input.clear();
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
            match client.fetch_message() {
                Ok(m) => {
                    if m.message.is_some() {
                        let chat_message = chat::ChatMessage::default(m.message.unwrap(), m.timestamp, m.username);
                        client.chat_log.put_line(chat_message);
                    }
                    if let Some(n_conn) = m.connections {
                        status_bar.patch(
                            vec![
                                BarComponent::status("Online".to_owned(), screen::Rect::new(0, 0, client.window.width, client.window.height)),
                                BarComponent::connected_clients(format!("{}", n_conn), screen::Rect::new(0, 0, client.window.width, client.window.height)),
                                BarComponent::login(client.login_name.clone().unwrap(), screen::Rect::new(0, 0, client.window.width, client.window.height))
                            ]
                        ).render(&mut screen_buf);
                    }
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock | io::ErrorKind::BrokenPipe => {},
                        _ => panic!("Something went really wrong {:?}", e),
                    }
                }
            }
            client.chat_log.render(&mut screen_buf);
            client_input.render(&mut screen_buf);
            screen_buf.render(&mut stdout)?;
            stdout.queue(MoveTo(client_input.inner.len() as u16, client.window.height as u16 - 1))?;
            stdout.flush()?;

        } else {
            match client.fetch_message() {
                Ok(m) => {
                    if m.message.is_some() {
                        let chat_message = chat::ChatMessage::default(m.message.unwrap(), m.timestamp, m.username);
                        client.chat_log.put_line(chat_message);
                    }
                    if let Some(n_conn) = m.connections {
                        status_bar.patch(
                            vec![
                                BarComponent::status("Online".to_owned(), screen::Rect::new(0, 0, client.window.width, client.window.height)),
                                BarComponent::connected_clients(format!("{}", n_conn), screen::Rect::new(0, 0, client.window.width, client.window.height)),
                                BarComponent::login(client.login_name.clone().unwrap(), screen::Rect::new(0, 0, client.window.width, client.window.height))
                            ]
                        ).render(&mut screen_buf);
                    }
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock | io::ErrorKind::BrokenPipe => {},
                        _ => panic!("Something went really wrong {:?}", e),
                    }
                }
            }
            client.chat_log.render(&mut screen_buf);
            client_input.render(&mut screen_buf);
            screen_buf.render(&mut stdout)?;
            stdout.queue(MoveTo(client_input.inner.len() as u16, client.window.height as u16 - 1))?;
            stdout.flush()?;
            // fetch_message(&mut client, &mut stat_bar);
            // render_chat(&client, &mut stdout)?;
        }
    }

    Ok(())
}
