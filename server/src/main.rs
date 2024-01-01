extern crate shared;
use shared::{Message, ParseMessageError, ParseMessageErrorKind};
use shared::{read_message, send_message};
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
use std::net::SocketAddr;
use std::str::FromStr;
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

    fn read_message(&self, stream: &mut TcpStream) -> Result<Message, io::Error> {
        let msg = read_message(stream)?;
        Ok(Message::from_str(&msg).expect("No issues here"))
    }

    fn register_listener(&mut self) -> Result<(), Box<dyn Error>> {
        self.poll.registry().register(&mut self.listener, Self::LISTENER, Interest::READABLE)?;
        Ok(())
    }

    fn register_client(&self, mut stream: TcpStream) -> Result<Token, io::Error> {
        let token = self.issue_token();
        // TODO: what do I do with this shit?
        std::thread::sleep(std::time::Duration::from_millis(100));
        let message = self.read_message(&mut stream)?;
        self.poll.registry().register(&mut stream, token, Interest::READABLE.add(Interest::WRITABLE))?;
        self.clients.borrow_mut().insert(token, Client::new(stream, message.username));
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
                    match self.register_client(stream) {
                        Ok(token) => {
                            println!("Accepted connection from {} with token {}", addr, <Token as Into<usize>>::into(token));
                            let system_msg = Message::new("System", Some(self.clients.borrow().len()), Some("***WELCOME TO CHAD***")).to_string();
                            send_message(system_msg.as_str(), &mut self.clients.borrow_mut().get_mut(&token).unwrap().stream).expect("Failed to write");
                            println!("Sent message: {}", system_msg);
                        },
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => eprintln!("Failed to register client: {} with {}", addr, e),
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
            let addr = socket.stream.peer_addr().expect("Failed to get address");
            println!("Got event on socket {:?} and address {} from {}", token, addr, socket.login_name);
            match read_message(&mut socket.stream) {
                Ok(m) => {
                    println!("Read from token={:?}, username {} saying {}", token, socket.login_name, m);
                    Self::broadcast_message(&socket.login_name.clone(), Message::from_str(&m).unwrap(), &mut clients)
                },
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    println!("Socket {:?} closed", token);
                    clients.remove(&token);
                },
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {},
                Err(e) => { panic!("Unexpected error {}", e) }
            }
            // match socket.stream.read(&mut buffer) {
            //     Ok(0) => {
            //         println!("Socket {:?} closed", token);
            //         clients.remove(&token);
            //     },
            //     Ok(n) => {
            //         println!("Read {} bytes from socket {:?} : {} saying {}", n, token, socket.login_name, String::from_utf8_lossy(&buffer));
            //         Self::broadcast_message(&socket.login_name.clone(), &buffer[0..n], &mut clients)
            //     },
            //     Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            //     },
            //     e => panic!("Error reading from socket {:?}: {:?}", token, e)
            // }
        }
    }

    fn broadcast_message(from: &str, message: Message, across: &mut HashMap<Token, Client>) {
        across.iter().for_each(|(tok, client)| {
            let mut stream = &client.stream;
            if let Some(m) = &message.message {
                println!("Broadcasting {} length", across.len());
                let broadcast_message = Message::new(from, Some(across.len()), Some(m)).to_string();
                send_message(&broadcast_message, &mut stream).expect("Failed to write");
            }
        })
    }
}


fn main() -> Result<(), ()> {
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
