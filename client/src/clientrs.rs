extern crate shared;
use shared::Message;
use shared::send_message;
use std::io;
use std::{net::{SocketAddr, TcpStream}, error::Error};
use std::io::Write;
use crossterm::terminal;
use crate::chat::ChatLog;

pub struct ClientInput {
    pub buffer: Vec<char>,
}

impl ClientInput {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn push(&mut self, ch: char) {
        self.buffer.push(ch)
    }

    pub fn backspace(&mut self) {
        self.buffer.pop();
    }
}

pub struct Window {
    pub height: usize,
    pub width: usize,
}

impl Window {
    fn new(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
        }
    }
}

pub struct Client {
    pub stream: Option<TcpStream>,
    pub login_name: Option<String>,
    pub chat_log: ChatLog,
    pub window: Window,
}

impl Client {
    pub fn new() -> Self {
        let (width, height) = terminal::size().unwrap();
        Self {
            stream: None,
            login_name: None,
            chat_log: ChatLog::new(height as usize, width as usize),
            window: Window::new(height as usize, width as usize),
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
        let msg = Message::without_message(self.login_name.as_ref().unwrap());
        self.stream.as_ref().unwrap().write_all(msg.to_string().as_bytes())?;
        // write!(&mut self.stream.as_ref().unwrap(), "{}", msg.to_string())?;
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
        let msg = Message::new(self.login_name.as_ref().unwrap(), None, Some(message));
        let see_msg = msg.to_string();
        send_message(msg.to_string().as_str(), self.stream.as_mut().unwrap())
    }
}