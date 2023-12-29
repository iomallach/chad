use std::{error::Error, fmt::Display, str::FromStr};
use std::result::Result;
use std::io::{self, BufRead};

pub enum MessageHeader {
    Username(String),
    Timestamp(String),
    Connections(usize),
    HasMessage,
}

impl ToString for MessageHeader {
    fn to_string(&self) -> String {
        match self {
            Self::Username(un) => format!("username:{}\r\n", un),
            Self::Timestamp(ts) => format!("timestamp:{}\r\n", ts),
            Self::Connections(n) => format!("connections:{}\r\n", n),
            Self::HasMessage => format!("has_message:\r\n")
        }
    }
}

impl FromStr for MessageHeader {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (header, value) = s.split_once(':').unwrap_or((s, s));

        match header {
            "username" => Ok(Self::Username(value.trim().to_owned())),
            "timestamp" => Ok(Self::Timestamp(value.trim().to_owned())),
            "connections" => Ok(Self::Connections(value.parse().expect("No parse error"))),
            "has_message" => Ok(Self::HasMessage),
            _ => Box::new(Err(std::io::Error::new()))
        }
    }
}

pub struct Message {
    pub username: String,
    pub timestamp: String,
    pub connections: Option<usize>,
    pub has_message: bool,
    pub message: Option<String>,
}

impl Message {
    pub fn new(username: &str, connections: Option<usize>, message: Option<&str>) -> Self {
        match message {
            None => {
                Self {
                    username: username.to_owned(),
                    timestamp: format!("{}", chrono::offset::Local::now()),
                    connections,
                    has_message: false,
                    message: None,
                }
            },
            Some(m) => {
                Self {
                    username: username.to_owned(),
                    timestamp: format!("{}", chrono::offset::Local::now()),
                    connections,
                    has_message: true,
                    message: Some(m.to_owned()),
                }
            },
        }
    }

    pub fn without_message(username: &str) -> Self {
        Self {
            username: username.to_owned(),
            timestamp: format!("{}", chrono::offset::Local::now()),
            connections: None,
            has_message: false,
            message: None,
        }
    }

    pub fn with_message(username: &str, message: &str) -> Self {
        Self {
            username: username.to_owned(),
            timestamp: format!("{}", chrono::offset::Local::now()),
            connections: None,
            has_message: true,
            message: Some(message.to_owned()),
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let mut str_repr = String::new();

        str_repr.push_str(&MessageHeader::Username(self.username.to_owned()).to_string());
        str_repr.push_str(&MessageHeader::Timestamp(self.timestamp.to_owned()).to_string());
        if self.has_message {
            str_repr.push_str(&MessageHeader::HasMessage.to_string());
            str_repr.push_str("\r\n");
            str_repr.push_str(self.message.as_ref().unwrap().as_str());
        }
        str_repr.push_str("\r\n\r\n");
        str_repr
    }
}

impl FromStr for Message {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_parts = s.split("\r\n");

        let mut username: Option<String> = None;
        let mut timestamp: Option<String> = None;
        let mut connections: Option<usize> = None;
        let mut has_message = false;
        let mut message: Option<String> = None;

        for raw_part in raw_parts {
            let header = match MessageHeader::from_str(raw_part) {
                Ok(h) => h,
                Err(_) => continue,
            };
            
            match header {
                MessageHeader::Username(un) => username = Some(un),
                MessageHeader::Timestamp(ts) => timestamp = Some(ts),
                MessageHeader::Connections(n) => connections = Some(n),
                MessageHeader::HasMessage => has_message = true,
            }
        }

        if has_message {
            let (raw_head, raw_tail) = s.split_once("\r\n\r\n").unwrap();
            let msg_string = raw_tail[0..raw_tail.len() - 4].to_owned();
            message = Some(msg_string)
        }
        Ok(Message {
            username: username.unwrap(),
            timestamp: timestamp.unwrap(),
            connections,
            has_message,
            message,
        })
    }
}

// TODO: take a buffer and return the number of bytes read
pub fn read_message<T: io::Read>(from: &mut T) -> Result<String, ParseMessageError> {
    let mut reader = io::BufReader::new(from);
    let mut message = String::new();
    let mut processed_headers = false;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Err(ParseMessageError{kind: ParseMessageErrorKind::WouldBlock}),
            Err(e) => panic!("Something went wrong {}", e),
            Ok(0) => return Err(ParseMessageError{kind: ParseMessageErrorKind::SocketClosed}),
            Ok(_) => {},
        }

        message.push_str(&line);

        if message.ends_with("\r\n\r\n") {
            if !message.contains(&MessageHeader::HasMessage.to_string()) || processed_headers {
                break;
            }
            processed_headers = true;
        }
    }
    Ok(message)
}

pub fn send_message<T: io::Write>(what: &str, to: &mut T,) -> Result<(), Box<dyn Error>> {
    to.write_all(what.as_bytes())?;
    Ok(())
}

#[derive(Debug)]
pub enum ParseMessageErrorKind {
    SocketClosed,
    UnknownHeader,
    WouldBlock,
}

impl PartialEq for ParseMessageErrorKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::SocketClosed, &Self::SocketClosed) => true,
            (&Self::UnknownHeader, &Self::UnknownHeader) => true,
            (&Self::WouldBlock, &Self::WouldBlock) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct ParseMessageError {
    pub kind: ParseMessageErrorKind,
}

impl Error for ParseMessageError {}

impl Display for ParseMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseMessageErrorKind::SocketClosed => write!(f, "Socket closed"),
            ParseMessageErrorKind::UnknownHeader => write!(f, "Unknown header"),
            ParseMessageErrorKind::WouldBlock => write!(f, "Would block"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message_without_message() {
        let message = Message {
            username: "testuser".to_owned(),
            timestamp: "2023-12-28".to_owned(),
            connections: None,
            has_message: false,
            message: None,
        }.to_string();
        assert_eq!("username:testuser\r\ntimestamp:2023-12-28\r\n\r\n\r\n", message);
    }

    #[test]
    fn test_create_message_with_message() {
        let message = Message {
            username: "testuser".to_owned(),
            timestamp: "2023-12-28".to_owned(),
            connections: None,
            has_message: true,
            message: Some("hello world".to_owned()),
        }.to_string();
        assert_eq!("username:testuser\r\ntimestamp:2023-12-28\r\nhas_message:\r\n\r\nhello world\r\n\r\n", message);
    }

    #[test]
    fn test_parse_message_without_message() {
        let raw_message = "username: testuser\r\ntimestamp: 2023-12-28\r\n\r\n\r\n";
        let message = Message::from_str(raw_message).unwrap();
        assert_eq!(message.username, "testuser");
        assert_eq!(message.timestamp, "2023-12-28");
        assert_eq!(message.has_message, false);
        assert_eq!(message.message, None);
    }

    #[test]
    fn test_parse_message_with_message() {
        let raw_message = "username: testuser\r\ntimestamp: 2023-12-28\r\nhas_message:\r\n\r\nhallowelt\r\n\r\n";
        let message = Message::from_str(raw_message).unwrap();
        assert_eq!(message.username, "testuser");
        assert_eq!(message.timestamp, "2023-12-28");
        assert_eq!(message.has_message, true);
        assert_eq!(message.message, Some("hallowelt".to_owned()));
    }

    #[test]
    fn test_read_message() {
        let mut message = "username:testuser\r\ntimestamp:2023-12-28\r\nhas_message:\r\n\r\nhallowelt\r\n\r\n".as_bytes();
        
        let message_copy = message;
        let msg = read_message(&mut message).unwrap();
        assert_eq!(message_copy, msg.as_bytes());
        assert_eq!(message_copy.len(), msg.len());
        assert!(msg.len() > 0);
    }
}