use std::io::{Read, Write, Result};
use std::cmp::min;

use super::balancer;

#[derive(Debug, Default)]
struct TestStream {
    read_data: Vec<u8>,
    index: usize,
}

//struct TestRequest { 
//    method: &'static str,
//    uri: &'static str,
//    version: &'static str,
//    data: &'static[u8],
//}
//
//impl TestRequest {
//    // Take some message and execute closure?
//    fn test(self) {
//
//    }
//}

impl Read for TestStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>  {
        let offset = min(self.read_data.len() - self.index, buf.len());
        buf[..offset].copy_from_slice(&self.read_data[self.index..self.index + offset]);
        self.index += offset;
        Ok(offset)
    }
}

impl Write for TestStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize>  {
        self.read_data = Vec::from(buf);
        Ok(self.read_data.len())
    }
    fn flush(&mut self) -> Result<()> {
        self.index = 0;
        Ok(())
    }
}

#[test]
fn test_read_next_line() {
    let request = b"TEST\r\nTWO\r\n";
    let mut request_buffer = vec![0u8; request.len()];
    request_buffer[..request.len()].clone_from_slice(request);
    let stream = &mut TestStream {
        read_data: request_buffer,
        index: 0,
    };
    // We cut off CRLF in read_next_line
    let line = balancer::read_next_line(stream, true).unwrap();
    assert_eq!("TEST".as_bytes(), line.as_bytes());
    let line = balancer::read_next_line(stream, true).unwrap();
    assert_eq!("TWO".as_bytes(), line.as_bytes());
}

#[test]
fn test_read_header() {
    let request = b"GET / HTTP/1.1\r\n\r\n\r\n";
    let request_buffer = request.to_vec();
    let stream = &mut TestStream {
        read_data: request_buffer,
        index: 0,
    };
    match balancer::read_header(stream) {
        Ok((header, _)) => {
            assert_eq!("GET", header.method());
            assert_eq!("/", header.uri());
            assert_eq!("HTTP/1.1", header.version());
        },
        Err(_) => panic!("Error"),
    }
}

#[test]
fn test_read_next_line_no_chop() {
    let request = b"TEST\r\nTWO\r\n";
    let mut request_buffer = vec![0u8; request.len()];
    request_buffer[..request.len()].clone_from_slice(request);
    let stream = &mut TestStream {
        read_data: request_buffer,
        index: 0,
    };
    // We cut off CRLF in read_next_line
    let line = balancer::read_next_line(stream, false).unwrap();
    assert_eq!("TEST\r\n".as_bytes(), line.as_bytes());
    let line = balancer::read_next_line(stream, false).unwrap();
    assert_eq!("TWO\r\n".as_bytes(), line.as_bytes());
}
