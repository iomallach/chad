use crate::ui::page::widget::Widget;
use crate::{client::ClientInput, state::action::Action};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Stylize;
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::block::{Position, Title};
use ratatui::widgets::{Block, Borders, Paragraph};
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct LoginPage {
    action_tx: UnboundedSender<Action>,
    input: ClientInput,
}

impl LoginPage {
    pub(crate) fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            input: ClientInput::new(),
        }
    }
}

impl Widget for LoginPage {
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
                    .send(Action::ConnectAndLogin { name: source })
                    .expect("Receiver unexpectedly dropped");
            }
            _ => {}
        }
    }
    fn update(&mut self, _state: crate::state::state::State) {}

    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let title = Title::from("Type your name and jump right into it".bold());
        let instructions = Title::from(Line::from(vec![
            "Enter ".into(),
            "your".green().bold(),
            " name".into(),
            " or".red().bold(),
            " press ^c to quit".into(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(symbols::border::THICK);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Fill(1),
                Constraint::Percentage(25),
            ])
            .split(frame.size());
        let center_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Fill(1),
                Constraint::Percentage(25),
            ])
            .split(chunks[1]);

        let input_area = center_chunks[1];

        frame.render_widget(
            Paragraph::new(self.input.get_ref().iter().collect::<String>())
                .centered()
                .block(block),
            input_area,
        );
        let mut buf_len = self.input.inner.len() as u16;
        if buf_len % 2 != 0 {
            buf_len += 1;
        }
        frame.set_cursor(
            input_area.x + input_area.width / 2 + buf_len / 2,
            input_area.y + 1,
        );
    }
}
