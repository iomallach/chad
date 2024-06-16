extern crate shared;
use crate::client::ClientInput;
use anyhow::Result;
use bytes::Bytes;
use chat::ChatLog;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal;
use futures::{FutureExt, StreamExt};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use shared::connection::Connection;
use shared::message::Message;
use shared::message::{ChatMessage, Login};
use std::io::stdout;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

mod chat;
mod client;
mod tui;

struct TerminalState;

impl TerminalState {
    fn enter() -> Result<Self> {
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All)
        )?;
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

#[tokio::main]
async fn main() -> Result<()> {
    let _terminal_state = TerminalState::enter()?;

    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(Terminal::new(CrosstermBackend::new(stdout()))?, action_tx);

    let mut connection: Option<Connection<OwnedWriteHalf, OwnedReadHalf>> = None;

    let mut chat_log = ChatLog::new(50);
    let mut login_name: Option<String> = None;
    let mut chatters: Vec<String> = Vec::new();

    loop {
        tui::draw(
            &mut app.terminal,
            &app.current_screen,
            &mut app.input_buffer,
            &chat_log,
            &chatters,
        )?;
        let event = app.event_reader.next().fuse();
        if let Some(ref mut conn) = connection {
            tokio::select! {
                maybe_frame = conn.read_frame() => {
                    let message = Message::from_frame(maybe_frame?)?;
                    match message {
                        Message::ChatMessage(m) => {
                            chat_log.put_line(chat::ChatMessage::new(
                                String::from_utf8(m.msg.to_vec())?,
                                m.sent_at.unwrap().time().to_string(),
                                String::from_utf8(m.name.to_vec())?,
                                " ".to_string(),
                            ))
                        },
                        Message::WelcomeMessage(m) => {
                            chat_log.put_line(chat::ChatMessage::new(
                                String::from_utf8(m.msg.to_vec())?,
                                String::from_utf8(m.sent_at.to_vec())?,
                                "System".to_string(),
                                " ".to_string()))
                        }
                        Message::UserEnteredChat(m) => {
                            chat_log.put_line(chat::ChatMessage::new(
                                String::from_utf8(m.msg.to_vec())?,
                                "".to_string(),
                                "System".to_string(),
                                " ".to_string()
                            ));
                        }
                        Message::WhoIsInChat(m) => {
                            chatters = m.chatters.iter().map(|c| {
                                let stringified_bytes = String::from_utf8(c.to_vec()).expect("Failed decoding bytes");
                                format!(" {}", stringified_bytes)
                            }).collect();
                        }
                        _ => panic!("We are hijacked, terminating!"),
                    }
                }
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            app.handle_key_event(event).await;
                        }
                        Some(Err(e)) => eprintln!("Error: {:?}", e),
                        None => break,
                    }
                }
                action = action_rx.recv() => {
                    // NOTE: recieves None if all senders are dropped or the senders were closed
                    if let Some(a) = action {
                        match a {
                            Action::UserInterrupt => break,
                            Action::SendMessage(msg) => {
                                conn.write_frame(
                                    Message::ChatMessage(
                                        ChatMessage::new(
                                            login_name.clone().unwrap().into(),
                                            None,
                                            msg.into()
                                    )
                                  ).into_frame()
                                ).await?;
                            }
                            _ => panic!("We are hijacked, terminating!"),
                        }
                    }
                }
            }
        } else {
            tokio::select! {
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            app.handle_key_event(event).await;
                        }
                        Some(Err(e)) => eprintln!("Error: {:?}", e),
                        None => break,
                    }
                }
                action = action_rx.recv() => {
                    // NOTE: recieves None if all senders are dropped or the senders were closed
                    if let Some(a) = action {
                        match a {
                            Action::Connect(login) => {
                                let stream = TcpStream::connect("127.0.0.1:8080").await?;
                                let (read_half, write_half) = stream.into_split();
                                connection = Some(Connection::new(read_half, write_half));
                                let message = Message::Login(Login::new(Bytes::copy_from_slice(login.as_bytes())));
                                // NOTE: find a way around matching and unwrapping, perhaps bv putting
                                // the connection inside a wrapper struct
                                connection.as_mut().unwrap().write_frame(message.into_frame()).await?;
                                login_name = Some(login);
                            }
                            Action::UserInterrupt => break,
                            _ => todo!("handle other actions")
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

struct App<B>
where
    B: Backend,
{
    input_buffer: ClientInput,
    terminal: Terminal<B>,
    event_reader: EventStream,
    current_screen: tui::CurrentScreen,
    action_tx: tokio::sync::mpsc::UnboundedSender<Action>,
}

impl<B> App<B>
where
    B: Backend,
{
    fn new(terminal: Terminal<B>, action_tx: tokio::sync::mpsc::UnboundedSender<Action>) -> Self {
        Self {
            input_buffer: ClientInput::new(),
            terminal,
            event_reader: EventStream::new(),
            current_screen: tui::CurrentScreen::Login,
            action_tx,
        }
    }

    async fn handle_key_event(&mut self, event: Event) {
        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Char(c) if event.modifiers.is_empty() => {
                    self.input_buffer.insert(c);
                }
                KeyCode::Char(c) if event.modifiers.contains(KeyModifiers::SHIFT) => {
                    self.input_buffer.insert_uppercase(c.to_uppercase());
                }
                KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.input_buffer.backspace_forward();
                }
                KeyCode::Backspace => {
                    self.input_buffer.backspace();
                }
                KeyCode::Left => {
                    self.input_buffer.left();
                }
                KeyCode::Right => {
                    self.input_buffer.right();
                }
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    // TODO: this should force the application to stop running
                    // may be via some boolean variable I can modify here
                    self.action_tx
                        .send(Action::UserInterrupt)
                        .expect("Reciever unexpectedly dropped");
                }
                KeyCode::Enter => {
                    let source: String = self.input_buffer.inner.iter().collect();
                    self.input_buffer.clear();
                    match self.current_screen {
                        tui::CurrentScreen::Login => {
                            self.current_screen = tui::CurrentScreen::Chatting;
                            self.action_tx
                                .send(Action::Connect(source))
                                .expect("Reciever unexpectedly dropped");
                        }
                        tui::CurrentScreen::Chatting => {
                            self.action_tx
                                .send(Action::SendMessage(source))
                                .expect("Reciever unexpectedly dropped");
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

enum Action {
    // Carries login name
    Connect(String),
    SendMessage(String),
    UserInterrupt,
}
