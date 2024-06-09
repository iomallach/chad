use std::vec::IntoIter;

use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use shared::parse_async::Frame;

#[derive(Clone, Debug)]
pub enum Message {
    Login(Login),
    Logout(Logout),
    ChatMessage(ChatMessage),
    SystemMessage(SystemMessage),
}

impl Message {
    pub fn from_frame(frame: Frame) -> Result<Self> {
        let mut parser = Parser::new(frame)?;
        let msg_kind = parser.next_bytes()?;

        match &msg_kind[..] {
            b"login" => Ok(Self::Login(Login::parse(parser)?)),
            b"logout" => Ok(Self::Logout(Logout::parse(parser)?)),
            b"chat_message" => Ok(Self::ChatMessage(ChatMessage::parse_new_message(parser)?)),
            b"system_message" => Ok(Self::SystemMessage(SystemMessage::parse(parser)?)),
            unknown => bail!("Unknown message kind: {:?}", unknown),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SystemMessage {
    pub msg: Bytes,
}

impl SystemMessage {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            msg: parser.next_bytes()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Login {
    pub name: Bytes,
}

impl Login {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            name: parser.next_bytes()?,
        })
    }
}

// NOTE: It may not be worth keeping the name here since the thread holds the name anyway and there
// can be no other client connected to the same thread
#[derive(Clone, Debug)]
pub struct Logout {
    pub name: Bytes,
}

impl Logout {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            name: parser.next_bytes()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub name: Bytes,
    pub sent_at: Option<chrono::NaiveDateTime>,
    pub msg: Bytes,
}

impl ChatMessage {
    fn parse_new_message(mut parser: Parser) -> Result<Self> {
        let name = parser.next_bytes()?;
        let msg = parser.next_bytes()?;

        Ok(Self {
            name,
            sent_at: None,
            msg,
        })
    }

    fn parse_broadcast_message(mut parser: Parser) -> Result<Self> {
        let name = parser.next_bytes()?;
        let msg = parser.next_bytes()?;
        let timestamp = parser.next_i64()?;
        let sent_at = chrono::NaiveDateTime::from_timestamp_millis(timestamp)
            .ok_or(anyhow!("Unable to parse {timestamp} into datetime"))?;

        Ok(Self {
            name,
            sent_at: Some(sent_at),
            msg,
        })
    }
}

struct Parser {
    frame: IntoIter<Frame>,
}

impl Parser {
    pub fn new(frame: Frame) -> Result<Self> {
        if let Frame::Array(a) = frame {
            return Ok(Self {
                frame: a.into_iter(),
            });
        }
        bail!("Expected array, got something else")
    }

    fn next(&mut self) -> Result<Frame> {
        self.frame.next().ok_or(anyhow!("End of frame"))
    }

    fn next_bytes(&mut self) -> Result<Bytes> {
        match self.next()? {
            Frame::Bulk(s) => Ok(s),
            Frame::Array(_) => bail!("Expected bulk string, found array"),
        }
    }

    fn next_i64(&mut self) -> Result<i64> {
        let bytes = self.next_bytes()?;
        let str_num = std::str::from_utf8(&bytes)?;
        let parsed_i64 = str_num.to_string().parse::<i64>()?;

        Ok(parsed_i64)
    }
}
