use std::io::{Read,Write};
use std::io;
use log::{info,warn};
use std::convert::TryInto;
use http_header::{
    Header, RequestHeader, 
};

struct HTTPLine([u8]);

// TODO: find end off http header so taht we can parse it.
// // header will be used to redirect request to target

pub fn handle_client<T: Read + Write>(mut stream: T)
    where T: Read + Write 
{
    loop {
        let line = match read_next_line(&mut stream, true) {
            Ok(line) => line,
            Err(e) => {
                warn!("Error reading from socket: {}", e);
                return
            }
        };
        info!("Read line {}", line);
        //Test if line is a http header
    }
}


pub fn read_next_line<T: Read + Write>(stream: &mut T, chop_eol: bool) -> io::Result<String> {
    // read bytes from stream until next cl rf is detected indicating new line todo: configure buffer size, prevent huge 'lines'
    let mut buf = Vec::new();
    let mut prev_byte_was_cr = false;
    loop {
        let byte = &mut vec![0u8; 1];    
        // bubble error up
        match stream.read(byte)  {
            Ok(read) => {
                if read == 0 {
                   return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "remote peer killed connection"))
                }
            },
            Err(e) => return Err(e),
        }
        // we might want to keep cl rf here because we are forwarding the request
        // or we could pass header to http crate
        // todo: parse with http header crate
        if byte[0] == b'\n' && prev_byte_was_cr {
            if chop_eol {
                buf.pop();
            } else {
                buf.push(byte[0]);
            }
            return Ok(String::from_utf8(buf).unwrap())
        }
        prev_byte_was_cr = byte[0] == b'\r';
        buf.push(byte[0]);
    }
}

pub fn read_header<T: Read + Write>(stream: &mut T) -> io::Result<(RequestHeader, Vec<u8>)> {
    let crlf = b"\r\n";
    let mut end_of_header = false;
    let mut buf = Vec::new(); 
    while end_of_header == false {
        let mut bytes = match read_next_line(stream, false) {
            Ok(line) => line.into_bytes(),
            Err(e) => panic!("{}", e),
        };
        if bytes == crlf {
            end_of_header = true;
        }
        buf.append(&mut bytes);
        //buf.push(bytes.as_slice());
    }
    println!("Got header bytes {:?}", buf);
    let (header, data) = Header::scan(buf.as_slice()).unwrap();
    let header: RequestHeader = Header::parse(header).unwrap().try_into().unwrap();
    Ok((header, data.to_vec()))
}

