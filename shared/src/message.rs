use std::vec::IntoIter;

use crate::parse_async::Frame;
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;

#[derive(Clone, Debug)]
pub enum Message {
    Login(Login),
    Logout(Logout),
    ChatMessage(ChatMessage),
    WelcomeMessage(WelcomeMessage),
    UserEnteredChat(UserEnteredChat),
    UserLeftChat(UserLeftChat),
    WhoIsInChat(WhoIsInChat),
}

impl Message {
    pub fn from_frame(frame: Frame) -> Result<Self> {
        let mut parser = Parser::new(frame)?;
        let msg_kind = parser.next_bytes()?;

        // TODO: figure out how to make use of the compiler here
        // instead of having to maintain strings
        match &msg_kind[..] {
            b"login" => Ok(Self::Login(Login::parse(parser)?)),
            b"logout" => Ok(Self::Logout(Logout::parse(parser)?)),
            b"chat_message" => Ok(Self::ChatMessage(ChatMessage::parse(parser)?)),
            b"welcome_message" => Ok(Self::WelcomeMessage(WelcomeMessage::parse(parser)?)),
            b"user_entered_chat" => Ok(Self::UserEnteredChat(UserEnteredChat::parse(parser)?)),
            b"user_left_chat" => Ok(Self::UserLeftChat(UserLeftChat::parse(parser)?)),
            b"who_is_in_chat" => Ok(Self::WhoIsInChat(WhoIsInChat::parse(parser)?)),
            unknown => bail!("Unknown message kind: {:?}", unknown),
        }
    }

    // TODO: probably better to implement From<T> instead of this
    pub fn into_frame(self) -> Frame {
        match self {
            Self::Login(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"login")));
                frame.push_bulk(Frame::Bulk(msg.name));
                frame
            }
            Self::Logout(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"logout")));
                frame.push_bulk(Frame::Bulk(msg.name));
                frame
            }
            Self::ChatMessage(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"chat_message")));
                frame.push_bulk(Frame::Bulk(msg.name));
                frame.push_bulk(Frame::Bulk(msg.msg));
                frame.push_bulk(Frame::Bulk(msg.sent_at));
                frame
            }
            Self::WelcomeMessage(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"welcome_message")));
                frame.push_bulk(Frame::Bulk(msg.sent_at));
                frame.push_bulk(Frame::Bulk(msg.msg));
                frame
            }
            Self::UserEnteredChat(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"user_entered_chat")));
                frame.push_bulk(Frame::Bulk(msg.name));
                frame.push_bulk(Frame::Bulk(msg.msg));
                frame
            }
            Self::UserLeftChat(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"user_left_chat")));
                frame.push_bulk(Frame::Bulk(msg.name));
                frame.push_bulk(Frame::Bulk(msg.msg));
                frame
            }
            Self::WhoIsInChat(msg) => {
                let mut frame = Frame::array();
                frame.push_bulk(Frame::Bulk(Bytes::from_static(b"who_is_in_chat")));

                let mut chatters_array = Frame::array();
                msg.chatters.iter().for_each(|el| {
                    chatters_array.push_bulk(Frame::Bulk(el.clone()));
                });
                frame.push_bulk(chatters_array);
                println!("View as frame: {:?}", frame);
                frame
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct WhoIsInChat {
    pub chatters: Vec<Bytes>,
}

impl WhoIsInChat {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            chatters: parser.next_array()?,
        })
    }

    pub fn new(chatters: Vec<Bytes>) -> Self {
        Self { chatters }
    }
}

#[derive(Clone, Debug)]
pub struct UserEnteredChat {
    pub msg: Bytes,
    pub name: Bytes,
}

impl UserEnteredChat {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            name: parser.next_bytes()?,
            msg: parser.next_bytes()?,
        })
    }

    pub fn new(msg: Bytes, name: Bytes) -> Self {
        Self { msg, name }
    }
}

#[derive(Clone, Debug)]
pub struct UserLeftChat {
    pub msg: Bytes,
    pub name: Bytes,
}

impl UserLeftChat {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            name: parser.next_bytes()?,
            msg: parser.next_bytes()?,
        })
    }

    pub fn new(msg: Bytes, name: Bytes) -> Self {
        Self { msg, name }
    }
}

#[derive(Clone, Debug)]
pub struct WelcomeMessage {
    pub msg: Bytes,
    pub sent_at: Bytes,
}

impl WelcomeMessage {
    fn parse(mut parser: Parser) -> Result<Self> {
        Ok(Self {
            sent_at: parser.next_bytes()?,
            msg: parser.next_bytes()?,
        })
    }

    pub fn new(msg: Bytes) -> Self {
        Self {
            msg,
            sent_at: Bytes::copy_from_slice(
                chrono::offset::Local::now()
                    .time()
                    .format("%H:%M:%S")
                    .to_string()
                    .as_bytes(),
            ),
        }
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

    pub fn new(name: Bytes) -> Self {
        Self { name }
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

    fn new(name: Bytes) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub name: Bytes,
    pub sent_at: Bytes,
    pub msg: Bytes,
}

impl ChatMessage {
    fn parse(mut parser: Parser) -> Result<Self> {
        let name = parser.next_bytes()?;
        let msg = parser.next_bytes()?;
        let sent_at = parser.next_bytes()?;

        Ok(Self { name, sent_at, msg })
    }

    pub fn new(name: Bytes, sent_at: chrono::DateTime<chrono::Local>, msg: Bytes) -> Self {
        let sent_at_fmt = sent_at.time().format("%H:%M:%S").to_string();
        Self {
            name,
            sent_at: sent_at_fmt.into(),
            msg,
        }
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

    fn next_array(&mut self) -> Result<Vec<Bytes>> {
        if let Frame::Array(a) = self.next()? {
            let mut array: Vec<Bytes> = Vec::new();
            let length = a.len();
            let mut inner_parser = Self::new(Frame::Array(a))?;

            for _ in 0..length {
                array.push(inner_parser.next_bytes()?);
            }
            return Ok(array);
        }
        bail!("Expected array, got something else")
    }

    fn next_i64(&mut self) -> Result<i64> {
        let bytes = self.next_bytes()?;
        let str_num = std::str::from_utf8(&bytes)?;
        let parsed_i64 = str_num.to_string().parse::<i64>()?;

        Ok(parsed_i64)
    }
}
