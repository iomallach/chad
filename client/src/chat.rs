use std::collections::VecDeque;

#[derive(Debug)]
pub struct ChatMessage {
    pub timestamp: String,
    pub username: String,
    pub msg: String,
    pub icon: String,
}

impl ChatMessage {
    pub fn new(msg: String, timestamp: String, username: String, icon: String) -> Self {
        Self {
            timestamp,
            username,
            msg,
            icon,
        }
    }
}

pub struct ChatLog {
    lines: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl ChatLog {
    pub fn new(max_messages: usize) -> Self {
        Self {
            lines: VecDeque::new(),
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

    pub fn get_lines(&self) -> &VecDeque<ChatMessage> {
        &self.lines
    }
}
