use std::collections::VecDeque;

use crossterm::style::Color;
use crossterm::style::{Stylize, style};

use crate::screen::Rect;
use crate::screen::screen_buffer::{ScreenBuffer, ScreenCell};

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
        Self::new(contents, Color::White, Color::Reset)
    }

    fn username(contents: String) -> Self {
        Self::new(contents, Color::Green, Color::Reset)
    }

    fn timestamp(contents: String) -> Self {
        Self::new(contents, Color::DarkYellow, Color::Reset)
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
        Self::new(msg, timestamp, username, Color::White, Color::Reset)
    }

    pub fn system(msg: String, timestamp: String, username: String) -> Self {
        Self::new(msg, timestamp, username, Color::Red, Color::Reset)
    }

    pub fn as_cells(&self) -> Vec<ScreenCell> {
        // TODO: duplicated code
        let mut timestamp: Vec<_> = self.timestamp.contents.chars().map(|c| {
            ScreenCell::new(c, self.timestamp.bg_color, self.timestamp.fg_color)
        }).collect();
        timestamp.push(ScreenCell::default());
        let mut username: Vec<_> = self.username.contents.chars().map(|c| {
            ScreenCell::new(c, self.username.bg_color, self.username.fg_color)
        }).collect();
        username.push(ScreenCell::default());
        let mut msg: Vec<_> = self.msg.contents.chars().map(|c| {
            ScreenCell::new(c, self.msg.bg_color, self.msg.fg_color)
        }).collect();
        let mut message_cells: Vec<ScreenCell> = Vec::new();

        message_cells.extend(timestamp);
        message_cells.extend(username);
        message_cells.extend(msg);

        message_cells
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
    rect: Rect,
    max_messages: usize
}

impl ChatLog {
    pub fn new(rect: Rect, max_messages: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            rect,
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

    pub fn render(&self, buf: &mut ScreenBuffer) {
        // TODO: this is flawed. Probably better to do it in the put line. The below code can lead to newer messages not being printed
        let mut n_lines = 0_f32;
        for (i, m) in self.lines.iter().enumerate() {
            let message_cells = m.as_cells();
            let need_lines = message_cells.len() as f32 / self.rect.w as f32;
            n_lines += need_lines.ceil();
            if n_lines > self.rect.h as f32 {
                break
            }
            // TODO: this doesn't take into account cases where previous message occupied multiple lines
            buf.fill_row(
                ScreenCell::default(),
                self.rect.y + i,
                Some(self.rect.x),
                Some(self.rect.w - self.rect.x));
            buf.put_cells(message_cells, self.rect.x, self.rect.y + i);
        }
    }
}