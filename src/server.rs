use std::net::{TcpListener, TcpStream};
struct Message {
    stream: TcpStream,
}
struct Server {
    listener: TcpListener,
    queue: message_queue<Message>,
}

impl Server {
    fn new() -> Self {
        Self{}
    }
}



