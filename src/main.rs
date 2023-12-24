use mio::Poll;
use mio::Events;
use mio::Token;
use mio::Interest;
use mio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::result::Result;


fn main() -> Result<(), ()> {
    println!("Starting TCP server at {}", "127.0.0.1:8080");
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    // TODO: catch errors
    let mut listener = TcpListener::bind(addr).map_err(|e| {
        eprintln!("Failed to bind to {}: {}", addr, e);
    })?;
    const LISTENER: Token = Token(0);
    let mut poll = Poll::new().map_err(|e| {
        eprintln!("Failed to create poll: {}", e);
    })?;
    let mut events = Events::with_capacity(128);

    poll.registry().register(&mut listener, LISTENER, Interest::READABLE).map_err(|e| {
        eprintln!("Failed to register listener: {}", e);
    })?;

    println!("Listenting on {}", addr);

    let mut next_socket_id: usize = 1;
    let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
    loop {
        poll.poll(&mut events, None).map_err(|e| {
            eprintln!("Failed to poll: {}", e)
        })?;
        for event in &events {
            match event.token() {
                LISTENER => {
                    loop {
                        match listener.accept() {
                            Ok((mut stream, addr)) => {
                                println!("Connection from {}", addr);
                                let token = Token(next_socket_id);
                                next_socket_id += 1;
                                poll.registry().register(&mut stream, token, Interest::READABLE.add(Interest::WRITABLE)).map_err(|e| {
                                    eprintln!("Failed to register {:?} {:?}", token, e)
                                })?;
                                writeln!(&mut stream, "Welcome to the chad!").unwrap();
                                sockets.insert(token, stream);
                                println!("Accepted connection from {}, saving the socket", addr);
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            e => panic!("Error accepting connection: {:?}", e)
                        }
                    }
                },
                token => {
                    if let Some(socket) = sockets.get_mut(&token) {
                        let addr = socket.local_addr().map_err(|e| {
                            eprintln!("Failed to get local addr: {}", e)
                        })?;
                        println!("Got event on socket {:?} and address {}", token, addr);
                        let mut buffer = [0; 8 * 1024];
                        match socket.read(&mut buffer) {
                            Ok(0) => {
                                println!("Socket {:?} closed", token);
                                sockets.remove(&token);
                                break;
                            },
                            Ok(n) => {
                                println!("Read {} bytes from socket {:?} saying {}", n, token, String::from_utf8_lossy(&buffer));
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            },
                            e => panic!("Error reading from socket {:?}: {:?}", token, e)
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
