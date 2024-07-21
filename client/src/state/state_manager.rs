use std::time::Duration;

use crate::state::state::ConnectionStatus;
use crate::state::{action::Action, state::State};
use anyhow::Result;
use bytes::Bytes;
use shared::message::Message;
use shared::{connection::Connection, message::ChatMessage};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::interval,
};

pub(crate) struct StateManager {
    state_tx: UnboundedSender<State>,
}

impl StateManager {
    pub fn new(state_tx: UnboundedSender<State>) -> Self {
        Self { state_tx }
    }

    pub async fn state_loop(
        &mut self,
        mut action_rx: UnboundedReceiver<Action>,
        termination_tx: broadcast::Sender<()>,
    ) -> Result<()> {
        let mut connection: Option<Connection<OwnedWriteHalf, OwnedReadHalf>> = None;
        let mut state = State::default();
        let mut ticker = interval(Duration::from_millis(500));
        self.state_tx.send(state.clone())?;

        loop {
            if let Some(ref mut conn) = connection {
                select! {
                    maybe_frame = conn.read_frame() => {
                        let server_message = Message::from_frame(maybe_frame?)?;
                        state.handle_server_message(server_message);
                    },
                    Some(ui_event) = action_rx.recv() => match ui_event {
                        Action::ConnectAndLogin { .. } => unreachable!("Impossible action when the connection is already established"),
                        Action::SendMessage { message } => {
                            conn.write_frame(
                                shared::message::Message::ChatMessage(
                                    ChatMessage::new(
                                        state.login_name.clone().expect("Empty login name").into(),
                                        None,
                                        message.into(),
                                    )
                                ).into_frame()
                            ).await?;
                            state.messages_sent += 1;
                        },
                        Action::Quit => {
                            let _ = termination_tx.send(());
                            break;
                        },
                    },
                    _ = ticker.tick() => {},
                }
            } else {
                select! {
                    Some(ui_event) = action_rx.recv() => match ui_event {
                        Action::ConnectAndLogin { name } => {
                            connection = Some(create_connection_handle("127.0.0.1:8080").await?);
                            let login_message = shared::message::Message::Login(
                                shared::message::Login::new(
                                    Bytes::copy_from_slice(name.as_bytes())
                                )
                            );
                            connection
                                .as_mut()
                                .expect("Connection suddenly closed shortly after opening")
                                .write_frame(login_message.into_frame()).await?;
                            state.login_name = Some(name);
                            state.connection_status = ConnectionStatus::Online
                        },
                        Action::SendMessage { .. } => unreachable!("Broken state: requesting to send a message when the client if offline"),
                        Action::Quit => break,
                    },
                    _ = ticker.tick() => {},
                }
            }
            self.state_tx.send(state.clone())?;
        }
        Ok(())
    }
}

async fn create_connection_handle(addr: &str) -> Result<Connection<OwnedWriteHalf, OwnedReadHalf>> {
    let stream = TcpStream::connect(addr).await?;
    let (read_half, write_half) = stream.into_split();
    Ok(Connection::new(read_half, write_half))
}
