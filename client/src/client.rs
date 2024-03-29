extern crate shared;
use shared::Message;
use shared::read_message;
use shared::send_message;
use std::io;
use std::{net::{SocketAddr, TcpStream}, error::Error};
use std::io::Write;
use crossterm::terminal;
use crate::chat::ChatLog;
use crate::screen::Rect;
use crate::screen::screen_buffer::ScreenBuffer;
use crate::screen::screen_buffer::ScreenCell;

pub struct ClientInput {
    pub inner: Vec<char>,
    rect: Rect,
    cursor: usize,
}

impl ClientInput {
    pub fn new(rect: Rect) -> Self {
        Self {
            inner: Vec::new(),
            rect,
            cursor: 0,
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.inner.remove(self.cursor);
        }
    }

    pub fn backspace_forward(&mut self) {
        if self.inner.len() > 0 && self.cursor < self.inner.len() {
            self.inner.remove(self.cursor);
        }
    }

    pub fn insert(&mut self, elem: char) {
        self.inner.insert(self.cursor, elem);
        self.cursor += 1;
    }

    pub fn insert_uppercase(&mut self, ch: std::char::ToUppercase) {
        for (idx, c) in ch.enumerate() {
            self.inner.insert(self.cursor + idx, c);
        }
        self.cursor += 1;
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cursor < self.inner.len() {
            self.cursor += 1;
        }
    }

    pub fn render(&mut self, buf: &mut ScreenBuffer) {
        let input_cells = self.inner.iter().map(|c| {
            ScreenCell::new(*c, crossterm::style::Color::Reset, crossterm::style::Color::White, false)
        }).collect::<Vec<_>>();
        buf.fill(ScreenCell::default(), self.rect.x as usize + input_cells.len(), self.rect.y.into(), self.rect.w.into());
        buf.put_cells(input_cells, self.rect.x.into(), self.rect.y.into());
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.cursor = 0;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }
}

pub struct Client {
    pub stream: Option<TcpStream>,
    pub login_name: Option<String>,
    pub chat_log: ChatLog,
    pub window: Rect,
}

impl Client {
    pub fn new() -> Self {
        let (width, height) = terminal::size().unwrap();
        Self {
            stream: None,
            login_name: None,
            // TODO: -4 is kind of a magic number, need to factor it out into configurations
            chat_log: ChatLog::new(Rect::new(2, 3, width as usize - 2, height as usize - 6), 256),
            window: Rect::new(0, 0, width as usize, height as usize),
        }
    }

    pub fn login(&mut self, login: &str) {
        self.login_name = Some(login.to_owned())
    }

    pub fn connect(&mut self, socket_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        if self.stream.is_some() {
            panic!("Already connected");
        }
        self.stream = Some(TcpStream::connect(socket_addr)?);
        let msg = Message::new(self.login_name.as_ref().unwrap(), None, Message::from_chrono(chrono::Local::now()), None);
        self.stream.as_ref().unwrap().write_all(msg.to_string().as_bytes())?;
        self.stream.as_ref().unwrap().set_nonblocking(true)?;
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        if self.stream.is_none() {
            panic!("Not connected");
        }
        self.stream = None;
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<(), io::Error> {
        let msg = Message::new(self.login_name.as_ref().unwrap(), None, Message::from_chrono(chrono::Local::now()), Some(message));
        send_message(msg.to_string().as_str(), self.stream.as_mut().unwrap())
    }

    pub fn fetch_message(&mut self) -> std::io::Result<Message> {
        if let Some(stream) = &mut self.stream {
            match read_message(stream) {
                Ok(m) => {
                    Ok(m)
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => Err(e),
                        io::ErrorKind::BrokenPipe => Err(e),
                        _ => panic!("Something went really wrong {:?}", e),
                    }
                }
            }
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "You are offline"))
        }
    }
}