use crate::{chat::ChatLog, client::ClientInput};
use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::{Stylize, Terminal},
    symbols,
    text::{Line, Span},
    widgets::{
        block::{Position, Title},
        Block, Borders, List, ListItem, Paragraph, Tabs, Widget,
    },
};

pub enum CurrentScreen {
    Login,
    Chatting,
}

pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &CurrentScreen,
    input_buffer: &mut ClientInput,
    chat_log: &ChatLog,
    chatters: &[String],
) -> Result<()> {
    match state {
        CurrentScreen::Login => draw_login_screen(terminal, state, input_buffer)?,
        CurrentScreen::Chatting => {
            draw_chatting_screen(terminal, state, input_buffer, chat_log, chatters)?
        }
    }
    Ok(())
}

pub fn draw_chatting_screen<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &CurrentScreen,
    input_buffer: &mut ClientInput,
    chat_log: &ChatLog,
    chatters: &[String],
) -> Result<()> {
    terminal.draw(|frame| {
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

        let chat_lines = chat_log.get_lines().iter().map(|l| {
            let msg = Line::from(Span::raw(format!(
                "{} {}{}: {}",
                l.timestamp, l.icon, l.username, l.msg
            )));
            ListItem::new(msg)
        });

        let chatters_lines = chatters.iter().map(|l| {
            let item = Line::from(Span::raw(l));
            ListItem::new(item)
        });

        frame.render_widget(List::new(chat_lines).block(chat_block), chat_area);
        frame.render_widget(
            Paragraph::new(input_buffer.inner.iter().collect::<String>()).block(input_block),
            input_area,
        );
        frame.render_widget(
            List::new(chatters_lines).block(chatters_block),
            chatters_area,
        );
        frame.render_widget(diagnostics_block, diagnostics_area);

        frame.set_cursor(
            input_area.x + 1 + input_buffer.inner.len() as u16,
            input_area.y + 1,
        )
    })?;

    Ok(())
}
pub fn draw_login_screen<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &CurrentScreen,
    input_buffer: &mut ClientInput,
) -> Result<()> {
    let title = Title::from("Login and start chadding".bold());
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

    terminal.draw(|frame| {
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
            Paragraph::new(input_buffer.inner.iter().collect::<String>())
                .centered()
                .block(block),
            input_area,
        );
        let mut buf_len = input_buffer.inner.len() as u16;
        if buf_len % 2 != 0 {
            buf_len += 1;
        }
        frame.set_cursor(
            input_area.x + input_area.width / 2 + buf_len / 2,
            input_area.y + 1,
        );
    })?;
    Ok(())
}
