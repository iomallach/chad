use std::io::stdout;
use std::io::Stdout;
use std::time::Duration;

use crate::state::action::Action;
use crate::state::state::State;
use anyhow::Result;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use futures::FutureExt;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::interval;

use super::dispatch::Dispatcher;

pub(crate) struct UiManager {
    action_tx: UnboundedSender<Action>,
}

impl UiManager {
    pub(crate) fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self { action_tx }
    }

    pub(crate) async fn ui_loop(&mut self, mut state_rx: UnboundedReceiver<State>) -> Result<()> {
        let mut terminal = enter_terminal_app()?;
        let state = state_rx
            .recv()
            .await
            .expect("State channel closed unexpectedly right at the start");
        let mut dispatcher = Dispatcher::new(self.action_tx.clone(), state);
        let mut ticker = interval(Duration::from_millis(250));
        let mut crossterm_events = EventStream::new();

        loop {
            select! {
                _ = ticker.tick() => {},
                Some(Ok(Event::Key(event))) = crossterm_events.next().fuse() => dispatcher.handle_key_event(event),
                Some(state) = state_rx.recv() => dispatcher.update(state),
            }

            terminal.draw(|frame| dispatcher.render(frame))?;
        }
        exit_terminal_app()?;
        Ok(())
    }
}

fn enter_terminal_app() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    install_panic_hook();

    execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All))?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

fn exit_terminal_app() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        exit_terminal_app().unwrap();
        original_hook(panic);
    }))
}
