use crossterm::event::KeyEvent;
use ratatui::prelude::Frame;

use crate::state::state::State;

// TODO: revisit the need for <P> down where and in render
pub(crate) trait Widget {
    fn render(&self, frame: &mut Frame);
    fn update(&mut self, state: State);
    fn handle_key_event(&mut self, key: KeyEvent);
}
