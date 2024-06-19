extern crate shared;
use crate::chat::ChatLog;
use anyhow::{anyhow, Result};
use std::io;
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpStream;

pub struct ClientInput {
    // TODO: make private when the flux move is over
    pub inner: Vec<char>,
    cursor: usize,
}

impl ClientInput {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
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

    pub fn clear(&mut self) {
        self.inner.clear();
        self.cursor = 0;
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn get_ref(&self) -> &[char] {
        &self.inner
    }
}

pub struct Client {
    pub stream: Option<TcpStream>,
    pub login_name: Option<String>,
    pub chat_log: ChatLog,
}

impl Client {
    pub fn new() -> Self {
        Self {
            stream: None,
            login_name: None,
            chat_log: ChatLog::new(256),
        }
    }

    pub fn login(&mut self, login: String) {
        self.login_name = Some(login)
    }

    pub async fn connect(&mut self, socket_addr: SocketAddr) -> Result<()> {
        if self.stream.is_some() {
            panic!("Already connected");
        }
        self.stream = Some(TcpStream::connect(socket_addr).await?);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if self.stream.is_none() {
            panic!("Not connected");
        }
        self.stream = None;
        Ok(())
    }

    pub fn send_message(&mut self, message: &str) -> Result<()> {
        //let msg = Message::new(
        //    self.login_name.as_ref().unwrap(),
        //    None,
        //    Message::from_chrono(chrono::Local::now()),
        //    Some(message),
        //);
        //send_message(msg.to_string().as_str(), self.stream.as_mut().unwrap())
        Ok(())
    }

    //pub fn fetch_message(&mut self) -> std::io::Result<Message> {
    //    if let Some(stream) = &mut self.stream {
    //        match read_message(stream) {
    //            Ok(m) => Ok(m),
    //            Err(e) => match e.kind() {
    //                io::ErrorKind::WouldBlock => Err(e),
    //                io::ErrorKind::BrokenPipe => Err(e),
    //                _ => panic!("Something went really wrong {:?}", e),
    //            },
    //        }
    //    } else {
    //        Err(io::Error::new(io::ErrorKind::BrokenPipe, "You are offline"))
    //    }
    //}
}
