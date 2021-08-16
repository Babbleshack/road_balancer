use std::io::{Read, Write};
use std::net::TcpStream;

//TODO: EnhancedStream.inner should be a reference to something implementing Read + Write traits,
//not a TcpStream
//
struct EnhancedStream {
    inner: TcpStream,
}

impl Read for EnhancedStream { 
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>  {
        self.inner.read(buf)
    }
}

impl Write for EnhancedStream {
    fn write(&mut self, buf: & [u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}


impl EnhancedStream {
    fn new(stream: TcpStream) -> Self { 
        Self {
            inner: stream,
        }
    }
    // parse // parse header and data
    // return itterator over data?
}

