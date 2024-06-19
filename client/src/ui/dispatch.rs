use crate::{
    state::{action::Action, state::State},
    ui::page::widget::Widget,
};
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::UnboundedSender;

use super::page::{chat_page::ChatPage, login_page::LoginPage};

enum ActivePage {
    Login,
    Chat,
}

pub(crate) struct Dispatcher {
    active_page: ActivePage,
    login_page: LoginPage,
    chat_page: ChatPage,
}

impl Dispatcher {
    pub(crate) fn new(action_tx: UnboundedSender<Action>, state: State) -> Self {
        Self {
            active_page: ActivePage::Login,
            login_page: LoginPage::new(action_tx.clone()),
            chat_page: ChatPage::new(action_tx, state),
        }
    }

    fn get_active_page_mut(&mut self) -> &mut dyn Widget {
        match self.active_page {
            ActivePage::Login => &mut self.login_page,
            ActivePage::Chat => &mut self.chat_page,
        }
    }

    fn get_active_page(&self) -> &dyn Widget {
        match self.active_page {
            ActivePage::Login => &self.login_page,
            ActivePage::Chat => &self.chat_page,
        }
    }

    pub(crate) fn handle_key_event(&mut self, key: KeyEvent) {
        self.get_active_page_mut().handle_key_event(key);
    }

    pub(crate) fn update(&mut self, state: State) {
        self.get_active_page_mut().update(state);
    }

    pub(crate) fn render(&self, frame: &mut ratatui::prelude::Frame) {
        self.get_active_page().render(frame);
    }
}
