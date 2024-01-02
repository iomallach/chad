pub mod parse;

use std::{str::FromStr, io};
use std::result::Result;

use parse::RequestParser;


pub enum MessageHeader {
    Username(String),
    Timestamp(String),
    Connections(usize),
    ContentLength(usize),
}

impl ToString for MessageHeader {
    fn to_string(&self) -> String {
        match self {
            Self::Username(un) => format!("Username:{}\r\n", un),
            Self::Timestamp(ts) => format!("Timestamp:{}\r\n", ts),
            Self::Connections(n) => format!("Connections:{}\r\n", n),
            Self::ContentLength(cl) => format!("Content-Length:{}\r\n\r\n", cl),
        }
    }
}

impl FromStr for MessageHeader {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (header, value) = s.split_once(':').unwrap_or((s, s));

        match header {
            "Username" => Ok(Self::Username(value.trim().to_owned())),
            "Timestamp" => Ok(Self::Timestamp(value.trim().to_owned())),
            "Connections" => Ok(Self::Connections(value.trim().parse().expect("No parse error"))),
            "Content-Length" => Ok(Self::ContentLength(value.trim().parse().expect("No parse error"))),
            x => {
                let error_msg = format!("Unexpected header: {}", x);
                let error = std::io::Error::new(io::ErrorKind::InvalidData, error_msg);
                Err(error)
            }
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub username: String,
    pub timestamp: String,
    pub connections: Option<usize>,
    pub content_length: usize,
    pub message: Option<String>,
}

impl Message {
    pub fn new(username: &str, connections: Option<usize>, timestamp: String, message: Option<&str>) -> Self {
        match message {
            None => {
                Self {
                    username: username.to_owned(),
                    timestamp,
                    connections,
                    content_length: 0,
                    message: None,
                }
            },
            Some(m) => {
                Self {
                    username: username.to_owned(),
                    timestamp,
                    connections,
                    content_length: m.len(),
                    message: Some(m.to_owned()),
                }
            },
        }
    }

    pub fn from_chrono(ts: chrono::DateTime<chrono::Local>) -> String {
        ts.format("%d-%m-%Y %H:%M:%S").to_string()
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username && self.timestamp == other.timestamp && self.connections == other.connections && self.content_length == other.content_length && self.message == other.message
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let mut str_repr = String::new();

        str_repr.push_str(&MessageHeader::Username(self.username.to_owned()).to_string());
        str_repr.push_str(&MessageHeader::Timestamp(self.timestamp.to_owned()).to_string());
        if let Some(conn_n) = self.connections {
            str_repr.push_str(&MessageHeader::Connections(conn_n).to_string());
        }
        if self.content_length > 0 {
            let content_len = &MessageHeader::ContentLength(self.message.as_ref().unwrap().len()).to_string();
            str_repr.push_str(&content_len);
            str_repr.push_str(self.message.as_ref().unwrap().as_str());
        } else {
            let content_len = &MessageHeader::ContentLength(0).to_string();
            str_repr.push_str(&content_len);
        }
        str_repr
    }
}

impl FromStr for Message {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RequestParser::new(&mut s.to_owned().as_bytes()).parse_message()
    }
}

pub fn read_message<T: io::Read>(from: &mut T) -> Result<Message, io::Error> {
    RequestParser::new(from).parse_message()
}

pub fn send_message<T: io::Write>(what: &str, to: &mut T,) -> Result<(), io::Error> {
    to.write_all(what.as_bytes())?;
    Ok(())
}
