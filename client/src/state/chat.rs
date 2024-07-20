use ratatui::layout::Rect;
use std::collections::VecDeque;

pub(crate) static USER_ICON: &str = " ";
pub(crate) static SYSTEM_ICON: &str = " ";

#[derive(Debug, Clone)]
pub(crate) struct ChatMessage {
    timestamp: String,
    user_name: String,
    msg: String,
    icon: String,
}

impl ChatMessage {
    pub(crate) fn new(user_name: String, timestamp: String, msg: String, icon: String) -> Self {
        Self {
            timestamp,
            user_name,
            msg,
            icon,
        }
    }

    pub(crate) fn length(&self) -> u16 {
        // Two spaces and a colon
        (self.timestamp.len() + self.user_name.len() + self.msg.len() + self.icon.len() + 3) as u16
    }
}

impl std::fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}: {}",
            self.timestamp, self.icon, self.user_name, self.msg
        )
    }
}

#[derive(Clone)]
pub(crate) struct ChatLog {
    messages: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl ChatLog {
    pub(crate) fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
        }
    }

    pub(crate) fn put_message(&mut self, line: ChatMessage) {
        if self.messages.len() + 1 > self.max_messages {
            self.messages.pop_front();
            self.messages.push_back(line)
        } else {
            self.messages.push_back(line);
        }
    }

    pub(crate) fn get_messages(&self) -> &VecDeque<ChatMessage> {
        &self.messages
    }

    pub(crate) fn get_fitting_messages(&self, area: &Rect) -> VecDeque<ChatMessage> {
        let mut fitting_messages: VecDeque<ChatMessage> = VecDeque::new();
        let mut lines_filled: u16 = 0;

        for m in self.messages.iter().rev() {
            // TODO: probably better to allocate once
            let lines_needed = divide_ceiled(m.length() as f32, area.width as f32);
            if (lines_needed + lines_filled) > area.height {
                break;
            } else {
                lines_filled += lines_needed;
                fitting_messages.push_front(m.clone());
            }
        }
        fitting_messages
    }
}

fn divide_ceiled(nom: f32, denom: f32) -> u16 {
    (nom / denom).ceil() as u16
}

impl Default for ChatLog {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 50,
        }
    }
}
