use std::collections::VecDeque;

use crossterm::style::Color;
use crossterm::style::{Stylize, style};

#[derive(Debug)]
pub struct MessagePiece {
    contents: String,
    fg_color: Color,
    bg_color: Color,
}

impl MessagePiece {
    fn new(contents: String, fg_color: Color, bg_color: Color) -> Self {
        Self {
            contents,
            fg_color,
            bg_color,
        }
    }

    fn message(contents: String) -> Self {
        Self::new(contents, Color::White, Color::Black)
    }

    fn username(contents: String) -> Self {
        Self::new(contents, Color::Green, Color::Black)
    }

    fn timestamp(contents: String) -> Self {
        Self::new(contents, Color::DarkYellow, Color::Black)
    }
}

#[derive(Debug)]
pub struct ChatMessage {
    timestamp: MessagePiece,
    username: MessagePiece,
    msg: MessagePiece,
    bg_color: Color,
    fg_color: Color,
}

impl ChatMessage {
    pub fn new(msg: String, timestamp: String, username: String, fg_color: Color, bg_color: Color) -> Self {
        Self {
            timestamp: MessagePiece::timestamp(timestamp),
            username: MessagePiece::username(username),
            msg: MessagePiece::message(msg),
            fg_color,
            bg_color,
        }
    }

    pub fn default(msg: String, timestamp: String, username: String) -> Self {
        Self::new(msg, timestamp, username, Color::White, Color::Black)
    }

    pub fn system(msg: String, timestamp: String, username: String) -> Self {
        Self::new(msg, timestamp, username, Color::Red, Color::Black)
    }
}

impl std::fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = style(&self.timestamp.contents).with(self.timestamp.fg_color);
        let username = style(&self.username.contents).with(self.username.fg_color);
        let message = style(&self.msg.contents).with(self.msg.fg_color);
        write!(f, "[{}][{}]: {}", timestamp, username, message)
    }
}

pub struct ChatLog {
    lines: VecDeque<ChatMessage>,
    height: usize,
    width: usize,
    max_messages: usize
}

impl ChatLog {
    pub fn new(height: usize, width: usize, max_messages: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            height,
            width,
            max_messages,
        }
    }

    pub fn put_line(&mut self, line: ChatMessage) {
        if self.lines.len() + 1 > self.max_messages {
            self.lines.pop_front();
            self.lines.push_back(line)
        } else {
            self.lines.push_back(line);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get(&self) -> &VecDeque<ChatMessage> {
        &self.lines
    }
}