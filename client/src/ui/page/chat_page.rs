use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::Stylize,
    symbols,
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, List, ListItem, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    client::ClientInput, state::action::Action, state::chat::ChatLog, state::state::State,
};

use super::widget::Widget;

const USER_ICON: &str = "";

struct ChatPageState {
    login_name: Option<String>,
    messages_sent: u64,
    chat_messages: ChatLog,
    online_users: HashSet<String>,
    time_online: u64,
}

impl From<State> for ChatPageState {
    fn from(value: State) -> Self {
        Self {
            login_name: value.login_name,
            messages_sent: value.messages_sent,
            chat_messages: value.chat_messages,
            online_users: value.online_users,
            time_online: value.timer.round() as u64,
        }
    }
}

pub(crate) struct ChatPage {
    action_tx: UnboundedSender<Action>,
    page_state: ChatPageState,
    input: ClientInput,
}

impl ChatPage {
    pub(crate) fn new(action_tx: UnboundedSender<Action>, state: State) -> Self {
        Self {
            action_tx,
            page_state: ChatPageState::from(state),
            input: ClientInput::new(),
        }
    }
}

impl Widget for ChatPage {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c) if key.modifiers.is_empty() => self.input.insert(c),
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.input.insert_uppercase(c.to_uppercase())
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.backspace_forward()
            }
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Left => self.input.left(),
            KeyCode::Right => self.input.right(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.action_tx
                    .send(Action::Quit)
                    .expect("Receiver unexpectedly dropped");
            }
            KeyCode::Enter => {
                let source: String = self.input.get_ref().iter().collect();
                self.input.clear();
                self.action_tx
                    .send(Action::SendMessage { message: source })
                    .expect("Receiver unexpectedly dropped");
            }
            _ => {}
        }
    }
    fn update(&mut self, state: State) {
        self.page_state = ChatPageState::from(state);
    }

    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .areas(frame.size());
        let [chat_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .areas(left);
        let [chatters_area, user_info_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(5)])
            .areas(right);

        let chat_block = Block::default()
            .title(Title::from("Messages".bold()).alignment(Alignment::Left))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);
        let input_block = Block::default()
            .title(Title::from("Input".bold()).alignment(Alignment::Left))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .green();

        let chatters_block = Block::default()
            .title(Title::from("Chatters".bold()).alignment(Alignment::Left))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);
        let user_info_block = Block::default()
            .title(Title::from("User info".bold()).alignment(Alignment::Left))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let chat_lines = self
            .page_state
            .chat_messages
            .get_fitting_messages(&chat_area.inner(&Margin::new(0, 1)))
            .into_iter()
            .map(|l| {
                let msg = Line::from(Span::raw(format!("{}", l)));
                ListItem::new(msg)
            });

        let chatters_lines = self.page_state.online_users.iter().map(|l| {
            let chatter = if *l == *self.page_state.login_name.as_ref().unwrap() {
                "You".green()
            } else {
                l.clone().white()
            };
            let item = Line::from(vec![Span::from(format!("{} ", USER_ICON)), chatter]);
            ListItem::new(item)
        });

        let user_info_lines = {
            let user_name_line =
                Line::from(self.page_state.login_name.as_ref().unwrap().to_string());
            let messages_sent = Line::from(format!("Sent: {}", self.page_state.messages_sent));
            let time_online = Line::from(format!("Online for {}s", self.page_state.time_online));
            vec![
                ListItem::new(user_name_line),
                ListItem::new(messages_sent),
                ListItem::new(time_online),
            ]
        };

        frame.render_widget(List::new(chat_lines).block(chat_block), chat_area);
        frame.render_widget(
            Paragraph::new(self.input.get_ref().iter().collect::<String>()).block(input_block),
            input_area,
        );
        frame.render_widget(
            List::new(chatters_lines).block(chatters_block),
            chatters_area,
        );
        frame.render_widget(
            List::new(user_info_lines).block(user_info_block),
            user_info_area,
        );

        frame.set_cursor(
            input_area.x + 1 + self.input.get_ref().len() as u16,
            input_area.y + 1,
        )
    }
}
