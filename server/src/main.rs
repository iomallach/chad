use anyhow::Result;
use tokio::{net::TcpListener, signal::ctrl_c};

extern crate shared;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8080").await?;
    let ctrl_c = ctrl_c();

    println!("Serving at 127.0.0.1:8080");
    server::run(tcp_listener, ctrl_c).await?;
    Ok(())
}
