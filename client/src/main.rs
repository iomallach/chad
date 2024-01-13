extern crate shared;
use parser::ParseResult;
use screen::BarComponent;
use screen::chat_frame::ChatFrame;
use screen::hint::Hint;
use std::io::{Write as _, self};
use std::{net::SocketAddr, error::Error, io::stdout};
use std::result::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, poll, Event, KeyCode, KeyModifiers};
use crossterm::{execute, QueueableCommand};
use crossterm::terminal;
use crate::client::{Client, ClientInput};
use crate::parser::{CommandParser, Command};

mod client;
mod chat;
mod parser;
mod screen;


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
    let mut screen_buf = screen::screen_buffer::ScreenBuffer::default(client.window.w, client.window.h);
    let mut status_bar = screen::BarBox::new(
        client.window.subrect(0, client.window.h - 2, 0, client.window.h - 1),
        vec![
            screen::BarComponent::status("Offline".to_owned(), screen::Rect::null()),
            screen::BarComponent::connected_clients(format!("{}", 0), screen::Rect::null()),
            screen::BarComponent::login("Not logged in".to_owned(), screen::Rect::null())
        ]
    );
    let mut header_bar = screen::BarBox::new(
        client.window.subrect(0, 0, 0, client.window.h - 1),
        vec![
            screen::BarComponent::header("Chad".to_owned(), screen::Rect::new(0, 0, client.window.w , client.window.h ))
        ]
    );
    let mut hint = Hint::new(
        "Type in /login <name> to join the fun",
        client.window.subrect(2, 2, 4, client.window.h - 1),
    );
    let mut client_input = ClientInput::new(
        client.window.subrect(0, client.window.h - 1, 0, client.window.h - 1),
    );
    let chat_frame = ChatFrame::new(&screen::Rect::new(0, 0, client.window.w , client.window.h ));

    hint.render(&mut screen_buf);
    chat_frame.render(&mut screen_buf);
    status_bar.render(&mut screen_buf);
    header_bar.render(&mut screen_buf);
    screen_buf.render(&mut stdout)?;
    screen_buf.reset_diff();
    stdout.queue(MoveTo(0, client.window.h as u16 - 1))?;
    stdout.flush()?;

    loop {
        if poll(std::time::Duration::from_millis(200))? {
            match read()? {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char(c) if event.modifiers.is_empty() => {
                            client_input.insert(c);
                        }
                        KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::SHIFT) => {
                            client_input.insert_uppercase(c.to_uppercase());
                        }
                        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                            client_input.backspace_forward();
                        }
                        KeyCode::Backspace => {
                            client_input.backspace();
                        }
                        KeyCode::Left => {
                            client_input.left();
                        }
                        KeyCode::Right => {
                            client_input.right();
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
                                                hint.patch("Disconnected");
                                            } else {
                                                hint.patch("Not connected");
                                            }
                                        }
                                        Command::Connect => {
                                            if client.login_name.is_none() {
                                                client_input.clear();
                                                hint.patch("You are not logged in, login via /login");
                                                continue;
                                            }
                                            let socket_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                                            match client.connect(socket_addr) {
                                                Ok(_) => {
                                                    hint.patch("Connected");
                                                },
                                                Err(e) => {
                                                    hint.patch(&format!("{}", e));
                                                }
                                            }
                                            client_input.clear();
                                        },
                                        Command::Login(l) => {
                                            client.login(&l);
                                            client_input.clear();
                                            hint.patch(&format!("Logged in as {}, now connect via /connect", l));
                                        }
                                    }
                                },
                                ParseResult::InvalidCommand => {
                                    hint.patch("No such command, try /login or /connect");
                                },
                                ParseResult::NoArgument => {
                                    hint.patch("No argument provided for /login");
                                },
                                ParseResult::NotACommand => {
                                    // TODO: move enirely into send_message
                                    if let Some(_) = client.stream {
                                        client.send_message(&source)?;
                                    } else {
                                        hint.patch("You are offline, connect with /connect");
                                    }
                                },
                                ParseResult::UnexpectedToken => {
                                    hint.patch("No such command, try /login or /connect");
                                },
                                ParseResult::EmptyInput => {},
                            }
                            client_input.clear();
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if client.stream.is_none() {
            status_bar.patch(
                vec![
                    BarComponent::status("Offline".to_owned(), screen::Rect::null()),
                    BarComponent::connected_clients(format!("{}", 0), screen::Rect::null())
                ]
            ).render(&mut screen_buf);
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
                            BarComponent::status("Online".to_owned(), screen::Rect::null()),
                            BarComponent::connected_clients(format!("{}", n_conn), screen::Rect::null()),
                            BarComponent::login(client.login_name.clone().unwrap(), screen::Rect::null())
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
        hint.render(&mut screen_buf);
        client.chat_log.render(&mut screen_buf);
        client_input.render(&mut screen_buf);
        // screen_buf.render(&mut stdout)?;
        screen_buf.render_diff(&mut stdout)?;
        screen_buf.reset_diff();
        stdout.queue(MoveTo(client_input.position() as u16, client.window.h as u16 - 1))?;
        stdout.flush()?;
    }

    Ok(())
}
