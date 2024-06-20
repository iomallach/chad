use shared::message::Message;
use std::collections::{HashSet, VecDeque};

static USER_ICON: &str = " ";
static SYSTEM_ICON: &str = " ";

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
    lines: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl ChatLog {
    pub(crate) fn new(max_messages: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            max_messages,
        }
    }

    pub(crate) fn put_line(&mut self, line: ChatMessage) {
        if self.lines.len() + 1 > self.max_messages {
            self.lines.pop_front();
            self.lines.push_back(line)
        } else {
            self.lines.push_back(line);
        }
    }

    pub(crate) fn get_lines(&self) -> &VecDeque<ChatMessage> {
        &self.lines
    }
}

impl Default for ChatLog {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            max_messages: 50,
        }
    }
}

#[derive(Clone)]
pub(crate) enum ConnectionStatus {
    Offline,
    Online,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Offline
    }
}

#[derive(Default, Clone)]
pub(crate) struct State {
    pub(crate) login_name: Option<String>,
    // TODO: String for simplicity, ideally should be: {user, msg, time, icon}
    pub(crate) chat_messages: ChatLog,
    // TODO: Same as above, ideally {name, icon, ?time_joined}
    pub(crate) online_users: HashSet<String>,
    pub(crate) connection_status: ConnectionStatus,
}

// TODO: perhaps it makes sense to return a Result from here
// instead of having to panic on every decode failure
// which is not expected to happen anyway
impl State {
    pub(crate) fn handle_server_message(&mut self, server_message: Message) {
        match server_message {
            Message::ChatMessage(m) => {
                let sent_at = m.sent_at.expect("Message DTTM gone").time().to_string();
                let chat_message = ChatMessage::new(
                    String::from_utf8(m.name.to_vec()).expect("Couldn't decode the name in utf8"),
                    sent_at,
                    String::from_utf8(m.msg.to_vec()).expect("Couldn't decode the message in utf8"),
                    USER_ICON.to_string(),
                );
                self.chat_messages.put_line(chat_message);
            }
            Message::WelcomeMessage(m) => {
                let msg =
                    String::from_utf8(m.msg.to_vec()).expect("Couldn't decode the message in utf8");
                let sent_at = String::from_utf8(m.sent_at.to_vec())
                    .expect("Couldn't decode the message in utf8");
                let chat_message =
                    ChatMessage::new("System".to_string(), sent_at, msg, SYSTEM_ICON.to_string());
                self.chat_messages.put_line(chat_message);
            }
            Message::UserEnteredChat(m) => {
                let msg =
                    String::from_utf8(m.msg.to_vec()).expect("Coudn't decode the message in utf8");
                let chat_message = ChatMessage::new(
                    "System".to_string(),
                    "".to_string(),
                    msg,
                    SYSTEM_ICON.to_string(),
                );
                self.chat_messages.put_line(chat_message);
            }
            Message::WhoIsInChat(m) => {
                self.online_users = m
                    .chatters
                    .iter()
                    .map(|c| {
                        String::from_utf8(c.to_vec())
                            .expect("Failed decoding bytes while parsig online users")
                    })
                    .collect();
            }
            Message::Login(_) | Message::Logout(_) => {
                unreachable!("Client must not receive server-side events")
            }
        }
    }
}
