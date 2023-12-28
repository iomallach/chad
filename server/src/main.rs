use chrono::DateTime;
use chrono::Local;
use mio::Poll;
use mio::Events;
use mio::Token;
use mio::Interest;
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;
use std::cell::RefCell;

struct Counter(usize);

impl Counter {
    fn new() -> Self {
        Self(0)
    }

    fn increment(&mut self) {
        self.0 += 1;
    }
}

struct Client {
    stream: TcpStream,
    login_name: String,
    connected_at: DateTime<Local>
}

impl Client {
    fn new(stream: TcpStream, login_name: String) -> Self {
        Self {
            stream,
            login_name,
            connected_at: chrono::offset::Local::now()
        }
    }
}

struct Server {
    listener: TcpListener,
    socket_addr: SocketAddr,
    poll: Poll,
    events: Events,
    next_socket_id: RefCell<Counter>,
    clients: RefCell<HashMap<Token, Client>>,
}

impl Server {
    const LISTENER: Token = Token(0);

    fn new(addr: &str, capacity: usize) -> Result<Self, Box<dyn Error>> {
        let socket_addr: SocketAddr = addr.parse()?;
        let listener = TcpListener::bind(socket_addr)?;
        let poll = Poll::new()?;
        let events = Events::with_capacity(capacity);
        Ok(Self {
            listener,
            socket_addr,
            poll,
            events,
            next_socket_id: RefCell::new(Counter::new()),
            clients: RefCell::new(HashMap::new()),
        })
    }

    fn issue_token(&self) -> Token {
        self.next_socket_id.borrow_mut().increment();
        Token(self.next_socket_id.borrow().0)
    }

    fn register_listener(&mut self) -> Result<(), Box<dyn Error>> {
        self.poll.registry().register(&mut self.listener, Self::LISTENER, Interest::READABLE)?;
        Ok(())
    }

    fn register_client(&self, mut stream: TcpStream) -> Result<Token, Box<dyn Error>> {
        let token = self.issue_token();
        let mut login_buf: [u8; 255] = [0; 255];
        
        let name = match stream.read(&mut login_buf) {
            Ok(0) => {
                eprintln!("Read 0 bytes, no login information provided");
                None
            },
            Ok(n) => {
                let name = std::str::from_utf8(&login_buf[..n]).expect("Reading name failed");
                Some(name.to_owned())
            }
            Err(e) => {
                eprintln!("Failure on handshake {}", e);
                None
            }
        };
        self.poll.registry().register(&mut stream, token, Interest::READABLE.add(Interest::WRITABLE))?;
        match name {
            None => self.clients.borrow_mut().insert(token, Client::new(stream, "".to_owned())),
            Some(c) => self.clients.borrow_mut().insert(token, Client::new(stream, c))
        };
        Ok(token)
    }

    fn poll(&mut self, timeout: Option<Duration>) -> Result<(), std::io::Error> {
        self.poll.poll(&mut self.events, timeout)
    }

    fn borrow_events(&self) -> &Events {
        &self.events
    }

    fn accept(&self, event: &Event) {
        match event.token() {
            Self::LISTENER => {
                self.handle_new_client();
            },
            token => {
                self.handle_stream(token);
            }
        }
    }

    fn handle_new_client(&self) {
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("Connection from {}", addr);
                    if let Ok(token) = self.register_client(stream) {
                        println!("Accepted connection from {} with token {}", addr, <Token as Into<usize>>::into(token));
                        self.clients.borrow_mut().get_mut(&token).unwrap().stream.write(b"\x06system***Welcome to Chad!***").expect("Failed to write");
                    } else {
                        eprintln!("Failed to register client: {}", addr);
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                e => panic!("Error accepting connection: {:?}", e)
            }
        }
    }

    fn handle_stream(&self, token: Token) {
        let mut clients = self.clients.borrow_mut();
        if let Some(socket) = clients.get_mut(&token) {
            let addr = socket.stream.local_addr().expect("Failed to get address");
            println!("Got event on socket {:?} and address {} from {}", token, addr, socket.login_name);
            let mut buffer = [0; 8 * 1024];
            match socket.stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Socket {:?} closed", token);
                    clients.remove(&token);
                },
                Ok(n) => {
                    println!("Read {} bytes from socket {:?} : {} saying {}", n, token, socket.login_name, String::from_utf8_lossy(&buffer));
                    Self::broadcast_message(&socket.login_name.clone(), &buffer[0..n], &mut clients)
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                },
                e => panic!("Error reading from socket {:?}: {:?}", token, e)
            }
        }
    }

    fn broadcast_message(from: &str, message: &[u8], across: &mut HashMap<Token, Client>) {
        across.iter().for_each(|(tok, client)| {
            let mut stream = &client.stream;
            let mut buffer: Vec<u8> = Vec::new();
            let len_byte: u8 = from.len() as u8;
            buffer.push(len_byte);
            let sender_name_bytes = from.as_bytes();
            buffer.extend(sender_name_bytes);
            buffer.extend(message);

            stream.write(&buffer).expect("Failed to broadcast");
            stream.flush().expect("Failed to flush");
            // write!(&mut stream, "{}", String::from_utf8_lossy(message)).expect("Failed to broadcast");
            // if tok != from {
            // }
        })
    }
}


fn main() -> Result<(), ()> {
    println!("{:?} {:?} {:?} {:?}", b"6", b"7", b"8", b"9");
    println!("Starting TCP server at {}", "127.0.0.1:8080");
    let mut server = Server::new("127.0.0.1:8080", 128).map_err(|e| {
        eprintln!("Failed to create server: {}", e);
    })?;
    server.register_listener().map_err(|e| {
        eprintln!("Failed to register listener: {}", e);
    })?;
    println!("Listenting on {}", server.socket_addr);

    loop {
        server.poll(None).map_err(|e| {
            eprintln!("Failed to poll: {}", e)
        })?;
        for event in server.borrow_events().iter() {
            server.accept(event);
        }
    }
}
