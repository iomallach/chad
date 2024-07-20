extern crate shared;
use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

mod client;
mod state;
mod ui;

use crate::state::state_manager::StateManager;
use crate::ui::ui_manager::UiManager;

#[tokio::main]
async fn main() -> Result<()> {
    let (state_tx, state_rx) = mpsc::unbounded_channel();
    let (action_tx, action_rx) = mpsc::unbounded_channel();
    let (termination_tx, termination_rx) = oneshot::channel();
    let mut state_manager = StateManager::new(state_tx);
    let mut ui_manager = UiManager::new(action_tx);

    let state_join_handle =
        tokio::spawn(async move { state_manager.state_loop(action_rx, termination_tx).await });
    let ui_join_handle =
        tokio::spawn(async move { ui_manager.ui_loop(state_rx, termination_rx).await });

    let (join1, join2) = tokio::try_join!(state_join_handle, ui_join_handle)?;

    join1?;
    join2?;
    Ok(())
}
