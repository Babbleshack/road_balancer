use core::panic;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::fs;
use std::io;

use serde::Deserialize;
use toml;

use log::{info,warn};
use simple_logger::SimpleLogger;

const BUFF_SIZE: usize = 1024;
const DEFAULT_PORT: u64 = 9090;

#[derive(Deserialize, Debug)]
struct Config {
    port: Option<u64>,
}

#[derive(Debug)]
enum ClientError {
    ZeroRead(String),
}


// TODO: find end off http header so taht we can parse it.
// // header will be used to redirect request to target

fn handle_client(mut stream: TcpStream) {
    println!("Handling connection {:?}", stream);
    //let mut reader = BufReader::new(stream);

    let mut buf = [0 as u8; BUFF_SIZE];
        match stream.read(&mut buf) {
            Ok(read) => {
                println!("Read {}", read); 
                let text = match str::from_utf8(&buf) {
                    Ok(v) => {
                        v
                    },
                    Err(e) => panic!("Error reading from buffer {}", e),
                };
                println!("Got message: {:}", text); 
                println!("Got message: {:?}", text); 
                buf.fill(0u8);
            },
            Err(e) => {
                println!("Error {}", e);
                stream.shutdown(std::net::Shutdown::Both).unwrap();
                std::process::exit(0 as i32);
            }
    };
}

fn handle_http_client(mut stream: TcpStream) {
    loop {
        match read_next_line(&mut stream) {
            Ok(line) => info!("Read Line: {}", line),
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                return
            }
        }
    }
}


fn read_next_line(stream: &mut TcpStream) -> io::Result<String> {
    // Read bytes from stream until next CL RF is detected indicating new line
    // TODO: configure buffer size, prevent huge 'lines'
    let mut buf = Vec::new();
    let mut prev_byte_was_cr = false;
    loop {
        let byte = &mut vec![0u8; 1];    
        // bubble error up
        match stream.read(byte)  {
            Ok(read) => {
                if read == 0 {
                    //return Err(ClientError::ZeroRead("Remote peer killed connection"))
                   return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Remote peer killed connection"))
                }
            },
            Err(e) => return Err(e),
        }
        // We might want to keep CL RF here because we are forwarding the request
        // or we could pass header to http crate
        // TODO: parse with http header crate
        if byte[0] == b'\n' && prev_byte_was_cr {
            buf.pop();
            return Ok(String::from_utf8(buf).unwrap())
        }
        prev_byte_was_cr = byte[0] == b'\r';
        buf.push(byte[0]);
    }
}

fn init_logger() {
    SimpleLogger::new().init().unwrap();
}

fn main() {
    init_logger();
    info!("Hello, from road_balancer, lets get balancing!");
    // Parse command args
    let path = std::env::args().nth(1).or(Some("config.toml".to_string())).unwrap();
    // Parse config
    let config = match fs::read_to_string(path) {
        Ok(conf) => conf,
        Err(e) => panic!("Error reading config {:?}", e),
    };
    let config: Config = match toml::from_str(&config) {
        Ok(conf) => conf,
        Err(e) => panic!("Error parsing config toml {:?}", e),
    };
    info!("Config {:?}", config);
    let port = config.port.or(Some(DEFAULT_PORT)).unwrap();
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(listener) => listener,
        Err(e) => panic!("Error binding to port {} : {}", port, e),
    };
    let port = listener.local_addr().unwrap();
    info!("Listening on port: {}", port);
    let stream = match listener.accept() {
        Ok(stream) => stream.0,
        Err(e) => panic!("Error {}", e) 
    };
    //handle_client(stream);
    handle_http_client(stream);
    info!("All done...");
}
