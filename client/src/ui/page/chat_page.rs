use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::Stylize,
    symbols,
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, List, ListItem, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    client::ClientInput, state::action::Action, state::state::ChatLog, state::state::State,
};

use super::widget::Widget;

struct ChatPageState {
    login_name: Option<String>,
    chat_messages: ChatLog,
    online_users: HashSet<String>,
}

impl From<State> for ChatPageState {
    fn from(value: State) -> Self {
        Self {
            login_name: value.login_name,
            chat_messages: value.chat_messages,
            online_users: value.online_users,
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
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .areas(frame.size());
        let [chat_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .areas(left);
        let [chatters_area, diagnostics_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
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
        let diagnostics_block = Block::default()
            .title(Title::from("Diagnostics".bold()).alignment(Alignment::Left))
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        // TODO: implemet display on chat message instead?
        let chat_lines = self.page_state.chat_messages.get_lines().iter().map(|l| {
            let msg = Line::from(Span::raw(format!("{}", l)));
            ListItem::new(msg)
        });

        let chatters_lines = self.page_state.online_users.iter().map(|l| {
            let item = Line::from(Span::raw(l));
            ListItem::new(item)
        });

        frame.render_widget(List::new(chat_lines).block(chat_block), chat_area);
        frame.render_widget(
            Paragraph::new(self.input.get_ref().iter().collect::<String>()).block(input_block),
            input_area,
        );
        frame.render_widget(
            List::new(chatters_lines).block(chatters_block),
            chatters_area,
        );
        frame.render_widget(diagnostics_block, diagnostics_area);

        frame.set_cursor(
            input_area.x + 1 + self.input.get_ref().len() as u16,
            input_area.y + 1,
        )
    }
}
