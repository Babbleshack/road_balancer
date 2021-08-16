mod balancer;
mod stream;
#[cfg(test)]
mod tests;

use core::panic;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::fs;

use serde::Deserialize;
use toml;

use log::info;
use simple_logger::SimpleLogger;

const DEFAULT_PORT: u64 = 9090;

#[derive(Deserialize, Debug)]
struct Config {
    port: Option<u64>,
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
    info!("Config path: {}", path);
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
    balancer::handle_client::<TcpStream>(stream);
    info!("All done...");
}
