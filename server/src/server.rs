use std::future::Future;
use std::time::SystemTime;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bytes::Bytes;
use shared::connection::write_frame_into;
use shared::message::UserEnteredChat;
use shared::message::WelcomeMessage;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::BufWriter;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

extern crate shared;
use shared::connection::Connection;
use shared::message::Message;

#[derive(Clone, Debug)]
pub struct Client {
    status: ClientStatus,
    name: String,
    connected_at: chrono::NaiveDateTime,
    messages_sent: u64,
}

#[derive(Clone, Debug)]
pub enum ClientStatus {
    Online,
    Offline,
}

impl Client {
    fn new(name: String, connected_at: chrono::NaiveDateTime) -> Self {
        Self {
            status: ClientStatus::Online,
            name,
            connected_at,
            messages_sent: 0,
        }
    }

    fn mark_offline(self) -> Self {
        Self {
            status: ClientStatus::Offline,
            name: self.name,
            connected_at: self.connected_at,
            messages_sent: self.messages_sent,
        }
    }

    fn increment_messages(&mut self) {
        self.messages_sent += 1;
    }
}

struct Shutdown {
    shutdown_announced: bool,
    shutdown_receiver: broadcast::Receiver<()>,
}

impl Shutdown {
    fn new(shutdown_receiver: broadcast::Receiver<()>) -> Self {
        Self {
            shutdown_announced: false,
            shutdown_receiver,
        }
    }

    fn shutdown_announced(&self) -> bool {
        self.shutdown_announced
    }

    async fn recv_shutdown(&mut self) {
        if self.shutdown_announced {
            return;
        }

        let _ = self.shutdown_receiver.recv().await;
        self.shutdown_announced = true;
    }
}

struct ConnectionHandler<W, R>
where
    W: AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
{
    connection: Connection<W, R>,
    shutdown: Shutdown,
    client_status_sender: mpsc::Sender<Client>,
    client_message_sender: broadcast::Sender<Message>,
    client_message_receiver: broadcast::Receiver<Message>,
    client: Option<Client>,
}

impl<W, R> ConnectionHandler<W, R>
where
    W: AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
{
    fn new(
        connection: Connection<W, R>,
        shutdown: Shutdown,
        client_status_sender: mpsc::Sender<Client>,
        client_message_sender: broadcast::Sender<Message>,
        client_message_receiver: broadcast::Receiver<Message>,
    ) -> Self {
        Self {
            connection,
            shutdown,
            client_status_sender,
            client_message_sender,
            client_message_receiver,
            client: None,
        }
    }

    async fn handle(&mut self) -> Result<()> {
        while !self.shutdown.shutdown_announced() {
            let maybe_frame = tokio::select! {
                frame = self.connection.read_frame() => frame,
                _ = self.shutdown.recv_shutdown() => {
                    return Ok(())
                }
                // TODO: doing via continue for now for a quick and dirty solution
                broadcasted_message = self.client_message_receiver.recv() => {
                    println!("Receiving broadcasted_message");
                    if let Ok(message) = broadcasted_message {
                        match message {
                            Message::ChatMessage(_) => {
                                let mut dbg_msg = BufWriter::new(Vec::new());
                                write_frame_into(&mut dbg_msg, message.clone().into_frame()).await?;
                                println!("Sending: {:?}", Bytes::copy_from_slice(dbg_msg.get_ref().as_slice()));
                                self.connection.write_frame(message.into_frame()).await?;
                            },
                            Message::UserEnteredChat(_) => {
                                self.connection.write_frame(message.into_frame()).await?;
                            }
                            unexpected => {
                                eprintln!("Expected a chat message, got {:?}", unexpected)
                            },
                        }
                    } else {
                        eprintln!("Error receiving broadcast: {:?}", broadcasted_message);
                    }
                    continue;
                }
            };

            // TODO: refactor message handling once it is parsed and verified
            match Message::from_frame(maybe_frame?) {
                Ok(msg) => match msg {
                    Message::Login(msg) => {
                        let now_timestamp = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)?
                            .as_millis();
                        self.client = Some(Client::new(
                            String::from_utf8(msg.name.to_vec())?,
                            chrono::NaiveDateTime::from_timestamp_millis(now_timestamp.try_into()?)
                                .ok_or(anyhow!("The clock might've gone backwards"))?,
                        ));
                        // NOTE: Safety: the client is initialized just before unwrapping, hence
                        // it's safe
                        self.client_status_sender
                            .send(self.client.as_ref().unwrap().clone())
                            .await?;

                        let message =
                            Message::WelcomeMessage(WelcomeMessage::new("Welcome to chad!".into()))
                                .into_frame();
                        self.connection.write_frame(message).await?;
                    }
                    Message::Logout(_) => {
                        // NOTE: Safety: unwrap below should never panic? LUL
                        let logged_out_client = self.client.take().unwrap().mark_offline();
                        self.client_status_sender.send(logged_out_client).await?;
                        return Ok(());
                    }
                    Message::ChatMessage(msg) => {
                        self.client_message_sender
                            .send(Message::ChatMessage(msg))
                            .map_err(|_| anyhow!("All receivers dropped the handle"))?;
                        // TODO: proooobably not safe to unwrap here, should be if-let with
                        // handling instead
                        self.client.as_mut().unwrap().increment_messages();
                    }
                    Message::WelcomeMessage(_) => bail!("We are hijacked, aborting immediately"),
                    Message::UserEnteredChat(_) => bail!("We are hijacked, aborting immediately"),
                },
                Err(e) => {
                    eprintln!("Protocol error: {}", e);
                }
            }
        }
        Ok(())
    }
}

pub struct Server {
    tcp_listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete: mpsc::Sender<()>,
    client_status_reciever: mpsc::Receiver<Client>,
    clients_connected: u64,
}

impl Server {
    pub async fn run(
        &mut self,
        client_status_sender: mpsc::Sender<Client>,
        notify_shutdown: broadcast::Sender<()>,
        client_message_sender: broadcast::Sender<Message>,
    ) -> Result<()> {
        loop {
            let (socket, address) = tokio::select! {
                conn = self.tcp_listener.accept() => conn?,
                client_connected = self.client_status_reciever.recv() => {
                    if let Some(client) = client_connected {
                        match client.status {
                            ClientStatus::Online => {
                                self.clients_connected += 1;
                                println!("New client connected: {:?}", client);
                                let _ = client_message_sender.send(Message::UserEnteredChat(UserEnteredChat::new(
                                    format!("{} joined the chat!", client.name).into(),
                                    client.name.into()
                                )));
                            }
                            ClientStatus::Offline => {
                                self.clients_connected -= 1;
                                println!("Client {:?} disconnected", client);
                            }
                        }
                    } else {
                        eprintln!("All receivers dropped the send handle");
                    }
                        continue;
                }
            };

            println!("Accepted connection from {}", address);
            let (read_half, write_half) = socket.into_split();
            let mut handler = ConnectionHandler::new(
                Connection::new(read_half, write_half),
                Shutdown::new(notify_shutdown.subscribe()),
                client_status_sender.clone(),
                client_message_sender.clone(),
                client_message_sender.subscribe(),
            );

            tokio::spawn(async move {
                if let Err(e) = handler.handle().await {
                    eprintln!("An error occured: {}", e);
                    let _ = handler
                        .client_status_sender
                        .send(handler.client.take().unwrap().mark_offline())
                        .await
                        .map_err(|e| {
                            eprintln!("Couldn't let the server know a client got disconnected");
                            e
                        });
                }
            });
        }
    }
}

pub async fn run(listener: TcpListener, shutdown_sig: impl Future) -> Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    // TODO: explore client status channel capacity
    let (client_status_sender, client_status_reciever) = mpsc::channel(1);
    let (client_message_sender, _) = broadcast::channel(1);
    let (shutdown_complete, mut shutdown_complete_reciever) = mpsc::channel(1);

    let mut server = Server {
        tcp_listener: listener,
        notify_shutdown: notify_shutdown.clone(),
        shutdown_complete,
        clients_connected: 0,
        client_status_reciever,
    };

    tokio::select! {
        run_res = server.run(client_status_sender, notify_shutdown, client_message_sender) => {
            if let Err(err) = run_res {
                eprintln!("Failed accepting connection: {}", err);
            }
        }
        _ = shutdown_sig => {
            println!("Received termination signal, shutting down gracefully");
        }
    }

    let Server {
        notify_shutdown,
        shutdown_complete,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete);

    let _ = shutdown_complete_reciever.recv().await;

    Ok(())
}
