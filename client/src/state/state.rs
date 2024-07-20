use crate::state::chat::{ChatLog, ChatMessage, SYSTEM_ICON, USER_ICON};
use shared::message::Message;
use std::collections::HashSet;

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
    pub(crate) messages_sent: u64,
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
                self.chat_messages.put_message(chat_message);
            }
            Message::WelcomeMessage(m) => {
                let msg =
                    String::from_utf8(m.msg.to_vec()).expect("Couldn't decode the message in utf8");
                let sent_at = String::from_utf8(m.sent_at.to_vec())
                    .expect("Couldn't decode the message in utf8");
                let chat_message =
                    ChatMessage::new("System".to_string(), sent_at, msg, SYSTEM_ICON.to_string());
                self.chat_messages.put_message(chat_message);
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
                self.chat_messages.put_message(chat_message);
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
